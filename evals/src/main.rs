//! Run the generation eval suite against a provider (BYO key from the OS keychain
//! or the provider's env var).
//!
//!   cargo run -p wf-evals            # Anthropic (default)
//!   cargo run -p wf-evals -- openai  # or: gemini | deepseek | moonshot | zhipu
//!
//! Exits non-zero if any prompt failed to compile, so it can gate prompt / language-
//! card changes in CI (with a key configured).

use wf_ai::{default_key_store, provider_for, KeyStore, ProviderKind};
use wf_evals::{run_one, summarize, PROMPTS};

fn parse_provider(arg: Option<&str>) -> ProviderKind {
    match arg {
        Some("openai") => ProviderKind::OpenAi,
        Some("gemini") => ProviderKind::Gemini,
        Some("deepseek") => ProviderKind::DeepSeek,
        Some("moonshot") | Some("kimi") => ProviderKind::Kimi,
        Some("zhipu") | Some("glm") => ProviderKind::Glm,
        _ => ProviderKind::Anthropic,
    }
}

fn main() {
    let kind = parse_provider(std::env::args().nth(1).as_deref());

    let key = match default_key_store().get(kind) {
        Some(k) => k,
        None => {
            eprintln!(
                "No API key for {}. Set {} (or store one in the keychain) and retry.",
                kind.display_name(),
                kind.key_env()
            );
            eprintln!("The harness logic is covered offline by `cargo test -p wf-evals`.");
            std::process::exit(2);
        }
    };

    let model = kind.default_model();
    let provider = provider_for(kind, key);
    println!("Running {} prompts against {} ({})\n", PROMPTS.len(), kind.display_name(), model);

    let results: Vec<_> = PROMPTS
        .iter()
        .map(|p| {
            let r = run_one(&*provider, model, p);
            let tag = if r.compiled { "ok  " } else { "FAIL" };
            let note = r.error.as_deref().map(|e| format!("  — {e}")).unwrap_or_default();
            println!("  [{tag}] {:<22} {} attempt(s){note}", p.id, r.attempts);
            r
        })
        .collect();

    let s = summarize(&results);
    println!(
        "\n{}/{} compiled ({:.0}%) · avg {:.1} attempt(s) among those that compiled.",
        s.compiled,
        s.total,
        s.compile_rate() * 100.0,
        s.avg_attempts
    );

    if s.compiled < s.total {
        std::process::exit(1);
    }
}
