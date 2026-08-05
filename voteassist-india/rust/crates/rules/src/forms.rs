//! Form-selection rules — "which ECI form does this situation need?"
//!
//! Every rule here answers one question: does the elector's stated
//! situation call for this particular form. All six rules run against the
//! same input every time (there is no short-circuiting), so the resulting
//! [`EvaluationTrace`] doubles as an explanation of the forms that do NOT
//! apply as well as the ones that do — useful both to a citizen ("why not
//! Form 7?") and to an auditor.
//!
//! Scope note on Form 8: since the Registration of Electors (Amendment)
//! Rules, 2022 (in force 1 August 2022), Form 8 absorbed what used to be a
//! separate Form 8A (within-constituency transposition) and Form 001 (EPIC
//! replacement) — there is no separate "Form 8A rule" in this module
//! because there is no longer a separate form in ECI practice; the
//! `form-8` knowledge-base entry is the citation for that whole
//! consolidated scope. See `crate::deadlines` for the DIFFERENT question of
//! *by when* a given form must be filed — this module only ever answers
//! *which* form, deliberately kept separate.

use serde::{Deserialize, Serialize};

use crate::citation::Citation;
use crate::trace::{EvaluationTrace, RuleStep, Verdict};

pub const RULE_FORM_6: &str = "forms.form_6";
pub const RULE_FORM_6A: &str = "forms.form_6a";
pub const RULE_FORM_7: &str = "forms.form_7";
pub const RULE_FORM_8: &str = "forms.form_8";
pub const RULE_FORM_12D: &str = "forms.form_12d";
pub const RULE_FORM_2_OR_2A: &str = "forms.form_2_or_2a";

/// Why the citizen is interacting with the electoral roll right now. This
/// is deliberately an enum of *reasons*, not forms — the whole point of
/// this module is to derive the form from the reason, not have the
/// citizen (or an upstream decision tree) pick the form themselves.
///
/// Not `Copy` (unlike `eligibility`'s input enums) because
/// `PostalBallotCategory::OtherNotifiedAbsenteeCategory` carries an owned
/// `String` description — every rule below matches on `&input.reason`
/// rather than moving it, so this costs nothing at the call site.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormReason {
    /// Never before registered as an elector anywhere — including when
    /// establishing a new place of ordinary residence is what prompted
    /// registering for the first time. A first registration is ALWAYS
    /// Form 6, never Form 8, even if a house move is what occasioned it —
    /// Form 8's "shifting of residence" branch is only for someone who was
    /// already an elector somewhere and has moved.
    FirstTimeRegistration,
    /// Already an elector, and has moved to a new place of ordinary
    /// residence (within the same Assembly Constituency or to a different
    /// one — both are Form 8 since the 2022 consolidation).
    AlreadyRegisteredAndShiftedResidence,
    CorrectionOfParticulars,
    LostOrDamagedEpic,
    MarkAsPersonWithDisability,
    ObjectToAnotherPersonsInclusion,
    SeekDeletionOfAnEntry,
    WantsPostalBallotForAnElection(PostalBallotCategory),
}

/// The Form 12D absentee-voter categories. Deliberately does NOT enumerate
/// "COVID-19 suspect/affected" as a permanent, standing category: that was
/// itself an ad hoc ECI notification for particular elections during the
/// pandemic (under the same general absentee-voter rule-making power), not
/// a fixed statutory class. Modelling it as an open, described category
/// (`OtherNotifiedAbsenteeCategory`) is the honest way to leave room for
/// whatever category ECI notifies for a given election without this crate
/// pretending to know it in advance or going stale when the next such
/// notification differs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostalBallotCategory {
    SeniorCitizen85OrOlder,
    PersonWithDisability,
    EssentialServiceEmployee,
    /// `description` should name the specific ECI-notified category (e.g.
    /// "COVID-19 suspect/affected, per ECI notification dated ..."), since
    /// this crate cannot hardcode one.
    OtherNotifiedAbsenteeCategory { description: String },
}

/// Everything the form-selection rules need. As with [`crate::eligibility::EligibilityInput`],
/// every field is `Option` and a `None` means "not asked / not yet known",
/// never "no".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormSelectionInput {
    /// Already on the electoral roll anywhere (as an ordinary elector —
    /// not a service voter, which has its own `is_service_voter_category`
    /// flag below since service electors sit in a separate part of the
    /// roll per RPA 1950 s.20).
    pub already_registered_elector: Option<bool>,
    /// A member of the Armed Forces of the Union (or a force to which the
    /// Army Act, 1950 applies), or otherwise in a notified service-voter
    /// category (RPA 1950 s.20). Takes precedence over every other
    /// reason: a service-voter-category person registers via Form 2/2A on
    /// the dedicated service-voter mechanism, not ordinary Form 6/6A/8,
    /// regardless of what else is true of their situation.
    pub is_service_voter_category: Option<bool>,
    /// An Indian citizen ordinarily resident outside India who has not
    /// acquired another country's citizenship (RPA 1950 s.20A) and either
    /// wants to register, or already is registered, as an overseas
    /// elector.
    pub is_overseas_elector_category: Option<bool>,
    pub reason: Option<FormReason>,
}

