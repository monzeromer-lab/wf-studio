//! The `wf://` custom-protocol serving: map a request path to a compiled
//! resource from the live [`CompiledSite`], and build the HTTP response.
//!
//! Pure and GPUI-free — the studio's webview host locks its shared site and
//! calls [`resolve`]/[`respond`] from the wry protocol closure.

use std::borrow::Cow;

use wf_core::CompiledSite;
use wry::http::{header::CONTENT_TYPE, Request, Response};

pub const MIME_HTML: &str = "text/html; charset=utf-8";
pub const MIME_CSS: &str = "text/css; charset=utf-8";
pub const MIME_JS: &str = "application/javascript; charset=utf-8";

/// The before/after diff-shell page (§4.1). Two overlaid iframes — the base
/// site (`/base`) underneath and the proposal (`/proposal`) on top, revealed by
/// a `clip-path` wipe that follows the cursor. A transparent overlay captures
/// the mouse (the iframes are `pointer-events: none` so it reaches the overlay),
/// and `window.__setClip(pct)` lets the host drive the split from GPUI.
pub const DIFF_SHELL: &str = r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  html, body { margin: 0; height: 100%; overflow: hidden; background: #0d0d12; }
  .frame { position: absolute; inset: 0; width: 100%; height: 100%; border: 0; pointer-events: none; }
  #after { clip-path: inset(0 0 0 50%); }
  #handle { position: absolute; top: 0; bottom: 0; left: 50%; width: 2px; margin-left: -1px;
            background: #7c5cff; box-shadow: 0 0 10px 1px rgba(124, 92, 255, 0.7); z-index: 20; }
  #grip { position: absolute; top: 50%; left: 50%; width: 28px; height: 28px; margin: -14px 0 0 -14px;
          border-radius: 50%; background: #7c5cff; box-shadow: 0 2px 10px rgba(0, 0, 0, 0.55); z-index: 21;
          display: flex; align-items: center; justify-content: center; color: #fff; font: 13px/1 sans-serif; }
  .tag { position: absolute; top: 20px; padding: 4px 10px; border-radius: 6px; z-index: 30; pointer-events: none;
         font: 700 10.5px/1 -apple-system, system-ui, sans-serif; letter-spacing: 0.06em; }
  #before-tag { left: 20px; background: rgba(0, 0, 0, 0.7); color: #c8c8d4; }
  #after-tag { right: 20px; background: rgba(124, 92, 255, 0.18); color: #b6a6ff; }
  #overlay { position: absolute; inset: 0; z-index: 10; cursor: ew-resize; }
</style>
</head>
<body>
  <iframe id="before" class="frame" src="/base"></iframe>
  <iframe id="after" class="frame" src="/proposal"></iframe>
  <div id="before-tag" class="tag">BEFORE</div>
  <div id="after-tag" class="tag">AFTER</div>
  <div id="handle"></div>
  <div id="grip">&#8646;</div>
  <div id="overlay"></div>
  <script>
    var after = document.getElementById('after');
    var handle = document.getElementById('handle');
    var grip = document.getElementById('grip');
    function setSplit(pct) {
      pct = Math.max(0, Math.min(100, pct));
      after.style.clipPath = 'inset(0 0 0 ' + pct + '%)';
      handle.style.left = pct + '%';
      grip.style.left = pct + '%';
    }
    document.getElementById('overlay').addEventListener('mousemove', function (e) {
      setSplit((e.clientX / window.innerWidth) * 100);
    });
    window.__setClip = setSplit;
    setSplit(50);
  </script>
</body>
</html>"#;

/// A fully self-contained page for one compiled site: the home page HTML with
/// its stylesheet and JS bundle inlined, so it renders with no external `wf://`
/// asset requests. The diff shell serves the base and proposal through this so
/// the two iframes never cross-load each other's `/styles.css` (§4.1).
pub fn self_contained(site: &CompiledSite) -> String {
    let html = site
        .pages
        .iter()
        .find(|p| p.route == "/")
        .or_else(|| site.pages.first())
        .map(|p| p.html.as_str())
        .unwrap_or("");
    // The SSG references assets relative to route depth; the home route (all the
    // diff shell serves) yields "./styles.css" / "./app.js".
    html.replace(
        r#"<link rel="stylesheet" href="./styles.css">"#,
        &format!("<style>\n{}\n</style>", site.css),
    )
    .replace(r#"<script src="./app.js"></script>"#, &format!("<script>\n{}\n</script>", site.js))
}

/// Map a request path to a compiled resource: the stylesheet, the JS bundle, or a
/// page by route (`/`, `/about`, `about/index.html`, …). `None` → 404.
pub fn resolve(site: &CompiledSite, path: &str) -> Option<(&'static str, Vec<u8>)> {
    let p = path.trim_start_matches('/');
    match p {
        "styles.css" => Some((MIME_CSS, site.css.clone().into_bytes())),
        "app.js" => Some((MIME_JS, site.js.clone().into_bytes())),
        _ => {
            let route = if p.is_empty() || p == "index.html" {
                "/".to_string()
            } else {
                format!("/{}", p.trim_end_matches("index.html").trim_end_matches('/'))
            };
            site.pages
                .iter()
                .find(|pg| pg.route == route)
                .map(|pg| (MIME_HTML, pg.html.clone().into_bytes()))
        }
    }
}

/// Build a `wf://` response from a resolved resource (200) or a 404.
pub fn respond(found: Option<(&'static str, Vec<u8>)>) -> Response<Cow<'static, [u8]>> {
    match found {
        // `no-store` is essential: after an inspector edit or AI change we recompile
        // and reload the SAME url, so WebKit must re-fetch index.html / styles.css /
        // app.js from the protocol handler rather than serve a cached (pre-edit) copy —
        // otherwise style/structure edits silently don't appear in the preview.
        Some((mime, bytes)) => Response::builder()
            .status(200)
            .header(CONTENT_TYPE, mime)
            .header("Cache-Control", "no-store, must-revalidate")
            .body(Cow::Owned(bytes))
            .unwrap(),
        None => Response::builder()
            .status(404)
            .header(CONTENT_TYPE, "text/plain")
            .body(Cow::Owned(b"not found".to_vec()))
            .unwrap(),
    }
}

/// Serve one `wf://` request from a compiled site (resolve + respond).
pub fn serve(request: &Request<Vec<u8>>, site: &CompiledSite) -> Response<Cow<'static, [u8]>> {
    respond(resolve(site, request.uri().path()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wf_core::compile_source;

    #[test]
    fn resolve_serves_compiled_page_css_and_js() {
        let site = compile_source("Page Home (path: \"/\") { Container { Text(\"hi\") } }").unwrap();

        // "/" and index.html resolve to the home page HTML with node-id stamps.
        for path in ["/", "/index.html", "index.html"] {
            let (mime, bytes) = resolve(&site, path).unwrap_or_else(|| panic!("no page for {path}"));
            assert_eq!(mime, MIME_HTML);
            assert!(String::from_utf8(bytes).unwrap().contains("data-wf-node="), "{path}: no stamps");
        }

        // Stylesheet + JS bundle by their conventional paths.
        assert_eq!(resolve(&site, "/styles.css").unwrap().0, MIME_CSS);
        let (js_mime, js) = resolve(&site, "/app.js").unwrap();
        assert_eq!(js_mime, MIME_JS);
        assert!(String::from_utf8(js).unwrap().contains("data-wf-node"));

        // Unknown path → 404 (None).
        assert!(resolve(&site, "/nope.txt").is_none());
    }

    #[test]
    fn self_contained_inlines_assets_and_drops_external_refs() {
        let site = compile_source("Page Home (path: \"/\") { Container { Text(\"hi\") } }").unwrap();
        let page = self_contained(&site);

        // The stylesheet and bundle are inlined…
        assert!(page.contains("<style>"), "css not inlined");
        assert!(page.contains("<script>"), "js not inlined");
        // …and the external asset requests are gone, so the iframe never reaches
        // back to a sibling scope's /styles.css.
        assert!(!page.contains(r#"href="./styles.css""#), "external css ref remains");
        assert!(!page.contains(r#"src="./app.js""#), "external js ref remains");
        // The rendered body (with node stamps) survives.
        assert!(page.contains("data-wf-node="), "body lost");
    }
}
