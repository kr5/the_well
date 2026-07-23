//! Validates every real knowledge-base source file against the actual
//! `knowledge-base/schema/entry.schema.json` schema-of-record — not just
//! against our Rust struct's serde mapping (which could silently accept
//! something the schema forbids, or vice versa). This is the check that
//! caught `service-voter.json`'s `relatedForms: ["form-2"]` not being in the
//! schema's enum before this crate existed; the schema was corrected instead
//! of the data, since Form 2 (service-voter registration) is legitimate.

use jsonschema::JSONSchema;
use serde_json::Value;

const SCHEMA_JSON: &str = include_str!("../../../../knowledge-base/schema/entry.schema.json");

const RAW_ENTRIES: &[(&str, &str)] = &[
    (
        "form-6.json",
        include_str!("../../../../knowledge-base/sources/form-6.json"),
    ),
    (
        "form-6a.json",
        include_str!("../../../../knowledge-base/sources/form-6a.json"),
    ),
    (
        "form-7.json",
        include_str!("../../../../knowledge-base/sources/form-7.json"),
    ),
    (
        "form-8.json",
        include_str!("../../../../knowledge-base/sources/form-8.json"),
    ),
    (
        "qualifying-dates.json",
        include_str!("../../../../knowledge-base/sources/qualifying-dates.json"),
    ),
    (
        "ordinary-residence-student.json",
        include_str!("../../../../knowledge-base/sources/ordinary-residence-student.json"),
    ),
    (
        "pwd-home-voting.json",
        include_str!("../../../../knowledge-base/sources/pwd-home-voting.json"),
    ),
    (
        "e-epic.json",
        include_str!("../../../../knowledge-base/sources/e-epic.json"),
    ),
    (
        "helpline-grievance.json",
        include_str!("../../../../knowledge-base/sources/helpline-grievance.json"),
    ),
    (
        "roll-search-polling-station.json",
        include_str!("../../../../knowledge-base/sources/roll-search-polling-station.json"),
    ),
    (
        "service-voter.json",
        include_str!("../../../../knowledge-base/sources/service-voter.json"),
    ),
];

#[test]
fn every_source_file_conforms_to_the_schema_of_record() {
    let schema: Value =
        serde_json::from_str(SCHEMA_JSON).expect("entry.schema.json must itself be valid JSON");
    let validator =
        JSONSchema::compile(&schema).expect("entry.schema.json must itself be a valid JSON Schema");

    let mut failures = Vec::new();
    for (filename, raw) in RAW_ENTRIES {
        let instance: Value = serde_json::from_str(raw)
            .unwrap_or_else(|e| panic!("{filename} is not valid JSON: {e}"));
        let messages: Vec<String> = match validator.validate(&instance) {
            Ok(()) => Vec::new(),
            Err(errors) => errors.map(|e| e.to_string()).collect(),
        };
        if !messages.is_empty() {
            failures.push(format!("{filename}: {}", messages.join("; ")));
        }
    }

    assert!(
        failures.is_empty(),
        "schema conformance failures:\n{}",
        failures.join("\n")
    );
}

/// The Rust struct in `crates::types::KnowledgeEntry` must also successfully
/// deserialize every real file — a schema-conformance pass alone wouldn't
/// catch a Rust-side field-name or enum-variant typo, since serde and
/// jsonschema validate independently.
#[test]
fn every_source_file_round_trips_through_the_rust_struct() {
    for (filename, raw) in RAW_ENTRIES {
        let entry: kb_content::KnowledgeEntry = serde_json::from_str(raw).unwrap_or_else(|e| {
            panic!("{filename} failed to deserialize into KnowledgeEntry: {e}")
        });
        let reserialized = serde_json::to_string(&entry).unwrap();
        let reparsed: kb_content::KnowledgeEntry = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(
            entry.id, reparsed.id,
            "{filename} lost data across a serialize/deserialize round trip"
        );
    }
}
