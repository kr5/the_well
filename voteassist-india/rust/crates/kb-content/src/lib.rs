//! Typed loader over VoteAssist India's curated, cited knowledge base.
//!
//! Direct Rust port of `packages/knowledge/src/index.ts`. See
//! docs/PRD-V2-RUST-PLATFORM.md Section 9 for the full content-authoring
//! model this implements.

pub mod entries;
pub mod search;
pub mod types;

use std::collections::HashMap;
use std::sync::OnceLock;

pub use search::{search_entries, search_entries_ranked, ScoredEntry};
pub use types::{KnowledgeEntry, KnowledgeSource, RelatedForm, ReviewStatus, SourceType, Topic};

fn all_entries() -> &'static Vec<KnowledgeEntry> {
    static ENTRIES: OnceLock<Vec<KnowledgeEntry>> = OnceLock::new();
    ENTRIES.get_or_init(entries::load_entries)
}

fn by_id() -> &'static HashMap<String, usize> {
    static INDEX: OnceLock<HashMap<String, usize>> = OnceLock::new();
    INDEX.get_or_init(|| {
        all_entries()
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id.clone(), i))
            .collect()
    })
}

pub fn knowledge_entries() -> &'static [KnowledgeEntry] {
    all_entries()
}

pub fn get_entry(id: &str) -> Option<&'static KnowledgeEntry> {
    by_id().get(id).map(|&i| &all_entries()[i])
}

#[derive(Debug, thiserror::Error)]
#[error("unknown knowledge base entry id \"{0}\"")]
pub struct UnknownEntryError(pub String);

pub fn require_entry(id: &str) -> Result<&'static KnowledgeEntry, UnknownEntryError> {
    get_entry(id).ok_or_else(|| UnknownEntryError(id.to_string()))
}

pub fn list_by_topic(topic: Topic) -> Vec<&'static KnowledgeEntry> {
    all_entries().iter().filter(|e| e.topic == topic).collect()
}

