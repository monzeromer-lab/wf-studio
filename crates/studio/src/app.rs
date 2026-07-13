//! `StudioApp` — the single GPUI entity that holds all Studio state and the
//! logic that drives it (a faithful port of the mock's `Component` class), plus
//! the **preview webview** embedded in the canvas.
//!
//! The chrome is native GPUI ([`crate::ui`]). The generated website renders in a
//! wry webview built as a *child* of the gpui window (`build_as_child`) and
//! wrapped in [`gpui_component::webview::WebView`]. On Linux the child path is
//! X11-only, so `main` forces X11 (XWayland); gpui 0.2.2's X11 `window_handle()`
//! is `unimplemented!()`, so we locate our window by `_NET_WM_PID` and hand
//! `build_as_child` a handle we construct. webkit's gtk widget is pumped from
//! GPUI's loop.

use std::borrow::Cow;
use std::time::Duration;

use anyhow::Context as _;
use gpui::{Context, Entity, PathPromptOptions, SharedString, Task, Window, prelude::*};
use gpui_component::input::{InputEvent, InputState};
use gpui_component::webview::WebView;
use wry::{
    WebViewBuilder,
    http::{Request, Response, header::CONTENT_TYPE},
};

use crate::state::*;
use crate::{ipc, site, ui};

/// The custom-protocol origin for the preview webview.
const ORIGIN: &str = "wf://localhost";

/// One step of a status pipeline (mock `pipeline(steps, done)`).
pub struct Step {
    pub status: Status,
    pub label: SharedString,
    pub dur_ms: u64,
    pub activity: Option<(Tone, SharedString)>,
}

impl Step {
    fn new(status: Status, label: impl Into<SharedString>, dur_ms: u64) -> Self {
        Self { status, label: label.into(), dur_ms, activity: None }
    }
    fn log(mut self, tone: Tone, text: impl Into<SharedString>) -> Self {
        self.activity = Some((tone, text.into()));
        self
    }
}

pub struct StudioApp {
    pub screen: Screen,
    pub ob_step: u8,
    pub show_advanced: bool,
    pub provider: ProviderId,
    pub model: SharedString,
    pub api_key: Entity<InputState>,
    pub tested: Tested,
    pub dir: Dir,
    pub device: Device,
    pub generated: bool,
    pub status: Status,
    pub sub_label: SharedString,
    pub busy: bool,
    pub prompt: Entity<InputState>,
    pub sel: Vec<Selection>,
    pub messages: Vec<Message>,
    pub attachments: Vec<Attachment>,
    pub active_skills: Vec<SkillId>,
    pub show_skills: bool,
    pub review_open: bool,
    pub chips: Vec<Chip>,
    pub applied_edits: AppliedEdits,
    pub show_settings: bool,
    pub show_activity: bool,
    pub show_history: bool,
    pub toast: bool,
    pub error_action: Option<ErrorAction>,
    pub dock_hint: SharedString,
    pub dock_shake: bool,
    pub pruning: bool,
    pub caching: bool,
    pub heal_attempts: u8,
    pub history: Vec<Checkpoint>,
    pub activity: Vec<ActivityItem>,
    pub clock: u32,
    /// The embedded website-preview webview (`None` if the embed failed).
    pub preview: Option<Entity<WebView>>,
    pipeline_task: Option<Task<()>>,
    hint_task: Option<Task<()>>,
    next_id: u64,
}

