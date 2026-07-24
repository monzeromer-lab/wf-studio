//! Domain model for the Studio: the enums, value types, and static data tables
//! ported from the mock's `Component` class (`docs/WebFluent Studio.dc.html`).
//!
//! Pure data — no GPUI. The live mutable state and the logic that drives it live
//! on [`crate::app::StudioApp`]; the rendering lives in [`crate::ui`].

use gpui::{Hsla, SharedString};

use crate::theme;

// ── Screens & top-level mode ────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Login/Onboarding kept for the future studio backend
pub enum Screen {
    Login,
    Home,
    Onboarding,
    Workspace,
    DsWorkspace,
}

/// Which centered dialog (if any) is open over the app window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modal {
    NewProject,
    Exit,
    SwapDs,
    Compile,
    Publish,
    Share,
    Settings,
    History,
    Profile,
}

/// Home dashboard filter tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeFilter {
    All,
    Website,
    System,
}

/// How the assistant is connected (onboarding + settings).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnMode {
    Key,
    Acp,
}

/// Result of a "Test connection" attempt against the selected provider.
#[derive(Debug, Clone, PartialEq)]
pub enum ConnTest {
    Untested,
    Testing,
    Ok,
    Failed(String),
}

// ── Projects (Home dashboard) ───────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    Website,
    System,
}

