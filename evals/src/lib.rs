//! The generation eval harness (IMPLEMENTATION_PLAN §4.4).
//!
//! A fixed suite of natural-language prompts is run through the real generation
//! loop ([`wf_core::generate_page`]) against a provider; each result is scored on
//! whether it compiled and how many self-heal rounds it took. This is how default
//! models are chosen and language-card / prompt regressions are caught with
//! evidence rather than vibes.
//!
//! Running the suite calls the model (needs a key + network), so it is a binary
//! (`cargo run -p wf-evals`), not a hermetic test. The harness logic itself is
//! unit-tested offline against [`wf_ai::ScriptedProvider`].

use wf_ai::{Provider, LANGUAGE_CARD};
use wf_core::{generate_page, GenConfig, GenError};

/// One golden prompt.
pub struct EvalPrompt {
    pub id: &'static str,
    /// `"en"` | `"ar"` — the suite is bilingual (NFR-3).
    pub lang: &'static str,
    /// Rough category (landing/menu/portfolio/…), for reporting.
    pub kind: &'static str,
    pub prompt: &'static str,
}

/// The M1 suite: landing pages, menus, portfolios, forms — Arabic + English.
pub const PROMPTS: &[EvalPrompt] = &[
    EvalPrompt { id: "cafe-landing-en", lang: "en", kind: "landing", prompt: "A landing page for a small coffee shop in Cairo: a hero with the name and a tagline, a short about paragraph, and a 'Visit us' call-to-action button." },
    EvalPrompt { id: "cafe-landing-ar", lang: "ar", kind: "landing", prompt: "صفحة هبوط لمقهى صغير في القاهرة: عنوان رئيسي باسم المقهى وشعار قصير، فقرة تعريف مختصرة، وزر لدعوة الزوّار للزيارة." },
    EvalPrompt { id: "menu-grid-en", lang: "en", kind: "menu", prompt: "A restaurant menu page showing three dishes in a grid, each with a name, a one-line description, and a price." },
    EvalPrompt { id: "portfolio-en", lang: "en", kind: "portfolio", prompt: "A personal portfolio page for a photographer: their name, a two-sentence bio, and a gallery of three placeholder photos." },
    EvalPrompt { id: "counter-en", lang: "en", kind: "interactive", prompt: "A simple page with a counter: a large number and plus and minus buttons that increase and decrease it." },
    EvalPrompt { id: "contact-form-en", lang: "en", kind: "form", prompt: "A contact page with a form: name, email, and message fields, and a submit button." },
    EvalPrompt { id: "startup-features-ar", lang: "ar", kind: "landing", prompt: "صفحة تعريفية لشركة تقنية ناشئة: عنوان جذّاب وثلاث ميزات معروضة في شبكة من ثلاثة أعمدة." },
    EvalPrompt { id: "pricing-en", lang: "en", kind: "pricing", prompt: "A pricing page with three plan cards side by side — Basic, Pro, and Team — each showing a price and a 'Choose' button." },
    EvalPrompt { id: "blog-index-en", lang: "en", kind: "blog", prompt: "A blog index page: a page heading and three post previews, each with a title and a short excerpt." },
    EvalPrompt { id: "faq-en", lang: "en", kind: "faq", prompt: "An FAQ page with four question-and-answer pairs under a heading." },
];

/// The outcome of running one prompt.
#[derive(Debug, Clone)]
pub struct EvalResult {
    pub id: String,
    pub compiled: bool,
    /// Model calls it took (1 = first try; >1 = self-heal rounds).
    pub attempts: usize,
    pub error: Option<String>,
}

/// Run one prompt through the generation loop against `provider`.
pub fn run_one(provider: &dyn Provider, model: &str, p: &EvalPrompt) -> EvalResult {
    let mut config = GenConfig::for_model(model);
    config.max_tokens = 8192;
    match generate_page(provider, LANGUAGE_CARD, p.prompt, &config) {
        Ok(o) => EvalResult { id: p.id.into(), compiled: true, attempts: o.attempts, error: None },
        Err(e) => {
            let attempts = match &e {
                GenError::Unrepaired { attempts, .. } => *attempts,
                _ => 0,
            };
            EvalResult { id: p.id.into(), compiled: false, attempts, error: Some(e.to_string()) }
        }
    }
}

/// Aggregate scores over a run.
#[derive(Debug, Clone)]
pub struct Summary {
    pub total: usize,
    pub compiled: usize,
    /// Mean attempts among prompts that compiled (1 = first try, lower is better).
    pub avg_attempts: f64,
}

impl Summary {
    pub fn compile_rate(&self) -> f64 {
        if self.total == 0 { 0.0 } else { self.compiled as f64 / self.total as f64 }
    }
}

/// Score a set of results: compile rate + mean self-heal rounds.
pub fn summarize(results: &[EvalResult]) -> Summary {
    let total = results.len();
    let compiled = results.iter().filter(|r| r.compiled).count();
    let attempts: usize = results.iter().filter(|r| r.compiled).map(|r| r.attempts).sum();
    let avg_attempts = if compiled == 0 { 0.0 } else { attempts as f64 / compiled as f64 };
    Summary { total, compiled, avg_attempts }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wf_ai::{ProviderKind, ScriptedProvider};

    const GOOD: &str = "```wf\nPage Home (path: \"/\") { Container { Text(\"hi\") } }\n```";
    const BAD: &str = "```wf\nPage Home (path: \"/\") { Ghost() }\n```";

    #[test]
    fn suite_is_non_empty_bilingual_and_unique() {
        assert!(PROMPTS.len() >= 10, "M1 wants at least 10 prompts");
        assert!(PROMPTS.iter().any(|p| p.lang == "ar"), "must include Arabic prompts");
        assert!(PROMPTS.iter().any(|p| p.lang == "en"), "must include English prompts");
        let ids: std::collections::HashSet<_> = PROMPTS.iter().map(|p| p.id).collect();
        assert_eq!(ids.len(), PROMPTS.len(), "prompt ids must be unique");
    }

    #[test]
    fn run_one_scores_a_good_generation() {
        let p = ScriptedProvider::with_text(GOOD);
        let r = run_one(&p, "test", &PROMPTS[0]);
        assert!(r.compiled);
        assert_eq!(r.attempts, 1);
        assert!(r.error.is_none());
    }

    #[test]
    fn run_one_scores_a_failed_generation() {
        let p = ScriptedProvider::new(ProviderKind::Anthropic);
        p.push_text(BAD).push_text(BAD).push_text(BAD);
        let r = run_one(&p, "test", &PROMPTS[0]);
        assert!(!r.compiled);
        assert!(r.error.is_some());
    }

    #[test]
    fn summarize_computes_rate_and_avg_attempts() {
        let results = vec![
            EvalResult { id: "a".into(), compiled: true, attempts: 1, error: None },
            EvalResult { id: "b".into(), compiled: true, attempts: 3, error: None },
            EvalResult { id: "c".into(), compiled: false, attempts: 3, error: Some("nope".into()) },
        ];
        let s = summarize(&results);
        assert_eq!(s.total, 3);
        assert_eq!(s.compiled, 2);
        assert!((s.compile_rate() - 2.0 / 3.0).abs() < 1e-9);
        assert!((s.avg_attempts - 2.0).abs() < 1e-9, "(1+3)/2 = 2 among the compiled");
    }
}
