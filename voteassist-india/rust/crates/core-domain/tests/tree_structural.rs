//! Rust port of packages/decision-engine/test/tree.test.ts.

use core_domain::{validate_tree, vote_assist_tree_v1, DecisionNode};

#[test]
fn tree_has_no_validation_issues() {
    let issues = validate_tree(vote_assist_tree_v1());
    assert!(
        issues.is_empty(),
        "expected no validation issues, got: {:?}",
        issues
    );
}

#[test]
fn every_question_option_has_en_and_hi_labels() {
    for node in vote_assist_tree_v1().nodes.values() {
        if let DecisionNode::Question(q) = node {
            assert!(!q.prompt.en.is_empty(), "node {} missing en prompt", q.id);
            assert!(
                q.prompt.other.contains_key("hi"),
                "node {} missing hi prompt",
                q.id
            );
            for option in &q.options {
                assert!(
                    !option.label.en.is_empty(),
                    "node {} option {} missing en label",
                    q.id,
                    option.value
                );
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
fn every_terminal_has_full_bilingual_content_and_citations() {
    for node in vote_assist_tree_v1().nodes.values() {
        if let DecisionNode::Terminal(t) = node {
            assert!(!t.outcome_title.en.is_empty());
            assert!(t.outcome_title.other.contains_key("hi"));
            assert!(!t.outcome_description.en.is_empty());
            assert!(t.outcome_description.other.contains_key("hi"));
            assert!(!t.caution.en.is_empty());
            assert!(t.caution.other.contains_key("hi"));
            assert!(
                !t.citations.is_empty(),
                "terminal {} has no citations",
                t.id
            );
            assert!(
                !t.deep_links.is_empty(),
                "terminal {} has no deep links",
                t.id
            );
            for link in &t.deep_links {
                url::Url::parse(&link.url).unwrap_or_else(|e| {
                    panic!(
                        "terminal {} has an invalid deep link URL {}: {e}",
                        t.id, link.url
                    )
                });
            }
        }
    }
}

#[test]
fn every_major_scenario_has_a_terminal() {
    let expected = [
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
    for id in expected {
        assert!(
            vote_assist_tree_v1().nodes.contains_key(id),
            "missing expected terminal {id}"
        );
    }
}
