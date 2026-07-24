//! Every few-shot example embedded in the AI language card MUST compile through
//! the studio gate (lex + parse + validate_semantics). If the engine's syntax or
//! builtin set drifts, this canary fails before the model is ever taught wrong.

use wf_core::compile_source;

const NOT_FOUND: &str = r#"Page NotFound (path: "/404", title: "Page Not Found") {
    Container(fadeIn) {
        Spacer(xl)
        Stack(gap: md) {
            Heading("404", h1, center, primary)
            Heading("Page Not Found", h2, center)
            Text("The page you are looking for does not exist or has been moved.", muted, center)
            Spacer()
            Row(justify: center) {
                Button("Go Home", primary, large) { navigate("/") }
            }
        }
        Spacer(xl)
    }
}"#;

const FEATURES: &str = r#"Component FeatureCard (title: String, description: String) {
    Card(elevated, scaleIn) {
        Card.Body {
            Heading(title, h2)
            Spacer(sm)
            Text(description, muted)
        }
    }
}

Page Features (path: "/features", title: "Features") {
    Container {
        Spacer()
        Heading("Why WebFluent", h1, center)
        Spacer()
        Grid(columns: 3, gap: md) {
            FeatureCard(title: "Fast", description: "Compiles to static HTML.")
            FeatureCard(title: "Typed", description: "Props are checked at compile time.")
            FeatureCard(title: "RTL-ready", description: "Logical CSS everywhere.")
        }
        Spacer(xl)
    }
}"#;

const COUNTER: &str = r#"Page Counter (path: "/counter", title: "Counter") {
    state counter = 0

    Container {
        Spacer()
        Card(elevated) {
            Card.Header { Heading("Counter", h2) }
            Card.Body {
                Row(align: center, gap: md) {
                    Button("-", large) { counter = counter - 1 }
                    Heading("{counter}", h2, primary)
                    Button("+", primary, large) { counter = counter + 1 }
                }
                Spacer(sm)
                Text("State updates re-render the view.", muted, small)
            }
        }
        Spacer(xl)
    }
}"#;

const ECHO: &str = r#"Page Echo (path: "/echo", title: "Echo") {
    state taskInput = ""
    state showHint = false

    Container {
        Spacer()
        Card(elevated) {
            Card.Body {
                Input(text, bind: taskInput, placeholder: "Type something...", label: "Input")
                Spacer(sm)
                if taskInput != "" {
                    Alert("You typed: {taskInput}", info)
                }
                Switch(bind: showHint, label: "Show hint")
                if showHint {
                    Text("Binding keeps state and UI in sync.", muted, small)
                }
            }
        }
        Spacer(xl)
    }
}"#;

const MARHABA: &str = r#"Page Marhaba (path: "/ar", title: "مرحبا") {
    Container(fadeIn) {
        Spacer(xl)
        Heading("مرحبا بك في ويب فلوينت", h1, center, slideUp)
        Spacer(sm)
        Text("لغة لبناء مواقع الويب بسهولة وسرعة.", muted, center)
        Spacer()
        Row(gap: md, justify: center) {
            Button("ابدأ الآن", primary, large) { navigate("/getting-started") }
            Button("الدليل", large) { navigate("/guide") }
        }
        Spacer(xl)
    }
}"#;

#[test]
fn all_language_card_examples_compile() {
    let examples = [
        ("NotFound", NOT_FOUND),
        ("Features", FEATURES),
        ("Counter", COUNTER),
        ("Echo", ECHO),
        ("Marhaba", MARHABA),
    ];
    let mut failures = Vec::new();
    for (name, src) in examples {
        if let Err(e) = compile_source(src) {
            failures.push(format!("  {name}: {e}"));
        }
    }
    assert!(
        failures.is_empty(),
        "language-card examples failed to compile:\n{}",
        failures.join("\n")
    );
}