impl StudioApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let prompt = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .auto_grow(1, 5)
                .placeholder("Describe the website you want to build\u{2026}")
        });
        let api_key = cx.new(|cx| InputState::new(window, cx).masked(true).placeholder("sk-ant-\u{2026}"));

        cx.subscribe_in(&prompt, window, |this, _, event: &InputEvent, window, cx| match event {
            InputEvent::Change => {
                if !this.dock_hint.is_empty() {
                    this.dock_hint = SharedString::default();
                    cx.notify();
                }
            }
            InputEvent::PressEnter { secondary: false } => this.submit_prompt(window, cx),
            _ => {}
        })
        .detach();

        cx.subscribe(&api_key, |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) && this.tested != Tested::Idle {
                this.tested = Tested::Idle;
                cx.notify();
            }
        })
        .detach();

        // Build the preview webview as a child of the gpui window and keep it
        // hidden until a site is generated.
        let preview = match build_preview(window) {
            Ok(webview) => {
                let entity = cx.new(|cx| WebView::new(webview, window, cx));
                entity.update(cx, |w, _| w.hide());
                Some(entity)
            }
            Err(error) => {
                eprintln!("wf-studio: could not embed the preview webview: {error:#}");
                None
            }
        };

        // Drive webkit's gtk widget from GPUI's loop (no gtk main loop of its
        // own), and apply any canvas-selection clicks the preview posted over
        // the IPC bridge meanwhile.
        #[cfg(target_os = "linux")]
        {
            let ipc_rx = ipc::take_receiver();
            cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor().timer(Duration::from_millis(8)).await;
                    if this.update(cx, |_, _| pump_gtk()).is_err() {
                        break;
                    }
                    let events = ipc::drain(&ipc_rx);
                    if !events.is_empty() && this.update(cx, |app, cx| app.apply_ipc(events, cx)).is_err() {
                        break;
                    }
                }
            })
            .detach();
        }

        Self {
            screen: Screen::Onboarding,
            ob_step: 0,
            show_advanced: false,
            provider: ProviderId::Anthropic,
            model: provider(ProviderId::Anthropic).default_model().into(),
            api_key,
            tested: Tested::Idle,
            dir: Dir::Rtl,
            device: Device::Desktop,
            generated: false,
            status: Status::Idle,
            sub_label: SharedString::default(),
            busy: false,
            prompt,
            sel: Vec::new(),
            messages: Vec::new(),
            attachments: Vec::new(),
            active_skills: Vec::new(),
            show_skills: false,
            review_open: false,
            chips: Vec::new(),
            applied_edits: AppliedEdits::default(),
            show_settings: false,
            show_activity: false,
            show_history: false,
            toast: false,
            error_action: None,
            dock_hint: SharedString::default(),
            dock_shake: false,
            pruning: true,
            caching: true,
            heal_attempts: 3,
            history: Vec::new(),
            activity: Vec::new(),
            clock: 634,
            preview,
            pipeline_task: None,
            hint_task: None,
            next_id: 0,
        }
    }

    // ── small helpers ───────────────────────────────────────────────────────
    fn next_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }
    pub fn provider(&self) -> &'static Provider {
        provider(self.provider)
    }
    pub fn prompt_text(&self, cx: &Context<Self>) -> String {
        self.prompt.read(cx).value().to_string()
    }
    fn set_prompt(&self, value: &str, window: &mut Window, cx: &mut Context<Self>) {
        let value = value.to_string();
        self.prompt.update(cx, move |s, cx| s.set_value(value, window, cx));
    }
    pub fn key_text(&self, cx: &Context<Self>) -> String {
        self.api_key.read(cx).value().to_string()
    }
    pub fn key_valid(&self, cx: &Context<Self>) -> bool {
        let k = self.key_text(cx);
        let k = k.trim();
        k.len() >= 10 && !k.chars().any(char::is_whitespace)
    }
    pub fn skill_suffix(&self) -> String {
        let names: Vec<&str> = self.active_skills.iter().map(|id| skill(*id).name).collect();
        if names.is_empty() {
            return String::new();
        }
        let plural = if names.len() > 1 { "s" } else { "" };
        format!(" I also applied your {} skill{plural}.", names.join(" and "))
    }

    // ── chat / activity / history ───────────────────────────────────────────
    fn push_msg(&mut self, role: Role, text: impl Into<SharedString>, tone: Tone, attachments: Vec<SharedString>) {
        self.messages.push(Message { role, tone, text: text.into(), attachments });
    }
    fn send_user(&mut self, text: String, window: &mut Window, cx: &mut Context<Self>) {
        let atts: Vec<SharedString> = self.attachments.iter().map(|a| a.name.clone()).collect();
        self.push_msg(Role::User, text, Tone::Plain, atts);
        self.attachments.clear();
        self.set_prompt("", window, cx);
    }
    fn push_activity(&mut self, tone: Tone, text: impl Into<SharedString>) {
        self.activity.insert(0, ActivityItem { tone, text: text.into() });
        self.activity.truncate(14);
    }
    fn push_checkpoint(&mut self, title: impl Into<SharedString>, edits: AppliedEdits) {
        self.clock += 3;
        for h in &mut self.history {
            h.current = false;
        }
        let time = fmt_clock(self.clock);
        self.history.insert(0, Checkpoint { title: title.into(), time: time.into(), edits, current: true });
    }

    // ── the status pipeline (mock `pipeline`) ───────────────────────────────
    fn run_pipeline(
        &mut self,
        steps: Vec<Step>,
        done: impl FnOnce(&mut StudioApp, &mut Context<StudioApp>) + 'static,
        cx: &mut Context<Self>,
    ) {
        self.pipeline_task = Some(cx.spawn(async move |this, cx| {
            for step in steps {
                let advanced = this
                    .update(cx, |app, cx| {
                        app.status = step.status;
                        app.sub_label = step.label;
                        app.busy = true;
                        if let Some((tone, text)) = step.activity {
                            app.push_activity(tone, text);
                        }
                        cx.notify();
                    })
                    .is_ok();
                if !advanced {
                    return;
                }
                cx.background_executor().timer(Duration::from_millis(step.dur_ms)).await;
            }
            let _ = this.update(cx, |app, cx| {
                app.busy = false;
                done(app, cx);
                cx.notify();
            });
        }));
    }

    // ── prompt submission & generation flows ────────────────────────────────
    pub fn submit_prompt(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        let raw = self.prompt_text(cx);
        let txt = raw.trim().to_string();

        if !self.generated {
            if txt.is_empty() {
                self.flash_hint("Describe your website first \u{2014} e.g. \u{201c}a caf\u{e9} in Cairo\u{201d}", cx);
                return;
            }
            self.send_user(raw, window, cx);
            if is_error_prompt(&txt) {
                self.run_error(ErrorAction::Generate, cx);
            } else {
                self.run_generate(cx);
            }
            return;
        }

        if self.review_open {
            if txt.is_empty() {
                self.flash_hint("Tell me what to adjust in these changes", cx);
                return;
            }
            self.send_user(raw, window, cx);
            self.run_inline(cx);
            return;
        }

        if txt.is_empty() {
            self.flash_hint("Type the change you want to make", cx);
            return;
        }
        self.send_user(raw, window, cx);
        if is_error_prompt(&txt) {
            self.run_error(ErrorAction::Edit, cx);
        } else if is_form_prompt(&txt) {
            self.run_attention(cx);
        } else {
            self.run_edit(cx);
        }
    }

    fn flash_hint(&mut self, msg: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.dock_hint = msg.into();
        self.dock_shake = true;
        self.hint_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(450)).await;
            let _ = this.update(cx, |a, cx| {
                a.dock_shake = false;
                cx.notify();
            });
            cx.background_executor().timer(Duration::from_millis(2350)).await;
            let _ = this.update(cx, |a, cx| {
                a.dock_hint = SharedString::default();
                cx.notify();
            });
        }));
        cx.notify();
    }

    pub fn send_or_cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            self.cancel(cx);
        } else {
            self.submit_prompt(window, cx);
        }
    }

    pub fn cancel(&mut self, cx: &mut Context<Self>) {
        self.pipeline_task = None;
        self.busy = false;
        self.status = if self.generated { Status::Compiled } else { Status::Idle };
        self.sub_label = SharedString::default();
        self.push_activity(Tone::Info, "Cancelled \u{2014} nothing was changed");
        cx.notify();
    }

    fn run_generate(&mut self, cx: &mut Context<Self>) {
        self.clear_selection(cx);
        self.show_settings = false;
        self.show_activity = false;
        self.activity.clear();
        let steps = vec![
            Step::new(Status::Generating, "Understanding your prompt\u{2026}", 1100),
            Step::new(Status::Compiling, "Composing five sections\u{2026}", 1200)
                .log(Tone::Info, "Composed header, hero, menu, about and footer"),
            Step::new(Status::SelfHeal, "Auto-fixing a broken menu link\u{2026}", 1150)
                .log(Tone::Warn, "Auto-fix: a menu link pointed nowhere \u{2014} repaired"),
        ];
        self.run_pipeline(
            steps,
            |app, cx| {
                app.generated = true;
                app.applied_edits = AppliedEdits::default();
                app.status = Status::Compiled;
                app.push_activity(Tone::Ok, "Compiled successfully \u{b7} 1.9s");
                app.push_checkpoint("Created Yasmine Caf\u{e9}", AppliedEdits::default());
                let msg = format!(
                    "Done \u{2014} your site is live with five sections: header, hero, menu, about and footer. I caught and repaired a broken link while compiling. Click any part of the preview to tweak it.{}",
                    app.skill_suffix()
                );
                app.push_msg(Role::Assistant, msg, Tone::Plain, vec![]);
                app.reload_preview(cx);
            },
            cx,
        );
    }

    fn run_edit(&mut self, cx: &mut Context<Self>) {
        let steps = vec![
            Step::new(Status::Generating, "Reading the selected section\u{2026}", 900),
            Step::new(Status::Compiling, "Preparing your changes\u{2026}", 1000),
        ];
        self.run_pipeline(
            steps,
            |app, _cx| {
                app.review_open = true;
                app.show_history = false;
                app.status = Status::Compiled;
                app.chips = vec![
                    app.chip(EditKey::Bigger, ChipKind::Style, "Heading size increased"),
                    app.chip(EditKey::Warm, ChipKind::Style, "Heading colour \u{2192} warm terracotta"),
                    app.chip(EditKey::Tint, ChipKind::Style, "Soft warm background added to the hero"),
                    app.chip(EditKey::Reserve, ChipKind::Text, "Button label \u{2192} \u{201c}Reserve a table\u{201d}"),
                ];
                let msg = format!(
                    "I\u{2019}ve proposed 4 changes to the hero. Keep the ones you like on the right, then apply.{}",
                    app.skill_suffix()
                );
                app.push_msg(Role::Assistant, msg, Tone::Plain, vec![]);
            },
            cx,
        );
    }

    fn run_inline(&mut self, cx: &mut Context<Self>) {
        let steps = vec![Step::new(Status::Compiling, "Folding in your tweak\u{2026}", 900)];
        self.run_pipeline(
            steps,
            |app, _cx| {
                app.status = Status::Compiled;
                let chip = app.chip(EditKey::Sub, ChipKind::Text, "Sub-headline reworded to be more inviting");
                app.chips.push(chip);
                app.push_activity(Tone::Info, "Added a change to the pending proposal");
                app.push_msg(Role::Assistant, "Added that tweak \u{2014} it\u{2019}s in the proposal on the right.", Tone::Plain, vec![]);
            },
            cx,
        );
    }

    fn run_error(&mut self, action: ErrorAction, cx: &mut Context<Self>) {
        self.clear_selection(cx);
        self.show_settings = false;
        self.show_activity = false;
        self.dock_hint = SharedString::default();
        let name = self.provider().name;
        let steps = vec![
            Step::new(Status::Generating, format!("Contacting {name}\u{2026}"), 1000),
            Step::new(Status::Compiling, "Waiting for a response\u{2026}", 1300).log(Tone::Info, format!("Sent request to {name}")),
        ];
        self.run_pipeline(
            steps,
            move |app, _cx| {
                app.status = Status::Error;
                app.error_action = Some(action);
                app.push_activity(Tone::Err, format!("Request to {name} timed out \u{2014} no response received"));
                app.push_msg(
                    Role::Assistant,
                    format!("I couldn\u{2019}t reach {name} \u{2014} the request timed out. Check your connection or API key, then try again. Your project is safe."),
                    Tone::Err,
                    vec![],
                );
            },
            cx,
        );
    }

    pub fn retry(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let action = self.error_action;
        self.status = if self.generated { Status::Compiled } else { Status::Idle };
        self.error_action = None;
        self.set_prompt("", window, cx);
        if action == Some(ErrorAction::Edit) {
            self.run_edit(cx);
        } else {
            self.run_generate(cx);
        }
    }

    fn run_attention(&mut self, cx: &mut Context<Self>) {
        self.clear_selection(cx);
        let n = self.heal_attempts;
        let mut steps = vec![
            Step::new(Status::Generating, "Adding an online order form\u{2026}", 950),
            Step::new(Status::Compiling, "Wiring up the form\u{2026}", 1000).log(Tone::Info, "Attempted to add an order form"),
        ];
        for i in 1..=n {
            steps.push(
                Step::new(Status::SelfHeal, format!("Attempt {i} of {n} \u{2014} retrying\u{2026}"), 800)
                    .log(Tone::Warn, format!("Auto-fix attempt {i} failed: missing payment provider")),
            );
        }
        self.run_pipeline(
            steps,
            move |app, _cx| {
                app.status = Status::Attention;
                app.toast = true;
                app.push_activity(Tone::Warn, "Needs your attention: order form couldn\u{2019}t be wired automatically");
                app.push_msg(
                    Role::Assistant,
                    format!("I tried {n} times but couldn\u{2019}t wire up the order form on my own \u{2014} it needs a payment provider from you. I left your design exactly as it was."),
                    Tone::Warn,
                    vec![],
                );
            },
            cx,
        );
    }

    fn chip(&mut self, key: EditKey, kind: ChipKind, label: impl Into<SharedString>) -> Chip {
        Chip { id: self.next_id(), key, kind, label: label.into(), accepted: true }
    }

    /// Reload the preview webview so it reflects the current applied edits.
    fn reload_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(preview) = &self.preview {
            let url = format!("{ORIGIN}/{}?{}", site::PREVIEW_ENTRY, self.applied_edits.query());
            preview.update(cx, |w, _| w.load_url(&url));
        }
    }

    // ── onboarding ──────────────────────────────────────────────────────────
    pub fn pick_provider(&mut self, id: ProviderId, cx: &mut Context<Self>) {
        self.provider = id;
        self.model = provider(id).default_model().into();
        cx.notify();
    }
    pub fn test_conn(&mut self, cx: &mut Context<Self>) {
        self.tested = if self.key_valid(cx) { Tested::Ok } else { Tested::Fail };
        cx.notify();
    }
    pub fn ob_next(&mut self, cx: &mut Context<Self>) {
        if self.ob_step == 1 && !self.key_valid(cx) {
            self.tested = Tested::Fail;
        } else {
            self.ob_step = (self.ob_step + 1).min(2);
        }
        cx.notify();
    }
    pub fn ob_back(&mut self, cx: &mut Context<Self>) {
        self.ob_step = self.ob_step.saturating_sub(1);
        cx.notify();
    }
    pub fn start_blank(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.screen = Screen::Studio;
        self.set_prompt("", window, cx);
        cx.notify();
    }
    pub fn pick_sample(&mut self, prompt: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.screen = Screen::Studio;
        self.set_prompt(prompt, window, cx);
        cx.notify();
    }
    pub fn start_with(&mut self, prompt: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.send_user(prompt.to_string(), window, cx);
        self.run_generate(cx);
    }

    // ── toolbar ─────────────────────────────────────────────────────────────
    pub fn set_dir(&mut self, dir: Dir, cx: &mut Context<Self>) {
        self.dir = dir;
        self.clear_selection(cx);
        cx.notify();
    }
    pub fn set_device(&mut self, device: Device, cx: &mut Context<Self>) {
        self.device = device;
        self.clear_selection(cx);
        cx.notify();
    }
    fn current_index(&self) -> Option<usize> {
        self.history.iter().position(|h| h.current)
    }
    pub fn can_undo(&self) -> bool {
        matches!(self.current_index(), Some(i) if i + 1 < self.history.len())
    }
    pub fn can_redo(&self) -> bool {
        matches!(self.current_index(), Some(i) if i > 0)
    }
    pub fn undo(&mut self, cx: &mut Context<Self>) {
        if let Some(i) = self.current_index()
            && i + 1 < self.history.len()
        {
            self.restore(i + 1, cx);
        }
    }
    pub fn redo(&mut self, cx: &mut Context<Self>) {
        if let Some(i) = self.current_index()
            && i > 0
        {
            self.restore(i - 1, cx);
        }
    }
    pub fn restore(&mut self, idx: usize, cx: &mut Context<Self>) {
        let Some(cp) = self.history.get(idx).cloned() else { return };
        self.applied_edits = cp.edits;
        self.clear_selection(cx);
        self.review_open = false;
        for (i, h) in self.history.iter_mut().enumerate() {
            h.current = i == idx;
        }
        self.push_activity(Tone::Info, format!("Restored: {}", cp.title));
        self.reload_preview(cx);
        cx.notify();
    }

    // ── IPC bridge ───────────────────────────────────────────────────────────
    fn apply_ipc(&mut self, events: Vec<ipc::Event>, cx: &mut Context<Self>) {
        for event in events {
            match event {
                ipc::Event::Select(Some(key)) => self.select_section(key, cx),
                ipc::Event::Select(None) => self.clear_sel(cx),
            }
        }
    }

    // ── canvas selection & quick inspector ──────────────────────────────────
    /// Toggle one section's membership in the (possibly multi-item) selection.
    pub fn select_section(&mut self, key: &'static str, cx: &mut Context<Self>) {
        if !(self.generated && !self.review_open && !self.busy) {
            return;
        }
        if let Some(pos) = self.sel.iter().position(|s| s.key == key) {
            self.sel.remove(pos);
        } else {
            self.sel.push(Selection { key });
        }
        self.sync_preview_selection(cx);
        cx.notify();
    }
    /// Drop one section from the selection (e.g. its chip's "×" in the chat
    /// dock) and clear its highlight in the preview to match.
    pub fn remove_selection(&mut self, key: &'static str, cx: &mut Context<Self>) {
        self.sel.retain(|s| s.key != key);
        self.sync_preview_selection(cx);
        cx.notify();
    }
    pub fn clear_sel(&mut self, cx: &mut Context<Self>) {
        self.clear_selection(cx);
        cx.notify();
    }
    /// Clear every selection and keep the preview's highlights in sync.
    fn clear_selection(&mut self, cx: &mut Context<Self>) {
        self.sel.clear();
        self.sync_preview_selection(cx);
    }
    /// Re-paint the preview's inline selection outlines to match `self.sel`
    /// exactly, so the two never drift apart regardless of which side (a
    /// preview click, or a chip removed in chat) changed the selection.
    fn sync_preview_selection(&mut self, cx: &mut Context<Self>) {
        let Some(preview) = &self.preview else { return };
        let keys: Vec<&str> = self.sel.iter().map(|s| s.key).collect();
        let keys_json = serde_json::to_string(&keys).unwrap_or_else(|_| "[]".into());
        let js = format!(
            "(function(keys){{document.querySelectorAll('[data-wf-el]').forEach(function(el){{\
             if(keys.indexOf(el.getAttribute('data-wf-el'))!==-1){{el.style.outline='2px solid #e2725b';el.style.outlineOffset='-2px';}}\
             else{{el.style.outline='';el.style.outlineOffset='';}}}});}})({keys_json});"
        );
        preview.update(cx, |w, _| {
            let _ = w.raw().evaluate_script(&js);
        });
    }
    pub fn pick_try(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.send_user(text.to_string(), window, cx);
        if is_form_prompt(text) {
            self.run_attention(cx);
        } else {
            self.run_edit(cx);
        }
    }
    fn apply_quick(&mut self, key: EditKey, on: bool, title: &'static str, cx: &mut Context<Self>) {
        self.applied_edits.set(key, on);
        self.push_activity(Tone::Ok, format!("Quick edit \u{b7} {title}"));
        self.push_checkpoint(title, self.applied_edits);
        self.reload_preview(cx);
        cx.notify();
    }
    pub fn insp_warm(&mut self, cx: &mut Context<Self>) {
        self.apply_quick(EditKey::Warm, true, "Heading colour \u{2192} terracotta", cx);
    }
    pub fn insp_dark(&mut self, cx: &mut Context<Self>) {
        self.apply_quick(EditKey::Warm, false, "Heading colour \u{2192} dark", cx);
    }
    pub fn insp_bigger(&mut self, cx: &mut Context<Self>) {
        self.apply_quick(EditKey::Bigger, true, "Heading size increased", cx);
    }
    pub fn insp_smaller(&mut self, cx: &mut Context<Self>) {
        self.apply_quick(EditKey::Bigger, false, "Heading size reduced", cx);
    }

    // ── review ──────────────────────────────────────────────────────────────
    pub fn toggle_chip(&mut self, id: u64, cx: &mut Context<Self>) {
        if let Some(c) = self.chips.iter_mut().find(|c| c.id == id) {
            c.accepted = !c.accepted;
            cx.notify();
        }
    }
    pub fn keep_all(&mut self, cx: &mut Context<Self>) {
        for c in &mut self.chips {
            c.accepted = true;
        }
        cx.notify();
    }
    pub fn clear_all(&mut self, cx: &mut Context<Self>) {
        for c in &mut self.chips {
            c.accepted = false;
        }
        cx.notify();
    }
    pub fn accepted_count(&self) -> usize {
        self.chips.iter().filter(|c| c.accepted).count()
    }
    pub fn apply_accepted(&mut self, cx: &mut Context<Self>) {
        let accepted: Vec<EditKey> = self.chips.iter().filter(|c| c.accepted).map(|c| c.key).collect();
        if accepted.is_empty() {
            return;
        }
        for key in &accepted {
            self.applied_edits.set(*key, true);
        }
        let n = accepted.len();
        self.review_open = false;
        self.chips.clear();
        self.clear_selection(cx);
        self.status = Status::Compiled;
        let plural = if n > 1 { "s" } else { "" };
        self.push_activity(Tone::Ok, format!("Applied {n} change{plural} to the hero"));
        self.push_checkpoint("Warmer, bolder hero", self.applied_edits);
        self.push_msg(Role::Assistant, format!("Applied {n} change{plural} and saved a checkpoint to your history."), Tone::Plain, vec![]);
        self.reload_preview(cx);
        cx.notify();
    }
    pub fn reset_proposal(&mut self, cx: &mut Context<Self>) {
        self.review_open = false;
        self.chips.clear();
        self.status = Status::Compiled;
        self.push_msg(Role::Assistant, "Discarded that proposal \u{2014} nothing changed.", Tone::Plain, vec![]);
        cx.notify();
    }

    // ── menus & settings ────────────────────────────────────────────────────
    pub fn toggle_settings(&mut self, cx: &mut Context<Self>) {
        self.show_settings = !self.show_settings;
        self.show_activity = false;
        cx.notify();
    }
    pub fn toggle_activity(&mut self, cx: &mut Context<Self>) {
        self.show_activity = !self.show_activity;
        self.show_settings = false;
        cx.notify();
    }
    pub fn toggle_history(&mut self, cx: &mut Context<Self>) {
        self.show_history = !self.show_history && !self.review_open;
        cx.notify();
    }
    pub fn toggle_advanced(&mut self, cx: &mut Context<Self>) {
        self.show_advanced = !self.show_advanced;
        cx.notify();
    }
    pub fn set_pruning(&mut self, cx: &mut Context<Self>) {
        self.pruning = !self.pruning;
        cx.notify();
    }
    pub fn set_caching(&mut self, cx: &mut Context<Self>) {
        self.caching = !self.caching;
        cx.notify();
    }
    pub fn set_heal_attempts(&mut self, n: u8, cx: &mut Context<Self>) {
        self.heal_attempts = n;
        cx.notify();
    }
    pub fn close_menus(&mut self, cx: &mut Context<Self>) {
        self.show_settings = false;
        self.show_activity = false;
        cx.notify();
    }
    pub fn set_model(&mut self, model: &str, cx: &mut Context<Self>) {
        self.model = model.to_string().into();
        cx.notify();
    }
    pub fn open_settings(&mut self, cx: &mut Context<Self>) {
        self.show_settings = true;
        self.show_activity = false;
        self.status = if self.generated { Status::Compiled } else { Status::Idle };
        self.error_action = None;
        cx.notify();
    }

    // ── skills ──────────────────────────────────────────────────────────────
    pub fn toggle_skills_menu(&mut self, cx: &mut Context<Self>) {
        self.show_skills = !self.show_skills;
        self.show_settings = false;
        self.show_activity = false;
        cx.notify();
    }
    pub fn close_skills(&mut self, cx: &mut Context<Self>) {
        self.show_skills = false;
        cx.notify();
    }
    pub fn toggle_skill(&mut self, id: SkillId, cx: &mut Context<Self>) {
        if let Some(pos) = self.active_skills.iter().position(|s| *s == id) {
            self.active_skills.remove(pos);
        } else {
            self.active_skills.push(id);
        }
        cx.notify();
    }
    pub fn remove_skill(&mut self, id: SkillId, cx: &mut Context<Self>) {
        self.active_skills.retain(|s| *s != id);
        cx.notify();
    }

    // ── attachments (native file picker) ────────────────────────────────────
    pub fn trigger_file(&mut self, cx: &mut Context<Self>) {
        let rx = cx.prompt_for_paths(PathPromptOptions { files: true, directories: false, multiple: true, prompt: None });
        cx.spawn(async move |this, cx| {
            if let Ok(Ok(Some(paths))) = rx.await {
                let _ = this.update(cx, |app, cx| {
                    for path in paths {
                        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "attachment".into());
                        let id = app.next_id();
                        app.attachments.push(Attachment { id, name: name.into() });
                    }
                    cx.notify();
                });
            }
        })
        .detach();
    }
    pub fn remove_attachment(&mut self, id: u64, cx: &mut Context<Self>) {
        self.attachments.retain(|a| a.id != id);
        cx.notify();
    }
    pub fn dismiss_toast(&mut self, cx: &mut Context<Self>) {
        self.toast = false;
        self.status = Status::Compiled;
        cx.notify();
    }
}

