//! Property-based tests superseding the TypeScript prototype's hand-enumerated
//! exhaustive-path test: instead of one fixed test walking every combination
//! by hand, this generates many random valid answer sequences and asserts the
//! same invariants must hold for all of them. See
//! docs/PRD-V2-RUST-PLATFORM.md Section 6.14 / Section 15.

use core_domain::{
    answer, create_session, get_current_node, is_session_complete, vote_assist_tree_v1,
    DecisionNode,
};
use proptest::prelude::*;

/// Follows a pseudo-random sequence of choice indices through the tree,
/// picking a valid option at each question node by index modulo the option
/// count, and returns the number of steps taken to reach a terminal.
fn walk_with_choices(choices: &[u8]) -> u32 {
    let tree = vote_assist_tree_v1();
    let mut state = create_session(tree).unwrap();
    let mut steps = 0u32;
    let mut choice_iter = choices.iter().cycle();

    loop {
        let node = get_current_node(tree, &state).unwrap();
        let DecisionNode::Question(q) = node else {
            return steps;
        };
        assert!(
            steps < 20,
            "session did not terminate within 20 steps — possible cycle"
        );
        let choice = *choice_iter.next().unwrap() as usize % q.options.len();
        let value = q.options[choice].value.clone();
        state = answer(tree, &state, &value).unwrap();
        steps += 1;
    }
}

proptest! {
    /// Every path, regardless of which valid options are picked at each
    /// question, reaches a terminal node within a bounded number of steps.
    #[test]
    fn every_random_valid_path_terminates(choices in prop::collection::vec(any::<u8>(), 1..30)) {
        let steps = walk_with_choices(&choices);
        prop_assert!(steps < 20);
    }

    /// No matter which valid path is walked, the reached terminal always
    /// carries at least one citation and one deep link — the same hard
    /// invariant `validate_tree` checks statically, re-verified dynamically
    /// here for every generated path.
    #[test]
    fn every_reached_terminal_is_fully_cited(choices in prop::collection::vec(any::<u8>(), 1..30)) {
        let tree = vote_assist_tree_v1();
        let mut state = create_session(tree).unwrap();
        let mut choice_iter = choices.iter().cycle();
        let mut steps = 0;

        loop {
            let node = get_current_node(tree, &state).unwrap();
            match node {
                DecisionNode::Terminal(t) => {
                    prop_assert!(!t.citations.is_empty());
                    prop_assert!(!t.deep_links.is_empty());
                    break;
                }
                DecisionNode::Question(q) => {
                    prop_assert!(steps < 20);
                    let choice = *choice_iter.next().unwrap() as usize % q.options.len();
                    let value = q.options[choice].value.clone();
                    state = answer(tree, &state, &value).unwrap();
                    steps += 1;
                }
            }
        }
        prop_assert!(is_session_complete(tree, &state).unwrap());
    }
}
