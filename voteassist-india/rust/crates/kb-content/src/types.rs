use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Mirrors `knowledge-base/schema/entry.schema.json` field-for-field. This is
/// the schema-of-record; if the two ever drift, `tests/schema_conformance.rs`
/// fails the build — see docs/PRD-V2-RUST-PLATFORM.md Section 9.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEntry {
    pub id: String,
    pub topic: Topic,
    pub title: String,
    pub summary: String,
    #[serde(default)]
    pub body: String,
    pub applicable_states: Vec<String>,
    pub source_type: SourceType,
    pub sources: Vec<KnowledgeSource>,
    pub last_verified_date: String,
    pub version: u32,
    #[serde(default)]
    pub related_forms: Vec<RelatedForm>,
    #[serde(default)]
    pub related_entities: Vec<String>,
    pub language: String,
    pub review_status: ReviewStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeSource {
    pub title: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum Topic {
    Registration,
    Correction,
    ShiftingOfResidence,
    DeletionObjection,
    Epic,
    OrdinaryResidence,
    NriVoter,
    ServiceVoter,
    PwdVoter,
    QualifyingDates,
    Grievance,
    PollingStation,
    RollSearch,
    Glossary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    EciOfficial,
    StateCeo,
    GazetteLaw,
    Sveep,
    PibRelease,
    CommunityPendingVerification,
}

/// Includes `form-2` (service-voter registration) alongside the four
/// consolidated electoral-roll forms and Form 12D (postal ballot/home
/// voting) — `knowledge-base/schema/entry.schema.json`'s enum was missing
/// `form-2` until this crate's schema-conformance test caught the mismatch
/// against the real `service-voter.json` entry; both were corrected together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum RelatedForm {
    #[serde(rename = "form-2")]
    Form2,
    #[serde(rename = "form-6")]
    Form6,
    #[serde(rename = "form-6a")]
    Form6a,
    #[serde(rename = "form-7")]
    Form7,
    #[serde(rename = "form-8")]
    Form8,
    #[serde(rename = "form-12d")]
    Form12d,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Draft,
    InReview,
    Verified,
    NeedsReverification,
}
