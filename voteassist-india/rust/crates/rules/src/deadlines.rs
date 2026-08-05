//! Deadline rules — "by when?"
//!
//! This is the one rule category in this crate where the ACCURACY BAR
//! forces genuine restraint: the exact number of days for "the last date
//! for filing claims and objections" in a roll-revision round, and the
//! exact date the roll is treated as frozen ahead of a specific poll, are
//! NOT standing statutory constants — they are fixed afresh by ECI
//! notification for each revision round / each election, and they
//! genuinely vary (different states, different rounds, different election
//! cycles have used different windows). Inventing a plausible-sounding
//! number here would be exactly the failure mode this crate exists to
//! avoid: a citizen told a specific date this crate made up, when the real
//! date is whatever the actual notification says.
//!
//! So every function in this module takes the actual dates as an injected
//! [`ElectionsCalendarInput`] rather than computing them from a formula,
//! and every rule reports [`Verdict::CannotDetermine`] — not a guess, not
//! "not satisfied" — when the relevant calendar date hasn't been supplied.
//! What this module DOES contribute on its own: the *test* of "has this
//! date passed relative to `as_of`", so a caller only ever has to plug in
//! the one number ECI actually published, not re-derive the comparison
//! logic at every call site.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::citation::Citation;
use crate::trace::{RuleStep, Verdict};

pub const RULE_CLAIMS_AND_OBJECTIONS_WINDOW_OPEN: &str = "deadlines.claims_and_objections_window_open";
pub const RULE_ROLL_NOT_YET_FROZEN_FOR_POLL: &str = "deadlines.roll_not_yet_frozen_for_poll";

/// The election-calendar facts this crate cannot derive on its own and
/// refuses to invent. Every field is `Option` for exactly that reason —
/// populate it from the actual ECI notification for the round/election in
/// question (e.g. a `mcc_windows`-style admin-entered calendar row, or a
/// future dedicated elections-calendar table); this crate has no opinion
/// on where that data lives, only on how to reason once it's supplied.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElectionsCalendarInput {
    /// The last date, in the CURRENT roll-revision round, on which Form 6/
    /// 6A/7/8 claims and objections may be filed (Registration of Electors
    /// Rules, 1960, rule 15 — the ERO/ECI fixes "a reasonable time," which
    /// this crate does not attempt to compute or default). `None` means
    /// "not yet notified for this round" — a real and common state, not
    /// missing data to be worked around.
    pub claims_and_objections_last_date: Option<NaiveDate>,
    /// The date on which the electoral roll used for a SPECIFIC,
    /// currently-notified poll is treated as frozen (no further
    /// additions/deletions processed against it for that poll), as fixed
    /// by the relevant ECI notification for that election. This is
    /// distinct from `claims_and_objections_last_date` above, which
    /// belongs to the ongoing periodic revision cycle, not to any one
    /// election. `None` means no such freeze date has been notified yet
    /// (or this election's calendar simply hasn't been supplied).
    pub roll_freeze_date_for_poll: Option<NaiveDate>,
}

/// Whether, as of `as_of`, the current roll-revision round's claims-and-
/// objections window is still open. The claim being tested is framed as
/// the OPEN state (`Satisfied` = still open) so [`Verdict::NotSatisfied`]
/// reads naturally as "window has closed" rather than double-negating.
pub fn evaluate_claims_and_objections_window_open(
    calendar: &ElectionsCalendarInput,
    as_of: NaiveDate,
) -> RuleStep {
    let citation = Citation::statute("Registration of Electors Rules, 1960", "rule 15");
    let description = "The claims-and-objections window for the current roll-revision round is still open.";
    match calendar.claims_and_objections_last_date {
        None => RuleStep {
            rule_id: RULE_CLAIMS_AND_OBJECTIONS_WINDOW_OPEN,
            description,
            citation,
            verdict: Verdict::CannotDetermine,
            detail: "No last-date-for-claims-and-objections has been supplied for the current round. This crate does not compute or default that date — it must come from the actual ECI/ERO notification for this revision round.".to_string(),
        },
        Some(last_date) if as_of <= last_date => RuleStep {
            rule_id: RULE_CLAIMS_AND_OBJECTIONS_WINDOW_OPEN,
            description,
            citation,
            verdict: Verdict::Satisfied,
            detail: format!("As of {as_of}, the notified last date ({last_date}) has not yet passed."),
        },
        Some(last_date) => RuleStep {
            rule_id: RULE_CLAIMS_AND_OBJECTIONS_WINDOW_OPEN,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: format!("As of {as_of}, the notified last date ({last_date}) has passed for this round — a fresh round's dates (or a specific election's Form 6/8/12D provisions) would need to be checked instead."),
        },
    }
}

