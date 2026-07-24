//! Election-jurisdiction model: which body administers which election type
//! in India, and what VoteAssist India commits to guiding a user through
//! for each. See docs/PRD-V3-COMPREHENSIVE-EXPANSION.md Section V3 for the
//! full rationale — this crate is the fix for PRD v2's implicit,
//! incomplete assumption that the Election Commission of India is the
//! only relevant authority.
//!
//! This crate is pure logic and reference data — zero I/O, no database
//! dependency — mirroring `core-domain`'s architecture. The reference
//! table below and `migrations/0004_forms_and_geo.sql`'s
//! `election_type_jurisdiction` seed data describe the same constitutional
//! facts and must be kept in sync by inspection (a mismatch there would be
//! a data-integrity bug, not a runtime one, since this crate never reads
//! from the database — it's the source of truth `crates/api` falls back
//! to before any database-backed reference-data lookup exists).

pub mod types;

pub use types::{AdministeringBody, ElectionType, JurisdictionRecord, VoteAssistScope};

use serde::{Deserialize, Serialize};
use ElectionType::*;

/// The full jurisdiction reference table, matching
/// `migrations/0004_forms_and_geo.sql`'s seed data row for row.
pub const JURISDICTION_TABLE: [JurisdictionRecord; 13] = [
    JurisdictionRecord {
        election_type: LokSabha,
        administering_body: AdministeringBody::Eci,
        is_direct_citizen_vote: true,
        scope: VoteAssistScope::FullGuidance,
    },
    JurisdictionRecord {
        election_type: RajyaSabha,
        administering_body: AdministeringBody::Eci,
        is_direct_citizen_vote: false,
        scope: VoteAssistScope::InformationalOnly,
    },
    JurisdictionRecord {
        election_type: LegislativeAssembly,
        administering_body: AdministeringBody::Eci,
        is_direct_citizen_vote: true,
        scope: VoteAssistScope::FullGuidance,
    },
    JurisdictionRecord {
        election_type: LegislativeCouncil,
        administering_body: AdministeringBody::Eci,
        is_direct_citizen_vote: false,
        scope: VoteAssistScope::InformationalOnly,
    },
    JurisdictionRecord {
        election_type: President,
        administering_body: AdministeringBody::Eci,
        is_direct_citizen_vote: false,
        scope: VoteAssistScope::InformationalOnly,
    },
    JurisdictionRecord {
        election_type: VicePresident,
        administering_body: AdministeringBody::Eci,
        is_direct_citizen_vote: false,
        scope: VoteAssistScope::InformationalOnly,
    },
    JurisdictionRecord {
        election_type: ByElection,
        administering_body: AdministeringBody::Eci,
        is_direct_citizen_vote: true,
        scope: VoteAssistScope::FullGuidance,
    },
    JurisdictionRecord {
        election_type: PanchayatGram,
        administering_body: AdministeringBody::StateElectionCommission,
        is_direct_citizen_vote: true,
        scope: VoteAssistScope::JurisdictionRoutingOnly,
    },
    JurisdictionRecord {
        election_type: PanchayatBlock,
        administering_body: AdministeringBody::StateElectionCommission,
        is_direct_citizen_vote: true,
        scope: VoteAssistScope::JurisdictionRoutingOnly,
    },
    JurisdictionRecord {
        election_type: PanchayatZilla,
        administering_body: AdministeringBody::StateElectionCommission,
        is_direct_citizen_vote: true,
        scope: VoteAssistScope::JurisdictionRoutingOnly,
    },
    JurisdictionRecord {
        election_type: MunicipalCorporation,
        administering_body: AdministeringBody::StateElectionCommission,
        is_direct_citizen_vote: true,
        scope: VoteAssistScope::JurisdictionRoutingOnly,
    },
    JurisdictionRecord {
        election_type: MunicipalCouncil,
        administering_body: AdministeringBody::StateElectionCommission,
        is_direct_citizen_vote: true,
        scope: VoteAssistScope::JurisdictionRoutingOnly,
    },
    JurisdictionRecord {
        election_type: NagarPanchayat,
        administering_body: AdministeringBody::StateElectionCommission,
        is_direct_citizen_vote: true,
        scope: VoteAssistScope::JurisdictionRoutingOnly,
    },
];

/// Look up the jurisdiction record for a given election type. Infallible —
/// `JURISDICTION_TABLE` is exhaustive over `ElectionType::ALL` by
/// construction, verified by `tests/table_completeness.rs`.
pub fn jurisdiction_for(election_type: ElectionType) -> JurisdictionRecord {
    JURISDICTION_TABLE
        .iter()
        .find(|r| r.election_type == election_type)
        .copied()
        .expect("JURISDICTION_TABLE is exhaustive over ElectionType — see table_completeness test")
}

/// The routing decision `core-domain`'s decision tree (or a bot/search
/// layer surfacing a local-body question) should act on for a given
/// election type. This is the single function that must be called before
/// ever producing a deep link for an election-related question — it is
/// the guard against the mistake this crate exists to prevent: sending a
/// citizen asking about a Panchayat election to voters.eci.gov.in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingDecision {
    /// Proceed with the standard ECI-oriented decision tree
    /// (`core_domain::vote_assist_tree_v1`/`v2`).
    UseEciDecisionTree,
    /// This is a real citizen concern but not one VoteAssist attempts
    /// registration guidance for (e.g. "can I vote for the Vice
    /// President?" — no, not directly). Explain why, do not route to a
    /// form.
    ExplainNoDirectAction,
    /// Route to the jurisdiction-routing terminal: explain that a State
    /// Election Commission, not the ECI, administers this, and deep-link
    /// to that state's SEC portal once known (PRD v3 Section V3.4).
    RouteToStateElectionCommission,
}

pub fn routing_decision_for(election_type: ElectionType) -> RoutingDecision {
    match jurisdiction_for(election_type).scope {
        VoteAssistScope::FullGuidance => RoutingDecision::UseEciDecisionTree,
        VoteAssistScope::InformationalOnly => RoutingDecision::ExplainNoDirectAction,
        VoteAssistScope::JurisdictionRoutingOnly => RoutingDecision::RouteToStateElectionCommission,
    }
}

/// Representation of a by-election (bypoll) to an ECI-administered seat.
/// Section 149 of the Representation of the People Act, 1951 requires a
/// bypoll to be held within six months of a seat falling vacant (with
/// limited exceptions) — see docs/PRD-V3-COMPREHENSIVE-EXPANSION.md
/// Section V3.4. A bypoll uses IDENTICAL Form 6/6A/7/8/2 registration
/// mechanics to a general election for that seat type: this struct exists
/// only to model the election-calendar/notification timeline, never to
/// introduce a new registration code path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ByElection {
    /// The seat type this bypoll fills — always one of the ECI-
    /// administered, direct-citizen-vote types (`LokSabha` or
    /// `LegislativeAssembly`); constructing one for any other election
    /// type is a logic error the constructor rejects.
    pub seat_type: ElectionType,
    pub constituency_name: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ByElectionError {
    #[error("by-elections only apply to direct-citizen-vote ECI seats (Lok Sabha or Legislative Assembly), not {0:?}")]
    InvalidSeatType(ElectionType),
}

impl ByElection {
    pub fn new(seat_type: ElectionType, constituency_name: impl Into<String>) -> Result<Self, ByElectionError> {
        match seat_type {
            LokSabha | LegislativeAssembly => {
                Ok(Self { seat_type, constituency_name: constituency_name.into() })
            }
            other => Err(ByElectionError::InvalidSeatType(other)),
        }
    }
}