impl ProjectKind {
    pub fn type_label(self) -> &'static str {
        match self {
            ProjectKind::Website => "Website",
            ProjectKind::System => "Design system",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Published/Shared used once publishing/sharing land
pub enum ProjectStatus {
    Published,
    Draft,
    Shared,
}

impl ProjectStatus {
    pub fn label(self) -> &'static str {
        match self {
            ProjectStatus::Published => "Published",
            ProjectStatus::Draft => "Draft",
            ProjectStatus::Shared => "Shared",
        }
    }
    pub fn color(self) -> Hsla {
        match self {
            ProjectStatus::Published => theme::success(),
            ProjectStatus::Draft => theme::text_caption(),
            ProjectStatus::Shared => theme::accent(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectTone {
    Accent,
    Violet,
    Teal,
    Blue,
}

impl ProjectTone {
    pub fn color(self) -> Hsla {
        match self {
            ProjectTone::Accent => theme::accent(),
            ProjectTone::Violet => theme::violet_soft(),
            ProjectTone::Teal => theme::tone_teal(),
            ProjectTone::Blue => theme::tone_blue(),
        }
    }
    pub fn tint(self) -> Hsla {
        match self {
            ProjectTone::Accent => theme::accent_tint(),
            ProjectTone::Violet => theme::violet_tint(),
            ProjectTone::Teal => theme::hexa(0x5CCB9A24),
            ProjectTone::Blue => theme::hexa(0x7FB0EE24),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Project {
    pub id: SharedString,
    pub kind: ProjectKind,
    pub name: SharedString,
    pub sub: SharedString,
    pub updated: SharedString,
    pub status: ProjectStatus,
    pub mono: SharedString,
    pub tone: ProjectTone,
}

/// The compile/generation status surfaced in the top-bar badge (FR-13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Idle,
    Compiling,
    Compiled,
    Error,
}

impl Status {
    /// `(label, dot color)`. `Idle` reads differently before/after first build.
    pub fn label_color(self, generated: bool) -> (&'static str, Hsla) {
        match self {
            Status::Idle if generated => ("Ready", theme::hex(0x9d938b)),
            Status::Idle => ("No project yet", theme::hex(0x9d938b)),
            Status::Compiling => ("Compiling\u{2026}", theme::accent()),
            Status::Compiled => ("Compiled", theme::success()),
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

/// Semantic tone for compile-log entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Warn tone used for compile warnings
pub enum Tone {
    Ok,
    Warn,
    Err,
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
    pub recommended: bool,
}

pub const PROVIDERS: &[Provider] = &[
    Provider {
        id: ProviderId::Anthropic,
        name: "Claude",
        by: "Anthropic",
        mono: "C",
        mono_bg: 0xc4634a,
        recommended: true,
    },
    Provider {
        id: ProviderId::OpenAI,
        name: "OpenAI",
        by: "GPT-5.6",
        mono: "O",
        mono_bg: 0x10a37f,
        recommended: false,
    },
    Provider {
        id: ProviderId::Gemini,
        name: "Gemini",
        by: "Google",
        mono: "G",
        mono_bg: 0x4285f4,
        recommended: false,
    },
    Provider {
        id: ProviderId::DeepSeek,
        name: "DeepSeek",
        by: "DeepSeek",
        mono: "D",
        mono_bg: 0x4d6bfe,
        recommended: false,
    },
    Provider {
        id: ProviderId::Kimi,
        name: "Kimi",
        by: "Moonshot",
        mono: "K",
        mono_bg: 0x6b4dfe,
        recommended: false,
    },
    Provider {
        id: ProviderId::Glm,
        name: "GLM",
        by: "Z.ai",
        mono: "Z",
        mono_bg: 0x3a7afe,
        recommended: false,
    },
];

pub fn provider(id: ProviderId) -> &'static Provider {
    PROVIDERS.iter().find(|p| p.id == id).unwrap()
}

// ════════════════════════════════════════════════════════════════════════════
// Cinematic workspace model (composer, inspector, review, blocks, API)
// ════════════════════════════════════════════════════════════════════════════

/// Which composer popover is open (attach / skills / model+permissions).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatMenu {
    Attach,
    Skills,
    Model,
}

/// Assistant reasoning effort.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effort {
    Fast,
    Balanced,
    Max,
}
impl Effort {
    pub fn label(self) -> &'static str {
        match self {
            Effort::Fast => "Fast",
            Effort::Balanced => "Balanced",
            Effort::Max => "Max",
        }
    }
}

/// How much the assistant may do without asking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Review,
    Safe,
    Manual,
}
impl Permission {
    pub fn label(self) -> &'static str {
        match self {
            Permission::Review => "Review each change",
            Permission::Safe => "Auto-apply safe edits",
            Permission::Manual => "Ask before anything",
        }
    }
    pub fn desc(self) -> &'static str {
        match self {
            Permission::Review => "Nothing sticks until you approve it",
            Permission::Safe => "Text & style apply; structure asks first",
            Permission::Manual => "Confirm every action, even reads",
        }
    }
}

/// Composer skills (mock `skillDefs`).
pub const SKILL_NAMES: &[&str] = &["Responsive layout", "SEO basics", "Accessibility (WCAG)", "Copywriting", "Motion & animation", "RTL polish"];

// ── Selectable site elements (inspector + outline) ──────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElKind {
    Text,
    Button,
    Image,
}

pub struct ElMeta {
    pub key: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
    pub kind: ElKind,
}

pub const ELEMENTS: &[ElMeta] = &[
    ElMeta { key: "brand", label: "Logo & brand", icon: "sparkle", kind: ElKind::Text },
    ElMeta { key: "nav", label: "Navigation", icon: "grid", kind: ElKind::Text },
    ElMeta { key: "headerCta", label: "Header button", icon: "plus", kind: ElKind::Button },
    ElMeta { key: "eyebrow", label: "Eyebrow pill", icon: "zap", kind: ElKind::Text },
    ElMeta { key: "heading", label: "Heading", icon: "type", kind: ElKind::Text },
    ElMeta { key: "sub", label: "Paragraph", icon: "type", kind: ElKind::Text },
    ElMeta { key: "cta", label: "Primary button", icon: "plus", kind: ElKind::Button },
    ElMeta { key: "cta2", label: "Secondary button", icon: "plus", kind: ElKind::Button },
    ElMeta { key: "visual", label: "Hero image", icon: "image", kind: ElKind::Image },
    ElMeta { key: "lineupTitle", label: "Lineup title", icon: "type", kind: ElKind::Text },
    ElMeta { key: "footer", label: "Footer", icon: "grid", kind: ElKind::Text },
];

pub fn element(key: &str) -> Option<&'static ElMeta> {
    ELEMENTS.iter().find(|e| e.key == key)
}

// ── Live style edits (inspector) ────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
}
impl Align {
    pub fn value(self) -> &'static str {
        match self {
            Align::Start => "start",
            Align::Center => "center",
            Align::End => "end",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ElEdit {
    pub color: Option<SharedString>,
    pub size: Option<f32>,
    pub weight: Option<u16>,
    pub align: Option<Align>,
    pub bg: Option<SharedString>,
    pub radius: Option<f32>,
}

/// A block the user added to the page from the outline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Text,
    Image,
    Button,
}
impl BlockType {
    pub fn label(self) -> &'static str {
        match self {
            BlockType::Text => "Text block",
            BlockType::Image => "Image block",
            BlockType::Button => "Button",
        }
    }
}

