use wf_core::compile_source;

const SAMPLE: &str = include_str!("../../../assets/sample.wf");

/// Golden test: the bundled Arabic sample must compile against the pinned
/// webfluent version through the studio compile path. This is the arbiter for
/// sample.wf syntax and a guard that Arabic content survives codegen (NFR-3).
#[test]
fn arabic_sample_compiles_to_html() {
    let site = compile_source(SAMPLE)
        .unwrap_or_else(|e| panic!("sample.wf must compile, got: {e}"));

    assert!(!site.pages.is_empty(), "sample produces at least one page");
    let html = &site.pages[0].html;
    assert!(
        html.contains("ويب فلونت"),
        "Arabic heading content must survive codegen"
    );
    assert!(
        html.contains("data-wf-node="),
        "studio-mode compile stamps node ids"
    );
}

#[test]
fn malformed_source_is_rejected() {
    assert!(
        compile_source("Page Broken (path: \"/\" {\n    Container {\n").is_err(),
        "an unterminated header must not compile"
    );
}