fn evaluate_form_2_or_2a(input: &FormSelectionInput) -> RuleStep {
    let citation = Citation::kb("service-voter");
    let description = "Form 2/2A — Service Voter Registration (RPA 1950 s.20).";
    match input.is_service_voter_category {
        None => RuleStep {
            rule_id: RULE_FORM_2_OR_2A,
            description,
            citation,
            verdict: Verdict::CannotDetermine,
            detail: "Service-voter category status was not supplied.".to_string(),
        },
        Some(true) => RuleStep {
            rule_id: RULE_FORM_2_OR_2A,
            description,
            citation,
            verdict: Verdict::Satisfied,
            detail: "A service-voter-category elector registers via Form 2 (or Form 2A for a service voter's spouse) on the dedicated service-voter mechanism, not ordinary Form 6/6A/8.".to_string(),
        },
        Some(false) => RuleStep {
            rule_id: RULE_FORM_2_OR_2A,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: "Not in a service-voter category.".to_string(),
        },
    }
}

fn evaluate_form_6a(input: &FormSelectionInput) -> RuleStep {
    let citation = Citation::kb("form-6a");
    let description = "Form 6A — Overseas (NRI) Elector Registration (RPA 1950 s.20A).";
    if input.is_service_voter_category == Some(true) {
        return RuleStep {
            rule_id: RULE_FORM_6A,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: "Service-voter-category registration takes precedence over overseas-elector registration.".to_string(),
        };
    }
    match (input.is_overseas_elector_category, input.already_registered_elector) {
        (None, _) => RuleStep {
            rule_id: RULE_FORM_6A,
            description,
            citation,
            verdict: Verdict::CannotDetermine,
            detail: "Overseas-elector category status was not supplied.".to_string(),
        },
        (Some(false), _) => RuleStep {
            rule_id: RULE_FORM_6A,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: "Not in the overseas-elector category.".to_string(),
        },
        (Some(true), Some(true)) => RuleStep {
            rule_id: RULE_FORM_6A,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: "Already registered as an overseas elector — a correction, EPIC replacement, or PwD marking for an existing entry is Form 8, not Form 6A, which is for first registering as an overseas elector.".to_string(),
        },
        (Some(true), _) => RuleStep {
            rule_id: RULE_FORM_6A,
            description,
            citation,
            verdict: Verdict::Satisfied,
            detail: "In the overseas-elector category and not already registered as one: Form 6A registers against the Indian address shown in the passport.".to_string(),
        },
    }
}

fn evaluate_form_6(input: &FormSelectionInput) -> RuleStep {
    let citation = Citation::kb("form-6");
    let description = "Form 6 — New Elector Registration.";
    if input.is_service_voter_category == Some(true) {
        return RuleStep {
            rule_id: RULE_FORM_6,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: "Service-voter-category registration takes precedence.".to_string(),
        };
    }
    if input.is_overseas_elector_category == Some(true) {
        return RuleStep {
            rule_id: RULE_FORM_6,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: "An overseas-elector-category first registration is Form 6A, not Form 6.".to_string(),
        };
    }
    match &input.reason {
        None => RuleStep {
            rule_id: RULE_FORM_6,
            description,
            citation,
            verdict: Verdict::CannotDetermine,
            detail: "No reason for the roll interaction was supplied.".to_string(),
        },
        Some(FormReason::FirstTimeRegistration) => RuleStep {
            rule_id: RULE_FORM_6,
            description,
            citation,
            verdict: Verdict::Satisfied,
            detail: "First-time registration (including when establishing a new place of ordinary residence is what prompted it) is always Form 6, per the 2022 consolidation that moved corrections/shifting for EXISTING electors to Form 8.".to_string(),
        },
        Some(_) => RuleStep {
            rule_id: RULE_FORM_6,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: "Reason given is not a first-time registration.".to_string(),
        },
    }
}

