# WebFluent Studio — Implementation Plan

Version 1.0 · 2026-07-13 · Companion to the Product & Business Requirements Document (PBRD v0.1)

This plan translates the PBRD into an engineering roadmap for the chosen stack:
**Rust + GPUI + gpui-component (wry webview) + the `webfluent` compiler crate + multi-provider AI (BYOK)**.

---

## 0. Current Status & Reconciliation (updated 2026-07-24)

**This document is the single source of truth.** A parallel plan grew in the engine
repo (`spec/STUDIO_INTEGRATION_PLAN.md`) while building the engine workstream and the
studio; it diverged on crate layout, edit model, and milestone numbering. It is now
**superseded** and retained only as an engine-workstream history. Three reconciliation
decisions (2026-07-24):

1. **Crate layout → refactor to the canonical §3.1 four-crate layout.** The studio was
   rewritten as a UI port and grew *studio-internal* compile/document/preview logic; the
   M0-scaffold `core`/`preview` were bypassed and `wf-core` no longer compiles (it predates
   the engine's `WebFluentError::EditError`). Plan: lift the studio-internal document/compile/
   proposal/self-heal logic into GPUI-free **`core`**, preview into **`preview`**; **revive
   and depend on `wf-ai`** (it builds and already implements §4.3 — six providers, two
   protocols, Gemini via its OpenAI-compat endpoint); studio keeps only GPUI.
2. **Edit model → the canonical §4.5 chips/proposal/review** (the studio UI is already built
   for it: `ChipKind::{Text,Style,Structure,Behavior}`, the Review panel, `Permission::Review`).
   The model returns full `.wf` (generation) or a replacement subtree (edit) as **plain fenced
   text** — no provider tool-use — so all six providers work uniformly. `apply_edits`/`EditOp`
   from the engine workstream is **retained as the *apply layer*** underneath chips (canonical
   §4.5's "apply accepted chips' span edits in reverse span order"), not as a model-output format.
3. **Milestones → the canonical M0–M5** below. The engine repo's M1–M6 numbering is retired.

### Crosswalk — work already done (do not rebuild), mapped onto this plan

| Built & tested (engine repo + `crates/studio`) | Canonical home | Status |
| - | - | - |
| Source `Span {start,end,line,col}` through lexer→AST | §5 **W1** | ✅ done |
| Deterministic `NodeId` (`Home:2.0.3`) + `data-wf-node` (studio-mode) + node map | §5 **W2/W3**, §4.2 | ✅ done |
| `apply_edits(source, &[EditOp])` — reverse-span span-edit engine + reparse-guard | §4.5 apply layer | ✅ done |
| `validate_semantics` (undefined component / route / duplicate) + structured `Diagnostic`s | §4.4 validate step, §5 **W5** | ✅ done (M4.E) |
| In-process `compile_studio` → `CompiledSite`; `wf://` serving; embedded webview; recompile-reload | §4.1 preview | ✅ done (studio-internal → lift to `preview`) |
| `WfProject` (multi-file sources + merged offset space + node map, keep-last-good) | §3.3 `Document` | ✅ built (studio-internal → lift to `core`) |
| Click-to-code (bridge `data-wf-node` → `resolve_node` → highlight) | §4.2, **M2** | ✅ done |
| Inspector (color/size/weight/align/bg/radius → `apply_edits`), outline, add-blocks | **M4** Quick Inspector | ✅ done (direct AST mutation, no LLM) |
| Review/chips UI surface (`ChipKind`, `review_items`, before/after) | §4.5 | 🟡 UI built on mock data — wire to real AST-diff in **M3** |
| `wf-ai`: `Provider` trait, Anthropic + OpenAI-compat adapters, SSE, dedicated-thread reqwest | §4.3, **M1** | ✅ owned by studio; generation loop + language card + KeyStore added in **M1** |

**Net position on the canonical scheme (updated 2026-07-24):** **M0–M4 done, and M5's
code parts done** (M5.1 FR-6 audit, M5.2 onboarding wiring + test-connection, M5.3 packaging
metadata + `docs/PACKAGING.md`). The **R0 four-crate refactor** landed (studio → GPUI-free
`wf-core` + `wf-preview` + `wf-ai` → `webfluent`), and `wf-core`/`wf-ai` carry the whole offline
brain — generation, scoped edits, diff/chips/proposal, history, self-heal + design-freeze, and
the eval harness — all hermetically tested; the studio wires each to the real UI. **The
pure-code product is complete.** What remains is *operational*: the per-provider eval pass (needs
BYO keys → pick default models), building/signing the bundles (add `assets/icon.png` + a GPL-3.0
`LICENSE`), the manual QA sweep, and shipping to alpha users. The M3.5 before/after scrub and the
real model picker (previously deferred) are now built.

### Build order (done → next)

- **R0 — crate refactor** ✅ — the 4-crate boundary (compile→`wf-core`, `wf://` serving→
  `wf-preview`; studio owns `wf-ai`; stale scaffold retired).
- **M1 — generation loop** ✅ — `ScriptedProvider`/`collect_text`, `generate_page`,
  `LANGUAGE_CARD`, keychain `KeyStore`, app wiring, `wf-evals`.
- **M3 — scoped edits + Visual Diff Review** ✅ — `edit_node` → `diff` → chips → review panel →
  `Proposal::apply_accepted` → reload, with inline re-prompt (FR-8). M3.5 before/after scrub ✅ —
  `WfProject::compile_variant` → `wf_preview::DIFF_SHELL` two-iframe (`/base`, `/proposal`)
  cursor-wipe, entered/left on proposal create / apply / discard.
- **M4 — guardrails + P1** ✅ — history/undo/restore (FR-14), runtime self-heal + design-freeze
  via a style fingerprint (FR-19–22), try-it chips (FR-9), RTL/device toggles + the engine's W4
  logical-CSS audit (FR-11/12).
- **M5 — alpha hardening** — code parts ✅ (FR-6 "no code visible" audit, onboarding wiring +
  test-connection, packaging metadata + PACKAGING.md). Operational remainder: per-provider eval
  pass to pick default models,
  packaging, the "zero code visible" audit (FR-6).

---

## 1. v1 Product Decisions (deltas from the PBRD)

| Decision | PBRD reference | Rationale |
| - | - | - |
| **v1 is free, bring-your-own-key (BYOK)** | Replaces FR-15 (EGP payments) | Payments deferred by owner decision. User pastes an API key for any supported provider. Removes Paymob/EasyKash dependency from the critical path entirely. |
| **Six AI providers at launch** | Extends §9 dependency on "LLM API (Claude)" | Anthropic (Claude), OpenAI, Google Gemini, DeepSeek, Moonshot (Kimi), Zhipu (GLM). BYOK makes multi-provider nearly free to support (see §4.3). |
| Everything else | FR-1…FR-14, FR-19…FR-22, all NFRs | Unchanged. **No code view, ever** remains a hard invariant. |

---

## 2. Validated Stack

All verified available on crates.io as of 2026-07-13:

| Crate | Version | Role | License |
| - | - | - | - |
| `gpui` | 0.2.2 | Application UI framework (windowing, rendering, entities) | Apache-2.0 |
| `gpui-component` | 0.5.1 | 60+ widgets: dock/panels, inputs, dropdowns, popovers, virtualized lists — plus the `webview` feature | Apache-2.0 |
| `lb-wry` | 0.53.3 | Longbridge's wry fork used by gpui-component's `WebView` (system webview embedding) | Apache-2.0/MIT |
| `webfluent` | 0.5.0-alpha | The language: lexer → parser (AST) → linter → codegen (HTML/CSS/JS/SSG/PDF/slides) + `Template` API | **GPL-3.0** |
| `gix` (or `git2`) | latest | Hidden version-history store (FR-14), enables v2 git-remote for free | Apache/MIT |
| `keyring` | latest | API keys in OS credential store (Secret Service / Keychain / Credential Manager) | Apache/MIT |
| `reqwest` + `tokio` + `serde` | latest | HTTP/streaming for AI providers | Apache/MIT |

**Two findings that shape this plan** (from inspecting `webfluent 0.5.0-alpha` source):

1. **The AST has no source spans and no stable node IDs.** `parser::UIElement` carries component/args/children but no location. Codegen emits no `data-*` identity attributes. Click-to-select (FR-3), scoped edits (FR-4), and per-chip apply (FR-7) all require node identity — so a **webfluent crate workstream is a first-class part of this plan** (§5). We own the crate, so this is scheduling, not risk.
2. **Diagnostics are already structured** (`error::Diagnostic { message, file, line, column, hint }`). This is exactly the machine-readable input the self-healing loop (FR-19/20) needs to feed back to the model.

**Licensing note:** `webfluent` is GPL-3.0. Linking it makes wf-studio's distributed binary GPL-3.0 unless we relicense. Since we hold copyright on webfluent, decide early: either ship wf-studio as GPL-3.0, or dual-license webfluent (e.g. GPL + commercial exception for wf-studio). Decide in M0; it affects nothing technically but affects distribution terms.

---

## 3. Architecture

### 3.1 Cargo workspace

```
wf-studio/
├── Cargo.toml                 # [workspace]
├── crates/
│   ├── studio/                # bin: GPUI app — windows, panels, all chrome
│   ├── core/                  # lib: project model, document state, AST-diff,
│   │                          #      proposals/chips, history, self-heal engine
│   ├── ai/                    # lib: provider abstraction + adapters, prompt
│   │                          #      templates, context pruning, key storage
│   └── preview/               # lib: webview host, wf:// protocol server,
│                              #      JS bridge (select/hover/errors/scrub IPC)
├── evals/                     # prompt suite + scoring harness (see §4.4)
└── docs/
```

`core`, `ai`, `preview` stay GPUI-free (plain async Rust) so they're testable headless and reusable if the UI layer ever changes. `studio` is the only crate that touches gpui/gpui-component.

### 3.2 The core loop (data flow)

```
user prompt ──► ai::generate ──► .wf source ──► webfluent parse+lint
                                                    │ Diagnostics? ─► self-heal loop (bounded)
                                                    ▼
                                       core::Document { source, AST, node-id map }
                                                    ▼
                                       webfluent codegen ─► html/css/js artifacts
                                                    ▼
                                       preview: wf://project/… custom protocol
                                                    ▼
                                       webview renders; bridge JS reports
                                       clicks / hovers / runtime errors ──► studio
```

Edits run the same pipe but produce a **Proposal** (shadow document + chip list) instead of mutating the live document; "Apply accepted" splices chips into the canonical source and commits a history checkpoint.

### 3.3 State model (the entities that matter)

- `Project` — folder on disk: `project.json`, `src/*.wf`, `assets/`, hidden history repo. Local-first (NFR-1, NFR-5).
- `Document` — canonical `.wf` source + parsed `Program` + `NodeId ↔ span` table. Single source of truth; the AST is always *derived* from source, never edited without re-serializing.
- `Selection` — `Option<Vec<NodeId>>` (v1: single node; the type allows multi-select later).
- `Proposal` — `{ base_revision, new_source, chips: Vec<Chip>, status }`. A `Chip` is `{ node_id, kind: Text|Style|Structure|Behavior, human_label, accepted: bool }` — human labels only, never syntax (FR-6).
- `CompileStatus` — `Idle | Generating | Compiling | SelfHealing{attempt} | Compiled | NeedsAttention` → drives the top-bar badge (FR-13, FR-21, FR-22).
- `ProviderConfig` — `{ provider, model, api_key_ref }`; key material lives only in the OS keyring.

---

## 4. Core Subsystem Designs

### 4.1 Preview pipeline (FR-2, FR-11, FR-12 · NFR-1)

- Serve compiled artifacts over a **wry custom protocol** (`wf://app/index.html`, `wf://app/app.js`, …) straight from an in-memory artifact store. No temp files, no localhost port, works offline (NFR-1).
- `gpui-component`'s `WebView` element hosts the wry webview inside the GPUI layout; we construct the `wry::WebView` ourselves (that's the crate's contract) with: custom protocol handler, init-script (the bridge), and IPC handler.
- **RTL/LTR toggle (FR-11):** bridge sets `document.documentElement.dir` + recompile with direction config; audit of logical-vs-physical CSS lives in the webfluent workstream (NFR-3).
- **Device toggle (FR-12):** studio constrains the WebView element's bounds to 375 / 768 / full-width. Pure layout, no webview tricks.
- **Before/after scrub (FR-5):** ✅ on a proposal, `WfProject::compile_variant` shadow-compiles the unaccepted source and the webview loads a minimal *diff shell* (`wf_preview::DIFF_SHELL`) with two overlaid iframes — `/base` (live document) under `/proposal` (variant), each served fully self-contained (`self_contained` inlines CSS/JS so the frames never cross-load assets). A `clip-path` wipe follows the cursor via a transparent capture overlay (the iframes are `pointer-events:none`); `window.__setClip(pct)` is also exposed so a host/GPUI control can drive the split over IPC. One webview instance; apply/discard reloads the live document.

### 4.2 DOM ↔ AST node identity (FR-3, FR-4, FR-7 — the reconciliation engine)

- webfluent gains spans + stable `NodeId`s and (behind a compile option) emits `data-wf-id="…"` on every generated element (§5). Studio keeps the `NodeId → span → AST node` table per compile.
- Bridge JS: `click` → nearest ancestor with `data-wf-id` → IPC to studio; studio draws the selection state and instructs the bridge to apply outline/hover classes (injected stylesheet, not inline styles, so we never contaminate user output).
- `NodeId` scheme: **structural path + sibling disambiguator** (e.g. hash of `Page[0]/Container[0]/Button[2]` + component name). Deterministic across recompiles of identical structure; a background re-parse re-syncs IDs after every accepted change (mitigates the AST-drift risk from PBRD §11 — drift only threatens *externally edited* files, which the Studio never produces in v1).

### 4.3 AI provider layer (BYOK) — six providers, two protocols

DeepSeek, Kimi (Moonshot), GLM (Zhipu), and OpenAI all speak the **OpenAI chat-completions protocol**; Gemini exposes an OpenAI-compatible endpoint as well. Anthropic has its own Messages API. So:

```rust
trait Provider {  // crates/ai
    fn stream_chat(&self, req: ChatRequest) -> BoxStream<ChatDelta>;
}
// impls: AnthropicAdapter, OpenAiCompatAdapter { base_url, auth_style }
```

| Provider | Adapter | Base URL |
| - | - | - |
| Claude | Anthropic (native, with prompt caching) | api.anthropic.com |
| OpenAI | OpenAI-compat | api.openai.com/v1 |
| DeepSeek | OpenAI-compat | api.deepseek.com/v1 |
| Kimi | OpenAI-compat | api.moonshot.ai/v1 |
| GLM | OpenAI-compat | open.bigmodel.cn/api/paas/v4 |
| Gemini | OpenAI-compat | generativelanguage.googleapis.com/v1beta/openai |

- Settings dropdown (in-place, with Advanced section — PBRD §4.1): provider picker, key field (stored via `keyring`, masked, "Test connection" button), model picker with sane defaults per provider.
- Keys never leave the machine except in the auth header of the provider call itself (NFR-5).
- **Context pruning (NFR-2):** generation sends system prompt + user prompt only; scoped edits send the selected node's source slice + a compact ancestry/tokens digest — not the whole project. System prompt is cached (Anthropic prompt caching; OpenAI-side caching is automatic) since it embeds the language spec and dominates token count.
- "claude code" note: v1 talks to the Anthropic API directly. Driving a locally installed Claude Code CLI as an optional agent backend is a v2 idea, recorded in §9.

### 4.4 Making models write valid WebFluent (the quiet hard problem)

No public model has seen WebFluent. Reliability comes from the harness, not the model:

1. **System prompt = language card**: compact grammar summary, the 50+ component catalog with signatures, style-block rules, 3–5 few-shot examples (Arabic + English content), hard rules (logical CSS properties only, no physical left/right — NFR-3).
2. **Validate → repair loop**: every model output is parsed + linted before it ever reaches the preview. On `Diagnostic`s, re-prompt with the diagnostic text (`line/column/hint`) — this is the same machinery as FR-19/20 self-healing, built once in `core`.
3. **Output contract**: model returns full `.wf` source (generation) or a replacement subtree (scoped edit) in fenced blocks; `ai` extracts and discards prose. The user never sees any of it (FR-6).
4. **Eval harness (`evals/`)**: ~30 golden prompts (landing pages, menus, portfolios; Arabic and English) scored on parse rate, lint-clean rate, repair rounds needed, and scoped-edit containment. Run per provider; this is how we choose default models and iterate the language card with evidence, and it's the regression gate for prompt changes.

### 4.5 Proposals, AST diff, and chips (FR-5, FR-7, FR-8)

- Diff the base AST vs. proposal AST keyed by `NodeId`: property-level changes (text arg, style property, modifier) → one chip each; node insert/remove/move → one structural chip. Labels are generated from a fixed template table ("Heading text changed", "Button color → primary"), optionally polished by the model later — but never syntax.
- **Apply accepted** re-serializes: start from base source, apply accepted chips' span edits (reverse span order), re-parse, recompile, history checkpoint (FR-14). Rejected chips simply never get spliced. Reset discards the proposal.
- **Inline re-prompt (FR-8)** mutates the pending proposal: same scoped-edit call, but the current proposal subtree is the base and chip states are preserved where the node survives.
- v1 chip granularity guardrail: if a proposal's diff produces > ~20 chips, group by node ("This section was redesigned — accept/reject as one"). Prevents chip-soup on big edits.

### 4.6 Self-healing (FR-19…FR-22 · NFR-7)

Two error sources, one engine:

- **Compile-time**: `webfluent` `Diagnostic`s from parse/lint/codegen.
- **Runtime**: bridge init-script installs `window.onerror` + `console.error` + unhandled-rejection hooks in the preview; reports over IPC.

Engine: bounded loop (default **3 attempts**) → error + offending node source → model → candidate fix → **design-freeze validation**: AST-diff the fix against current source; if any `StyleBlock`, theme token, or layout-affecting arg changed, the fix is *rejected* and retried with a stricter instruction (NFR-7, enforced structurally — this is the AST paying rent, not a prompt-hope). Every attempt is logged to the activity log surfaced behind the status badge (FR-21, human wording, no stack traces). After N failures → non-blocking "needs attention" notice (FR-22); badge shows amber, app stays usable.

### 4.7 History (FR-14) and storage

- `gix` repo in a hidden dir (`.wfstudio/history`), auto-commit on every accepted change / generation with a human summary ("Changed hero heading") as the message. UI: "History" panel with timestamped entries + one-click restore. Zero git vocabulary (FR-14); v2 "connect a git remote" (FR-17) becomes a push of an existing repo rather than a migration.

---

## 5. webfluent Crate Workstream (parallel track)

Changes Studio needs from the language crate — each small, none speculative:

| # | Change | Needed by | Milestone |
| - | - | - | - |
| W1 | Thread `Span { line, col, byte_range }` through lexer → AST nodes (`UIElement`, `StyleProperty`, args) | node identity, chip splicing | M2 |
| W2 | Deterministic `NodeId` (structural path hash) computed at parse; expose `Program::node_table()` | FR-3/4/7 | M2 |
| W3 | `CompileOptions { emit_node_ids: bool, direction: Ltr\|Rtl\|Auto }` — emit `data-wf-id` only for Studio builds; production output stays clean | FR-3, FR-11 | M2 |
| W4 | Audit `codegen/css.rs` + themes for physical properties → logical (`margin-inline-start` etc.) | NFR-3, FR-11 | M4 |
| W5 | Machine-readable diagnostics already exist — add error *codes* for the self-heal prompt table | FR-19/20 | M4 |
| W6 | (nice-to-have) `render_html` split into cacheable per-artifact codegen to speed watch-recompiles | NFR-4 | M4+ |

Pin `webfluent` by exact version per milestone; Studio CI runs the eval suite against each webfluent bump.

---

## 6. Milestones

Sized for one experienced Rust developer full-time; webfluent workstream (§5) interleaves. Weeks are estimates, exit criteria are not.

### M0 — Platform spike + app shell (≈ 2 weeks) — *de-risk before building*
- Workspace scaffold (`studio`/`core`/`ai`/`preview`), CI (fmt, clippy, test).
- GPUI window with the three-zone layout: top bar (status badge, settings dropdown), preview canvas, bottom prompt dock.
- **Spike A (the go/no-go):** gpui-component `WebView` + lb-wry rendering webfluent output on this dev machine. Linux/Wayland child-webviews are the #1 platform risk (dev box is Wayland; webkit2gtk is installed). Fallback ladder if broken: run under XWayland → detached preview window (plain wry/gtk window beside the GPUI window) → treat Windows/macOS as alpha platforms with Linux-X11 dev mode. **Decide and write it down.**
- **Spike B:** Arabic text shaping/bidi in GPUI inputs (users will type Arabic prompts into Studio chrome).
- Licensing decision from §2.
- **Exit:** a `.wf` file from disk compiles and renders inside (or beside) the Studio window on Linux; webview approach documented.

### M1 — Generation loop (≈ 2 weeks) → FR-1, FR-2, FR-13
- `ai` crate: `Provider` trait, Anthropic + OpenAI-compat adapters, streaming, `keyring` storage, settings UI (provider/key/model, test-connection).
- Language card system prompt v1; generation → validate → repair loop v0 (parse/lint errors only, 3 attempts).
- Custom-protocol artifact serving; auto-reload on recompile; status badge states wired end-to-end.
- First 10 eval prompts running in `evals/`.
- **Exit:** *Flow A*: cold start → prompt ("مطعم صغير في القاهرة…") → live preview **< 60 s** (NFR-4), with a provider key as the only setup.

### M2 — Node identity + click-to-select (≈ 2–3 weeks) → FR-3, groundwork for FR-4/5/7
- webfluent W1–W3 (spans, NodeIds, `data-wf-id` emission).
- Bridge v1: click/hover capture, selection outline via injected stylesheet, IPC both ways.
- `core` node table: `NodeId ↔ span ↔ AST node`; selection state in studio.
- **Exit:** click any rendered element → outlined in preview, correct node highlighted in Studio state; survives recompile; zero mis-selections across the eval projects.

### M3 — Scoped edits + Visual Diff Review (≈ 3 weeks) → FR-4, FR-5, FR-7, FR-8 — *the product's heart*
- Scoped-edit prompting with context pruning (NFR-2; measure the token reduction, target 60%).
- Proposal engine: AST diff → chips, shadow compile, diff-shell scrub UI, chip list panel (accept/reject), Apply accepted / Reset, inline re-prompt.
- Containment test in evals: **no source changes outside the selected node's span** across the suite (the PBRD's scoped-edit-accuracy metric).
- **Exit:** *Flow C* end-to-end on a representative project without reconciliation failure (Release Criterion #3).

### M4 — Guardrails + P1 features (≈ 2 weeks) → FR-9…FR-12, FR-14, FR-19…FR-22
- Quick Inspector (color, font-size) as direct AST mutation — no LLM round-trip (FR-10).
- "Try it" suggestion chips from a per-component table (FR-9).
- RTL/LTR + device-size toggles (FR-11/12) + webfluent W4 logical-properties audit.
- Runtime-error capture → full self-healing engine with design-freeze validation, activity log, needs-attention notice (FR-19–22, NFR-7).
- History checkpoints + restore panel (FR-14).
- **Exit:** all MVP P0/P1 FRs demoable; self-heal never alters a style block across eval-suite fault injections.

### M5 — Alpha hardening (≈ 2 weeks) → Release Criteria
- First-run onboarding: pick provider → paste key → three sample prompts.
- Full eval pass per provider; pick recommended default models; RTL verification against an Arabic test project (Release Criterion #5).
- Packaging: Linux (AppImage/.deb), Windows (WebView2 bootstrap), macOS (dmg) — signed where feasible; local-only crash/error log.
- Manual QA sweep of PBRD §10; **zero code visible anywhere** audit (FR-6).
- Ship to 5–10 alpha users; instrument nothing that violates NFR-5 (feedback via conversation, not telemetry).

**Total: ~13 weeks to alpha.** Cut lines if pressed: FR-8 inline re-prompt and FR-9 chips are the safest deferrals; M0–M3 are not compressible without hollowing out the product.

---

## 7. Requirement Traceability

| Requirement | Where | Milestone |
| - | - | - |
| FR-1, FR-2 | §4.1, §4.3, §4.4 | M1 |
| FR-3 | §4.2 | M2 |
| FR-4, FR-5, FR-7, FR-8 | §4.5 | M3 |
| FR-6 (no code, ever) | invariant in every UI surface; audited | M5 gate |
| FR-9, FR-10 | M4 features | M4 |
| FR-11, FR-12, FR-13 | §4.1, badge | M1/M4 |
| FR-14 | §4.7 | M4 |
| FR-15 | **replaced by BYOK** (§1) | — |
| FR-19…FR-22 | §4.6 | M1 (seed) / M4 (full) |
| NFR-1…NFR-7 | §4.1–§4.6, §5 W4 | continuous; eval-gated |

---

## 8. Top Risks & Mitigations

1. **Webview embedding on Linux/Wayland** — gpui-component's WebView is experimental; upstream wry child-views don't support Wayland. *Mitigation:* M0 Spike A with a written fallback ladder (XWayland → detached preview window → Windows/macOS-first alpha). The lb-wry fork exists precisely for GPUI, so odds are reasonable — but we prove it in week 1, not week 10.
2. **Model fluency in WebFluent** — no model has seen the language. *Mitigation:* §4.4 harness (language card + validate/repair + eval suite); default-model choice driven by eval scores, not vibes.
3. **Chip-splicing correctness** (span edits on re-serialized source) — subtle off-by-one territory. *Mitigation:* property-level chips first, structural chips second; heavy round-trip tests (`source → AST → edit → source → AST` equality) in `core`.
4. **webfluent alpha churn** — Studio and language evolve together. *Mitigation:* exact-version pins, eval suite as compatibility gate, §5 table as the only sanctioned change list.
5. **GPL-3.0 propagation** from webfluent into the shipped app. *Mitigation:* M0 licensing decision (we own the copyright; dual-licensing is available).
6. **GPUI 0.2 API churn** — young public crate. *Mitigation:* pin minor versions; gpui-component tracks gpui for us; upgrade on milestone boundaries only.

---

## 9. Deferred (v2+) — recorded so they shape, but don't enter, v1

- EGP payments via Paymob/EasyKash (original FR-15) — re-enters when pricing exists.
- Figma import (FR-16); git remotes UI (FR-17 — the hidden gix repo is already the substrate); public share links via Cloudflare Workers + R2 (FR-18); PDF preview toggle (webfluent already renders PDF — expose when Paged-Media UX is designed); multi-select editing; Claude Code CLI as an optional local agent backend; Arabic localization of Studio chrome itself (M0 Spike B informs feasibility).
