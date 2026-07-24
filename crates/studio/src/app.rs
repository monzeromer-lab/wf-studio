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

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use webfluent::CompiledSite;

use anyhow::Context as _;
use gpui::{Context, Entity, SharedString, Task, Window, prelude::*};
use gpui_component::input::{InputEvent, InputState};
use gpui_component::webview::WebView;
use wry::{WebViewBuilder, http::Request};

use crate::state::*;
use crate::{ipc, site, ui};

/// The custom-protocol origin for the preview webview.
const ORIGIN: &str = "wf://localhost";

pub struct StudioApp {
    pub screen: Screen,
    pub ob_step: u8,
    pub provider: ProviderId,
    pub api_key: Entity<InputState>,
    /// BYO-key store (OS keychain → env): restores the selected provider's key
    /// into `api_key` on startup and persists a key when generation uses it.
    keys: Box<dyn wf_ai::KeyStore>,
    pub dir: Dir,
    pub device: Device,
    pub generated: bool,
    pub status: Status,
    pub busy: bool,
    pub prompt: Entity<InputState>,
    pub messages: Vec<Message>,
    pub review_open: bool,
    /// The pending AI edit under review (real chips), the file it edits, and the
    /// node it targets (for inline re-prompt, FR-8).
    proposal: Option<wf_core::Proposal>,
    proposal_file: Option<String>,
    proposal_node: Option<String>,
    /// The source we last attempted to self-heal (attempt each version once).
    heal_tried: Option<String>,
    pub pruning: bool,
    pub caching: bool,
    pub heal_attempts: u8,
    // ── shell: auth, home, projects, modals (cinematic redesign) ────────────
    pub login_email: Entity<InputState>,
    pub login_pw: Entity<InputState>,
    pub login_busy: bool,
    pub home_filter: HomeFilter,
    pub projects: Vec<Project>,
    pub current_project: Option<SharedString>,
    pub new_type: ProjectKind,
    pub modal: Option<Modal>,
    pub conn_mode: ConnMode,
    pub acp_url: Entity<InputState>,
    pub acp_connected: bool,
    // ── workspace: composer, selection, inspector, review, blocks ───────────
    pub chat_open: bool,
    pub panel_open: bool,
    pub chat_menu: Option<ChatMenu>,
    pub ds_picker_open: bool,
    pub api_panel_open: bool,
    pub chat_model: SharedString,
    pub effort: Effort,
    pub permission: Permission,
    pub skills: Vec<usize>,
    pub api_spec: Option<ApiSpec>,
    pub spa_mode: bool,
    pub applied_ds: Option<SharedString>,
    pub pending_ds: Option<SharedString>,
    pub selection: Vec<SharedString>,
    /// Real preview selection: the `data-wf-node` ids clicked in the compiled
    /// site (resolve via `project.resolve_node`). Replaces the mock `selection`
    /// as M3 wires the inspector to real nodes.
    pub sel_nodes: Vec<SharedString>,
    pub edits: HashMap<SharedString, ElEdit>,
    pub added_blocks: Vec<BlockType>,
    pub event_order: [usize; 3],
    pub review_split: f32,
    // ── design-system workspace: tabs, tokens, live specimens ───────────────
    pub ds_tab: DsTab,
    pub ds_sel: Option<DsSel>,
    pub ds_rtl: bool,
    pub ds_colors: Vec<DsColorToken>,
    pub ds_types: Vec<DsTypeToken>,
    pub ds_demo: DsDemo,
    // ── modals: publish, settings, share, mcp, toast ────────────────────────
    pub publish_tab: PublishTab,
    pub deploying: bool,
    pub published: bool,
    pub export_kind: ExportKind,
    pub settings_tab: SettingsTab,
    pub share_role: ShareRole,
    pub link_access: LinkAccess,
    pub collab_mk: ShareRole,
    pub collab_ah: ShareRole,
    pub share_menu: Option<ShareMenu>,
    pub mcp_list: Vec<McpServer>,
    pub mcp_name: Entity<InputState>,
    pub mcp_cmd: Entity<InputState>,
    mcp_next_id: u64,
    pub toast_note: Option<Toast>,
    toast_task: Option<Task<()>>,
    /// The embedded website-preview webview (`None` if the embed failed).
    pub preview: Option<Entity<WebView>>,
    /// The live WebFluent project: `.wf` sources + the latest compile.
    project: wf_core::WfProject,
    /// Source-snapshot history for undo/redo/restore (FR-14).
    history: wf_core::History,
    /// The output the `wf://` protocol serves, shared with the serve closure.
    /// Swapped on recompile, then the webview reloads to pick it up.
    output: Arc<RwLock<CompiledSite>>,
    pipeline_task: Option<Task<()>>,
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
        let keys: Box<dyn wf_ai::KeyStore> = Box::new(wf_ai::default_key_store());
        let login_email = cx.new(|cx| InputState::new(window, cx).default_value("rana@studio.sa").placeholder("you@email.com"));
        let login_pw = cx.new(|cx| InputState::new(window, cx).masked(true).placeholder("Password"));
        let acp_url = cx.new(|cx| InputState::new(window, cx).placeholder("wss://localhost:4000  \u{b7}  or:  npx my-agent --acp"));
        let mcp_name = cx.new(|cx| InputState::new(window, cx).placeholder("Name"));
        let mcp_cmd = cx.new(|cx| InputState::new(window, cx).placeholder("Command or URL"));

        cx.subscribe_in(&prompt, window, |this, _, event: &InputEvent, window, cx| {
            if let InputEvent::PressEnter { secondary: false } = event {
                if this.screen == Screen::DsWorkspace {
                    this.ds_send(window, cx);
                } else {
                    this.send_prompt(window, cx);
                }
            }
        })
        .detach();

        // Seed the in-memory WebFluent project, compile it, and publish the result
        // to the shared handle the `wf://` serve closure reads.
        let project = wf_core::WfProject::seed();
        match project.error() {
            Some(err) => eprintln!("wf-studio: seed project failed to compile: {err}"),
            None => eprintln!(
                "wf-studio: seed project compiled ({} page(s), {} nodes)",
                project.compiled().pages.len(),
                project.compiled().node_map.len(),
            ),
        }
        let mut history = wf_core::History::new();
        history.checkpoint("Started your project", project.snapshot());
        let output = Arc::new(RwLock::new(project.compiled().clone()));

