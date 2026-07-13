use wf_core::Document;

const SAMPLE: &str = include_str!("../../../assets/sample.wf");

/// Golden test: the bundled Arabic sample must compile against the pinned
/// webfluent version. This is the arbiter for sample.wf syntax.
#[test]
fn arabic_sample_compiles_to_html() {
    let doc = Document::new(SAMPLE);
    let artifacts = doc.compile().unwrap_or_else(|diags| {
        panic!("sample.wf must compile, got diagnostics: {diags:?}");
    });

    let index = artifacts.get("index.html").expect("index.html artifact");
    assert_eq!(index.mime, "text/html");

    let html = std::str::from_utf8(&index.bytes).expect("utf-8 html");
    assert!(
        html.contains("مرحباً"),
        "Arabic content must survive codegen"
    );
    assert!(
        html.contains("<html") || html.contains("<!DOCTYPE"),
        "render_html returns a full document"
    );
}

#[test]
fn artifacts_lookup_normalizes_paths() {
    let doc = Document::new(SAMPLE);
    let artifacts = doc.compile().unwrap();
    assert!(artifacts.get("/").is_some(), "root resolves to index.html");
    assert!(artifacts.get("/index.html").is_some());
    assert!(artifacts.get("missing.js").is_none());
}

#[test]
fn malformed_source_yields_located_diagnostics() {
    let doc = Document::new("Page Broken (path: \"/\" {\n    Container {\n");
    let diags = doc.compile().expect_err("must fail");
    assert!(!diags.is_empty());
    let d = &diags[0];
    assert!(!d.message.is_empty());
    assert!(
        d.line.is_some(),
        "lexer/parser errors should carry a location, got: {d:?}"
    );
}
