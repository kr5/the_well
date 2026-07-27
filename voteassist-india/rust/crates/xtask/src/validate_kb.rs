//! Validates every `knowledge-base/sources/*.json` file against
//! `knowledge-base/schema/entry.schema.json` at runtime — the same check
//! `crates/kb-content`'s own `tests/schema_conformance.rs` performs, but
//! that test only covers a fixed list of files `include_str!`'d at
//! compile time (kb-content is deliberately zero-filesystem-I/O — see
//! that crate's own module doc). This command discovers files
//! dynamically via `std::fs::read_dir`, so it also catches a brand-new
//! file (e.g. a promoted translation draft, or a new entry
//! `kb-content`'s source list hasn't been updated to include yet)
//! without needing a Rust code change first.

use std::path::Path;

use jsonschema::JSONSchema;
use serde_json::Value;

pub fn validate_kb(sources_dir: &Path, schema_path: &Path) -> Result<(), String> {
    let schema_text =
        std::fs::read_to_string(schema_path).map_err(|e| format!("failed to read {schema_path:?}: {e}"))?;
    let schema: Value =
        serde_json::from_str(&schema_text).map_err(|e| format!("{schema_path:?} is not valid JSON: {e}"))?;
    let validator = JSONSchema::compile(&schema)
        .map_err(|e| format!("{schema_path:?} is not a valid JSON Schema: {e}"))?;

    let mut paths: Vec<_> = std::fs::read_dir(sources_dir)
        .map_err(|e| format!("failed to read directory {sources_dir:?}: {e}"))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();

    if paths.is_empty() {
        return Err(format!("no .json files found in {sources_dir:?}"));
    }

    let mut failures = Vec::new();
    for path in &paths {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) => {
                failures.push(format!("{}: failed to read: {e}", path.display()));
                continue;
            }
        };
        let instance: Value = match serde_json::from_str(&text) {
            Ok(instance) => instance,
            Err(e) => {
                failures.push(format!("{}: not valid JSON: {e}", path.display()));
                continue;
            }
        };

        let messages: Vec<String> = match validator.validate(&instance) {
            Ok(()) => Vec::new(),
            Err(errors) => errors.map(|e| e.to_string()).collect(),
        };
        if !messages.is_empty() {
            failures.push(format!("{}: {}", path.display(), messages.join("; ")));
        }
    }

    if failures.is_empty() {
        println!("{} entries validated OK against {}", paths.len(), schema_path.display());
        Ok(())
    } else {
        Err(format!(
            "{} of {} entries failed schema validation:\n{}",
            failures.len(),
            paths.len(),
            failures.join("\n")
        ))
    }
}