/// Which view the right-hand context panel shows (mock `rm`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightMode {
    Working,
    Review,
    Multi,
    Inspector,
    Outline,
    Start,
}

// ── API integration (OpenAPI) ───────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct Endpoint {
    pub method: &'static str,
    pub path: &'static str,
    pub desc: &'static str,
    pub bound: bool,
}

#[derive(Debug, Clone)]
pub struct ApiSpec {
    pub name: &'static str,
    pub version: &'static str,
    pub base: &'static str,
    pub endpoints: Vec<Endpoint>,
}

pub fn sample_api_spec() -> ApiSpec {
    ApiSpec {
        name: "layali-api.yaml",
        version: "OpenAPI 3.1",
        base: "https://api.layali.app/v1",
        endpoints: vec![
            Endpoint { method: "GET", path: "/venues", desc: "List rooftop venues", bound: true },
            Endpoint { method: "GET", path: "/events", desc: "Upcoming lineup", bound: true },
            Endpoint { method: "POST", path: "/reservations", desc: "Create a booking", bound: true },
            Endpoint { method: "GET", path: "/reservations/{id}", desc: "Booking status", bound: false },
            Endpoint { method: "POST", path: "/newsletter", desc: "Subscribe to updates", bound: false },
        ],
    }
}

/// Method → `(fg, tinted bg)` for the endpoint pills.
pub fn method_colors(method: &str) -> (Hsla, Hsla) {
    match method {
        "GET" => (theme::accent(), theme::accent_tint()),
        "POST" => (theme::success(), theme::hexa(0x5CCB9A24)),
        "PUT" => (theme::warning(), theme::warning_tint()),
        "DELETE" => (theme::danger(), theme::danger_tint()),
        _ => (theme::text_muted(), theme::bg_raised()),
    }
}

// ── Modals: publish / settings / share / toast / mcp ────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishTab {
    Deploy,
    Export,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    Static,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    Providers,
    Mcp,
    Advanced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareRole {
    Edit,
    View,
}
impl ShareRole {
    pub fn label(self) -> &'static str {
        match self {
            ShareRole::Edit => "Can edit",
            ShareRole::View => "Can view",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkAccess {
    Restricted,
    Anyone,
}
impl LinkAccess {
    pub fn label(self) -> &'static str {
        match self {
            LinkAccess::Restricted => "Restricted",
            LinkAccess::Anyone => "Anyone with the link",
        }
    }
    pub fn desc(self) -> &'static str {
        match self {
            LinkAccess::Restricted => "Only invited people can open this",
            LinkAccess::Anyone => "Can view the published site",
        }
    }
}