        // Build the preview webview as a child of the gpui window and keep it
        // hidden until a site is generated.
        let preview = match build_preview(window, output.clone()) {
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

        // Restore the default provider's saved key (OS keychain → env fallback).
        if let Some(k) = keys.get(wf_ai::ProviderKind::Anthropic) {
            api_key.update(cx, |s, cx| s.set_value(k, window, cx));
        }

        Self {
            screen: Screen::Login,
            ob_step: 0,
            provider: ProviderId::Anthropic,
            api_key,
            keys,
            dir: Dir::Rtl,
            device: Device::Desktop,
            generated: false,
            status: Status::Idle,
            busy: false,
            prompt,
            messages: Vec::new(),
            review_open: false,
            proposal: None,
            proposal_file: None,
            proposal_node: None,
            heal_tried: None,
            pruning: true,
            caching: true,
            heal_attempts: 3,
            login_email,
            login_pw,
            login_busy: false,
            home_filter: HomeFilter::All,
            projects: seed_projects(),
            current_project: None,
            new_type: ProjectKind::Website,
            modal: None,
            conn_mode: ConnMode::Key,
            acp_url,
            acp_connected: false,
            chat_open: true,
            panel_open: true,
            chat_menu: None,
            ds_picker_open: false,
            api_panel_open: false,
            chat_model: "sonnet".into(),
            effort: Effort::Balanced,
            permission: Permission::Review,
            skills: vec![0],
            api_spec: None,
            spa_mode: true,
            applied_ds: Some("ds1".into()),
            pending_ds: None,
            selection: Vec::new(),
            sel_nodes: Vec::new(),
            edits: HashMap::new(),
            added_blocks: Vec::new(),
            event_order: [0, 1, 2],
            review_split: 50.0,
            ds_tab: DsTab::Foundations,
            ds_sel: None,
            ds_rtl: false,
            ds_colors: ds_color_tokens(),
            ds_types: ds_type_tokens(),
            ds_demo: DsDemo::default(),
            publish_tab: PublishTab::Deploy,
            deploying: false,
            published: false,
            export_kind: ExportKind::Static,
            settings_tab: SettingsTab::Providers,
            share_role: ShareRole::Edit,
            link_access: LinkAccess::Restricted,
            collab_mk: ShareRole::Edit,
            collab_ah: ShareRole::View,
            share_menu: None,
            mcp_list: seed_mcp(),
            mcp_name,
            mcp_cmd,
            mcp_next_id: 4,
            toast_note: None,
            toast_task: None,
            preview,
            project,
            history,
            output,
            pipeline_task: None,
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
    /// The selected provider as the AI-crate kind.
    fn provider_kind(&self) -> wf_ai::ProviderKind {
        match self.provider {
            ProviderId::Anthropic => wf_ai::ProviderKind::Anthropic,
            ProviderId::OpenAI => wf_ai::ProviderKind::OpenAi,
            ProviderId::Gemini => wf_ai::ProviderKind::Gemini,
            ProviderId::DeepSeek => wf_ai::ProviderKind::DeepSeek,
            ProviderId::Kimi => wf_ai::ProviderKind::Kimi,
            ProviderId::Glm => wf_ai::ProviderKind::Glm,
        }
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
    /// Persist the current key-field value to the key store, under the selected
    /// provider. Called when generation uses the key, so a pasted key survives a
    /// restart without an explicit save step.
    fn save_current_key(&self, cx: &Context<Self>) {
        let key = self.key_text(cx);
        let key = key.trim();
        if key.is_empty() {
            return;
        }
        if let Err(e) = self.keys.set(self.provider_kind(), key) {
            eprintln!("wf-studio: could not save API key to the keychain: {e}");
        }
    }
    // ── chat / activity / history ───────────────────────────────────────────
    fn push_msg(&mut self, role: Role, text: impl Into<SharedString>) {
        self.messages.push(Message { role, text: text.into() });
    }

    /// Push the current canvas state into the (persistent) preview page via
    /// Recompile the project, publish the new output to the shared serve handle,
    /// and reload the preview so it shows the result. The AI / inspector edit
    /// flows drive this in M3; for now it is the manual recompile hook.
    #[allow(dead_code)] // wired to edit triggers in M3
    pub fn recompile_and_reload(&mut self, cx: &mut Context<Self>) {
        self.project.recompile();
        if let Ok(mut out) = self.output.write() {
            *out = self.project.compiled().clone();
        }
        if let Some(preview) = &self.preview {
            preview.update(cx, |w, _| {
                let _ = w.raw().load_url(&format!("{ORIGIN}/{}", site::PREVIEW_ENTRY));
            });
        }
    }

    /// A preview element was clicked (its `data-wf-node` id). Resolve it to code,
    /// remember the selection, and highlight it in the preview.
    pub fn select_node(&mut self, node_id: impl Into<SharedString>, additive: bool, cx: &mut Context<Self>) {
        let id = node_id.into();
        if additive {
            if let Some(pos) = self.sel_nodes.iter().position(|k| *k == id) {
                self.sel_nodes.remove(pos);
            } else {
                self.sel_nodes.push(id.clone());
            }
        } else {
            self.sel_nodes = vec![id.clone()];
        }
        match self.project.resolve_node(&id) {
            Some(r) => eprintln!(
                "wf-studio: selected {id} in {} [{}] -> {}",
                r.info.component,
                r.file,
                r.source_slice.lines().next().unwrap_or("")
            ),
            None => eprintln!("wf-studio: selected {id} (not in node map)"),
        }
        // Mirror into the mock `selection` so the reused inspector panel renders
        // for the real node (its controls now emit EditOps — see `set_*`).
        self.selection = self.sel_nodes.clone();
        self.highlight_nodes(cx);
        cx.notify();
    }

    /// Clear the preview node selection.
    pub fn deselect_node(&mut self, cx: &mut Context<Self>) {
        if self.sel_nodes.is_empty() {
            return;
        }
        self.sel_nodes.clear();
        self.selection.clear();
        self.highlight_nodes(cx);
        cx.notify();
    }

    /// The page element tree for the outline panel, derived from the live compile.
    pub fn outline(&self) -> Vec<wf_core::OutlineNode> {
        self.project.outline()
    }

    /// Whether a preview node id is in the current selection (drives outline row highlight).
    pub fn node_selected(&self, id: &str) -> bool {
        self.sel_nodes.iter().any(|s| s.as_ref() == id)
    }

    /// The leading token of the selected element's source — its element type.
    fn selected_element_name(&self) -> Option<String> {
        let id = self.sel_nodes.first()?;
        let r = self.project.resolve_node(id)?;
        let name = r
            .source_slice
            .split(|c: char| c == '(' || c == ' ' || c == '{' || c == '\n')
            .next()
            .unwrap_or("")
            .to_string();
        (!name.is_empty()).then_some(name)
    }

    /// One-tap "try it" edit suggestions for the selected element (FR-9):
    /// `(label, instruction)`.
    pub fn try_it_suggestions(&self) -> Vec<(SharedString, SharedString)> {
        let Some(name) = self.selected_element_name() else { return Vec::new() };
        crate::state::try_it_suggestions(&name)
            .iter()
            .map(|(l, i)| (SharedString::from(*l), SharedString::from(*i)))
            .collect()
    }

    /// Outline the currently-selected nodes in the preview via `evaluate_script`.
    fn highlight_nodes(&self, cx: &mut Context<Self>) {
        let Some(preview) = &self.preview else { return };
        // JS: clear prior outlines, then outline each selected `data-wf-node`.
        const HIGHLIGHT_JS: &str = "(function(){\
          document.querySelectorAll('[data-wf-sel]').forEach(function(e){e.removeAttribute('data-wf-sel');e.style.outline='';e.style.outlineOffset='';});\
          __IDS__.forEach(function(id){\
            var el=document.querySelector('[data-wf-node=\"'+id+'\"]');\
            if(el){el.setAttribute('data-wf-sel','');el.style.outline='2px solid #7C5CFF';el.style.outlineOffset='1px';}\
          });\
        })();";
        let ids: Vec<&str> = self.sel_nodes.iter().map(|s| s.as_ref()).collect();
        let list = serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string());
        let js = HIGHLIGHT_JS.replace("__IDS__", &list);
        preview.update(cx, |w, _| {
            let _ = w.raw().evaluate_script(&js);
        });
    }

    /// `window.__wfApply(...)` — dir, device, selection, live edits, the review
    /// wipe, and any added blocks. No page reload, so inspector tweaks are live.
    pub fn sync_preview(&self, cx: &mut Context<Self>) {
        let Some(preview) = &self.preview else { return };
        let mut edits = serde_json::Map::new();
        for (k, e) in &self.edits {
            let mut o = serde_json::Map::new();
            if let Some(c) = &e.color {
                o.insert("color".into(), serde_json::json!(c.as_ref()));
            }
            if let Some(s) = e.size {
                o.insert("size".into(), serde_json::json!(s));
            }
            if let Some(w) = e.weight {
                o.insert("weight".into(), serde_json::json!(w));
            }
            if let Some(a) = e.align {
                o.insert("align".into(), serde_json::json!(a.value()));
            }
            if let Some(b) = &e.bg {
                o.insert("bg".into(), serde_json::json!(b.as_ref()));
            }
            if let Some(r) = e.radius {
                o.insert("radius".into(), serde_json::json!(r));
            }
            edits.insert(k.to_string(), serde_json::Value::Object(o));
        }
        let blocks: Vec<&str> = self
            .added_blocks
            .iter()
            .map(|b| match b {
                BlockType::Text => "text",
                BlockType::Image => "image",
                BlockType::Button => "button",
            })
            .collect();
        let state = serde_json::json!({
            "dir": if self.dir == Dir::Rtl { "rtl" } else { "ltr" },
            "device": match self.device { Device::Desktop => "desktop", Device::Tablet => "tablet", Device::Mobile => "mobile" },
            "review": self.review_open,
            "reviewSplit": self.review_split,
            "selection": self.selection.iter().map(|s| s.as_ref()).collect::<Vec<&str>>(),
            "addedBlocks": blocks,
            "eventOrder": self.event_order,
            "edits": serde_json::Value::Object(edits),
        });
        let js = format!("window.__wfApply && window.__wfApply({state});");
        preview.update(cx, |w, _| {
            let _ = w.raw().evaluate_script(&js);
        });
    }

    // ── onboarding ──────────────────────────────────────────────────────────
    pub fn pick_provider(&mut self, id: ProviderId, window: &mut Window, cx: &mut Context<Self>) {
        // Persist the outgoing provider's key, switch, then restore the incoming
        // provider's saved key into the field (empty if none).
        self.save_current_key(cx);
        self.provider = id;
        let loaded = self.keys.get(self.provider_kind()).unwrap_or_default();
        self.api_key.update(cx, |s, cx| s.set_value(loaded, window, cx));
        cx.notify();
    }

    // ── shell: auth, home, projects, modals (cinematic redesign) ─────────────
    pub fn login_ready(&self, cx: &Context<Self>) -> bool {
        self.login_email.read(cx).value().trim().len() > 3 && !self.login_pw.read(cx).value().is_empty()
    }
    pub fn sign_in(&mut self, cx: &mut Context<Self>) {
        if self.login_busy {
            return;
        }
        self.login_busy = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(1100)).await;
            let _ = this.update(cx, |a, cx| {
                a.login_busy = false;
                a.screen = Screen::Home;
                cx.notify();
            });
        })
        .detach();
    }
    pub fn sso_sign_in(&mut self, cx: &mut Context<Self>) {
        self.screen = Screen::Home;
        cx.notify();
    }
    pub fn sign_out(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.screen = Screen::Login;
        self.modal = None;
        self.current_project = None;
        self.login_pw.update(cx, |s, cx| s.set_value("", window, cx));
        cx.notify();
    }
    pub fn set_home_filter(&mut self, f: HomeFilter, cx: &mut Context<Self>) {
        self.home_filter = f;
        cx.notify();
    }
    pub fn current_project(&self) -> Option<&Project> {
        let id = self.current_project.as_ref()?;
        self.projects.iter().find(|p| &p.id == id)
    }
    pub fn open_project(&mut self, id: SharedString, cx: &mut Context<Self>) {
        let Some(p) = self.projects.iter().find(|p| p.id == id).cloned() else { return };
        self.current_project = Some(p.id.clone());
        self.modal = None;
        match p.kind {
            ProjectKind::System => {
                self.screen = Screen::DsWorkspace;
                self.messages.clear();
                self.push_msg(
                    Role::Assistant,
                    "This is your design system. Tell me to add a token, restyle a component, or generate a new one \u{2014} I\u{2019}ll apply it across the kit.",
                );
            }
            ProjectKind::Website => {
                let built = p.id.as_ref() == "p1";
                self.screen = Screen::Workspace;
                self.generated = built;
                self.review_open = false;
                self.messages.clear();
                if built {
                    self.status = Status::Compiled;
                    self.push_msg(
                        Role::Assistant,
                        "Welcome back \u{2014} your site is live. Ask for a change, or click any element to tweak it.",
                    );
                } else {
                    self.status = Status::Idle;
                }
                self.sync_preview(cx);
            }
        }
        cx.notify();
    }
    pub fn new_project(&mut self, cx: &mut Context<Self>) {
        self.open_modal(Modal::NewProject, cx);
    }
    pub fn set_new_type(&mut self, kind: ProjectKind, cx: &mut Context<Self>) {
        self.new_type = kind;
        cx.notify();
    }
    pub fn create_project(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.modal = None;
        match self.new_type {
            ProjectKind::System => {
                self.screen = Screen::DsWorkspace;
                self.current_project = Some("new".into());
                self.messages.clear();
                self.push_msg(
                    Role::Assistant,
                    "Let\u{2019}s build your design system. Describe your brand \u{2014} colors, type, feel \u{2014} and I\u{2019}ll generate a starter kit of tokens and components.",
                );
            }
            ProjectKind::Website => {
                self.ob_step = 0;
                self.generated = false;
                self.review_open = false;
                self.messages.clear();
                self.set_prompt("", window, cx);
                self.screen = Screen::Onboarding;
            }
        }
        cx.notify();
    }
    pub fn request_exit(&mut self, cx: &mut Context<Self>) {
        self.open_modal(Modal::Exit, cx);
    }
    pub fn confirm_exit(&mut self, cx: &mut Context<Self>) {
        self.screen = Screen::Home;
        self.modal = None;
        self.current_project = None;
        cx.notify();
    }
    pub fn open_modal(&mut self, m: Modal, cx: &mut Context<Self>) {
        self.modal = Some(m);
        cx.notify();
    }
    pub fn close_modal(&mut self, cx: &mut Context<Self>) {
        self.modal = None;
        self.share_menu = None;
        cx.notify();
    }
    pub fn open_profile(&mut self, cx: &mut Context<Self>) {
        self.open_modal(Modal::Profile, cx);
    }

    // ── onboarding (cinematic) ───────────────────────────────────────────────
    pub fn set_conn_mode(&mut self, m: ConnMode, cx: &mut Context<Self>) {
        self.conn_mode = m;
        cx.notify();
    }
    pub fn goto_step(&mut self, i: u8, cx: &mut Context<Self>) {
        if i <= self.ob_step {
            self.ob_step = i;
            cx.notify();
        }
    }
    pub fn next_step(&mut self, cx: &mut Context<Self>) {
        self.ob_step = (self.ob_step + 1).min(2);
        cx.notify();
    }
    pub fn prev_step(&mut self, cx: &mut Context<Self>) {
        self.ob_step = self.ob_step.saturating_sub(1);
        cx.notify();
    }
    pub fn enter_workspace(&mut self, prompt: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.screen = Screen::Workspace;
        self.generated = false;
        self.status = Status::Idle;
        self.review_open = false;
        self.messages.clear();
        self.set_prompt(prompt, window, cx);
        cx.notify();
    }
    pub fn skip_onboarding(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.enter_workspace("", window, cx);
    }
    pub fn pick_starter(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let preset = match id {
            "venue" => "A rooftop live-music venue in Riyadh \u{2014} Arabic-first, dark and cinematic, with a lineup and table booking.",
            "cafe" => "A cozy caf\u{e9} landing page with a menu, hours and a location map.",
            "portfolio" => "A photographer\u{2019}s portfolio with a hero, gallery grid and contact.",
            "product" => "A launch page for a new product with a hero, features and pricing.",
            _ => "",
        };
        self.enter_workspace(preset, window, cx);
    }

    // ── toolbar ─────────────────────────────────────────────────────────────
    pub fn set_dir(&mut self, dir: Dir, cx: &mut Context<Self>) {
        self.dir = dir;
        self.sync_preview(cx);
        cx.notify();
    }
    pub fn set_device(&mut self, device: Device, cx: &mut Context<Self>) {
        self.device = device;
        self.sync_preview(cx);
        cx.notify();
    }
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }
    pub fn undo(&mut self, cx: &mut Context<Self>) {
        let Some(sources) = self.history.undo().map(|r| r.sources.clone()) else { return };
        self.project.restore_sources(sources);
        self.recompile_and_reload(cx);
        cx.notify();
    }
    pub fn redo(&mut self, cx: &mut Context<Self>) {
        let Some(sources) = self.history.redo().map(|r| r.sources.clone()) else { return };
        self.project.restore_sources(sources);
        self.recompile_and_reload(cx);
        cx.notify();
    }
    /// History revisions for the version panel: `(summary, is_current)`, oldest first.
    pub fn history_entries(&self) -> Vec<(SharedString, bool)> {
        let cur = self.history.cursor();
        self.history
            .entries()
            .iter()
            .enumerate()
            .map(|(i, r)| (SharedString::from(r.summary.clone()), i == cur))
            .collect()
    }
    /// Restore a specific history revision (a History-panel click).
    pub fn restore_revision(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(sources) = self.history.restore(index).map(|r| r.sources.clone()) else { return };
        self.project.restore_sources(sources);
        self.recompile_and_reload(cx);
        cx.notify();
    }

    // ── IPC bridge ───────────────────────────────────────────────────────────
    fn apply_ipc(&mut self, events: Vec<ipc::Event>, cx: &mut Context<Self>) {
        for event in events {
            match event {
                ipc::Event::Select { key, additive } => self.select_node(key, additive, cx),
                ipc::Event::Deselect => self.deselect_node(cx),
                ipc::Event::PageLoaded => {
                    // Re-apply the selection outline after a reload cleared it.
                    self.sync_preview(cx);
                    self.highlight_nodes(cx);
                }
                ipc::Event::RuntimeError { message } => self.on_runtime_error(message, cx),
            }
        }
    }

    /// A preview runtime error: try to self-heal the page — fix the bug without
    /// changing the design (§4.6). Bounded: each source version is attempted once,
    /// so a fix that doesn't clear the error never loops.
    fn on_runtime_error(&mut self, message: String, cx: &mut Context<Self>) {
        if self.busy || !self.generated || self.proposal.is_some() {
            return;
        }
        let file = "src/pages/Home.wf".to_string();
        let Some(source) = self.project.file_source(&file).map(str::to_string) else { return };
        // Attempt each exact source only once (a give-up leaves it, so no re-heal loop).
        if self.heal_tried.as_deref() == Some(&source) {
            return;
        }
        self.heal_tried = Some(source.clone());
        let key = self.key_text(cx).trim().to_string();
        if key.is_empty() {
            return; // no key — nothing we can do; stay quiet
        }

        let kind = self.provider_kind();
        let mut config = wf_core::GenConfig::for_model(kind.default_model());
        config.max_tokens = 8192;
        let provider = wf_ai::provider_for(kind, key);

        self.busy = true;
        self.status = Status::Compiling;
        self.push_msg(Role::Assistant, "Caught a runtime error in the preview — trying to fix it without touching your design\u{2026}");
        cx.notify();

        self.pipeline_task = Some(cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { wf_core::self_heal(&*provider, &source, &message, &config) })
                .await;
            let _ = this.update(cx, |a, cx| {
                a.busy = false;
                match result {
                    Ok(outcome) => {
                        a.project.set_source(&file, outcome.source);
                        a.recompile_and_reload(cx);
                        a.history.checkpoint("Self-healed a runtime error", a.project.snapshot());
                        a.status = Status::Compiled;
                        a.push_msg(Role::Assistant, format!("Fixed it in {} attempt(s) — your design was untouched.", outcome.attempts));
                        a.show_toast(ToastTone::Success, "Self-healed a runtime error \u{2014} your design was untouched.", cx);
                    }
                    Err(e) => {
                        // Non-blocking: flag it and keep the app usable (FR-22).
                        a.status = Status::Error;
                        a.push_msg(Role::Assistant, format!("A runtime error needs your attention — I couldn't fix it safely without changing the design: {e}"));
                        a.show_toast(ToastTone::Idle, "A runtime error needs attention \u{2014} see the chat.", cx);
                    }
                }
                cx.notify();
            });
        }));
    }


    // ════════════════════════════════════════════════════════════════════════
    // Cinematic workspace: build flow, review, selection, inspector, blocks
    // ════════════════════════════════════════════════════════════════════════
    pub fn compile_text(&self) -> &'static str {
        match self.status {
            Status::Compiling => "Generating your page\u{2026}",
            Status::Compiled => "Your page is live",
            Status::Error => "That didn't compile",
            Status::Idle => "Ready when you are",
        }
    }
    pub fn compile_sub(&self) -> &'static str {
        match self.status {
            Status::Compiling => "writing WebFluent \u{b7} validating \u{b7} self-healing",
            Status::Compiled => "click any element in the preview to tweak it",
            Status::Error => "see the chat for what went wrong",
            Status::Idle => "describe a site to get started",
        }
    }
    pub fn chat_model_label(&self) -> String {
        model_def(&self.chat_model).name.replace("Claude ", "")
    }
    pub fn right_mode(&self) -> RightMode {
        if self.busy {
            RightMode::Working
        } else if self.review_open && self.selection.is_empty() {
            RightMode::Review
        } else if self.selection.len() > 1 {
            RightMode::Multi
        } else if self.selection.len() == 1 && self.generated {
            RightMode::Inspector
        } else if self.generated {
            RightMode::Outline
        } else {
            RightMode::Start
        }
    }

    pub fn send_prompt(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.prompt_text(cx);
        // A pending proposal → refine it (FR-8); a selected element → a scoped edit;
        // otherwise generate.
        if self.proposal.is_some() {
            self.reprompt(text, window, cx);
        } else if self.generated && !self.sel_nodes.is_empty() {
            self.edit(text, window, cx);
        } else {
            self.build(text, window, cx);
        }
    }
    pub fn run_suggestion(&mut self, text: &str, window: &mut Window, cx: &mut Context<Self>) {
        self.build(text.to_string(), window, cx);
    }
    /// Design-system assistant: send the composer text (or a suggestion).
    pub fn ds_send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let t = self.prompt_text(cx).trim().to_string();
        if t.is_empty() {
            return;
        }
        self.push_msg(Role::User, t);
        self.set_prompt("", window, cx);
        self.push_msg(
            Role::Assistant,
            "Updated your design system \u{2014} I adjusted the tokens and regenerated the affected components. Review them on the canvas.",
        );
        self.show_toast(ToastTone::Success, "Design system updated \u{b7} saved to version history.", cx);
    }
    pub fn ds_run(&mut self, text: &str, cx: &mut Context<Self>) {
        self.push_msg(Role::User, text.to_string());
        self.push_msg(Role::Assistant, "Done \u{2014} applied across your tokens and components. Take a look.");
        self.show_toast(ToastTone::Success, "Design system updated.", cx);
    }

    // ── design-system: tabs, selection, foundations & specimen edits ─────────
    pub fn set_ds_tab(&mut self, tab: DsTab, cx: &mut Context<Self>) {
        self.ds_tab = tab;
        self.ds_sel = None;
        cx.notify();
    }
    pub fn ds_select(&mut self, kind: DsSelKind, id: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.ds_sel = Some(DsSel { kind, id: id.into() });
        cx.notify();
    }
    pub fn ds_clear_sel(&mut self, cx: &mut Context<Self>) {
        self.ds_sel = None;
        cx.notify();
    }
    pub fn toggle_ds_rtl(&mut self, cx: &mut Context<Self>) {
        self.ds_rtl = !self.ds_rtl;
        cx.notify();
    }
    pub fn ds_add_color(&mut self, cx: &mut Context<Self>) {
        let id = SharedString::from(format!("c-new{}", self.next_id()));
        self.ds_colors.push(DsColorToken {
            id: id.clone(),
            name: "New token".into(),
            val: 0x7FB3B0,
            role: "Untitled role".into(),
            group: "Custom".into(),
        });
        self.ds_sel = Some(DsSel { kind: DsSelKind::Color, id });
        self.show_toast(ToastTone::Success, "Added a color token \u{2014} pick its value on the right.", cx);
        cx.notify();
    }
    /// Repaint the selected color token from a swatch pick.
    pub fn set_ds_color_val(&mut self, id: &str, val: u32, cx: &mut Context<Self>) {
        if let Some(c) = self.ds_colors.iter_mut().find(|c| c.id.as_ref() == id) {
            c.val = val;
            cx.notify();
        }
    }
    pub fn set_ds_type_weight(&mut self, id: &str, w: u16, cx: &mut Context<Self>) {
        if let Some(t) = self.ds_types.iter_mut().find(|t| t.id.as_ref() == id) {
            t.weight = w;
            cx.notify();
        }
    }
    pub fn bump_ds_type_size(&mut self, id: &str, delta: f32, cx: &mut Context<Self>) {
        if let Some(t) = self.ds_types.iter_mut().find(|t| t.id.as_ref() == id) {
            t.size = (t.size + delta).clamp(10.0, 72.0);
            cx.notify();
        }
    }
    pub fn bump_ds_type_tracking(&mut self, id: &str, delta: f32, cx: &mut Context<Self>) {
        if let Some(t) = self.ds_types.iter_mut().find(|t| t.id.as_ref() == id) {
            t.tracking = (t.tracking + delta).clamp(-4.0, 12.0);
            cx.notify();
        }
    }
    pub fn set_ds_btn_variant(&mut self, v: DsBtnVariant, cx: &mut Context<Self>) {
        self.ds_demo.button_variant = v;
        cx.notify();
    }
    pub fn set_ds_btn_size(&mut self, s: DsBtnSize, cx: &mut Context<Self>) {
        self.ds_demo.button_size = s;
        cx.notify();
    }
    pub fn set_ds_chip_kind(&mut self, k: DsChipKind, cx: &mut Context<Self>) {
        self.ds_demo.chip_kind = k;
        cx.notify();
    }
    pub fn set_ds_status_tone(&mut self, t: DsStatusTone, cx: &mut Context<Self>) {
        self.ds_demo.status_tone = t;
        cx.notify();
    }
    pub fn set_ds_avatar_tone(&mut self, t: DsAvatarTone, cx: &mut Context<Self>) {
        self.ds_demo.avatar_tone = t;
        cx.notify();
    }
    pub fn toggle_ds_demo(&mut self, cx: &mut Context<Self>) {
        self.ds_demo.toggle = !self.ds_demo.toggle;
        cx.notify();
    }
    pub fn bump_ds_slider(&mut self, delta: i16, cx: &mut Context<Self>) {
        self.ds_demo.slider = (self.ds_demo.slider as i16 + delta).clamp(0, 100) as u8;
        cx.notify();
    }
    pub fn set_ds_tabs_active(&mut self, i: u8, cx: &mut Context<Self>) {
        self.ds_demo.tabs_active = i;
        cx.notify();
    }
    pub fn ds_generate_sel(&mut self, cx: &mut Context<Self>) {
        let Some(sel) = self.ds_sel.clone() else { return };
        let label = ds_comp(sel.id.as_ref()).map(|c| c.label).unwrap_or("component");
        self.push_msg(Role::User, format!("Generate the {label} component."));
        self.push_msg(
            Role::Assistant,
            format!("Generating {label} \u{2014} I\u{2019}ll add it as a live, editable specimen with its variants and states."),
        );
        self.show_toast(ToastTone::Success, format!("{label} queued for generation."), cx);
        cx.notify();
    }

    // ── design-system: inspector selection accessors ────────────────────────
    pub fn ds_summary(&self) -> (usize, usize, usize, usize) {
        (self.ds_colors.len(), self.ds_types.len(), ds_comp_counts().1, DS_RADII.len())
    }
    pub fn ds_selected_color(&self) -> Option<&DsColorToken> {
        let sel = self.ds_sel.as_ref()?;
        (sel.kind == DsSelKind::Color).then(|| self.ds_colors.iter().find(|c| c.id == sel.id)).flatten()
    }
    pub fn ds_selected_type(&self) -> Option<&DsTypeToken> {
        let sel = self.ds_sel.as_ref()?;
        (sel.kind == DsSelKind::Type).then(|| self.ds_types.iter().find(|t| t.id == sel.id)).flatten()
    }
    pub fn ds_selected_comp(&self) -> Option<&'static DsComp> {
        let sel = self.ds_sel.as_ref()?;
        (sel.kind == DsSelKind::Comp).then(|| ds_comp(sel.id.as_ref())).flatten()
    }

    /// Generate a page from the prompt: call the selected provider with the
    /// language card, validate + self-heal the reply (in `wf-core`), then swap
    /// the Home source and reload the preview. BYOK — a provider key is the only
    /// setup (IMPLEMENTATION_PLAN M1, Flow A). The loop blocks (the provider
    /// streams on its own thread), so it runs on the background executor and the
    /// result is applied back on the UI thread.
    pub fn build(&mut self, text: String, window: &mut Window, cx: &mut Context<Self>) {
        let prompt = text.trim().to_string();
        if prompt.is_empty() || self.busy {
            return;
        }
        self.push_msg(Role::User, prompt.clone());
        self.set_prompt("", window, cx);
        self.selection.clear();
        self.sel_nodes.clear();
        self.chat_menu = None;

        let key = self.key_text(cx).trim().to_string();
        if key.is_empty() {
            self.push_msg(Role::Assistant, "Add a provider API key in Settings, then send your prompt again.");
            self.show_toast(ToastTone::Idle, "No API key \u{2014} add one in Settings to generate.", cx);
            cx.notify();
            return;
        }
        // Remember the key for next time (per provider), now that it is in use.
        self.save_current_key(cx);

        let kind = self.provider_kind();
        let mut config = wf_core::GenConfig::for_model(kind.default_model());
        config.max_tokens = 8192;
        let provider = wf_ai::provider_for(kind, key);

        self.busy = true;
        self.status = Status::Compiling;
        cx.notify();

        self.pipeline_task = Some(cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { wf_core::generate_page(&*provider, wf_ai::LANGUAGE_CARD, &prompt, &config) })
                .await;
            let _ = this.update(cx, |a, cx| {
                a.busy = false;
                match result {
                    Ok(outcome) => {
                        a.project.set_source("src/pages/Home.wf", outcome.source);
                        a.recompile_and_reload(cx);
                        // recompile validates again; it should agree with generation.
                        let compile_err = a.project.error().map(|e| e.to_string());
                        match compile_err {
                            Some(err) => {
                                a.status = Status::Error;
                                a.push_msg(Role::Assistant, format!("The page was generated but the preview failed to compile: {err}"));
                            }
                            None => {
                                a.generated = true;
                                a.status = Status::Compiled;
                                let healed = outcome.attempts.saturating_sub(1);
                                let note = if healed > 0 {
                                    format!("Done \u{2014} your page is live (self-healed {healed} compile issue(s) along the way). Click any part of the preview to tweak it.")
                                } else {
                                    "Done \u{2014} your page is live. Click any part of the preview to tweak it.".to_string()
                                };
                                a.push_msg(Role::Assistant, note);
                                a.sync_preview(cx);
                                a.history.checkpoint("Generated a page", a.project.snapshot());
                            }
                        }
                    }
                    Err(e) => {
                        a.status = Status::Error;
                        a.push_msg(Role::Assistant, format!("I couldn't build a working page: {e}"));
                        a.show_toast(ToastTone::Idle, "Generation failed \u{2014} see the chat for details.", cx);
                    }
                }
                cx.notify();
            });
        }));
    }

    /// A scoped AI edit of the selected element: run `edit_node` off-thread, diff
    /// the result into a `Proposal`, and open the review panel with real chips.
    pub fn edit(&mut self, instruction: String, window: &mut Window, cx: &mut Context<Self>) {
        let instruction = instruction.trim().to_string();
        if instruction.is_empty() || self.busy {
            return;
        }
        let Some(node_id) = self.sel_nodes.first().cloned() else {
            return self.build(instruction, window, cx);
        };
        let Some(resolved) = self.project.resolve_node(&node_id) else { return };
        let file = resolved.file.clone();
        let Some(base) = self.project.file_source(&file).map(str::to_string) else { return };
        self.run_edit(base, node_id.to_string(), file, instruction, window, cx);
    }

    /// Refine the pending proposal with a follow-up instruction (FR-8): edit the
    /// same node in the CURRENT proposal, but keep diffing against the original
    /// base so the review still shows the whole change from the live document.
    pub fn reprompt(&mut self, instruction: String, window: &mut Window, cx: &mut Context<Self>) {
        let instruction = instruction.trim().to_string();
        if instruction.is_empty() || self.busy {
            return;
        }
        let (Some(node), Some(file)) = (self.proposal_node.clone(), self.proposal_file.clone()) else {
            return;
        };
        let Some(edit_base) = self.proposal.as_ref().map(|p| p.proposal().to_string()) else { return };
        self.run_edit(edit_base, node, file, instruction, window, cx);
    }

    /// Shared edit spine: run `edit_node` on `edit_base` off-thread, then diff the
    /// result against the live (unapplied) file source into a `Proposal`. `edit`
    /// passes the original file as `edit_base`; `reprompt` passes the current proposal.
    fn run_edit(&mut self, edit_base: String, node: String, file: String, instruction: String, window: &mut Window, cx: &mut Context<Self>) {
        let key = self.key_text(cx).trim().to_string();
        if key.is_empty() {
            self.push_msg(Role::Assistant, "Add a provider API key in Settings, then send your edit again.");
            self.show_toast(ToastTone::Idle, "No API key \u{2014} add one in Settings.", cx);
            cx.notify();
            return;
        }
        self.save_current_key(cx);

        let kind = self.provider_kind();
        let mut config = wf_core::GenConfig::for_model(kind.default_model());
        config.max_tokens = 8192;
        let provider = wf_ai::provider_for(kind, key);
        let node_for_edit = node.clone();

        self.push_msg(Role::User, instruction.clone());
        self.set_prompt("", window, cx);
        self.busy = true;
        self.status = Status::Compiling;
        cx.notify();

        self.pipeline_task = Some(cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { wf_core::edit_node(&*provider, &edit_base, &node_for_edit, &instruction, &config) })
                .await;
            let _ = this.update(cx, |a, cx| {
                a.busy = false;
                match result {
                    Ok(outcome) => {
                        // Diff against the live file source (unchanged until Apply),
                        // so re-prompts still show the full change from the document.
                        let diff_base = a.project.file_source(&file).unwrap_or_default().to_string();
                        match wf_core::Proposal::new(diff_base, outcome.source) {
                            Ok(proposal) => {
                                let n = proposal.len();
                                a.proposal = Some(proposal);
                                a.proposal_file = Some(file);
                                a.proposal_node = Some(node);
                                a.selection.clear();
                                a.sel_nodes.clear();
                                a.highlight_nodes(cx);
                                a.review_open = true;
                                a.status = Status::Compiled;
                                let msg = if n == 0 {
                                    "That already looks the way you asked — no changes needed.".to_string()
                                } else {
                                    format!("Ready — {n} change(s) to review and keep on the right.")
                                };
                                a.push_msg(Role::Assistant, msg);
                            }
                            Err(e) => {
                                a.status = Status::Error;
                                a.push_msg(Role::Assistant, format!("I made the edit but couldn't diff it: {e}"));
                            }
                        }
                    }
                    Err(e) => {
                        a.status = Status::Error;
                        a.push_msg(Role::Assistant, format!("I couldn't make that edit: {e}"));
                    }
                }
                cx.notify();
            });
        }));
    }

    // ── review ───────────────────────────────────────────────────────────────
    /// The review chips to render: `(kind, label, accepted)` from the pending proposal.
    pub fn review_chips(&self) -> Vec<(ChipKind, SharedString, bool)> {
        match &self.proposal {
            Some(p) => p
                .chips()
                .iter()
                .enumerate()
                .map(|(i, c)| (map_chip_kind(c.kind), SharedString::from(c.label.clone()), p.is_accepted(i)))
                .collect(),
            None => Vec::new(),
        }
    }
    pub fn kept_count(&self) -> usize {
        self.proposal.as_ref().map(|p| p.accepted_count()).unwrap_or(0)
    }
    pub fn toggle_keep(&mut self, i: usize, cx: &mut Context<Self>) {
        if let Some(p) = self.proposal.as_mut() {
            p.toggle(i);
            cx.notify();
        }
    }
    pub fn review_keep_all(&mut self, cx: &mut Context<Self>) {
        if let Some(p) = self.proposal.as_mut() {
            for i in 0..p.len() {
                p.set_accepted(i, true);
            }
            cx.notify();
        }
    }
    pub fn review_clear_all(&mut self, cx: &mut Context<Self>) {
        if let Some(p) = self.proposal.as_mut() {
            for i in 0..p.len() {
                p.set_accepted(i, false);
            }
            cx.notify();
        }
    }
    pub fn apply_review(&mut self, cx: &mut Context<Self>) {
        let Some(file) = self.proposal_file.clone() else {
            self.review_open = false;
            cx.notify();
            return;
        };
        let applied = match self.proposal.as_ref() {
            Some(p) => p.apply_accepted().map(|src| (src, p.accepted_count())),
            None => return,
        };
        match applied {
            Ok((source, count)) => {
                self.project.set_source(&file, source);
                self.recompile_and_reload(cx);
                self.history.checkpoint(format!("Applied {count} change(s)"), self.project.snapshot());
                self.proposal = None;
                self.proposal_file = None;
                self.proposal_node = None;
                self.review_open = false;
                self.status = Status::Compiled;
                self.show_toast(ToastTone::Success, format!("Applied {count} change(s)."), cx);
            }
            Err(e) => {
                self.push_msg(Role::Assistant, format!("Some of those changes conflicted and couldn't be applied together: {e}. Try keeping fewer, or apply all."));
                self.show_toast(ToastTone::Idle, "Couldn't apply that combination \u{2014} see the chat.", cx);
            }
        }
        cx.notify();
    }
    pub fn discard_review(&mut self, cx: &mut Context<Self>) {
        self.proposal = None;
        self.proposal_file = None;
        self.proposal_node = None;
        self.review_open = false;
        self.status = Status::Compiled;
        self.show_toast(ToastTone::Idle, "Changes discarded \u{2014} your design was left untouched.", cx);
        cx.notify();
    }

    // ── selection ─────────────────────────────────────────────────────────────
    pub fn deselect(&mut self, cx: &mut Context<Self>) {
        self.selection.clear();
        self.sync_preview(cx);
        cx.notify();
    }
    pub fn remove_from_selection(&mut self, key: &str, cx: &mut Context<Self>) {
        self.selection.retain(|k| k.as_ref() != key);
        self.sync_preview(cx);
        cx.notify();
    }
    pub fn sel_kind(&self) -> Option<ElKind> {
        if self.selection.len() != 1 {
            return None;
        }
        let key = self.selection[0].as_ref();
        if let Some(idx) = key.strip_prefix("add").and_then(|n| n.parse::<usize>().ok()) {
            return self.added_blocks.get(idx).map(|b| match b {
                BlockType::Text => ElKind::Text,
                BlockType::Image => ElKind::Image,
                BlockType::Button => ElKind::Button,
            });
        }
        element(key).map(|e| e.kind)
    }
    pub fn sel_label(&self, key: &str) -> SharedString {
        if let Some(idx) = key.strip_prefix("add").and_then(|n| n.parse::<usize>().ok()) {
            return self.added_blocks.get(idx).map(|b| b.label().into()).unwrap_or_else(|| "Block".into());
        }
        element(key).map(|e| SharedString::from(e.label)).unwrap_or_else(|| "Element".into())
    }
    pub fn sel_icon(&self, key: &str) -> &'static str {
        if key.starts_with("add") {
            return "plus";
        }
        element(key).map(|e| e.icon).unwrap_or("target")
    }

    // ── inspector (live edits) ────────────────────────────────────────────────
    /// Apply structured edits to the currently-selected node, then recompile the
    /// project and reload the preview — the real "edit by selecting" spine.
    fn apply_ops(&mut self, ops: Vec<webfluent::EditOp>, cx: &mut Context<Self>) {
        let Some(id) = self.sel_nodes.first().cloned() else { return };
        match self.project.edit_node(id.as_ref(), &ops) {
            Ok(()) => self.recompile_and_reload(cx),
            Err(e) => eprintln!("wf-studio: edit failed: {e}"),
        }
    }

    /// Emit a `SetStyle { prop, value }` edit on the selected node.
    fn edit_style(&mut self, prop: &str, value: String, cx: &mut Context<Self>) {
        let Some(id) = self.sel_nodes.first().cloned() else { return };
        let op = webfluent::EditOp::SetStyle { node: id.to_string(), prop: prop.to_string(), value };
        self.apply_ops(vec![op], cx);
    }

    pub fn set_color(&mut self, v: SharedString, cx: &mut Context<Self>) {
        self.edit_style("color", format!("\"{v}\""), cx);
    }
    pub fn set_size(&mut self, v: f32, cx: &mut Context<Self>) {
        self.edit_style("font-size", format!("\"{v}px\""), cx);
    }
    pub fn set_weight(&mut self, v: u16, cx: &mut Context<Self>) {
        self.edit_style("font-weight", v.to_string(), cx);
    }
    pub fn set_align(&mut self, v: Align, cx: &mut Context<Self>) {
        self.edit_style("text-align", format!("\"{}\"", v.value()), cx);
    }
    pub fn set_bg(&mut self, v: SharedString, cx: &mut Context<Self>) {
        self.edit_style("background", format!("\"{v}\""), cx);
    }
    pub fn set_radius(&mut self, v: f32, cx: &mut Context<Self>) {
        self.edit_style("border-radius", format!("\"{v}px\""), cx);
    }
    pub fn reset_style(&mut self, cx: &mut Context<Self>) {
        for key in self.selection.clone() {
            self.edits.insert(key, ElEdit::default());
        }
        self.sync_preview(cx);
        cx.notify();
    }
    pub fn edit_for(&self, key: &str) -> ElEdit {
        self.edits.get(key).cloned().unwrap_or_default()
    }

    // ── outline / blocks ──────────────────────────────────────────────────────
    pub fn add_block(&mut self, kind: BlockType, cx: &mut Context<Self>) {
        // Append the block as a child of the selected element (a real AppendChild
        // edit → recompile → reload).
        let Some(target) = self.sel_nodes.first().cloned() else {
            self.show_toast(ToastTone::Idle, "Select an element first, then add a block into it.", cx);
            return;
        };
        let (wf, label) = match kind {
            BlockType::Text => ("Text(\"New text\")", "text block"),
            BlockType::Image => ("Image(src: \"/placeholder.png\")", "image"),
            BlockType::Button => ("Button(\"Button\", primary)", "button"),
        };
        self.apply_ops(
            vec![webfluent::EditOp::AppendChild { node: target.to_string(), wf: wf.to_string() }],
            cx,
        );
        self.show_toast(ToastTone::Success, format!("Added a {label}."), cx);
        cx.notify();
    }

    // ── composer menus & options ──────────────────────────────────────────────
    pub fn toggle_chat_menu(&mut self, m: ChatMenu, cx: &mut Context<Self>) {
        self.chat_menu = if self.chat_menu == Some(m) { None } else { Some(m) };
        cx.notify();
    }
    pub fn close_chat_menu(&mut self, cx: &mut Context<Self>) {
        self.chat_menu = None;
        cx.notify();
    }
    /// Close every composer popover (attach/skills/model/DS/API) — used by the
    /// click-outside backdrop.
    pub fn close_composer_menus(&mut self, cx: &mut Context<Self>) {
        self.chat_menu = None;
        self.ds_picker_open = false;
        self.api_panel_open = false;
        cx.notify();
    }
    pub fn set_chat_model(&mut self, id: &str, cx: &mut Context<Self>) {
        self.chat_model = id.to_string().into();
        cx.notify();
    }
    pub fn set_effort(&mut self, e: Effort, cx: &mut Context<Self>) {
        self.effort = e;
        cx.notify();
    }
    pub fn set_permission(&mut self, p: Permission, cx: &mut Context<Self>) {
        self.permission = p;
        cx.notify();
    }
    pub fn toggle_skill_idx(&mut self, i: usize, cx: &mut Context<Self>) {
        if let Some(pos) = self.skills.iter().position(|s| *s == i) {
            self.skills.remove(pos);
        } else {
            self.skills.push(i);
        }
        cx.notify();
    }

    // ── design-system picker ──────────────────────────────────────────────────
    pub fn toggle_ds_picker(&mut self, cx: &mut Context<Self>) {
        self.ds_picker_open = !self.ds_picker_open;
        cx.notify();
    }
    pub fn applied_ds_name(&self) -> SharedString {
        self.applied_ds
            .as_ref()
            .and_then(|id| self.projects.iter().find(|p| &p.id == id))
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "No design system".into())
    }
    pub fn choose_ds(&mut self, id: SharedString, cx: &mut Context<Self>) {
        self.ds_picker_open = false;
        match &self.applied_ds {
            Some(cur) if *cur != id => {
                self.pending_ds = Some(id);
                self.modal = Some(Modal::SwapDs);
            }
            _ => self.applied_ds = Some(id),
        }
        cx.notify();
    }
    pub fn clear_ds(&mut self, cx: &mut Context<Self>) {
        if self.applied_ds.is_some() {
            self.pending_ds = None;
            self.ds_picker_open = false;
            self.modal = Some(Modal::SwapDs);
        }
        cx.notify();
    }
    pub fn confirm_swap_ds(&mut self, cx: &mut Context<Self>) {
        self.applied_ds = self.pending_ds.take();
        self.modal = None;
        let msg = if self.applied_ds.is_some() { "Design system applied to this project." } else { "Design system removed." };
        self.show_toast(ToastTone::Success, msg, cx);
        cx.notify();
    }
    pub fn cancel_swap_ds(&mut self, cx: &mut Context<Self>) {
        self.pending_ds = None;
        self.modal = None;
        cx.notify();
    }

    // ── API integration panel ─────────────────────────────────────────────────
    pub fn toggle_api_panel(&mut self, cx: &mut Context<Self>) {
        self.api_panel_open = !self.api_panel_open;
        self.chat_menu = None;
        cx.notify();
    }
    pub fn attach_openapi(&mut self, cx: &mut Context<Self>) {
        self.api_spec = Some(sample_api_spec());
        self.api_panel_open = true;
        self.chat_menu = None;
        self.show_toast(ToastTone::Success, "OpenAPI spec parsed \u{2014} 5 endpoints found.", cx);
        cx.notify();
    }
    pub fn toggle_endpoint(&mut self, i: usize, cx: &mut Context<Self>) {
        if let Some(spec) = &mut self.api_spec
            && let Some(ep) = spec.endpoints.get_mut(i)
        {
            ep.bound = !ep.bound;
            cx.notify();
        }
    }
    pub fn toggle_spa(&mut self, cx: &mut Context<Self>) {
        self.spa_mode = !self.spa_mode;
        cx.notify();
    }
    pub fn remove_api_spec(&mut self, cx: &mut Context<Self>) {
        self.api_spec = None;
        self.api_panel_open = false;
        cx.notify();
    }
    pub fn api_bound_count(&self) -> usize {
        self.api_spec.as_ref().map(|s| s.endpoints.iter().filter(|e| e.bound).count()).unwrap_or(0)
    }

    // ── collapse toggles ──────────────────────────────────────────────────────
    pub fn toggle_chat(&mut self, cx: &mut Context<Self>) {
        self.chat_open = !self.chat_open;
        cx.notify();
    }
    pub fn toggle_panel(&mut self, cx: &mut Context<Self>) {
        self.panel_open = !self.panel_open;
        cx.notify();
    }

    // ════════════════════════════════════════════════════════════════════════
    // Modals: toast, publish, settings, share, history
    // ════════════════════════════════════════════════════════════════════════
    pub fn show_toast(&mut self, tone: ToastTone, msg: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.toast_note = Some(Toast { tone, msg: msg.into() });
        self.toast_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(4200)).await;
            let _ = this.update(cx, |a, cx| {
                a.toast_note = None;
                cx.notify();
            });
        }));
        cx.notify();
    }
    pub fn dismiss_note(&mut self, cx: &mut Context<Self>) {
        self.toast_task = None;
        self.toast_note = None;
        cx.notify();
    }

    // ── publish ───────────────────────────────────────────────────────────────
    pub fn set_publish_tab(&mut self, t: PublishTab, cx: &mut Context<Self>) {
        self.publish_tab = t;
        cx.notify();
    }
    pub fn set_export_kind(&mut self, k: ExportKind, cx: &mut Context<Self>) {
        self.export_kind = k;
        cx.notify();
    }
    pub fn do_publish(&mut self, cx: &mut Context<Self>) {
        self.deploying = true;
        cx.notify();
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_millis(1700)).await;
            let _ = this.update(cx, |a, cx| {
                a.deploying = false;
                a.published = true;
                cx.notify();
            });
        })
        .detach();
    }
    pub fn copy_link(&mut self, cx: &mut Context<Self>) {
        self.show_toast(ToastTone::Idle, "Link copied to clipboard.", cx);
    }

    // ── settings ──────────────────────────────────────────────────────────────
    pub fn set_settings_tab(&mut self, t: SettingsTab, cx: &mut Context<Self>) {
        self.settings_tab = t;
        cx.notify();
    }
    pub fn toggle_ctx(&mut self, cx: &mut Context<Self>) {
        self.pruning = !self.pruning;
        cx.notify();
    }
    pub fn toggle_prompt_cache(&mut self, cx: &mut Context<Self>) {
        self.caching = !self.caching;
        cx.notify();
    }
    pub fn inc_heal(&mut self, cx: &mut Context<Self>) {
        self.heal_attempts = (self.heal_attempts + 1).min(5);
        cx.notify();
    }
    pub fn dec_heal(&mut self, cx: &mut Context<Self>) {
        self.heal_attempts = self.heal_attempts.saturating_sub(1).max(1);
        cx.notify();
    }
    pub fn toggle_mcp(&mut self, id: u64, cx: &mut Context<Self>) {
        if let Some(m) = self.mcp_list.iter_mut().find(|m| m.id == id) {
            m.on = !m.on;
            cx.notify();
        }
    }
    pub fn remove_mcp(&mut self, id: u64, cx: &mut Context<Self>) {
        self.mcp_list.retain(|m| m.id != id);
        cx.notify();
    }
    pub fn add_mcp(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.mcp_name.read(cx).value().trim().to_string();
        let cmd = self.mcp_cmd.read(cx).value().trim().to_string();
        if name.is_empty() || cmd.is_empty() {
            return;
        }
        let id = self.mcp_next_id;
        self.mcp_next_id += 1;
        self.mcp_list.push(McpServer { id, name: name.into(), meta: cmd.into(), on: true });
        self.mcp_name.update(cx, |s, cx| s.set_value("", window, cx));
        self.mcp_cmd.update(cx, |s, cx| s.set_value("", window, cx));
        self.show_toast(ToastTone::Success, "MCP server added and connected.", cx);
    }
    pub fn connect_acp(&mut self, cx: &mut Context<Self>) {
        if self.acp_url.read(cx).value().trim().len() < 6 {
            return;
        }
        self.acp_connected = true;
        self.show_toast(ToastTone::Success, "Agent connected over ACP.", cx);
        cx.notify();
    }
    pub fn disconnect_acp(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.acp_connected = false;
        self.acp_url.update(cx, |s, cx| s.set_value("", window, cx));
        cx.notify();
    }

    // ── share ─────────────────────────────────────────────────────────────────
    pub fn toggle_share_menu(&mut self, m: ShareMenu, cx: &mut Context<Self>) {
        self.share_menu = if self.share_menu == Some(m) { None } else { Some(m) };
        cx.notify();
    }
    pub fn close_share_menu(&mut self, cx: &mut Context<Self>) {
        self.share_menu = None;
        cx.notify();
    }
    pub fn set_share_role(&mut self, r: ShareRole, cx: &mut Context<Self>) {
        self.share_role = r;
        self.share_menu = None;
        cx.notify();
    }
    pub fn set_link_access(&mut self, a: LinkAccess, cx: &mut Context<Self>) {
        self.link_access = a;
        self.share_menu = None;
        cx.notify();
    }
    pub fn set_collab_role(&mut self, i: usize, r: ShareRole, cx: &mut Context<Self>) {
        if i == 1 {
            self.collab_mk = r;
        } else {
            self.collab_ah = r;
        }
        self.share_menu = None;
        cx.notify();
    }
    pub fn invite_sent(&mut self, cx: &mut Context<Self>) {
        self.show_toast(ToastTone::Success, "Invitation sent \u{2014} they\u{2019}ll get an email.", cx);
    }

    // ── history ───────────────────────────────────────────────────────────────
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

/// Build the preview webview as a child of the gpui window, serving the live
/// [`CompiledSite`] held in `output` over `wf://`.
fn build_preview(window: &mut Window, output: Arc<RwLock<CompiledSite>>) -> anyhow::Result<wry::WebView> {
    let builder = WebViewBuilder::new()
        .with_custom_protocol("wf".into(), move |_id, request| {
            let found = output.read().ok().and_then(|site| wf_preview::resolve(&site, request.uri().path()));
            wf_preview::respond(found)
        })
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

fn boot_script() -> String {
    // Inject the click/lifecycle bridge before the compiled site's own scripts.
    ipc::BRIDGE_JS.to_string()
}

/// Map a diff chip kind (wf-core) to the studio's review chip kind.
fn map_chip_kind(k: wf_core::ChipKind) -> ChipKind {
    match k {
        wf_core::ChipKind::Text => ChipKind::Text,
        wf_core::ChipKind::Style => ChipKind::Style,
        wf_core::ChipKind::Structure => ChipKind::Structure,
        wf_core::ChipKind::Behavior => ChipKind::Behavior,
    }
}
