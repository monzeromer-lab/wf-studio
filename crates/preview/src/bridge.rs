/// Injected into every preview page before load (wry initialization script).
///
/// M0 scope: forward runtime errors to the studio over the ipc channel —
/// the seed of the self-healing loop (FR-19). The M2 bridge (click-to-select,
/// hover outlines, scrub sync) grows in this file.
pub const BRIDGE_JS: &str = r#"
(function () {
    if (window.__wfBridge) { return; }

    function post(kind, payload) {
        try {
            window.ipc.postMessage(JSON.stringify({ kind: kind, payload: payload }));
        } catch (_) { /* ipc unavailable: nothing useful to do */ }
    }

    window.__wfBridge = { post: post };

    window.addEventListener('error', function (event) {
        post('runtime-error', {
            message: String(event.message || event.error || 'unknown error'),
            source: String(event.filename || ''),
            line: event.lineno || 0,
        });
    });

    window.addEventListener('unhandledrejection', function (event) {
        post('runtime-error', { message: 'Unhandled rejection: ' + String(event.reason), source: '', line: 0 });
    });

    var origConsoleError = console.error;
    console.error = function () {
        post('console-error', { message: Array.prototype.slice.call(arguments).map(String).join(' ') });
        return origConsoleError.apply(console, arguments);
    };

    window.addEventListener('DOMContentLoaded', function () {
        post('page-loaded', {});
    });
})();
"#;
