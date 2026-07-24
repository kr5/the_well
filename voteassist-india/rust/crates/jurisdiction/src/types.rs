use serde::{Deserialize, Serialize};

/// Mirrors the Postgres `election_type` enum in
/// `migrations/0001_extensions_and_enums.sql` — every variant here must
/// have a corresponding value in `migrations/0004_forms_and_geo.sql`'s
/// `election_type_jurisdiction` seed data, and vice versa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElectionType {
    LokSabha,
    RajyaSabha,
    LegislativeAssembly,
    LegislativeCouncil,
    President,
    VicePresident,
    ByElection,
    PanchayatGram,
    PanchayatBlock,
    PanchayatZilla,
    MunicipalCorporation,
    MunicipalCouncil,
    NagarPanchayat,
}

impl ElectionType {
    pub const ALL: [ElectionType; 13] = [
        ElectionType::LokSabha,
        ElectionType::RajyaSabha,
        ElectionType::LegislativeAssembly,
        ElectionType::LegislativeCouncil,
        ElectionType::President,
        ElectionType::VicePresident,
        ElectionType::ByElection,
        ElectionType::PanchayatGram,
        ElectionType::PanchayatBlock,
        ElectionType::PanchayatZilla,
        ElectionType::MunicipalCorporation,
        ElectionType::MunicipalCouncil,
        ElectionType::NagarPanchayat,
    ];

    /// Whether this election type is one of the three Panchayati Raj
    /// tiers (Gram/Block/Zilla) — grouped because they share identical
    /// jurisdiction and routing behaviour (all State Election Commission,
    /// all `jurisdiction_routing_only`).
    pub fn is_panchayat_tier(self) -> bool {
        matches!(
            self,
            ElectionType::PanchayatGram | ElectionType::PanchayatBlock | ElectionType::PanchayatZilla
        )
    }

    /// Whether this election type is one of the three Urban Local Body
    /// tiers (Municipal Corporation/Council, Nagar Panchayat).
    pub fn is_municipal_tier(self) -> bool {
        matches!(
            self,
            ElectionType::MunicipalCorporation | ElectionType::MunicipalCouncil | ElectionType::NagarPanchayat
        )
    }

    /// Whether this election type falls under a State Election Commission
    /// rather than the ECI (Article 243K/243ZA) — see
    /// docs/PRD-V3-COMPREHENSIVE-EXPANSION.md Section V3.2.
    pub fn is_local_body(self) -> bool {
        self.is_panchayat_tier() || self.is_municipal_tier()
    }
}

/// Mirrors the Postgres `administering_body` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdministeringBody {
    Eci,
    StateElectionCommission,
}

/// Mirrors the Postgres `voteassist_scope` enum — how much VoteAssist
/// commits to guiding a user through this election type, as opposed to
/// only explaining who's in charge of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoteAssistScope {
    /// Full registration/correction/EPIC guidance — this is what the
    /// core-domain decision tree covers.
    FullGuidance,
    /// VoteAssist can correctly answer "does this apply to me / can I
    /// vote in this directly," but does not attempt registration guidance
    /// (e.g. Rajya Sabha, an indirect election by sitting MLAs).
    InformationalOnly,
    /// VoteAssist's only role is pointing the user at the right official
    /// body (a State Election Commission) — it does not attempt to
    /// replicate that body's own procedures nationally, since there is no
    /// unified national portal or rule set for local-body elections.
    JurisdictionRoutingOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct JurisdictionRecord {
    pub election_type: ElectionType,
    pub administering_body: AdministeringBody,
    pub is_direct_citizen_vote: bool,
    pub scope: VoteAssistScope,
}
