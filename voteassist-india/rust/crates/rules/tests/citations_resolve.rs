//! Cross-crate consistency check, mirroring `core-domain`'s
//! `tests/knowledge_integration.rs`: every `Citation::KnowledgeBase` this
//! crate hands out from a rule must actually resolve to a real, loaded
//! `kb-content` entry. `rules`'s production code does not depend on
//! `kb-content` (see this crate's `Cargo.toml`), so this is a dev-only
//! guard against a citation silently rotting if a knowledge-base entry is
//! renamed or removed.

use chrono::NaiveDate;
use rules::{
    eligibility::{evaluate_eligibility, Citizenship, EligibilityInput, ResidenceSituation},
    forms::{evaluate_form_selection, FormReason, FormSelectionInput, PostalBallotCategory},
    Citation,
};

fn all_traced_kb_entry_ids() -> Vec<String> {
    let mut ids = Vec::new();

    // Eligibility: exercise every ResidenceSituation variant so every
    // branch's citation is visited at least once.
    for residence in [
        ResidenceSituation::OrdinarilyResidentAtClaimedAddress,
        ResidenceSituation::StudentAwayForStudies,
        ResidenceSituation::TemporarilyAbsentIntendingToReturn,
        ResidenceSituation::NoLongerResidentAndNotReturning,
        ResidenceSituation::OverseasElectorUnderSection20a,
    ] {
        let input = EligibilityInput {
            date_of_birth: Some(NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()),
            as_of: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            citizenship: Some(Citizenship::Indian),
            residence: Some(residence),
            court_declared_unsound_mind: Some(false),
            disqualified_for_electoral_offence: Some(false),
        };
        for step in evaluate_eligibility(&input).steps {
            if let Citation::KnowledgeBase { entry_id } = step.citation {
                ids.push(entry_id);
            }
        }
    }

    // Forms: exercise every reason so every form rule's citation fires.
    let reasons = [
        FormReason::FirstTimeRegistration,
        FormReason::AlreadyRegisteredAndShiftedResidence,
        FormReason::CorrectionOfParticulars,
        FormReason::LostOrDamagedEpic,
        FormReason::MarkAsPersonWithDisability,
        FormReason::ObjectToAnotherPersonsInclusion,
        FormReason::SeekDeletionOfAnEntry,
        FormReason::WantsPostalBallotForAnElection(PostalBallotCategory::SeniorCitizen85OrOlder),
    ];
    for reason in reasons {
        let input = FormSelectionInput {
            already_registered_elector: Some(true),
            is_service_voter_category: Some(false),
            is_overseas_elector_category: Some(false),
            reason: Some(reason),
        };
        for step in evaluate_form_selection(&input).steps {
            if let Citation::KnowledgeBase { entry_id } = step.citation {
                ids.push(entry_id);
            }
        }
    }

    // Service-voter form rule, and Form 6A's own-branch citation.
    let service_input = FormSelectionInput {
        already_registered_elector: Some(false),
        is_service_voter_category: Some(true),
        is_overseas_elector_category: Some(false),
        reason: Some(FormReason::FirstTimeRegistration),
    };
    for step in evaluate_form_selection(&service_input).steps {
        if let Citation::KnowledgeBase { entry_id } = step.citation {
            ids.push(entry_id);
        }
    }

    let overseas_input = FormSelectionInput {
        already_registered_elector: Some(false),
        is_service_voter_category: Some(false),
        is_overseas_elector_category: Some(true),
        reason: Some(FormReason::FirstTimeRegistration),
    };
    for step in evaluate_form_selection(&overseas_input).steps {
        if let Citation::KnowledgeBase { entry_id } = step.citation {
            ids.push(entry_id);
        }
    }

    ids.sort();
    ids.dedup();
    ids
}

#[test]
fn every_knowledge_base_citation_this_crate_hands_out_resolves() {
    let ids = all_traced_kb_entry_ids();
    assert!(!ids.is_empty(), "expected at least one KnowledgeBase citation to be exercised");
    for id in &ids {
        assert!(
            kb_content::get_entry(id).is_some(),
            "rules crate cites knowledge-base entry \"{id}\" which does not exist in kb-content — a citation has rotted"
        );
    }
}

#[test]
fn expected_entry_ids_are_exactly_the_ones_this_crate_relies_on() {
    let ids = all_traced_kb_entry_ids();
    let expected: Vec<String> = [
        "form-6",
        "form-6a",
        "form-7",
        "form-8",
        "ordinary-residence-student",
        "pwd-home-voting",
        "qualifying-dates",
        "service-voter",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    assert_eq!(ids, expected, "the set of knowledge-base entries this crate cites has changed — update this test deliberately if that's expected");
}