impl Render for StudioApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        ui::render(self, window, cx)
    }
}

// ── the preview webview ─────────────────────────────────────────────────────
#[cfg(target_os = "linux")]
fn pump_gtk() {
    while gtk::events_pending() {
        gtk::main_iteration_do(false);
    }
}

/// Build the preview webview as a child of the gpui window, serving `CafeSite`.
fn build_preview(window: &mut Window) -> anyhow::Result<wry::WebView> {
    let builder = WebViewBuilder::new()
        .with_custom_protocol("wf".into(), |_id, request| serve(request))
        .with_initialization_script(boot_script())
        .with_ipc_handler(|request: Request<String>| ipc::on_message(request.into_body()))
        .with_url(format!("{ORIGIN}/{}", site::PREVIEW_ENTRY));

    #[cfg(target_os = "linux")]
    {
        let _ = window; // gpui's X11 window_handle() is unimplemented; we find it ourselves.
        let xid = find_gpui_window().context("locating the gpui X11 window")?;
        let webview = builder
            .build_as_child(&X11Parent(xid))
            .context("embedding the preview webview into the gpui X11 window")?;
        eprintln!("wf-studio: preview webview embedded into window 0x{xid:x}");
        Ok(webview)
    }
    #[cfg(not(target_os = "linux"))]
    {
        use raw_window_handle::HasWindowHandle;
        let handle = window.window_handle().context("gpui window has no raw handle")?;
        builder.build_as_child(&handle).context("embedding the preview webview")
    }
}

