//! Walks new v2-specific paths through the expanded tree, exercising the
//! engine against real new nodes rather than only the v1 subset.

use core_domain::{answer, create_session, get_current_node, is_session_complete, vote_assist_tree_v2};

#[test]
fn government_transfer_qualifying_for_service_voter_reaches_form_2() {
    let tree = vote_assist_tree_v2();
    let mut state = create_session(tree).unwrap();
    state = answer(tree, &state, "yes").unwrap(); // registered
    state = answer(tree, &state, "moved").unwrap();
    state = answer(tree, &state, "government_job_transfer").unwrap();
    state = answer(tree, &state, "yes_qualifies").unwrap();
    assert!(is_session_complete(tree, &state).unwrap());
    assert_eq!(get_current_node(tree, &state).unwrap().id(), "terminal_service_voter");
}

#[test]
fn private_job_transfer_never_reaches_service_voter() {
    let tree = vote_assist_tree_v2();
    let mut state = create_session(tree).unwrap();
    state = answer(tree, &state, "yes").unwrap();
    state = answer(tree, &state, "moved").unwrap();
    state = answer(tree, &state, "private_job_transfer").unwrap();
    state = answer(tree, &state, "same").unwrap();
    assert_eq!(get_current_node(tree, &state).unwrap().id(), "terminal_form8_shift");
}

#[test]
fn moved_and_name_change_reaches_the_combined_terminal() {
    let tree = vote_assist_tree_v2();
    let mut state = create_session(tree).unwrap();
    state = answer(tree, &state, "yes").unwrap();
    state = answer(tree, &state, "moved_and_name_change").unwrap();
    assert_eq!(get_current_node(tree, &state).unwrap().id(), "terminal_form8_shift_and_correction");
}

#[test]
fn wrongful_deletion_is_distinguished_from_fresh_reregistration() {
    let tree = vote_assist_tree_v2();

    let mut fresh = create_session(tree).unwrap();
    fresh = answer(tree, &fresh, "yes").unwrap();
    fresh = answer(tree, &fresh, "object_delete").unwrap();
    fresh = answer(tree, &fresh, "my_entry_deleted").unwrap();
    assert_eq!(get_current_node(tree, &fresh).unwrap().id(), "terminal_form6_new");

    let mut contested = create_session(tree).unwrap();
    contested = answer(tree, &contested, "yes").unwrap();
    contested = answer(tree, &contested, "object_delete").unwrap();
    contested = answer(tree, &contested, "deletion_was_wrongful").unwrap();
    assert_eq!(get_current_node(tree, &contested).unwrap().id(), "terminal_deleted_entry_contest");
}

#[test]
fn tribal_remote_or_slum_reaches_the_honest_local_verification_terminal() {
    let tree = vote_assist_tree_v2();
    let mut state = create_session(tree).unwrap();
    state = answer(tree, &state, "no").unwrap();
    state = answer(tree, &state, "adult").unwrap();
    state = answer(tree, &state, "resident").unwrap();
    state = answer(tree, &state, "tribal_remote_or_slum").unwrap();
    assert_eq!(get_current_node(tree, &state).unwrap().id(), "terminal_address_proof_local_verification");
}

#[test]
fn no_standard_documents_reaches_the_alternatives_terminal() {
    let tree = vote_assist_tree_v2();
    let mut state = create_session(tree).unwrap();
    state = answer(tree, &state, "no").unwrap();
    state = answer(tree, &state, "adult").unwrap();
    state = answer(tree, &state, "resident").unwrap();
    state = answer(tree, &state, "lacking_standard_documents").unwrap();
    state = answer(tree, &state, "has_none_of_these").unwrap();
    assert_eq!(get_current_node(tree, &state).unwrap().id(), "terminal_documents_alternatives");
}

#[test]
fn eighty_five_plus_does_not_require_form_8() {
    let tree = vote_assist_tree_v2();
    let mut state = create_session(tree).unwrap();
    state = answer(tree, &state, "yes").unwrap();
    state = answer(tree, &state, "senior_85").unwrap();
    let node = get_current_node(tree, &state).unwrap().as_terminal().unwrap();
    assert_eq!(node.id, "terminal_senior_postal_ballot");
    assert!(node.recommended_forms.is_empty());
}

#[test]
fn polling_station_changed_without_move_is_not_a_form_8_scenario() {
    let tree = vote_assist_tree_v2();
    let mut state = create_session(tree).unwrap();
    state = answer(tree, &state, "yes").unwrap();
    state = answer(tree, &state, "polling_station_changed").unwrap();
    let node = get_current_node(tree, &state).unwrap().as_terminal().unwrap();
    assert_eq!(node.id, "terminal_polling_station_changed");
    assert!(node.recommended_forms.is_empty());
}
