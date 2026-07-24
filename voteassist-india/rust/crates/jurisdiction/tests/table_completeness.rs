use jurisdiction::{
    jurisdiction_for, routing_decision_for, AdministeringBody, ByElection, ElectionType,
    RoutingDecision, VoteAssistScope, JURISDICTION_TABLE,
};

#[test]
fn jurisdiction_table_covers_every_election_type_exactly_once() {
    assert_eq!(JURISDICTION_TABLE.len(), ElectionType::ALL.len());
    for et in ElectionType::ALL {
        let matches = JURISDICTION_TABLE.iter().filter(|r| r.election_type == et).count();
        assert_eq!(matches, 1, "{et:?} should appear exactly once in JURISDICTION_TABLE");
    }
}

#[test]
fn eci_administered_types_match_expected_set() {
    let eci_types: Vec<ElectionType> = JURISDICTION_TABLE
        .iter()
        .filter(|r| r.administering_body == AdministeringBody::Eci)
        .map(|r| r.election_type)
        .collect();

    for et in [
        ElectionType::LokSabha,
        ElectionType::RajyaSabha,
        ElectionType::LegislativeAssembly,
        ElectionType::LegislativeCouncil,
        ElectionType::President,
        ElectionType::VicePresident,
        ElectionType::ByElection,
    ] {
        assert!(eci_types.contains(&et), "{et:?} should be ECI-administered");
    }
}

#[test]
fn all_local_body_types_are_sec_administered_and_routing_only() {
    for et in ElectionType::ALL {
        if et.is_local_body() {
            let record = jurisdiction_for(et);
            assert_eq!(record.administering_body, AdministeringBody::StateElectionCommission);
            assert_eq!(record.scope, VoteAssistScope::JurisdictionRoutingOnly);
        }
    }
}

#[test]
fn full_guidance_types_are_exactly_the_direct_eci_registration_scenarios() {
    let full_guidance: Vec<ElectionType> = JURISDICTION_TABLE
        .iter()
        .filter(|r| r.scope == VoteAssistScope::FullGuidance)
        .map(|r| r.election_type)
        .collect();
    assert_eq!(full_guidance.len(), 3);
    assert!(full_guidance.contains(&ElectionType::LokSabha));
    assert!(full_guidance.contains(&ElectionType::LegislativeAssembly));
    assert!(full_guidance.contains(&ElectionType::ByElection));
}

#[test]
fn indirect_elections_never_route_to_a_registration_flow() {
    for et in [
        ElectionType::RajyaSabha,
        ElectionType::LegislativeCouncil,
        ElectionType::President,
        ElectionType::VicePresident,
    ] {
        assert!(!jurisdiction_for(et).is_direct_citizen_vote);
        assert_eq!(routing_decision_for(et), RoutingDecision::ExplainNoDirectAction);
    }
}

#[test]
fn panchayat_and_municipal_questions_never_reach_the_eci_tree() {
    for et in ElectionType::ALL {
        if et.is_local_body() {
            assert_eq!(routing_decision_for(et), RoutingDecision::RouteToStateElectionCommission);
        }
    }
}

#[test]
fn by_election_only_constructs_for_direct_eci_seat_types() {
    assert!(ByElection::new(ElectionType::LokSabha, "Wayanad").is_ok());
    assert!(ByElection::new(ElectionType::LegislativeAssembly, "Chinchwad").is_ok());
    assert!(ByElection::new(ElectionType::RajyaSabha, "N/A").is_err());
    assert!(ByElection::new(ElectionType::PanchayatGram, "N/A").is_err());
}