#[cfg(target_os = "linux")]
struct X11Parent(u32);

#[cfg(target_os = "linux")]
impl raw_window_handle::HasWindowHandle for X11Parent {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        let handle = raw_window_handle::XlibWindowHandle::new(self.0 as core::ffi::c_ulong);
        let raw = raw_window_handle::RawWindowHandle::Xlib(handle);
        // Safe: the window id belongs to our own live gpui window.
        Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(raw) })
    }
}

/// Find our own top-level window by matching `_NET_WM_PID` to this process.
#[cfg(target_os = "linux")]
fn find_gpui_window() -> anyhow::Result<u32> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{AtomEnum, ConnectionExt};

    let (conn, screen) = x11rb::connect(None).context("connect to X server")?;
    let root = conn.setup().roots[screen].root;
    let pid_atom = conn.intern_atom(false, b"_NET_WM_PID")?.reply()?.atom;
    let my_pid = std::process::id();

    let mut stack = vec![root];
    let mut visited = 0u32;
    while let Some(win) = stack.pop() {
        visited += 1;
        if visited > 4096 {
            break;
        }
        if win != root
            && let Ok(reply) = conn.get_property(false, win, pid_atom, AtomEnum::CARDINAL, 0, 1)?.reply()
            && reply.value32().and_then(|mut v| v.next()) == Some(my_pid)
        {
            return Ok(win);
        }
        if let Ok(tree) = conn.query_tree(win)?.reply() {
            stack.extend(tree.children);
        }
    }
    anyhow::bail!("no window found with _NET_WM_PID = {my_pid}")
}