fn evaluate_form_7(input: &FormSelectionInput) -> RuleStep {
    let citation = Citation::kb("form-7");
    let description = "Form 7 — Objection to Inclusion / Claim for Deletion.";
    match &input.reason {
        None => RuleStep {
            rule_id: RULE_FORM_7,
            description,
            citation,
            verdict: Verdict::CannotDetermine,
            detail: "No reason for the roll interaction was supplied.".to_string(),
        },
        Some(FormReason::ObjectToAnotherPersonsInclusion) | Some(FormReason::SeekDeletionOfAnEntry) => RuleStep {
            rule_id: RULE_FORM_7,
            description,
            citation,
            verdict: Verdict::Satisfied,
            detail: "Objecting to another person's inclusion, or seeking deletion of an entry (e.g. after death, a duplicate registration, or someone no longer resident at the address), is Form 7.".to_string(),
        },
        Some(_) => RuleStep {
            rule_id: RULE_FORM_7,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: "Reason given is not an objection or deletion request.".to_string(),
        },
    }
}

fn evaluate_form_8(input: &FormSelectionInput) -> RuleStep {
    let citation = Citation::kb("form-8");
    let description = "Form 8 — Shifting of Residence, Correction of Entries, EPIC Replacement, or PwD Marking (post-2022 consolidation; absorbs the former Form 8A).";
    if input.is_service_voter_category == Some(true) {
        return RuleStep {
            rule_id: RULE_FORM_8,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: "Service-voter-category electors use the dedicated service-voter mechanism for changes to their own entry, not Form 8.".to_string(),
        };
    }
    match &input.reason {
        None => RuleStep {
            rule_id: RULE_FORM_8,
            description,
            citation,
            verdict: Verdict::CannotDetermine,
            detail: "No reason for the roll interaction was supplied.".to_string(),
        },
        Some(
            FormReason::AlreadyRegisteredAndShiftedResidence
            | FormReason::CorrectionOfParticulars
            | FormReason::LostOrDamagedEpic
            | FormReason::MarkAsPersonWithDisability,
        ) => RuleStep {
            rule_id: RULE_FORM_8,
            description,
            citation,
            verdict: Verdict::Satisfied,
            detail: "Shifting of residence for an existing elector (same or different Assembly Constituency), correction of particulars, EPIC replacement, and PwD marking are all Form 8 since the Registration of Electors (Amendment) Rules, 2022.".to_string(),
        },
        Some(_) => RuleStep {
            rule_id: RULE_FORM_8,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: "Reason given does not fall under Form 8's consolidated scope.".to_string(),
        },
    }
}

fn evaluate_form_12d(input: &FormSelectionInput) -> RuleStep {
    let citation = Citation::kb("pwd-home-voting");
    let description = "Form 12D — Postal Ballot / Home Voting for eligible absentee categories at a specific election.";
    match &input.reason {
        None => RuleStep {
            rule_id: RULE_FORM_12D,
            description,
            citation,
            verdict: Verdict::CannotDetermine,
            detail: "No reason for the roll interaction was supplied.".to_string(),
        },
        Some(FormReason::WantsPostalBallotForAnElection(category)) => {
            if input.already_registered_elector == Some(false) {
                RuleStep {
                    rule_id: RULE_FORM_12D,
                    description,
                    citation,
                    verdict: Verdict::NotSatisfied,
                    detail: "Form 12D is only available to an already-registered elector for a specific election — register first (Form 6/6A) before an absentee-category postal ballot request can apply.".to_string(),
                }
            } else {
                let category_note = match category {
                    PostalBallotCategory::SeniorCitizen85OrOlder => "senior citizen aged 85 or older".to_string(),
                    PostalBallotCategory::PersonWithDisability => "person with disability".to_string(),
                    PostalBallotCategory::EssentialServiceEmployee => "essential-service employee on election duty".to_string(),
                    PostalBallotCategory::OtherNotifiedAbsenteeCategory { description: notified } => {
                        format!("ECI-notified absentee category ({notified})")
                    }
                };
                RuleStep {
                    rule_id: RULE_FORM_12D,
                    description,
                    citation,
                    verdict: Verdict::Satisfied,
                    detail: format!(
                        "Category matched: {category_note}. Form 12D is the right form for the category — see `crate::deadlines` for the SEPARATE question of whether the filing window for a specific, currently-notified election is open, which this rule does not determine."
                    ),
                }
            }
        }
        Some(_) => RuleStep {
            rule_id: RULE_FORM_12D,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: "Reason given is not a postal-ballot/home-voting request.".to_string(),
        },
    }
}

