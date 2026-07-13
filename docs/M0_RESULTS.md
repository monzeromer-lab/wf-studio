# M0 Results — Platform Spike Verdicts

Status: **go decision reached** · 2026-07-13 · Companion to `docs/IMPLEMENTATION_PLAN.md` §6 M0

## Verdict summary

| Question | Verdict | Evidence |
| - | - | - |
| webfluent 0.5.0-alpha pin + Arabic sample syntax | **✅ PASS** | `cargo test -p wf-core` — 3/3, Arabic survives codegen |
| gpui 0.2.2 builds on bare system (no gtk dev libs) | **✅ PASS** | `cargo build -p gpui` clean, 1m41s (pure-Rust Wayland/Vulkan stack) |
| `wf-studio` binary builds + links on Wayland | **✅ PASS** | after `libxkbcommon-x11-dev` (below) + 4 gpui-0.2 trait-import fixes |
| Detached preview (tao+wry) on Wayland (**Spike A go/no-go**) | **✅ PASS** | `wf-studio` launched on GNOME/Wayland; detached window renders compiled HTML — two clean runs, bridge `page-loaded`, no runtime errors |
| `wf://` custom protocol on webkitgtk | **✅ PASS** | in-memory `index.html` served + loaded (proven by the render above) |
| Prompt → compiled preview loop end-to-end | **✅ PASS** | `Document::compile` → `ArtifactStore.publish` → `wf://` serve → webview render, verified via the bridge IPC |
| Embedded child webview under X11/XWayland | _deferred_ | Linux v1 ships **detached** (see below); embedded is the macOS/Windows path |
| Arabic shaping/bidi in GPUI chrome (labels + input) — **Spike B** | ⏳ _needs eyeball_ | Arabic labels + placeholder wired in `top_bar`/`prompt_dock`; GNOME/Wayland blocks headless screenshots (D-Bus `AccessDenied`, no `wlr-screencopy`), so confirm visually on next manual launch |

## Build prerequisites (Linux)

The wry/tao preview stack needs GTK/WebKit **dev** headers (runtime libs alone
are not enough — `glib-sys` fails at `glib-2.0.pc`). Ubuntu/Debian:

```bash
sudo apt install libglib2.0-dev libgtk-3-dev libwebkit2gtk-4.1-dev libsoup-3.0-dev
```

Verified available on this machine (Ubuntu 26.04): webkit2gtk-4.1-dev 2.52.3.

**GPUI needs one more dev lib.** The `gpui` *crate* compiles with none of the
above, but linking the `wf-studio` **binary** pulls in `-lxkbcommon-x11` (blade
X11 keyboard path). Runtime lib ships by default; the dev symlink does not:

```bash
sudo apt install libxkbcommon-x11-dev
```

`libxkbcommon-dev` was already present; only the `-x11` companion was missing.
Vulkan runtime (`libvulkan.so.1`) + ICDs (incl. `lvp` software fallback) are
present, so GPUI starts on this box once the binary links.

## Architecture consequences (locked during M0 planning)

- **Linux preview = detached window** (`crates/preview/src/detached.rs`): tao event
  loop on a dedicated thread (`with_any_thread`), wry webview built into the tao
  window's gtk vbox via `build_gtk` — the only route that works on **both**
  Wayland and X11. Raw-handle child webviews (`build_as_child`) are X11-only
  upstream; `lb-wry 0.53.3` is a verbatim republish of upstream wry with no
  GPUI patches.
- **macOS/Windows preview = embedded** (gpui-component `WebView`), left as a
  seam; published `gpui-wry` README: "Only supports macOS and Windows
  currently", renders **on top** of GPUI content — no chrome may overlap the
  preview rect, overlays must hide the webview.
- **Serving**: `wf://localhost/<path>` custom protocol from an in-memory
  versioned `ArtifactStore`; reloads cache-bust with `?v=<version>`.
- If embedded-on-Linux lands upstream (gpui-component git main has an
  unreleased `build_gtk` example), it slots in behind the same seam.

## Upstream watch

- `longbridge/gpui-component` main branch: Linux webview example
  (`examples/webview`, gtk::Fixed + `build_gtk`) — unpublished as of 2026-07-13.
- `gpui` 0.2.x: whether `Window` exposes `HasWindowHandle` (needed for
  embedded `build_as_child`) — checked during task 5 spike: _pending_.

## Open owner decisions

- **License**: wf-studio links GPL-3.0 `webfluent` → either ship wf-studio as
  GPL-3.0 or dual-license webfluent (owner holds copyright). No LICENSE file
  until decided.