fn serve(request: Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    match site::resource(request.uri().path()) {
        Some((mime, bytes)) => Response::builder().status(200).header(CONTENT_TYPE, mime).body(Cow::Borrowed(bytes)).unwrap(),
        None => Response::builder().status(404).header(CONTENT_TYPE, "text/plain").body(Cow::Borrowed(b"not found".as_slice())).unwrap(),
    }
}

fn boot_script() -> String {
    format!(
        r#"
window.__resources = {{
  "https://unpkg.com/react@18.3.1/umd/react.production.min.js": "{ORIGIN}/vendor/react.js",
  "https://unpkg.com/react-dom@18.3.1/umd/react-dom.production.min.js": "{ORIGIN}/vendor/react-dom.js"
}};
{bridge}
"#,
        bridge = ipc::BRIDGE_JS
    )
}

// ── prompt intent detection (mock `ERR` / `FORM` regexes) ───────────────────
fn contains_any(hay: &str, needles: &[&str]) -> bool {
    let low = hay.to_lowercase();
    needles.iter().any(|n| low.contains(*n))
}
fn is_error_prompt(text: &str) -> bool {
    contains_any(text, &["offline", "timeout", "no internet", "network", "crash", "\u{641}\u{634}\u{644}", "\u{627}\u{646}\u{642}\u{637}\u{627}\u{639}"])
}
fn is_form_prompt(text: &str) -> bool {
    contains_any(text, &["form", "order", "checkout", "payment", "\u{62f}\u{641}\u{639}", "\u{637}\u{644}\u{628}"])
}