/// Evaluate every form-selection rule against `input` and return the full,
/// ordered [`EvaluationTrace`]. Use [`recommended_forms`] to extract just
/// the forms whose rule was `Satisfied`.
pub fn evaluate_form_selection(input: &FormSelectionInput) -> EvaluationTrace {
    let mut trace = EvaluationTrace::new();
    trace.push(evaluate_form_6(input));
    trace.push(evaluate_form_6a(input));
    trace.push(evaluate_form_7(input));
    trace.push(evaluate_form_8(input));
    trace.push(evaluate_form_12d(input));
    trace.push(evaluate_form_2_or_2a(input));
    trace
}

/// A recommended form, as extracted from an [`EvaluationTrace`] produced by
/// [`evaluate_form_selection`] — a small, typed convenience over matching
/// on `rule_id` strings directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormRecommendation {
    Form6,
    Form6a,
    Form7,
    Form8,
    Form12d,
    Form2Or2a,
}

const FORM_RULE_IDS: [(FormRecommendation, &str); 6] = [
    (FormRecommendation::Form6, RULE_FORM_6),
    (FormRecommendation::Form6a, RULE_FORM_6A),
    (FormRecommendation::Form7, RULE_FORM_7),
    (FormRecommendation::Form8, RULE_FORM_8),
    (FormRecommendation::Form12d, RULE_FORM_12D),
    (FormRecommendation::Form2Or2a, RULE_FORM_2_OR_2A),
];

