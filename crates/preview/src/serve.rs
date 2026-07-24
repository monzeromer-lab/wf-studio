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
        Some((mime, bytes)) => Response::builder()
            .status(200)
            .header(CONTENT_TYPE, mime)
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
}
