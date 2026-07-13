//! Domain model for the Studio: the enums, value types, and static data tables
//! ported from the mock's `Component` class (`docs/WebFluent Studio.dc.html`).
//!
//! Pure data — no GPUI. The live mutable state and the logic that drives it live
//! on [`crate::app::StudioApp`]; the rendering lives in [`crate::ui`].

use gpui::{Hsla, SharedString};

use crate::theme;

// ── Screens & top-level mode ────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Onboarding,
    Studio,
}

/// The compile/generation status surfaced in the top-bar badge (FR-13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Idle,
    Generating,
    Compiling,
    SelfHeal,
    Compiled,
    Attention,
    Error,
}

impl Status {
    /// `(label, dot color)`. `Idle` reads differently before/after first build.
    pub fn label_color(self, generated: bool) -> (&'static str, Hsla) {
        match self {
            Status::Idle if generated => ("Ready", theme::hex(0x9d938b)),
            Status::Idle => ("No project yet", theme::hex(0x9d938b)),
            Status::Generating => ("Generating\u{2026}", theme::accent()),
            Status::Compiling => ("Compiling\u{2026}", theme::accent()),
            Status::SelfHeal => ("Self-healing\u{2026}", theme::warn()),
            Status::Compiled => ("Compiled", theme::success()),
            Status::Attention => ("Needs attention", theme::warn()),
            Status::Error => ("Couldn\u{2019}t compile", theme::danger()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Rtl,
    Ltr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Device {
    Desktop,
    Tablet,
    Mobile,
}

/// API-key connection test state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tested {
    Idle,
    Ok,
    Fail,
}

/// Which action the error overlay should retry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorAction {
    Generate,
    Edit,
}

/// Semantic tone shared by chat bubbles and activity/log dots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Plain,
    Ok,
    Info,
    Warn,
    Err,
}

impl Tone {
    pub fn dot(self) -> Hsla {
        match self {
            Tone::Ok => theme::success(),
            Tone::Warn => theme::warn(),
            Tone::Err => theme::danger(),
            Tone::Info | Tone::Plain => theme::muted(),
        }
    }
}

// ── Providers & models ──────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderId {
    Anthropic,
    OpenAI,
    Gemini,
    DeepSeek,
    Kimi,
    Glm,
}

pub struct Provider {
    pub id: ProviderId,
    pub name: &'static str,
    pub by: &'static str,
    pub mono: &'static str,
    pub mono_bg: u32,
    pub models: &'static [&'static str],
    pub recommended: bool,
}

impl Provider {
    pub fn default_model(&self) -> &'static str {
        self.models[0]
    }
}

pub const PROVIDERS: &[Provider] = &[
    Provider {
        id: ProviderId::Anthropic,
        name: "Claude",
        by: "Anthropic",
        mono: "C",
        mono_bg: 0xc4634a,
        models: &["Claude Sonnet 4.5", "Claude Haiku 4.5"],
        recommended: true,
    },
    Provider {
        id: ProviderId::OpenAI,
        name: "OpenAI",
        by: "GPT-4.1",
        mono: "O",
        mono_bg: 0x10a37f,
        models: &["GPT-4.1", "GPT-4.1 mini"],
        recommended: false,
    },
    Provider {
        id: ProviderId::Gemini,
        name: "Gemini",
        by: "Google",
        mono: "G",
        mono_bg: 0x4285f4,
        models: &["Gemini 2.5 Pro", "Gemini 2.5 Flash"],
        recommended: false,
    },
    Provider {
        id: ProviderId::DeepSeek,
        name: "DeepSeek",
        by: "DeepSeek",
        mono: "D",
        mono_bg: 0x4d6bfe,
        models: &["DeepSeek V3", "DeepSeek R1"],
        recommended: false,
    },
    Provider {
        id: ProviderId::Kimi,
        name: "Kimi",
        by: "Moonshot",
        mono: "K",
        mono_bg: 0x6b4dfe,
        models: &["Kimi K2", "Kimi K2 Turbo"],
        recommended: false,
    },
    Provider {
        id: ProviderId::Glm,
        name: "GLM",
        by: "Zhipu",
        mono: "Z",
        mono_bg: 0x3a7afe,
        models: &["GLM-4.6", "GLM-4.5 Air"],
        recommended: false,
    },
];

