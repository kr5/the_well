//! Elector eligibility rules — the "is this person eligible?" half of this
//! crate, grounded in Representation of the People Act, 1950 (RPA 1950)
//! ss. 16, 19 and 20/20A.
//!
//! Each rule below tests exactly one statutory condition and returns a
//! [`RuleStep`] with its own [`Verdict`] (never a combined yes/no) — the
//! driver [`evaluate_eligibility`] assembles them into one
//! [`EvaluationTrace`], and it is `EvaluationTrace::overall()` (not any
//! individual rule) that answers the citizen-facing "am I eligible"
//! question, always with the full "because x, y, z" trail attached.
//!
//! What this module does NOT do: it does not decide anyone's eligibility.
//! It evaluates the facts it is given against the rules it knows about. A
//! `Satisfied` overall verdict is "appears eligible based on what you told
//! us," not a legal determination — the ERO (Electoral Registration
//! Officer) is the actual decision-maker, per RPA 1950 s.19B/Registration
//! of Electors Rules 1960.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::citation::Citation;
use crate::qualifying_date::{applicable_qualifying_date, is_18_or_older_on, next_qualifying_date_after};
use crate::trace::{EvaluationTrace, RuleStep, Verdict};

pub const RULE_AGE_18: &str = "eligibility.age_18_on_qualifying_date";
pub const RULE_CITIZENSHIP: &str = "eligibility.citizenship";
pub const RULE_ORDINARY_RESIDENCE: &str = "eligibility.ordinary_residence";
pub const RULE_NOT_UNSOUND_MIND: &str = "eligibility.not_disqualified_unsound_mind";
pub const RULE_NOT_CORRUPT_PRACTICE: &str = "eligibility.not_disqualified_electoral_offence";

/// Citizenship status as relevant to elector eligibility (RPA 1950 s.19(a)
/// requires the person to be "a citizen of India").
///
/// Deliberately distinguishes an Overseas Citizen of India (OCI)
/// cardholder from an Indian citizen: OCI is a lifelong-visa-like status
/// under the Citizenship Act, 1955, NOT citizenship, and OCI cardholders
/// are NOT eligible to be electors on that status alone. This is a
/// frequent, genuine point of citizen confusion (OCI documentation is
/// sometimes mistaken for proof of citizenship) that this crate exists to
/// get right rather than paper over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Citizenship {
    Indian,
    ForeignNational,
    OciCardholderNotCitizen,
}

/// The applicant's residence situation, covering the well-known
/// "ordinary residence" nuances RPA 1950 s.20 and long-standing ERO
/// practice raise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResidenceSituation {
    /// Lives at the address being claimed with no complicating factor.
    OrdinarilyResidentAtClaimedAddress,
    /// A student living away from their native address in a hostel/mess
    /// for studies. Per SVEEP guidance (the `ordinary-residence-student`
    /// knowledge-base entry) they may register at EITHER their native
    /// address OR their present hostel/mess address, but not both — this
    /// crate can only confirm the residence test is satisfied under
    /// *whichever one* they've chosen; detecting an attempted double
    /// registration is out of scope (it requires cross-referencing rolls
    /// this crate has no access to, being zero-I/O).
    StudentAwayForStudies,
    /// Temporarily away from the claimed address (e.g. a short posting,
    /// medical treatment, travel) with a settled intention to return.
    /// "Ordinary residence" tolerates temporary absence — this does not,
    /// by itself, disqualify someone from claiming the address they
    /// intend to return to.
    TemporarilyAbsentIntendingToReturn,
    /// No longer resident at, and not intending to return to, the address
    /// in question. This is the situation Form 7 deletion exists for, not
    /// a basis for continued registration there — see `crate::forms`.
    NoLongerResidentAndNotReturning,
    /// An Indian citizen ordinarily resident outside India (an NRI in the
    /// popular sense) who has NOT acquired citizenship of another country,
    /// registering under RPA 1950 s.20A against the address in India shown
    /// in their passport. This is a genuinely distinct legal basis from
    /// ordinary "ordinary residence" — s.20A creates the entitlement to be
    /// registered directly, without any residence-in-India test, precisely
    /// because the whole premise is that the person is NOT resident in
    /// India.
    OverseasElectorUnderSection20a,
}

