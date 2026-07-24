//! Property-based invariants for the expanded v2 tree, mirroring
//! proptest_invariants.rs's coverage of v1.

use core_domain::{answer, create_session, get_current_node, is_session_complete, vote_assist_tree_v2, DecisionNode};
use proptest::prelude::*;

fn walk_with_choices(choices: &[u8]) -> u32 {
    let tree = vote_assist_tree_v2();
    let mut state = create_session(tree).unwrap();
    let mut steps = 0u32;
    let mut choice_iter = choices.iter().cycle();

    loop {
        let node = get_current_node(tree, &state).unwrap();
        let DecisionNode::Question(q) = node else {
            return steps;
        };
        assert!(steps < 20, "session did not terminate within 20 steps — possible cycle");
        let choice = *choice_iter.next().unwrap() as usize % q.options.len();
        let value = q.options[choice].value.clone();
        state = answer(tree, &state, &value).unwrap();
        steps += 1;
    }
}

proptest! {
    #[test]
    fn every_random_valid_path_terminates(choices in prop::collection::vec(any::<u8>(), 1..30)) {
        let steps = walk_with_choices(&choices);
        prop_assert!(steps < 20);
    }

    #[test]
    fn every_reached_terminal_is_fully_cited(choices in prop::collection::vec(any::<u8>(), 1..30)) {
        let tree = vote_assist_tree_v2();
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
