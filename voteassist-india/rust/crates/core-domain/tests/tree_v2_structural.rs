//! Structural validity tests for the expanded v2 tree, mirroring
//! tree_structural.rs's checks for v1.

use core_domain::{validate_tree, vote_assist_tree_v2, DecisionNode};

#[test]
fn tree_v2_has_no_validation_issues() {
    let issues = validate_tree(vote_assist_tree_v2());
    assert!(issues.is_empty(), "expected no validation issues, got: {:?}", issues);
}

#[test]
fn tree_v2_every_question_option_has_en_and_hi_labels() {
    for node in vote_assist_tree_v2().nodes.values() {
        if let DecisionNode::Question(q) = node {
            assert!(!q.prompt.en.is_empty(), "node {} missing en prompt", q.id);
            assert!(q.prompt.other.contains_key("hi"), "node {} missing hi prompt", q.id);
            for option in &q.options {
                assert!(!option.label.en.is_empty(), "node {} option {} missing en label", q.id, option.value);
                assert!(
                    option.label.other.contains_key("hi"),
                    "node {} option {} missing hi label",
                    q.id,
                    option.value
                );
            }
        }
    }
}

#[test]
fn tree_v2_every_terminal_has_full_bilingual_content_and_citations() {
    for node in vote_assist_tree_v2().nodes.values() {
        if let DecisionNode::Terminal(t) = node {
            assert!(!t.outcome_title.en.is_empty());
            assert!(t.outcome_title.other.contains_key("hi"));
            assert!(!t.outcome_description.en.is_empty());
            assert!(t.outcome_description.other.contains_key("hi"));
            assert!(!t.caution.en.is_empty());
            assert!(t.caution.other.contains_key("hi"));
            assert!(!t.citations.is_empty(), "terminal {} has no citations", t.id);
            assert!(!t.deep_links.is_empty(), "terminal {} has no deep links", t.id);
            for link in &t.deep_links {
                url::Url::parse(&link.url)
                    .unwrap_or_else(|e| panic!("terminal {} has an invalid deep link URL {}: {e}", t.id, link.url));
            }
        }
    }
}

#[test]
fn tree_v2_covers_every_new_scenario_from_the_v2_spec() {
    let expected_new_terminals = [
        "terminal_form8_shift_and_correction",
        "terminal_deleted_entry_contest",
        "terminal_senior_postal_ballot",
        "terminal_documents_alternatives",
        "terminal_address_proof_local_verification",
        "terminal_gender_marker_update",
        "terminal_polling_station_changed",
    ];
    for id in expected_new_terminals {
        assert!(vote_assist_tree_v2().nodes.contains_key(id), "missing expected v2 terminal {id}");
    }
}

#[test]
fn tree_v2_still_covers_every_v1_scenario() {
    let expected_v1_terminals = [
        "terminal_roll_search",
        "terminal_not_yet_eligible",
        "terminal_qualifying_date",
        "terminal_form6a",
        "terminal_service_voter",
        "terminal_form6_new",
        "terminal_form6_student_hostel",
        "terminal_form6_no_fixed_address",
        "terminal_form8_correction",
        "terminal_form8_shift",
        "terminal_eepic_download",
        "terminal_form8_epic_replacement",
        "terminal_pwd_marking",
        "terminal_form7",
    ];
    for id in expected_v1_terminals {
        assert!(vote_assist_tree_v2().nodes.contains_key(id), "v2 dropped v1 terminal {id}");
    }
}

#[test]
fn tree_v2_has_grown_meaningfully_beyond_v1() {
    let v1_len = core_domain::vote_assist_tree_v1().nodes.len();
    let v2_len = vote_assist_tree_v2().nodes.len();
    assert!(v2_len > v1_len, "v2 ({v2_len} nodes) should be strictly larger than v1 ({v1_len} nodes)");
}