/// Everything the eligibility rules in this module need. Every field
/// except `as_of` is `Option` — a missing field is how "not asked" or "not
/// yet known" is represented, and the corresponding rule below reports
/// [`Verdict::CannotDetermine`] rather than guessing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EligibilityInput {
    pub date_of_birth: Option<NaiveDate>,
    /// The date this eligibility check is being made as of — needed to
    /// compute the applicable qualifying date. Not optional: without any
    /// reference date at all, no qualifying date can be identified, so a
    /// caller must always supply "today" (or whatever assessment date is
    /// relevant) even when every other field is unknown.
    pub as_of: NaiveDate,
    pub citizenship: Option<Citizenship>,
    pub residence: Option<ResidenceSituation>,
    /// Whether a competent court has declared this person of unsound mind
    /// (RPA 1950 s.16(1)(b)). Note the honest limitation: this crate
    /// cannot itself assess soundness of mind — the statute keys off a
    /// specific court declaration, which is exactly the yes/no/unknown
    /// this field carries, nothing more clinical than that.
    pub court_declared_unsound_mind: Option<bool>,
    /// Whether this person is currently disqualified from voting under
    /// the law relating to corrupt practices and electoral offences (RPA
    /// 1950 s.16(1)(c), read with RPA 1951's disqualification provisions).
    /// This crate does not adjudicate that determination itself — it can
    /// only reflect a yes/no/unknown sourced from an authoritative record
    /// (e.g. a court order, an ECI disqualification notification).
    pub disqualified_for_electoral_offence: Option<bool>,
}

fn evaluate_age(input: &EligibilityInput) -> RuleStep {
    let citation = Citation::kb("qualifying-dates");
    let qd = applicable_qualifying_date(input.as_of);
    match input.date_of_birth {
        None => RuleStep {
            rule_id: RULE_AGE_18,
            description: "Must be 18 years of age on or before the applicable qualifying date (RPA 1950 s.14(b), as amended 2022).",
            citation,
            verdict: Verdict::CannotDetermine,
            detail: format!(
                "Date of birth was not supplied. The applicable qualifying date as of {as_of} is {qd}, but age cannot be checked against it without a date of birth.",
                as_of = input.as_of
            ),
        },
        Some(dob) => {
            if is_18_or_older_on(dob, qd) {
                RuleStep {
                    rule_id: RULE_AGE_18,
                    description: "Must be 18 years of age on or before the applicable qualifying date (RPA 1950 s.14(b), as amended 2022).",
                    citation,
                    verdict: Verdict::Satisfied,
                    detail: format!("Born {dob}: already 18 or older on the applicable qualifying date, {qd}."),
                }
            } else {
                let next = next_qualifying_date_after(qd);
                RuleStep {
                    rule_id: RULE_AGE_18,
                    description: "Must be 18 years of age on or before the applicable qualifying date (RPA 1950 s.14(b), as amended 2022).",
                    citation,
                    verdict: Verdict::NotSatisfied,
                    detail: format!(
                        "Born {dob}: not yet 18 on the applicable qualifying date, {qd}. Since 2022 there are four qualifying dates a year, so re-check against the next one, {next}, once it becomes applicable — this crate does not assume they will qualify then, only that {qd} is not met now."
                    ),
                }
            }
        }
    }
}

fn evaluate_citizenship(input: &EligibilityInput) -> RuleStep {
    let citation = Citation::statute("Representation of the People Act, 1950", "s. 19(a)");
    let description = "Must be a citizen of India.";
    match input.citizenship {
        None => RuleStep {
            rule_id: RULE_CITIZENSHIP,
            description,
            citation,
            verdict: Verdict::CannotDetermine,
            detail: "Citizenship status was not supplied.".to_string(),
        },
        Some(Citizenship::Indian) => RuleStep {
            rule_id: RULE_CITIZENSHIP,
            description,
            citation,
            verdict: Verdict::Satisfied,
            detail: "Confirmed Indian citizen.".to_string(),
        },
        Some(Citizenship::ForeignNational) => RuleStep {
            rule_id: RULE_CITIZENSHIP,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: "Not an Indian citizen.".to_string(),
        },
        Some(Citizenship::OciCardholderNotCitizen) => RuleStep {
            rule_id: RULE_CITIZENSHIP,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: "An OCI (Overseas Citizen of India) card is a lifelong-visa status under the Citizenship Act, 1955, not Indian citizenship — an OCI cardholder who has not separately acquired Indian citizenship does not meet RPA 1950 s.19(a).".to_string(),
        },
    }
}

