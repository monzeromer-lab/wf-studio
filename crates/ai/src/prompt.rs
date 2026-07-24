//! Prompt templates for generation (IMPLEMENTATION_PLAN §4.3/§4.4).
//!
//! [`LANGUAGE_CARD`] is the system prompt that teaches a model to write valid
//! WebFluent — no public model has seen the language, so reliability comes from
//! this card plus the validate → repair loop, not the model. It was distilled
//! from the engine's `spec/SPEC.md` and the real `site/` sources; every few-shot
//! example embedded in it is compile-verified (see wf-core's
//! `tests/language_card.rs`).

/// The WebFluent language card: output contract + grammar + component catalog +
/// style/RTL rules + compile-verified few-shot examples. Sent as the system
/// prompt on every generation (§4.3 caches it; it dominates token count).
pub const LANGUAGE_CARD: &str = include_str!("language_card.md");

/// The design-system card: the WebFluent grammar PLUS the design-system domain
/// (the designsystems.com model, page inventory, token model, doc components, and
/// a worked example). Sent as the system prompt when generating a design-system
/// project instead of [`LANGUAGE_CARD`], so the output is a browsable multi-page
/// DS site (foundations/token galleries, type specimen, component library).
pub const DESIGN_SYSTEM_CARD: &str = include_str!("design_system_card.md");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_card_is_present_and_sane() {
        assert!(LANGUAGE_CARD.len() > 2000, "card should be substantial");
        assert!(LANGUAGE_CARD.contains("OUTPUT CONTRACT"), "must state the return-only-wf contract");
        assert!(LANGUAGE_CARD.contains("```wf"), "must include fenced few-shot examples");
        assert!(LANGUAGE_CARD.contains("margin-inline"), "must teach the logical-CSS RTL rule");
    }

    #[test]
    fn design_system_card_is_present_and_sane() {
        assert!(DESIGN_SYSTEM_CARD.len() > 4000, "DS card should be substantial");
        assert!(DESIGN_SYSTEM_CARD.contains("OUTPUT CONTRACT"), "must state the return-only-wf contract");
        assert!(DESIGN_SYSTEM_CARD.contains("```wf"), "must include a worked example");
        // The DS domain + the static-rendering guardrail must both be present.
        assert!(DESIGN_SYSTEM_CARD.contains("design system"), "must teach the DS domain");
        assert!(DESIGN_SYSTEM_CARD.contains("STATIC RENDERING"), "must carry the no-interpolation rule");
    }
}
