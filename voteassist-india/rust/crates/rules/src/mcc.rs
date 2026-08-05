//! MCC (Model Code of Conduct) applicability.
//!
//! The MCC is enforced by the Election Commission of India under its
//! plenary constitutional power (Article 324) to superintend, direct, and
//! control elections — it is not itself a numbered statutory provision
//! with fixed dates; a state's MCC window is a live administrative fact
//! (announced when ECI schedules an election for that state, lifted when
//! results are declared / the process concludes), never a formula this
//! crate could compute from a calendar. Per the crate root docs, that live
//! fact MUST be read from an injected input, never hardcoded.
//!
//! [`McWindow`] mirrors the `mcc_windows` table row shape used by
//! `channel_core::mcc_gate` and `admin_app::pages::mcc_panel`
//! (`state_name`/`window_start`/`window_end`/`is_active`) field-for-field,
//! so a caller can pass the same rows it already fetched for the admin
//! panel or the broadcast gate straight into this crate without
//! translation. This crate does not fetch those rows itself — that would
//! violate its zero-I/O constraint.
//!
//! Consistency note: `channel_core::mcc_gate::is_mcc_active_for_state`'s
//! actual query is `WHERE s.name = $1 AND w.is_active` — it trusts the
//! human-maintained `is_active` flag (toggled by a legal reviewer via the
//! admin MCC panel) directly, and does NOT independently recompute
//! "active" from `window_start`/`window_end`. [`evaluate_mcc_applicability`]
//! below deliberately does the same: it treats `is_active` as the
//! authority, and only uses the window's dates to make the trace `detail`
//! readable for a human. Having this crate apply a different,
//! date-based reinterpretation of the same flag would create two
//! disagreeing sources of truth for the same fact — worse than one shared
//! source of truth being wrong, since at least a single source can be
//! corrected in one place.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::citation::Citation;
use crate::trace::{RuleStep, Verdict};

pub const RULE_MCC_NOT_IN_FORCE: &str = "mcc.not_in_force_for_state";

/// Mirrors `admin_app::server_fns::MccWindowView` / the `mcc_windows` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McWindow {
    pub state_name: String,
    pub window_start: NaiveDate,
    pub window_end: Option<NaiveDate>,
    pub is_active: bool,
}

/// Whether the MCC is NOT currently in force for `state_name` (`Satisfied`
/// = clear to proceed with normal broadcast/mobilization content; framed
/// as the "clear" state, matching `crate::deadlines`'s convention of
/// naming the claim after the outcome that should read as good news).
///
/// `windows` should be every `McWindow` row relevant to `state_name` (in
/// practice: the full or state-filtered result of a `mcc_windows` query) —
/// this function does the state-name matching itself so a caller can pass
/// the whole table through unfiltered.
///
/// There is no `Verdict::CannotDetermine` case that turns on missing
/// *rows* — an empty or non-matching `windows` slice for a state simply
/// means no MCC window has ever been declared for it, which is a genuine,
/// confidently-known `Satisfied` (not-in-force), not an unknown. The
/// tri-state's `CannotDetermine` exists for missing INPUT, not for an
/// input that is present and says "nothing here."
pub fn evaluate_mcc_applicability(windows: &[McWindow], state_name: &str) -> RuleStep {
    let citation = Citation::statute("Constitution of India", "Art. 324 (ECI's plenary power to enforce the non-statutory Model Code of Conduct)");
    let description = "The Model Code of Conduct is not currently in force for this state.";

    match windows.iter().find(|w| w.is_active && w.state_name == state_name) {
        None => RuleStep {
            rule_id: RULE_MCC_NOT_IN_FORCE,
            description,
            citation,
            verdict: Verdict::Satisfied,
            detail: format!("No active MCC window is declared for \"{state_name}\"."),
        },
        Some(active) => RuleStep {
            rule_id: RULE_MCC_NOT_IN_FORCE,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: format!(
                "An active MCC window is declared for \"{state_name}\", started {start}{end}.",
                start = active.window_start,
                end = active
                    .window_end
                    .map(|d| format!(", scheduled to end {d}"))
                    .unwrap_or_else(|| " with no end date recorded yet".to_string()),
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn no_windows_at_all_is_satisfied_not_unknown() {
        let step = evaluate_mcc_applicability(&[], "Maharashtra");
        assert_eq!(step.verdict, Verdict::Satisfied);
    }

    #[test]
    fn active_window_for_the_named_state_is_not_satisfied() {
        let windows = vec![McWindow {
            state_name: "Maharashtra".to_string(),
            window_start: ymd(2026, 2, 1),
            window_end: None,
            is_active: true,
        }];
        let step = evaluate_mcc_applicability(&windows, "Maharashtra");
        assert_eq!(step.verdict, Verdict::NotSatisfied);
        assert!(step.detail.contains("Maharashtra"));
    }

    #[test]
    fn active_window_for_a_different_state_does_not_affect_this_one() {
        let windows = vec![McWindow {
            state_name: "Kerala".to_string(),
            window_start: ymd(2026, 2, 1),
            window_end: None,
            is_active: true,
        }];
        let step = evaluate_mcc_applicability(&windows, "Maharashtra");
        assert_eq!(step.verdict, Verdict::Satisfied);
    }

    #[test]
    fn closed_window_is_ignored_even_if_dates_would_still_span_as_of() {
        // Mirrors channel-core: is_active is authoritative, not the dates.
        let windows = vec![McWindow {
            state_name: "Maharashtra".to_string(),
            window_start: ymd(2026, 1, 1),
            window_end: Some(ymd(2026, 12, 31)),
            is_active: false,
        }];
        let step = evaluate_mcc_applicability(&windows, "Maharashtra");
        assert_eq!(step.verdict, Verdict::Satisfied);
    }

    #[test]
    fn citation_names_article_324_not_a_fabricated_statute_section() {
        let step = evaluate_mcc_applicability(&[], "Maharashtra");
        match step.citation {
            Citation::Statute { ref act, .. } => assert_eq!(act, "Constitution of India"),
            _ => panic!("expected a Statute citation"),
        }
    }
}