fn evaluate_ordinary_residence(input: &EligibilityInput) -> RuleStep {
    let description = "Must be ordinarily resident in the constituency claimed (RPA 1950 s.19(b)/s.20), or registering under the s.20A overseas-elector entitlement.";
    match input.residence {
        None => RuleStep {
            rule_id: RULE_ORDINARY_RESIDENCE,
            description,
            citation: Citation::statute("Representation of the People Act, 1950", "s. 19(b), s. 20"),
            verdict: Verdict::CannotDetermine,
            detail: "Residence situation was not supplied.".to_string(),
        },
        Some(ResidenceSituation::OrdinarilyResidentAtClaimedAddress) => RuleStep {
            rule_id: RULE_ORDINARY_RESIDENCE,
            description,
            citation: Citation::statute("Representation of the People Act, 1950", "s. 19(b), s. 20"),
            verdict: Verdict::Satisfied,
            detail: "Ordinarily resident at the claimed address.".to_string(),
        },
        Some(ResidenceSituation::StudentAwayForStudies) => RuleStep {
            rule_id: RULE_ORDINARY_RESIDENCE,
            description,
            citation: Citation::kb("ordinary-residence-student"),
            verdict: Verdict::Satisfied,
            detail: "A student away for studies may register at EITHER their native address OR their present hostel/mess address, not both — this rule only confirms one valid basis for registration exists, not which one was chosen or that both weren't claimed.".to_string(),
        },
        Some(ResidenceSituation::TemporarilyAbsentIntendingToReturn) => RuleStep {
            rule_id: RULE_ORDINARY_RESIDENCE,
            description,
            citation: Citation::statute("Representation of the People Act, 1950", "s. 20"),
            verdict: Verdict::Satisfied,
            detail: "Temporary absence with a settled intention to return does not break ordinary residence at the claimed address.".to_string(),
        },
        Some(ResidenceSituation::NoLongerResidentAndNotReturning) => RuleStep {
            rule_id: RULE_ORDINARY_RESIDENCE,
            description,
            citation: Citation::statute("Representation of the People Act, 1950", "s. 20"),
            verdict: Verdict::NotSatisfied,
            detail: "No longer ordinarily resident at, and not intending to return to, the address in question — this is a basis for a Form 7 deletion request at that address, not for continued registration there.".to_string(),
        },
        Some(ResidenceSituation::OverseasElectorUnderSection20a) => RuleStep {
            rule_id: RULE_ORDINARY_RESIDENCE,
            description,
            citation: Citation::kb("form-6a"),
            verdict: Verdict::Satisfied,
            detail: "Registering under RPA 1950 s.20A as an overseas elector against the Indian address shown in the passport — s.20A is a standalone entitlement that does not require ordinary residence in India at all.".to_string(),
        },
    }
}

fn evaluate_not_unsound_mind(input: &EligibilityInput) -> RuleStep {
    let citation = Citation::statute("Representation of the People Act, 1950", "s. 16(1)(b)");
    let description = "Must not be disqualified for having been declared of unsound mind by a competent court.";
    match input.court_declared_unsound_mind {
        None => RuleStep {
            rule_id: RULE_NOT_UNSOUND_MIND,
            description,
            citation,
            verdict: Verdict::CannotDetermine,
            detail: "Whether a competent court has made such a declaration was not supplied.".to_string(),
        },
        Some(false) => RuleStep {
            rule_id: RULE_NOT_UNSOUND_MIND,
            description,
            citation,
            verdict: Verdict::Satisfied,
            detail: "No court declaration of unsound mind on record.".to_string(),
        },
        Some(true) => RuleStep {
            rule_id: RULE_NOT_UNSOUND_MIND,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: "A competent court has declared this person of unsound mind — RPA 1950 s.16(1)(b) disqualifies from registration for as long as that declaration stands.".to_string(),
        },
    }
}

fn evaluate_not_disqualified_for_electoral_offence(input: &EligibilityInput) -> RuleStep {
    let citation = Citation::statute("Representation of the People Act, 1950", "s. 16(1)(c)");
    let description = "Must not be disqualified from voting under the law relating to corrupt practices and electoral offences.";
    match input.disqualified_for_electoral_offence {
        None => RuleStep {
            rule_id: RULE_NOT_CORRUPT_PRACTICE,
            description,
            citation,
            verdict: Verdict::CannotDetermine,
            detail: "Whether such a disqualification is currently in force was not supplied.".to_string(),
        },
        Some(false) => RuleStep {
            rule_id: RULE_NOT_CORRUPT_PRACTICE,
            description,
            citation,
            verdict: Verdict::Satisfied,
            detail: "No corrupt-practice/electoral-offence disqualification on record.".to_string(),
        },
        Some(true) => RuleStep {
            rule_id: RULE_NOT_CORRUPT_PRACTICE,
            description,
            citation,
            verdict: Verdict::NotSatisfied,
            detail: "Currently disqualified from voting for a corrupt practice or electoral offence under RPA 1950 s.16(1)(c) (read with RPA 1951's disqualification provisions) — this crate reflects that record, it does not itself determine guilt or the disqualification period.".to_string(),
        },
    }
}