/// Which Share-dialog dropdown is currently open (mock `shareMenu`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareMenu {
    InviteRole,
    LinkAccess,
    Collab(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastTone {
    Success,
    Idle,
}
impl ToastTone {
    /// `(icon, fg, tint bg, border)`.
    pub fn style(self) -> (&'static str, Hsla, Hsla, Hsla) {
        match self {
            ToastTone::Success => ("check-circle", theme::success(), theme::success_tint(), theme::hexa(0x5CCB9A59)),
            ToastTone::Idle => ("check", theme::text_soft(), theme::bg_hover(), theme::line_strong()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub tone: ToastTone,
    pub msg: SharedString,
}

/// An MCP server row (Settings → MCP servers).
#[derive(Debug, Clone)]
pub struct McpServer {
    pub id: u64,
    pub name: SharedString,
    pub meta: SharedString,
    pub on: bool,
}

pub fn seed_mcp() -> Vec<McpServer> {
    let mk = |id, name: &str, meta: &str, on| McpServer { id, name: name.to_string().into(), meta: meta.to_string().into(), on };
    vec![
        mk(1, "Payments \u{b7} Stripe", "npx stripe-mcp", true),
        mk(2, "Content \u{b7} Sanity", "https://mcp.sanity.io", true),
        mk(3, "Analytics", "npx analytics-mcp", false),
    ]
}

/// A collaborator on the Share dialog (static demo data).
pub struct Collaborator {
    pub initials: &'static str,
    pub name: &'static str,
    pub owner: bool,
    pub online: bool,
}

pub const COLLABORATORS: &[Collaborator] = &[
    Collaborator { initials: "RS", name: "Rana Saeed (you)", owner: true, online: true },
    Collaborator { initials: "MK", name: "Maya Kamal", owner: false, online: true },
    Collaborator { initials: "AH", name: "Ali Hassan", owner: false, online: false },
];

/// Build-log entry (Compile-log / Activity modal). Populated from real compiles
/// on `StudioApp` (see `record_compile`), newest first.
#[derive(Clone)]
pub struct CompileEntry {
    pub title: SharedString,
    pub ms: SharedString,
    pub time: SharedString,
    pub note: SharedString,
    pub note_tone: Tone,
    pub icon: &'static str,
    pub dot: fn() -> Hsla,
    pub detail: Option<SharedString>,
}

/// One-tap "try it" edit suggestions per element type (FR-9): `(label, the
/// instruction sent to the scoped-edit loop)`. A generic fallback covers the rest.
pub fn try_it_suggestions(element: &str) -> &'static [(&'static str, &'static str)] {
    match element {
        "Button" | "IconButton" => &[
            ("Make it primary", "make this button use the primary color"),
            ("Make it large", "make this button large"),
            ("Add an icon", "add a fitting leading icon to this button"),
        ],
        "Heading" => &[
            ("Make it bigger", "make this heading one level larger"),
            ("Center it", "center this heading"),
        ],
        "Text" => &[
            ("Muted", "make this text muted"),
            ("Emphasize", "make this text bold"),
            ("Center it", "center this text"),
        ],
        "Container" | "Row" | "Column" | "Stack" | "Grid" | "Card" => &[
            ("Add padding", "add comfortable padding inside this element"),
            ("Center contents", "center the contents"),
            ("Add a subtle background", "give this a subtle surface background"),
        ],
        "Image" => &[
            ("Round the corners", "give this image rounded corners"),
            ("Add a shadow", "give this image a soft shadow"),
        ],
        _ => &[
            ("Add spacing", "add a little spacing around this element"),
            ("Restyle it", "give this element a cleaner, more modern style without changing its content"),
        ],
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub text: SharedString,
}

// ════════════════════════════════════════════════════════════════════════════
// Design-system workspace (mock `dsWorkspace`): foundations, a component
// catalog, and a live-specimen inspector. Ported from `WebFluent Studio.dc.html`.
// ════════════════════════════════════════════════════════════════════════════

/// Which specimen-canvas tab is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsTab {
    Foundations,
    Components,
    Preview,
}
impl DsTab {
    pub const ALL: [DsTab; 3] = [DsTab::Foundations, DsTab::Components, DsTab::Preview];
    pub fn label(self) -> &'static str {
        match self {
            DsTab::Foundations => "Foundations",
            DsTab::Components => "Components",
            DsTab::Preview => "Preview",
        }
    }
}

/// What the inspector is editing (mock `state.dsSel.kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsSelKind {
    Color,
    Type,
    Comp,
}

/// A live design-system selection — a token or component `id`.
#[derive(Debug, Clone, PartialEq)]
pub struct DsSel {
    pub kind: DsSelKind,
    pub id: SharedString,
}

// ── foundations: color / type / radii tokens (mutable) ──────────────────────
#[derive(Debug, Clone)]
pub struct DsColorToken {
    pub id: SharedString,
    pub name: SharedString,
    pub val: u32,
    pub role: SharedString,
    pub group: SharedString,
}

/// Seed color tokens (mock `state.dsColorTokens`).
pub fn ds_color_tokens() -> Vec<DsColorToken> {
    let mk = |id: &'static str, name: &'static str, val: u32, role: &'static str, group: &'static str| DsColorToken {
        id: id.into(),
        name: name.into(),
        val,
        role: role.into(),
        group: group.into(),
    };
    vec![
        mk("c-accent", "Accent", 0x93C0F2, "Primary action \u{b7} links", "Brand"),
        mk("c-violet", "Violet", 0x8A6DF2, "Hero & brand gradient", "Brand"),
        mk("c-ink", "Ink", 0xF4F6FB, "Primary text", "Neutral"),
        mk("c-surface", "Surface", 0x14161C, "Panels & cards", "Neutral"),
        mk("c-base", "Base", 0x0D0E12, "App background", "Neutral"),
        mk("c-success", "Success", 0x5CCB9A, "Confirmed \u{b7} live", "Semantic"),
        mk("c-warning", "Warning", 0xE9BE6A, "Self-heal \u{b7} caution", "Semantic"),
        mk("c-danger", "Danger", 0xEF7A85, "Errors \u{b7} destructive", "Semantic"),
    ]
}

/// Group color-token indices by their `group`, preserving first-seen order
/// (Brand / Neutral / Semantic / any custom group), for the Foundations grid.
pub fn ds_color_groups(tokens: &[DsColorToken]) -> Vec<(SharedString, Vec<usize>)> {
    let mut groups: Vec<(SharedString, Vec<usize>)> = Vec::new();
    for (i, t) in tokens.iter().enumerate() {
        if let Some(g) = groups.iter_mut().find(|(name, _)| name == &t.group) {
            g.1.push(i);
        } else {
            groups.push((t.group.clone(), vec![i]));
        }
    }
    groups
}

/// The inspector's fixed color palette (swatch picker).
pub const DS_SWATCHES: &[u32] =
    &[0x93C0F2, 0x8A6DF2, 0x5CCB9A, 0xE9BE6A, 0xEF7A85, 0x7FB3B0, 0xF4F6FB, 0x9AA3B2];

#[derive(Debug, Clone)]
pub struct DsTypeToken {
    pub id: SharedString,
    pub name: SharedString,
    pub family: SharedString,
    pub size: f32,
    pub weight: u16,
    pub tracking: f32,
    pub sample: SharedString,
    pub ar: bool,
}
impl DsTypeToken {
    /// The embedded font family this style renders in.
    pub fn font(&self) -> &'static str {
        if self.ar {
            theme::FONT_ARABIC
        } else if self.family.as_ref() == "Space Grotesk" {
            theme::FONT_DISPLAY
        } else {
            theme::FONT_UI
        }
    }
    pub fn meta_text(&self) -> String {
        format!("{} \u{b7} {}", self.family, self.weight)
    }
}