/// Whether, as of `as_of`, the electoral roll for a specific, currently-
/// notified poll is NOT yet frozen (`Satisfied` = not yet frozen, i.e.
/// still updatable for that poll).
pub fn evaluate_roll_not_yet_frozen_for_poll(
    calendar: &ElectionsCalendarInput,
    as_of: NaiveDate,
) -> RuleStep {
    let citation = Citation::missing(
        "The specific date on which a roll is frozen ahead of a given poll is fixed per-election by ECI notification (typically tied to that election's notification under RPA 1951 s.30 and the Conduct of Elections Rules, 1961's nomination timeline), not a single standing rule/section this crate can cite generically. Citing a specific section here would overstate how standardised this date actually is across elections — see this module's doc comment.",
    );
    let description = "The electoral roll for a specific, currently-notified poll is not yet frozen.";
    match calendar.roll_freeze_date_for_poll {
        None => RuleStep {
            rule_id: RULE_ROLL_NOT_YET_FROZEN_FOR_POLL,
            description,
            citation,
            verdict: Verdict::CannotDetermine,
            detail: "No roll-freeze date has been supplied for a specific poll. This crate does not compute or default that date — it must come from the actual ECI notification for that election.".to_string(),
        },
        Some(freeze_date) if as_of < freeze_date => RuleStep {
            rule_id: RULE_ROLL_NOT_YET_FROZEN_FOR_POLL,
            description,
            citation,
            verdict: Verdict::Satisfied,
            detail: format!("As of {as_of}, the notified freeze date ({freeze_date}) has not yet arrived."),
        },
        Some(freeze_date) => RuleStep {
            rule_id: RULE_ROLL_NOT_YET_FROZEN_FOR_POLL,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: format!("As of {as_of}, the roll is frozen for this poll from {freeze_date} — no further additions/deletions are processed against it for this election."),
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
    fn missing_last_date_is_cannot_determine_never_not_satisfied() {
        let calendar = ElectionsCalendarInput::default();
        let step = evaluate_claims_and_objections_window_open(&calendar, ymd(2026, 3, 1));
        assert_eq!(step.verdict, Verdict::CannotDetermine);
    }

    #[test]
    fn window_still_open_before_notified_last_date() {
        let calendar = ElectionsCalendarInput {
            claims_and_objections_last_date: Some(ymd(2026, 3, 15)),
            ..Default::default()
        };
        let step = evaluate_claims_and_objections_window_open(&calendar, ymd(2026, 3, 1));
        assert_eq!(step.verdict, Verdict::Satisfied);
    }

    #[test]
    fn window_still_open_exactly_on_notified_last_date() {
        let calendar = ElectionsCalendarInput {
            claims_and_objections_last_date: Some(ymd(2026, 3, 15)),
            ..Default::default()
        };
        let step = evaluate_claims_and_objections_window_open(&calendar, ymd(2026, 3, 15));
        assert_eq!(step.verdict, Verdict::Satisfied);
    }

    #[test]
    fn window_closed_after_notified_last_date() {
        let calendar = ElectionsCalendarInput {
            claims_and_objections_last_date: Some(ymd(2026, 3, 15)),
            ..Default::default()
        };
        let step = evaluate_claims_and_objections_window_open(&calendar, ymd(2026, 3, 16));
        assert_eq!(step.verdict, Verdict::NotSatisfied);
    }

    #[test]
    fn missing_freeze_date_is_cannot_determine() {
        let calendar = ElectionsCalendarInput::default();
        let step = evaluate_roll_not_yet_frozen_for_poll(&calendar, ymd(2026, 3, 1));
        assert_eq!(step.verdict, Verdict::CannotDetermine);
        // This one deliberately has no standing statutory citation to give
        // (see the module doc) — make sure that honesty is visible, not
        // papered over with a fabricated section number.
        assert!(step.citation.is_missing());
    }

    #[test]
    fn roll_not_yet_frozen_before_notified_freeze_date() {
        let calendar = ElectionsCalendarInput {
            roll_freeze_date_for_poll: Some(ymd(2026, 5, 1)),
            ..Default::default()
        };
        let step = evaluate_roll_not_yet_frozen_for_poll(&calendar, ymd(2026, 4, 20));
        assert_eq!(step.verdict, Verdict::Satisfied);
    }

    #[test]
    fn roll_frozen_from_notified_freeze_date_onward() {
        let calendar = ElectionsCalendarInput {
            roll_freeze_date_for_poll: Some(ymd(2026, 5, 1)),
            ..Default::default()
        };
        let on_date = evaluate_roll_not_yet_frozen_for_poll(&calendar, ymd(2026, 5, 1));
        assert_eq!(on_date.verdict, Verdict::NotSatisfied);
        let after_date = evaluate_roll_not_yet_frozen_for_poll(&calendar, ymd(2026, 5, 2));
        assert_eq!(after_date.verdict, Verdict::NotSatisfied);
    }
}