/// Evaluate every eligibility rule against `input` and return the full,
/// ordered [`EvaluationTrace`]. Call `.overall()` on the result for the
/// single tri-state verdict, and iterate `.steps` (or the `satisfied`/
/// `not_satisfied`/`cannot_determine` helpers) for the "because x, y, z"
/// explanation.
pub fn evaluate_eligibility(input: &EligibilityInput) -> EvaluationTrace {
    let mut trace = EvaluationTrace::new();
    trace.push(evaluate_age(input));
    trace.push(evaluate_citizenship(input));
    trace.push(evaluate_ordinary_residence(input));
    trace.push(evaluate_not_unsound_mind(input));
    trace.push(evaluate_not_disqualified_for_electoral_offence(input));
    trace
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_eligible_input(as_of: NaiveDate) -> EligibilityInput {
        EligibilityInput {
            date_of_birth: Some(NaiveDate::from_ymd_opt(1990, 5, 20).unwrap()),
            as_of,
            citizenship: Some(Citizenship::Indian),
            residence: Some(ResidenceSituation::OrdinarilyResidentAtClaimedAddress),
            court_declared_unsound_mind: Some(false),
            disqualified_for_electoral_offence: Some(false),
        }
    }

    #[test]
    fn fully_satisfied_input_is_overall_satisfied() {
        let input = base_eligible_input(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        let trace = evaluate_eligibility(&input);
        assert_eq!(trace.overall(), Verdict::Satisfied);
        assert_eq!(trace.steps.len(), 5);
    }

    #[test]
    fn missing_date_of_birth_yields_cannot_determine_not_ineligible() {
        let mut input = base_eligible_input(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        input.date_of_birth = None;
        let trace = evaluate_eligibility(&input);
        // This is the crate's single most important behavioural guarantee:
        // a missing input must never present as "not eligible".
        assert_eq!(trace.overall(), Verdict::CannotDetermine);
        assert!(trace.overall() != Verdict::NotSatisfied);
        let age_step = trace.steps.iter().find(|s| s.rule_id == RULE_AGE_18).unwrap();
        assert_eq!(age_step.verdict, Verdict::CannotDetermine);
    }

    #[test]
    fn every_field_missing_yields_cannot_determine_with_five_unknown_steps() {
        let input = EligibilityInput {
            date_of_birth: None,
            as_of: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            citizenship: None,
            residence: None,
            court_declared_unsound_mind: None,
            disqualified_for_electoral_offence: None,
        };
        let trace = evaluate_eligibility(&input);
        assert_eq!(trace.overall(), Verdict::CannotDetermine);
        assert_eq!(trace.cannot_determine().count(), 5);
    }

    #[test]
    fn underage_applicant_is_not_satisfied_on_age_but_other_rules_still_run() {
        let mut input = base_eligible_input(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        input.date_of_birth = Some(NaiveDate::from_ymd_opt(2010, 6, 1).unwrap()); // 15 as of 2026-01-01
        let trace = evaluate_eligibility(&input);
        assert_eq!(trace.overall(), Verdict::NotSatisfied);
        let age_step = trace.steps.iter().find(|s| s.rule_id == RULE_AGE_18).unwrap();
        assert_eq!(age_step.verdict, Verdict::NotSatisfied);
        // Citizenship still evaluated independently, not short-circuited.
        let citizenship_step = trace.steps.iter().find(|s| s.rule_id == RULE_CITIZENSHIP).unwrap();
        assert_eq!(citizenship_step.verdict, Verdict::Satisfied);
    }

    #[test]
    fn oci_cardholder_is_not_satisfied_on_citizenship() {
        let mut input = base_eligible_input(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        input.citizenship = Some(Citizenship::OciCardholderNotCitizen);
        let trace = evaluate_eligibility(&input);
        assert_eq!(trace.overall(), Verdict::NotSatisfied);
        let step = trace.steps.iter().find(|s| s.rule_id == RULE_CITIZENSHIP).unwrap();
        assert_eq!(step.verdict, Verdict::NotSatisfied);
        assert!(step.detail.contains("OCI"));
    }

    #[test]
    fn nri_overseas_elector_under_section_20a_satisfies_residence_without_being_resident_in_india() {
        let mut input = base_eligible_input(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        input.residence = Some(ResidenceSituation::OverseasElectorUnderSection20a);
        let trace = evaluate_eligibility(&input);
        assert_eq!(trace.overall(), Verdict::Satisfied);
        let step = trace.steps.iter().find(|s| s.rule_id == RULE_ORDINARY_RESIDENCE).unwrap();
        assert_eq!(step.citation, Citation::kb("form-6a"));
    }

    #[test]
    fn student_away_for_studies_satisfies_residence() {
        let mut input = base_eligible_input(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        input.residence = Some(ResidenceSituation::StudentAwayForStudies);
        let trace = evaluate_eligibility(&input);
        let step = trace.steps.iter().find(|s| s.rule_id == RULE_ORDINARY_RESIDENCE).unwrap();
        assert_eq!(step.verdict, Verdict::Satisfied);
        assert_eq!(step.citation, Citation::kb("ordinary-residence-student"));
    }

    #[test]
    fn temporary_absence_does_not_break_ordinary_residence() {
        let mut input = base_eligible_input(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        input.residence = Some(ResidenceSituation::TemporarilyAbsentIntendingToReturn);
        let trace = evaluate_eligibility(&input);
        let step = trace.steps.iter().find(|s| s.rule_id == RULE_ORDINARY_RESIDENCE).unwrap();
        assert_eq!(step.verdict, Verdict::Satisfied);
    }

    #[test]
    fn no_longer_resident_and_not_returning_fails_residence() {
        let mut input = base_eligible_input(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        input.residence = Some(ResidenceSituation::NoLongerResidentAndNotReturning);
        let trace = evaluate_eligibility(&input);
        assert_eq!(trace.overall(), Verdict::NotSatisfied);
    }

    #[test]
    fn unsound_mind_declaration_disqualifies() {
        let mut input = base_eligible_input(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        input.court_declared_unsound_mind = Some(true);
        let trace = evaluate_eligibility(&input);
        assert_eq!(trace.overall(), Verdict::NotSatisfied);
        let step = trace.steps.iter().find(|s| s.rule_id == RULE_NOT_UNSOUND_MIND).unwrap();
        assert_eq!(step.verdict, Verdict::NotSatisfied);
    }

    #[test]
    fn electoral_offence_disqualification_fails_overall_even_with_age_unknown() {
        let mut input = base_eligible_input(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        input.date_of_birth = None;
        input.disqualified_for_electoral_offence = Some(true);
        let trace = evaluate_eligibility(&input);
        // A confirmed disqualification wins over an unrelated unknown —
        // see EvaluationTrace::overall's documented precedence.
        assert_eq!(trace.overall(), Verdict::NotSatisfied);
    }

    #[test]
    fn boundary_turning_18_exactly_on_each_qualifying_date_is_satisfied() {
        for (birth_month, birth_day, qd_month) in [(1, 1, 1), (4, 1, 4), (7, 1, 7), (10, 1, 10)] {
            let dob = NaiveDate::from_ymd_opt(2008, birth_month, birth_day).unwrap();
            let as_of = NaiveDate::from_ymd_opt(2026, qd_month, 1).unwrap();
            let mut input = base_eligible_input(as_of);
            input.date_of_birth = Some(dob);
            let trace = evaluate_eligibility(&input);
            assert_eq!(
                trace.overall(),
                Verdict::Satisfied,
                "born {dob} should be 18 as of qualifying date {as_of}"
            );
        }
    }

    #[test]
    fn boundary_turning_18_one_day_after_each_qualifying_date_is_not_yet_satisfied() {
        for (birth_month, birth_day, qd_month) in [(1, 2, 1), (4, 2, 4), (7, 2, 7), (10, 2, 10)] {
            let dob = NaiveDate::from_ymd_opt(2008, birth_month, birth_day).unwrap();
            let as_of = NaiveDate::from_ymd_opt(2026, qd_month, 1).unwrap();
            let mut input = base_eligible_input(as_of);
            input.date_of_birth = Some(dob);
            let trace = evaluate_eligibility(&input);
            assert_eq!(
                trace.overall(),
                Verdict::NotSatisfied,
                "born {dob} should not yet be 18 as of qualifying date {as_of}"
            );
        }
    }
}