/// Seed type tokens (mock `state.dsTypeTokens`).
pub fn ds_type_tokens() -> Vec<DsTypeToken> {
    let mk = |id: &'static str, name: &'static str, family: &'static str, size: f32, weight: u16, tracking: f32, sample: &'static str, ar: bool| DsTypeToken {
        id: id.into(),
        name: name.into(),
        family: family.into(),
        size,
        weight,
        tracking,
        sample: sample.into(),
        ar,
    };
    vec![
        mk("t-display", "Display", "Space Grotesk", 40.0, 700, -2.0, "Build by conversation", false),
        mk("t-title", "Title", "Space Grotesk", 26.0, 600, -1.0, "Choose your provider", false),
        mk("t-body", "Body", "Manrope", 16.0, 500, 0.0, "Review every change before it ships.", false),
        mk("t-label", "Label", "Manrope", 12.0, 700, 8.0, "PEOPLE WITH ACCESS", false),
        mk("t-arabic", "Arabic", "IBM Plex Sans Arabic", 24.0, 600, 0.0, "تعالَ نشرب قهوة سوا", true),
    ]
}

/// Font-weight options offered in the type inspector.
pub const DS_WEIGHTS: &[(u16, &str)] = &[(400, "Reg"), (500, "Med"), (600, "Semi"), (700, "Bold")];

#[derive(Debug, Clone, Copy)]
pub struct DsRadius {
    pub name: &'static str,
    pub val: f32,
}
pub const DS_RADII: &[DsRadius] = &[
    DsRadius { name: "Badge", val: 6.0 },
    DsRadius { name: "Control", val: 9.0 },
    DsRadius { name: "Row", val: 11.0 },
    DsRadius { name: "Card", val: 14.0 },
    DsRadius { name: "Modal", val: 26.0 },
];