pub fn provider(id: ProviderId) -> &'static Provider {
    PROVIDERS.iter().find(|p| p.id == id).unwrap()
}

// ── Skills ──────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillId {
    Seo,
    A11y,
    Rtl,
    Menu,
    Booking,
    Analytics,
}

pub struct Skill {
    pub id: SkillId,
    pub name: &'static str,
    pub desc: &'static str,
}

pub const SKILLS: &[Skill] = &[
    Skill { id: SkillId::Seo, name: "SEO optimization", desc: "Meta tags, clean headings, sitemap" },
    Skill { id: SkillId::A11y, name: "Accessibility", desc: "WCAG contrast, labels, alt text" },
    Skill { id: SkillId::Rtl, name: "Arabic RTL polish", desc: "Logical CSS, Arabic typography" },
    Skill { id: SkillId::Menu, name: "Restaurant menu", desc: "Structured menu with EGP prices" },
    Skill { id: SkillId::Booking, name: "Table booking", desc: "Reservation form + confirmation" },
    Skill { id: SkillId::Analytics, name: "Privacy analytics", desc: "Cookieless visit tracking" },
];

pub fn skill(id: SkillId) -> &'static Skill {
    SKILLS.iter().find(|s| s.id == id).unwrap()
}

// ── Starter & sample prompts ────────────────────────────────────────────────
pub struct Starter {
    pub chip: &'static str,
    pub prompt: &'static str,
}

pub const STARTERS: &[Starter] = &[
    Starter {
        chip: "\u{2615}\u{fe0f} Cairo caf\u{e9}",
        prompt: "\u{645}\u{642}\u{647}\u{649} \u{635}\u{63a}\u{64a}\u{631} \u{641}\u{64a} \u{648}\u{633}\u{637} \u{627}\u{644}\u{642}\u{627}\u{647}\u{631}\u{629} \u{64a}\u{642}\u{62f}\u{645} \u{642}\u{647}\u{648}\u{629} \u{62a}\u{631}\u{643}\u{64a} \u{648}\u{62d}\u{644}\u{648}\u{64a}\u{627}\u{62a} \u{634}\u{631}\u{642}\u{64a}\u{629}\u{60c} \u{645}\u{639} \u{642}\u{627}\u{626}\u{645}\u{629} \u{637}\u{639}\u{627}\u{645} \u{648}\u{645}\u{648}\u{627}\u{639}\u{64a}\u{62f}",
    },
    Starter {
        chip: "Boutique shop",
        prompt: "A landing page for a boutique clothing shop with a hero, featured products, and contact details",
    },
    Starter {
        chip: "Portfolio",
        prompt: "A clean portfolio for a freelance designer with an about section, selected work, and a contact form",
    },
];

pub struct Sample {
    pub icon: &'static str,
    pub icon_bg: (u32, u32),
    pub title: &'static str,
    pub desc: &'static str,
    pub prompt: &'static str,
}

pub const SAMPLES: &[Sample] = &[
    Sample {
        icon: "\u{2615}",
        icon_bg: (0xc4634a, 0x8a3f28),
        title: "Cairo caf\u{e9} or restaurant",
        desc: "Menu, hours, and a reservation prompt \u{2014} in Arabic.",
        prompt: STARTERS[0].prompt,
    },
    Sample {
        icon: "\u{1F45C}",
        icon_bg: (0xb0779a, 0x7a4d63),
        title: "Boutique shop landing page",
        desc: "Hero, featured products, and a contact section.",
        prompt: STARTERS[1].prompt,
    },
    Sample {
        icon: "\u{1F4BC}",
        icon_bg: (0x6a7fb0, 0x455680),
        title: "Freelancer portfolio",
        desc: "About, selected work, and a way to get in touch.",
        prompt: STARTERS[2].prompt,
    },
];