/// Which forms this trace's rules came out `Satisfied` for. Empty if none
/// matched (including if every rule was `CannotDetermine` — this function
/// does not distinguish "no form applies" from "not enough information,"
/// deliberately: check `trace.cannot_determine()` separately before
/// treating an empty result as "no form needed."
pub fn recommended_forms(trace: &EvaluationTrace) -> Vec<FormRecommendation> {
    FORM_RULE_IDS
        .iter()
        .filter_map(|(form, rule_id)| {
            trace
                .steps
                .iter()
                .find(|s| s.rule_id == *rule_id)
                .filter(|s| s.verdict.is_satisfied())
                .map(|_| *form)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank_input() -> FormSelectionInput {
        FormSelectionInput {
            already_registered_elector: None,
            is_service_voter_category: None,
            is_overseas_elector_category: None,
            reason: None,
        }
    }

    #[test]
    fn first_time_registration_recommends_only_form_6() {
        let input = FormSelectionInput {
            already_registered_elector: Some(false),
            is_service_voter_category: Some(false),
            is_overseas_elector_category: Some(false),
            reason: Some(FormReason::FirstTimeRegistration),
        };
        let trace = evaluate_form_selection(&input);
        assert_eq!(recommended_forms(&trace), vec![FormRecommendation::Form6]);
    }

    #[test]
    fn overseas_first_registration_recommends_only_form_6a() {
        let input = FormSelectionInput {
            already_registered_elector: Some(false),
            is_service_voter_category: Some(false),
            is_overseas_elector_category: Some(true),
            reason: Some(FormReason::FirstTimeRegistration),
        };
        let trace = evaluate_form_selection(&input);
        assert_eq!(recommended_forms(&trace), vec![FormRecommendation::Form6a]);
    }

    #[test]
    fn already_registered_overseas_elector_needing_correction_gets_form_8_not_6a() {
        let input = FormSelectionInput {
            already_registered_elector: Some(true),
            is_service_voter_category: Some(false),
            is_overseas_elector_category: Some(true),
            reason: Some(FormReason::CorrectionOfParticulars),
        };
        let trace = evaluate_form_selection(&input);
        assert_eq!(recommended_forms(&trace), vec![FormRecommendation::Form8]);
    }

    #[test]
    fn objection_to_inclusion_recommends_form_7() {
        let input = FormSelectionInput {
            already_registered_elector: Some(true),
            is_service_voter_category: Some(false),
            is_overseas_elector_category: Some(false),
            reason: Some(FormReason::ObjectToAnotherPersonsInclusion),
        };
        let trace = evaluate_form_selection(&input);
        assert_eq!(recommended_forms(&trace), vec![FormRecommendation::Form7]);
    }

    #[test]
    fn deletion_request_recommends_form_7() {
        let input = FormSelectionInput {
            already_registered_elector: Some(true),
            is_service_voter_category: Some(false),
            is_overseas_elector_category: Some(false),
            reason: Some(FormReason::SeekDeletionOfAnEntry),
        };
        let trace = evaluate_form_selection(&input);
        assert_eq!(recommended_forms(&trace), vec![FormRecommendation::Form7]);
    }

    #[test]
    fn shifted_residence_recommends_form_8() {
        let input = FormSelectionInput {
            already_registered_elector: Some(true),
            is_service_voter_category: Some(false),
            is_overseas_elector_category: Some(false),
            reason: Some(FormReason::AlreadyRegisteredAndShiftedResidence),
        };
        let trace = evaluate_form_selection(&input);
        assert_eq!(recommended_forms(&trace), vec![FormRecommendation::Form8]);
    }

    #[test]
    fn lost_epic_recommends_form_8() {
        let input = FormSelectionInput {
            already_registered_elector: Some(true),
            is_service_voter_category: Some(false),
            is_overseas_elector_category: Some(false),
            reason: Some(FormReason::LostOrDamagedEpic),
        };
        let trace = evaluate_form_selection(&input);
        assert_eq!(recommended_forms(&trace), vec![FormRecommendation::Form8]);
    }

    #[test]
    fn pwd_marking_recommends_form_8() {
        let input = FormSelectionInput {
            already_registered_elector: Some(true),
            is_service_voter_category: Some(false),
            is_overseas_elector_category: Some(false),
            reason: Some(FormReason::MarkAsPersonWithDisability),
        };
        let trace = evaluate_form_selection(&input);
        assert_eq!(recommended_forms(&trace), vec![FormRecommendation::Form8]);
    }

    #[test]
    fn postal_ballot_for_registered_senior_recommends_form_12d() {
        let input = FormSelectionInput {
            already_registered_elector: Some(true),
            is_service_voter_category: Some(false),
            is_overseas_elector_category: Some(false),
            reason: Some(FormReason::WantsPostalBallotForAnElection(
                PostalBallotCategory::SeniorCitizen85OrOlder,
            )),
        };
        let trace = evaluate_form_selection(&input);
        assert_eq!(recommended_forms(&trace), vec![FormRecommendation::Form12d]);
    }

    #[test]
    fn postal_ballot_request_from_an_unregistered_person_does_not_get_form_12d() {
        let input = FormSelectionInput {
            already_registered_elector: Some(false),
            is_service_voter_category: Some(false),
            is_overseas_elector_category: Some(false),
            reason: Some(FormReason::WantsPostalBallotForAnElection(
                PostalBallotCategory::PersonWithDisability,
            )),
        };
        let trace = evaluate_form_selection(&input);
        assert!(recommended_forms(&trace).is_empty());
        let step = trace.steps.iter().find(|s| s.rule_id == RULE_FORM_12D).unwrap();
        assert_eq!(step.verdict, Verdict::NotSatisfied);
    }

    #[test]
    fn other_notified_absentee_category_is_open_ended_not_hardcoded() {
        let input = FormSelectionInput {
            already_registered_elector: Some(true),
            is_service_voter_category: Some(false),
            is_overseas_elector_category: Some(false),
            reason: Some(FormReason::WantsPostalBallotForAnElection(
                PostalBallotCategory::OtherNotifiedAbsenteeCategory {
                    description: "COVID-19 suspect/affected, per ECI notification dated 2020".to_string(),
                },
            )),
        };
        let trace = evaluate_form_selection(&input);
        assert_eq!(recommended_forms(&trace), vec![FormRecommendation::Form12d]);
        let step = trace.steps.iter().find(|s| s.rule_id == RULE_FORM_12D).unwrap();
        assert!(step.detail.contains("COVID-19"));
    }

    #[test]
    fn service_voter_category_recommends_form_2_or_2a_and_overrides_everything_else() {
        let input = FormSelectionInput {
            already_registered_elector: Some(false),
            is_service_voter_category: Some(true),
            is_overseas_elector_category: Some(true), // deliberately contradictory-looking input
            reason: Some(FormReason::FirstTimeRegistration),
        };
        let trace = evaluate_form_selection(&input);
        assert_eq!(recommended_forms(&trace), vec![FormRecommendation::Form2Or2a]);
    }

    #[test]
    fn blank_input_yields_all_cannot_determine_and_no_recommendations() {
        let trace = evaluate_form_selection(&blank_input());
        assert_eq!(trace.steps.len(), 6);
        assert_eq!(trace.cannot_determine().count(), 6);
        assert!(recommended_forms(&trace).is_empty());
    }

    #[test]
    fn trace_explains_why_forms_were_not_selected_too() {
        let input = FormSelectionInput {
            already_registered_elector: Some(false),
            is_service_voter_category: Some(false),
            is_overseas_elector_category: Some(false),
            reason: Some(FormReason::FirstTimeRegistration),
        };
        let trace = evaluate_form_selection(&input);
        let form7_step = trace.steps.iter().find(|s| s.rule_id == RULE_FORM_7).unwrap();
        assert_eq!(form7_step.verdict, Verdict::NotSatisfied);
        assert!(!form7_step.detail.is_empty());
    }
}