// ── demo specimen state (mock `state.dsDemo`) ───────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsBtnVariant {
    Primary,
    Secondary,
    Ghost,
    Soft,
}
impl DsBtnVariant {
    pub const ALL: [DsBtnVariant; 4] = [DsBtnVariant::Primary, DsBtnVariant::Secondary, DsBtnVariant::Ghost, DsBtnVariant::Soft];
    pub fn label(self) -> &'static str {
        match self {
            DsBtnVariant::Primary => "Primary",
            DsBtnVariant::Secondary => "Secondary",
            DsBtnVariant::Ghost => "Ghost",
            DsBtnVariant::Soft => "Soft",
        }
    }
    /// `(fill, foreground, border)` for the specimen.
    pub fn style(self) -> (Option<Hsla>, Hsla, Option<Hsla>) {
        match self {
            DsBtnVariant::Primary => (Some(theme::accent()), theme::accent_contrast(), None),
            DsBtnVariant::Secondary => (Some(theme::bg_raised()), theme::text_soft(), Some(theme::line_strong())),
            DsBtnVariant::Ghost => (None, theme::text_soft(), None),
            DsBtnVariant::Soft => (Some(theme::accent_tint()), theme::accent(), None),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsBtnSize {
    Sm,
    Md,
    Lg,
}
impl DsBtnSize {
    pub const ALL: [DsBtnSize; 3] = [DsBtnSize::Sm, DsBtnSize::Md, DsBtnSize::Lg];
    pub fn label(self) -> &'static str {
        match self {
            DsBtnSize::Sm => "Small",
            DsBtnSize::Md => "Medium",
            DsBtnSize::Lg => "Large",
        }
    }
    /// `(height, horizontal pad, text size)`.
    pub fn metrics(self) -> (f32, f32, f32) {
        match self {
            DsBtnSize::Sm => (30.0, 14.0, 12.5),
            DsBtnSize::Md => (38.0, 18.0, 13.5),
            DsBtnSize::Lg => (46.0, 22.0, 15.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsChipKind {
    Accent,
    Skill,
    Success,
    Warning,
    Danger,
}
impl DsChipKind {
    pub const ALL: [DsChipKind; 5] = [DsChipKind::Accent, DsChipKind::Skill, DsChipKind::Success, DsChipKind::Warning, DsChipKind::Danger];
    pub fn label(self) -> &'static str {
        match self {
            DsChipKind::Accent => "Accent",
            DsChipKind::Skill => "Skill",
            DsChipKind::Success => "Success",
            DsChipKind::Warning => "Warning",
            DsChipKind::Danger => "Danger",
        }
    }
    /// `(foreground, tinted background)`.
    pub fn colors(self) -> (Hsla, Hsla) {
        match self {
            DsChipKind::Accent => (theme::accent(), theme::accent_tint()),
            DsChipKind::Skill => (theme::violet_soft(), theme::violet_tint()),
            DsChipKind::Success => (theme::success(), theme::success_tint()),
            DsChipKind::Warning => (theme::warning(), theme::warning_tint()),
            DsChipKind::Danger => (theme::danger(), theme::danger_tint()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsStatusTone {
    Compiled,
    Compiling,
    Idle,
    Attention,
    Error,
}
impl DsStatusTone {
    pub const ALL: [DsStatusTone; 5] =
        [DsStatusTone::Compiled, DsStatusTone::Compiling, DsStatusTone::Idle, DsStatusTone::Attention, DsStatusTone::Error];
    pub fn label(self) -> &'static str {
        match self {
            DsStatusTone::Compiled => "Compiled",
            DsStatusTone::Compiling => "Compiling",
            DsStatusTone::Idle => "Idle",
            DsStatusTone::Attention => "Attention",
            DsStatusTone::Error => "Error",
        }
    }
    pub fn color(self) -> Hsla {
        match self {
            DsStatusTone::Compiled => theme::success(),
            DsStatusTone::Compiling => theme::accent(),
            DsStatusTone::Idle => theme::text_muted(),
            DsStatusTone::Attention => theme::warning(),
            DsStatusTone::Error => theme::danger(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsAvatarTone {
    Blue,
    Violet,
    Teal,
    Accent,
}
impl DsAvatarTone {
    pub const ALL: [DsAvatarTone; 4] = [DsAvatarTone::Blue, DsAvatarTone::Violet, DsAvatarTone::Teal, DsAvatarTone::Accent];
    pub fn label(self) -> &'static str {
        match self {
            DsAvatarTone::Blue => "Blue",
            DsAvatarTone::Violet => "Violet",
            DsAvatarTone::Teal => "Teal",
            DsAvatarTone::Accent => "Accent",
        }
    }
    /// `(foreground, tinted background)`.
    pub fn colors(self) -> (Hsla, Hsla) {
        match self {
            DsAvatarTone::Blue => (theme::tone_blue(), theme::hexa(0x7FB0EE24)),
            DsAvatarTone::Violet => (theme::violet_soft(), theme::violet_tint()),
            DsAvatarTone::Teal => (theme::tone_teal(), theme::hexa(0x5CCB9A24)),
            DsAvatarTone::Accent => (theme::accent(), theme::accent_tint()),
        }
    }
}

/// Live-specimen demo state shared by the Components tab and the inspector.
#[derive(Debug, Clone)]
pub struct DsDemo {
    pub button_variant: DsBtnVariant,
    pub button_size: DsBtnSize,
    pub button_label: SharedString,
    pub toggle: bool,
    pub slider: u8,
    pub chip_kind: DsChipKind,
    pub chip_label: SharedString,
    pub input_ph: SharedString,
    pub status_tone: DsStatusTone,
    pub avatar_tone: DsAvatarTone,
    pub avatar_initials: SharedString,
    pub tabs_active: u8,
}
impl Default for DsDemo {
    fn default() -> Self {
        Self {
            button_variant: DsBtnVariant::Primary,
            button_size: DsBtnSize::Md,
            button_label: "Reserve a table".into(),
            toggle: true,
            slider: 62,
            chip_kind: DsChipKind::Accent,
            chip_label: "Live tonight".into(),
            input_ph: "you@example.com".into(),
            status_tone: DsStatusTone::Compiled,
            avatar_tone: DsAvatarTone::Blue,
            avatar_initials: "RS".into(),
            tabs_active: 0,
        }
    }
}

// ── component catalog (mock `dsCompCats`, reconstructed from the template) ───
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsCompKind {
    // live, editable specimens
    Button,
    IconButton,
    Input,
    Textarea,
    Chip,
    Toggle,
    Slider,
    Status,
    Avatar,
    Card,
    Seg,
    // schematic placeholders (not yet generated)
    Fab,
    Link,
    Select,
    Checkbox,
    Radio,
    Date,
    Table,
    List,
    Navbar,
    Sidebar,
    Crumbs,
    Pager,
    Accordion,
    Modal,
    Toast,
    Progress,
    Tooltip,
    Container,
    Grid,
    Stack,
    Divider,
    Carousel,
}
impl DsCompKind {
    /// Whether this component renders a live, editable specimen (vs. a
    /// not-yet-generated schematic placeholder).
    pub fn ready(self) -> bool {
        matches!(
            self,
            DsCompKind::Button
                | DsCompKind::IconButton
                | DsCompKind::Input
                | DsCompKind::Textarea
                | DsCompKind::Chip
                | DsCompKind::Toggle
                | DsCompKind::Slider
                | DsCompKind::Status
                | DsCompKind::Avatar
                | DsCompKind::Card
                | DsCompKind::Seg
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DsComp {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: DsCompKind,
}
impl DsComp {
    pub fn ready(&self) -> bool {
        self.kind.ready()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DsCompCat {
    pub cat: &'static str,
    pub items: &'static [DsComp],
}

use DsCompKind as K;
pub const DS_CATALOG: &[DsCompCat] = &[
    DsCompCat {
        cat: "Actions",
        items: &[
            DsComp { id: "button", label: "Button", kind: K::Button },
            DsComp { id: "iconbtn", label: "Icon button", kind: K::IconButton },
            DsComp { id: "fab", label: "Floating action", kind: K::Fab },
            DsComp { id: "link", label: "Link", kind: K::Link },
        ],
    },
    DsCompCat {
        cat: "Forms",
        items: &[
            DsComp { id: "input", label: "Input", kind: K::Input },
            DsComp { id: "textarea", label: "Textarea", kind: K::Textarea },
            DsComp { id: "select", label: "Select", kind: K::Select },
            DsComp { id: "checkbox", label: "Checkbox", kind: K::Checkbox },
            DsComp { id: "radio", label: "Radio group", kind: K::Radio },
            DsComp { id: "toggle", label: "Toggle", kind: K::Toggle },
            DsComp { id: "slider", label: "Slider", kind: K::Slider },
            DsComp { id: "date", label: "Date picker", kind: K::Date },
        ],
    },
    DsCompCat {
        cat: "Data display",
        items: &[
            DsComp { id: "avatar", label: "Avatar", kind: K::Avatar },
            DsComp { id: "chip", label: "Chip", kind: K::Chip },
            DsComp { id: "status", label: "Status pill", kind: K::Status },
            DsComp { id: "card", label: "Card", kind: K::Card },
            DsComp { id: "table", label: "Table", kind: K::Table },
            DsComp { id: "list", label: "List", kind: K::List },
        ],
    },
    DsCompCat {
        cat: "Navigation",
        items: &[
            DsComp { id: "navbar", label: "Navbar", kind: K::Navbar },
            DsComp { id: "sidebar", label: "Sidebar", kind: K::Sidebar },
            DsComp { id: "crumbs", label: "Breadcrumbs", kind: K::Crumbs },
            DsComp { id: "pager", label: "Pagination", kind: K::Pager },
            DsComp { id: "seg", label: "Tabs", kind: K::Seg },
            DsComp { id: "accordion", label: "Accordion", kind: K::Accordion },
        ],
    },
    DsCompCat {
        cat: "Feedback",
        items: &[
            DsComp { id: "modal", label: "Modal", kind: K::Modal },
            DsComp { id: "toast", label: "Toast", kind: K::Toast },
            DsComp { id: "progress", label: "Progress", kind: K::Progress },
            DsComp { id: "tooltip", label: "Tooltip", kind: K::Tooltip },
        ],
    },
    DsCompCat {
        cat: "Layout",
        items: &[
            DsComp { id: "container", label: "Container", kind: K::Container },
            DsComp { id: "grid", label: "Grid", kind: K::Grid },
            DsComp { id: "stack", label: "Stack", kind: K::Stack },
            DsComp { id: "divider", label: "Divider", kind: K::Divider },
            DsComp { id: "carousel", label: "Carousel", kind: K::Carousel },
        ],
    },
];

/// Look up a catalog component by `id`.
pub fn ds_comp(id: &str) -> Option<&'static DsComp> {
    DS_CATALOG.iter().flat_map(|c| c.items).find(|c| c.id == id)
}

/// `(ready, total)` component counts across the catalog.
pub fn ds_comp_counts() -> (usize, usize) {
    let all = DS_CATALOG.iter().flat_map(|c| c.items);
    let total = all.clone().count();
    let ready = all.filter(|c| c.ready()).count();
    (ready, total)
}

/// Planned-variant chips shown for a not-yet-generated component.
pub fn ds_planned_variants(kind: DsCompKind) -> &'static [&'static str] {
    match kind {
        DsCompKind::Select => &["Single", "Multi", "Searchable", "Grouped"],
        DsCompKind::Checkbox => &["Default", "Indeterminate", "Card", "Disabled"],
        DsCompKind::Radio => &["Vertical", "Horizontal", "Card", "Disabled"],
        DsCompKind::Date => &["Single date", "Range", "With time", "Inline"],
        DsCompKind::Table => &["Simple", "Sortable", "Selectable", "Sticky header"],
        DsCompKind::List => &["Bulleted", "Icon", "Two-line", "Interactive"],
        DsCompKind::Navbar => &["Solid", "Transparent", "With search", "Sticky"],
        DsCompKind::Sidebar => &["Expanded", "Collapsed", "Icon rail", "Nested"],
        DsCompKind::Modal => &["Dialog", "Sheet", "Confirm", "Fullscreen"],
        DsCompKind::Toast => &["Success", "Info", "Warning", "Error"],
        DsCompKind::Progress => &["Linear", "Circular", "Stepped", "Indeterminate"],
        _ => &["Default", "Compact", "With icon", "Disabled"],
    }
}
