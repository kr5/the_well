use crate::types::KnowledgeEntry;

/// Curated source-of-truth JSON files live in `/knowledge-base/sources` at
/// the repo root (shared across the whole project — both this crate and the
/// TypeScript prototype's `@voteassist/knowledge` package read the same
/// files during the migration window) so non-engineers can review and PR
/// knowledge changes without touching Rust or TypeScript code.
macro_rules! entry {
    ($file:literal) => {
        include_str!(concat!("../../../../knowledge-base/sources/", $file))
    };
}

const RAW_ENTRIES: &[&str] = &[
    entry!("form-6.json"),
    entry!("form-6a.json"),
    entry!("form-7.json"),
    entry!("form-8.json"),
    entry!("qualifying-dates.json"),
    entry!("ordinary-residence-student.json"),
    entry!("pwd-home-voting.json"),
    entry!("e-epic.json"),
    entry!("helpline-grievance.json"),
    entry!("roll-search-polling-station.json"),
    entry!("service-voter.json"),
];

pub fn load_entries() -> Vec<KnowledgeEntry> {
    RAW_ENTRIES
        .iter()
        .map(|raw| {
            serde_json::from_str(raw).unwrap_or_else(|e| {
                panic!(
                    "a knowledge-base/sources/*.json file failed to deserialize into KnowledgeEntry \
                     (this is a build-time content-quality invariant, not a runtime possibility): {e}"
                )
            })
        })
        .collect()
}
