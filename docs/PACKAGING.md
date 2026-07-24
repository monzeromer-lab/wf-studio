# Packaging WebFluent Studio

Bundle the `wf-studio` binary into a native installer per platform (canonical plan
§6 M5). Bundle metadata lives in `crates/studio/Cargo.toml`
(`[package.metadata.bundle]`); shared package fields (license, authors, description)
are in the workspace `[workspace.package]`.

## Runtime dependencies (what the machine needs to *run* the app)

The preview is a **wry** webview embedded in the GPUI window, so each platform needs
its system web engine:

| Platform | Web engine | Ship / require |
| - | - | - |
| **Linux** | webkit2gtk (GTK3) | Depend on `libwebkit2gtk-4.1` + GTK3; the app forces **X11** (XWayland under Wayland — see `main.rs` / the `never-remove-gpui` design note). |
| **Windows** | WebView2 | Bundle the **WebView2 Evergreen bootstrapper**; the MSI should install it if absent. |
| **macOS** | WKWebView | Built into macOS — nothing to ship. |

## Build

```bash
cargo build --release -p wf-studio
```

## Bundle (cargo-bundle)

```bash
cargo install cargo-bundle
cargo bundle --release -p wf-studio      # → .app (macOS) · .deb (Linux) · .msi (Windows)
```

- **AppImage** (Linux): cargo-bundle emits `.deb`; for an AppImage, run the release
  binary through `cargo-appimage` or `appimagetool` (pull in webkit2gtk + GTK libs).
- **dmg** (macOS): build the `.app`, then `hdiutil create` / `create-dmg`.
- **Signing**: codesign + notarize on macOS, and Authenticode-sign the MSI on Windows,
  where a certificate is available.

## Before the first release (TODO)

1. **Icon** — add `assets/icon.png` (512×512) and uncomment `icon = [...]` in the
   bundle metadata (cargo-bundle derives `.icns`/`.ico` from it).
2. **LICENSE file** — the app links the **GPL-3.0** `webfluent` compiler, so the
   distributed binary is **GPL-3.0-or-later**. Add a `LICENSE` file with the full
   GPL-3.0 text (or dual-license `webfluent` first — see §2 of the plan) before
   distributing.
3. **Version** — bump `version` in `[workspace.package]`.
4. **Crash/error log** — local-only, per NFR-5 (no telemetry).
