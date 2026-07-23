//! Rust port of packages/knowledge/test/entries.test.ts.

use kb_content::{
    get_entry, knowledge_entries, list_by_topic, require_entry, search_entries, Topic,
};
use std::collections::HashSet;

#[test]
fn has_at_least_one_entry_loaded() {
    assert!(!knowledge_entries().is_empty());
}

#[test]
fn has_no_duplicate_ids() {
    let ids: Vec<&str> = knowledge_entries().iter().map(|e| e.id.as_str()).collect();
    let unique: HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(unique.len(), ids.len());
}

#[test]
fn every_entry_has_at_least_one_valid_https_source_and_nonempty_summary_body() {
    for entry in knowledge_entries() {
        assert!(!entry.sources.is_empty(), "{} has no sources", entry.id);
        assert!(!entry.summary.is_empty());
        assert!(!entry.body.is_empty());
        for source in &entry.sources {
            let url = url::Url::parse(&source.url).unwrap_or_else(|e| panic!("{}: {e}", entry.id));
            assert!(matches!(url.scheme(), "http" | "https"));
        }
    }
}

#[test]
fn every_entry_declares_a_parseable_last_verified_date() {
    for entry in knowledge_entries() {
        chrono::NaiveDate::parse_from_str(&entry.last_verified_date, "%Y-%m-%d")
            .unwrap_or_else(|e| panic!("{}: {e}", entry.id));
    }
}

#[test]
fn get_entry_resolves_known_id_and_none_for_unknown() {
    assert!(get_entry("form-8").unwrap().title.contains("Form 8"));
    assert!(get_entry("does-not-exist").is_none());
}

#[test]
fn require_entry_errors_for_unknown_id() {
    assert!(require_entry("does-not-exist").is_err());
}

#[test]
fn list_by_topic_filters_correctly() {
    let shifting = list_by_topic(Topic::ShiftingOfResidence);
    assert!(!shifting.is_empty());
    assert!(shifting
        .iter()
        .all(|e| e.topic == Topic::ShiftingOfResidence));
}

#[test]
fn search_entries_finds_form_8_for_shifting_of_residence() {
    let results = search_entries("shifting of residence");
    assert!(results.iter().any(|e| e.id == "form-8"));
}

#[test]
fn search_entries_returns_nothing_for_empty_query() {
    assert!(search_entries("   ").is_empty());
}
