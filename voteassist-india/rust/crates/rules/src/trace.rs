//! The tri-state [`Verdict`] and the [`EvaluationTrace`] every rule category
//! in this crate evaluates through. This is the crate's most important
//! design decision (see the crate root docs): the API is built around
//! producing an explanation from the start, not a bare answer with an
//! explanation bolted on afterward.

use serde::{Deserialize, Serialize};

use crate::citation::Citation;

/// A rule's answer to the single yes/no claim it tests — with a mandatory
/// third state.
///
/// Never collapse a missing input into [`Verdict::NotSatisfied`]. That
/// collapse is this crate's single worst failure mode: a citizen who
/// simply didn't answer a question (or whose channel — IVR, WhatsApp —
/// never collected it) must never be told "you appear ineligible" when the
/// honest answer is "we don't know yet." Every rule function in
/// `crate::eligibility`, `crate::forms`, `crate::deadlines`, and `crate::mcc`
/// returns [`Verdict::CannotDetermine`] whenever a required input field is
/// `None`, as a matter of construction, not convention — there is no code
/// path in this crate that silently defaults an absent input to "no".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Satisfied,
    NotSatisfied,
    /// Insufficient input to evaluate this rule at all — distinct from,
    /// and never conflated with, [`Verdict::NotSatisfied`].
    CannotDetermine,
}

impl Verdict {
    pub fn is_satisfied(self) -> bool {
        matches!(self, Verdict::Satisfied)
    }

    pub fn is_not_satisfied(self) -> bool {
        matches!(self, Verdict::NotSatisfied)
    }

    pub fn is_cannot_determine(self) -> bool {
        matches!(self, Verdict::CannotDetermine)
    }
}

/// One rule's contribution to a trace: which rule (`rule_id`), what it's
/// actually testing (`description`), what backs that test (`citation`),
/// what it concluded (`verdict`), and a plain-language `detail` filled in
/// per-evaluation (e.g. the actual computed qualifying date, the actual
/// age) so a citizen or auditor sees the specific reasoning, not just a
/// static rule description repeated verbatim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleStep {
    pub rule_id: &'static str,
    pub description: &'static str,
    pub citation: Citation,
    pub verdict: Verdict,
    pub detail: String,
}

/// The result of evaluating a whole rule set (an eligibility check, a
/// form-selection pass, a deadline check) — every [`RuleStep`] that fired,
/// in evaluation order, so a consumer can render "you appear eligible
/// BECAUSE x, y, z" or audit exactly why the engine said what it said.
///
/// Deliberately a flat `Vec`, not a nested tree: every rule category in
/// this crate evaluates a fixed, small, independent set of named rules
/// (there is no rule-depends-on-rule chaining), so a flat ordered list is
/// the honest shape of the data, not a simplification of something more
/// complex.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvaluationTrace {
    pub steps: Vec<RuleStep>,
}

impl EvaluationTrace {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, step: RuleStep) {
        self.steps.push(step);
    }

    /// Combine every step's verdict into one overall verdict:
    ///
    /// 1. Any [`Verdict::NotSatisfied`] step wins outright. A confirmed
    ///    disqualifying fact (e.g. "declared of unsound mind by a
    ///    competent court") is not softened by an unrelated unknown
    ///    elsewhere — that would bury a real negative under a fake
    ///    "maybe."
    /// 2. Otherwise, any [`Verdict::CannotDetermine`] step wins. This is
    ///    the safe default this crate insists on: never report overall
    ///    [`Verdict::Satisfied`] on partial information.
    /// 3. Only if every step is [`Verdict::Satisfied`] is the overall
    ///    verdict [`Verdict::Satisfied`].
    pub fn overall(&self) -> Verdict {
        if self.steps.iter().any(|s| s.verdict.is_not_satisfied()) {
            Verdict::NotSatisfied
        } else if self.steps.iter().any(|s| s.verdict.is_cannot_determine()) {
            Verdict::CannotDetermine
        } else {
            Verdict::Satisfied
        }
    }

    pub fn satisfied(&self) -> impl Iterator<Item = &RuleStep> {
        self.steps.iter().filter(|s| s.verdict.is_satisfied())
    }

    pub fn not_satisfied(&self) -> impl Iterator<Item = &RuleStep> {
        self.steps.iter().filter(|s| s.verdict.is_not_satisfied())
    }

    pub fn cannot_determine(&self) -> impl Iterator<Item = &RuleStep> {
        self.steps.iter().filter(|s| s.verdict.is_cannot_determine())
    }

    /// Every citation referenced anywhere in this trace — the full source
    /// list a consumer would show alongside the verdict.
    pub fn citations(&self) -> Vec<&Citation> {
        self.steps.iter().map(|s| &s.citation).collect()
    }

    /// Steps whose citation is the honest [`Citation::Missing`] placeholder
    /// — surfaced separately so an admin audit view can flag them, per this
    /// crate's absolute rule that an uncited claim must never look like a
    /// sourced one.
    pub fn uncited_steps(&self) -> impl Iterator<Item = &RuleStep> {
        self.steps.iter().filter(|s| s.citation.is_missing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(verdict: Verdict) -> RuleStep {
        RuleStep {
            rule_id: "test.rule",
            description: "a test rule",
            citation: Citation::kb("qualifying-dates"),
            verdict,
            detail: "detail".to_string(),
        }
    }

    #[test]
    fn overall_is_satisfied_when_every_step_is() {
        let mut trace = EvaluationTrace::new();
        trace.push(step(Verdict::Satisfied));
        trace.push(step(Verdict::Satisfied));
        assert_eq!(trace.overall(), Verdict::Satisfied);
    }

    #[test]
    fn overall_is_not_satisfied_if_any_step_is_even_with_an_unknown_elsewhere() {
        let mut trace = EvaluationTrace::new();
        trace.push(step(Verdict::Satisfied));
        trace.push(step(Verdict::NotSatisfied));
        trace.push(step(Verdict::CannotDetermine));
        assert_eq!(trace.overall(), Verdict::NotSatisfied);
    }

    #[test]
    fn overall_is_cannot_determine_when_no_step_fails_but_one_is_unknown() {
        let mut trace = EvaluationTrace::new();
        trace.push(step(Verdict::Satisfied));
        trace.push(step(Verdict::CannotDetermine));
        assert_eq!(trace.overall(), Verdict::CannotDetermine);
    }

    #[test]
    fn empty_trace_is_vacuously_satisfied() {
        // No rules registered means no rule failed and none is unknown —
        // an empty rule *set* (not an empty *input*) is the only way to
        // reach this, and callers in this crate always register a fixed,
        // non-empty rule set per category, so this is a documented edge
        // case rather than a real code path.
        let trace = EvaluationTrace::new();
        assert_eq!(trace.overall(), Verdict::Satisfied);
    }

    #[test]
    fn uncited_steps_are_detectable() {
        let mut trace = EvaluationTrace::new();
        let mut missing = step(Verdict::Satisfied);
        missing.citation = Citation::missing("no source attached yet");
        trace.push(missing);
        trace.push(step(Verdict::Satisfied));
        assert_eq!(trace.uncited_steps().count(), 1);
    }
}
