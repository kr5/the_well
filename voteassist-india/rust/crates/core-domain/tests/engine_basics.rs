//! Rust port of packages/decision-engine/test/engine.test.ts.

use core_domain::{
    answer, create_session, estimate_progress, get_current_node, is_session_complete,
    vote_assist_tree_v1, Answer, DecisionNode, EngineError,
};

#[test]
fn creates_a_session_at_the_start_node() {
    let tree = vote_assist_tree_v1();
    let state = create_session(tree).unwrap();
    assert_eq!(state.current_node_id, "start");
    assert!(state.history.is_empty());
}

#[test]
fn walks_the_nri_path_to_form_6a() {
    let tree = vote_assist_tree_v1();
    let mut state = create_session(tree).unwrap();
    state = answer(tree, &state, "no").unwrap();
    state = answer(tree, &state, "adult").unwrap();
    state = answer(tree, &state, "nri").unwrap();

    assert!(is_session_complete(tree, &state).unwrap());
    let node = get_current_node(tree, &state).unwrap();
    assert_eq!(node.id(), "terminal_form6a");
    assert_eq!(
        state.history,
        vec![
            Answer {
                question_id: "start".into(),
                value: "no".into()
            },
            Answer {
                question_id: "not_registered_age".into(),
                value: "adult".into()
            },
            Answer {
                question_id: "citizenship_check".into(),
                value: "nri".into()
            },
        ]
    );
}

#[test]
fn walks_the_student_hostel_path() {
    let tree = vote_assist_tree_v1();
    let mut state = create_session(tree).unwrap();
    state = answer(tree, &state, "no").unwrap();
    state = answer(tree, &state, "adult").unwrap();
    state = answer(tree, &state, "resident").unwrap();
    state = answer(tree, &state, "student_hostel").unwrap();
    state = answer(tree, &state, "hostel").unwrap();
    assert_eq!(
        get_current_node(tree, &state).unwrap().id(),
        "terminal_form6_student_hostel"
    );
}

#[test]
fn rejects_an_invalid_answer_value() {
    let tree = vote_assist_tree_v1();
    let state = create_session(tree).unwrap();
    let err = answer(tree, &state, "not-a-real-option").unwrap_err();
    assert!(matches!(err, EngineError::InvalidAnswer { .. }));
}

#[test]
fn refuses_to_answer_a_terminal_node() {
    let tree = vote_assist_tree_v1();
    let mut state = create_session(tree).unwrap();
    state = answer(tree, &state, "not_sure").unwrap(); // -> terminal_roll_search
    assert!(is_session_complete(tree, &state).unwrap());
    let err = answer(tree, &state, "anything").unwrap_err();
    assert!(matches!(err, EngineError::TerminalReached { .. }));
}

#[test]
fn progress_increases_monotonically_to_one_at_a_terminal() {
    let tree = vote_assist_tree_v1();
    let mut state = create_session(tree).unwrap();
    let mut progress = estimate_progress(tree, &state).unwrap();
    assert_eq!(progress, 0.0);

    for value in ["no", "adult", "resident", "student_hostel", "hostel"] {
        let next = answer(tree, &state, value).unwrap();
        let next_progress = estimate_progress(tree, &next).unwrap();
        assert!(next_progress >= progress);
        state = next;
        progress = next_progress;
    }
    assert_eq!(progress, 1.0);
}

/// Exhaustive path enumeration: every combination of answers reaches a
/// terminal node within a bounded number of steps. Rust port of the same
/// exhaustive-path test in the TypeScript prototype's engine.test.ts.
#[test]
fn every_combination_of_answers_terminates() {
    fn enumerate(tree: &core_domain::DecisionTree, state: core_domain::EngineState, depth: u32) {
        assert!(
            depth <= 20,
            "path did not terminate within 20 steps — possible cycle"
        );
        let node = get_current_node(tree, &state).unwrap();
        if let DecisionNode::Question(q) = node {
            for option in q.options.clone() {
                enumerate(
                    tree,
                    answer(tree, &state, &option.value).unwrap(),
                    depth + 1,
                );
            }
        }
    }

    let tree = vote_assist_tree_v1();
    enumerate(tree, create_session(tree).unwrap(), 0);
}
