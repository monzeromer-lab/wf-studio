//! One-off: generate a DESIGN SYSTEM live with the DS card and compile it, to
//! validate the design-system system prompt end to end.
//!
//!   cargo run -p wf-evals --example ds_live -- deepseek   # or anthropic|openai|…
//!
//! Uses a BYO key from the OS keychain or the provider env var. Writes the raw
//! generated `.wf` to the path in WF_DS_OUT (default /tmp/ds_generated.wf).

use wf_ai::{collect_text, default_key_store, provider_for, ChatMessage, ChatRequest, KeyStore, Provider, ProviderKind, DESIGN_SYSTEM_CARD};
use wf_core::{compile_source, generate_page, GenConfig};

/// Extract the first fenced ```wf block. Tolerates a truncated (unclosed) fence.
fn extract_wf(text: &str) -> String {
    if let Some(start) = text.find("```wf") {
        let after = &text[start + 5..];
        // Closed fence → take up to the closing ```; unclosed (truncated) → take the rest.
        let body = after.find("```").map(|end| &after[..end]).unwrap_or(after);
        return body.trim().to_string();
    }
    // No fence at all: strip any stray leading/trailing markers.
    text.trim().trim_start_matches("```wf").trim_start_matches("```").trim_end_matches("```").trim().to_string()
}

fn main() {
    let kind = match std::env::args().nth(1).as_deref() {
        Some("openai") => ProviderKind::OpenAi,
        Some("gemini") => ProviderKind::Gemini,
        Some("deepseek") => ProviderKind::DeepSeek,
        Some("moonshot") | Some("kimi") => ProviderKind::Kimi,
        Some("zhipu") | Some("glm") => ProviderKind::Glm,
        _ => ProviderKind::Anthropic,
    };
    let Some(key) = default_key_store().get(kind) else {
        eprintln!("No API key for {} — set {} or store one in the keychain.", kind.display_name(), kind.key_env());
        std::process::exit(2);
    };
    let model = kind.default_model();
    let provider = provider_for(kind, key);
    let out = std::env::var("WF_DS_OUT").unwrap_or_else(|_| "/tmp/ds_generated.wf".to_string());

    // Scoped small so a complete site fits the token budget (a full DS site is large).
    let prompt = "A small, focused design system for a fintech dashboard: an indigo brand \
        ramp and a neutral gray scale (color foundation), a compact type scale, and TWO \
        documented components — Button and Card — each with variants, states, and usage \
        do/don't. Keep it to Getting Started, Principles, Foundations/Color, and those two \
        component pages. Concise.";

    let max_tokens: u32 = std::env::var("WF_DS_MAX").ok().and_then(|s| s.parse().ok()).unwrap_or(30000);
    let mut config = GenConfig::for_model(model);
    config.max_tokens = max_tokens;

    // First, a single RAW call so we can inspect exactly what the model emits.
    eprintln!("Raw generation with {} ({})…", kind.display_name(), model);
    let req = ChatRequest {
        model: model.to_string(),
        max_tokens,
        messages: vec![ChatMessage::system(DESIGN_SYSTEM_CARD), ChatMessage::user(prompt)],
    };
    let raw = collect_text(Provider::stream_chat(&*provider, req)).unwrap_or_else(|e| format!("STREAM ERROR: {e}"));
    let raw_path = format!("{out}.raw.md");
    let _ = std::fs::write(&raw_path, &raw);
    let wf = extract_wf(&raw);
    let _ = std::fs::write(&out, &wf);
    println!("RAW  reply_len={}  wf_len={}  -> {raw_path} , {out}", raw.len(), wf.len());
    match compile_source(&wf) {
        Ok(site) => println!("RAW COMPILE OK  pages={}  nodes={}", site.pages.len(), site.node_map.len()),
        Err(e) => println!("RAW COMPILE FAILED: {e}"),
    }
    println!();

    eprintln!("Now the full generate+repair loop…\n");
    let t = std::time::Instant::now();
    match generate_page(&*provider, DESIGN_SYSTEM_CARD, prompt, &config) {
        Ok(outcome) => {
            let _ = std::fs::write(&out, &outcome.source);
            println!("GENERATED  attempts={}  source_len={}  in {:.1}s  -> {out}", outcome.attempts, outcome.source.len(), t.elapsed().as_secs_f32());
            match compile_source(&outcome.source) {
                Ok(site) => {
                    println!("COMPILE OK  pages={}  nodes={}", site.pages.len(), site.node_map.len());
                    for pg in &site.pages {
                        println!("  route {}", pg.route);
                    }
                    // Sanity: did the model inline literal hex swatches (not `{hex}`)?
                    let src = &outcome.source;
                    println!("uses literal hex in style: {}", src.contains("background: \"#") || src.contains("background:\"#"));
                    println!("still has brace interpolation in style: {}", src.contains("background: \"{") || src.contains("background:\"{"));
                }
                Err(e) => println!("COMPILE FAILED: {e}"),
            }
        }
        Err(e) => println!("GENERATION FAILED: {e}"),
    }
}