// ── Canvas selection sections (the mock's `SEL` table) ──────────────────────
pub struct SelInfo {
    pub key: &'static str,
    pub label: &'static str,
    pub tries: [&'static str; 2],
    /// Whether the floating Quick Inspector applies (mock shows it for `heading`).
    pub inspector: bool,
}

/// The selectable regions of the generated site, in document order.
pub const SECTIONS: &[SelInfo] = &[
    SelInfo { key: "header", label: "Header", tries: ["Make it translucent", "Add a phone number"], inspector: false },
    SelInfo { key: "nav", label: "Navigation", tries: ["Add a \u{201c}Contact\u{201d} link", "Make the links bolder"], inspector: false },
    SelInfo { key: "hero", label: "Hero section", tries: ["Make it warmer and bigger", "Add an online order form"], inspector: false },
    SelInfo { key: "heading", label: "Heading", tries: ["Make it warmer and bigger", "Shorten the wording"], inspector: true },
    SelInfo { key: "subheading", label: "Sub-headline", tries: ["Reword it to be more inviting", "Make it shorter"], inspector: false },
    SelInfo { key: "cta", label: "Call-to-action button", tries: ["Change label to \u{201c}Reserve a table\u{201d}", "Add an online order form"], inspector: false },
    SelInfo { key: "menu", label: "Menu list", tries: ["Add a chef\u{2019}s-pick badge", "Show prices in bold"], inspector: false },
    SelInfo { key: "about", label: "About section", tries: ["Make it warmer and bigger", "Shorten the wording"], inspector: false },
    SelInfo { key: "hours", label: "Hours & location", tries: ["Add a phone number", "Highlight today\u{2019}s hours"], inspector: false },
    SelInfo { key: "footer", label: "Footer", tries: ["Add social links", "Make it darker"], inspector: false },
];

pub fn section(key: &str) -> &'static SelInfo {
    SECTIONS.iter().find(|s| s.key == key).unwrap_or(&SECTIONS[0])
}

/// Map an arbitrary (e.g. bridge-supplied) string back to one of `SECTIONS`'
/// own `&'static str` keys, or `None` if it isn't a recognized section.
pub fn section_key(key: &str) -> Option<&'static str> {
    SECTIONS.iter().find(|s| s.key == key).map(|s| s.key)
}

// ── Chips, edits, history, activity, messages ───────────────────────────────
/// The four edit categories from the plan (§4.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ChipKind {
    Text,
    Style,
    Structure,
    Behavior,
}

impl ChipKind {
    pub fn label(self) -> &'static str {
        match self {
            ChipKind::Text => "Text",
            ChipKind::Style => "Style",
            ChipKind::Structure => "Structure",
            ChipKind::Behavior => "Behavior",
        }
    }
}

/// The five demo edit knobs the hero proposal toggles (mock `appliedEdits`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKey {
    Bigger,
    Warm,
    Tint,
    Reserve,
    Sub,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AppliedEdits {
    pub bigger: bool,
    pub warm: bool,
    pub tint: bool,
    pub reserve: bool,
    pub sub: bool,
}

impl AppliedEdits {
    pub fn set(&mut self, key: EditKey, v: bool) {
        match key {
            EditKey::Bigger => self.bigger = v,
            EditKey::Warm => self.warm = v,
            EditKey::Tint => self.tint = v,
            EditKey::Reserve => self.reserve = v,
            EditKey::Sub => self.sub = v,
        }
    }
    /// Serialize as CafeSite dc-import props (query params for the preview).
    pub fn query(&self) -> String {
        format!(
            "bigger={}&warm={}&tint={}&reserve={}&sub={}",
            self.bigger, self.warm, self.tint, self.reserve, self.sub
        )
    }
}

#[derive(Debug, Clone)]
pub struct Chip {
    pub id: u64,
    pub key: EditKey,
    pub kind: ChipKind,
    pub label: SharedString,
    pub accepted: bool,
}

#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub title: SharedString,
    pub time: SharedString,
    pub edits: AppliedEdits,
    pub current: bool,
}

#[derive(Debug, Clone)]
pub struct ActivityItem {
    pub tone: Tone,
    pub text: SharedString,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub tone: Tone,
    pub text: SharedString,
    pub attachments: Vec<SharedString>,
}

#[derive(Debug, Clone)]
pub struct Attachment {
    pub id: u64,
    pub name: SharedString,
}

/// A live selection on the canvas.
#[derive(Debug, Clone)]
pub struct Selection {
    pub key: &'static str,
}

/// Format minutes-since-midnight as a 12-hour clock, e.g. `10:34 AM`.
pub fn fmt_clock(minutes: u32) -> String {
    let h24 = (minutes / 60) % 24;
    let mm = minutes % 60;
    let h12 = if h24.is_multiple_of(12) { 12 } else { h24 % 12 };
    let ampm = if h24 < 12 { "AM" } else { "PM" };
    format!("{h12}:{mm:02} {ampm}")
}
