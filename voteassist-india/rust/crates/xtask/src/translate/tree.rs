//! Translates every `core_domain::LocalizedText` value found in a
//! decision-tree JSON file (`tree_v1.json`/`tree_v2.json`) into a draft
//! copy of that same tree with the target locale added.
//!
//! `LocalizedText` serializes as a flat object with an `"en"` key plus
//! zero or more other locale keys (`core-domain/src/types.rs`'s
//! `#[serde(flatten)] other: BTreeMap<String, String>`) — this walk
//! detects that shape structurally (an object with a string `"en"` key)
//! rather than needing to know the tree's node schema, so it works
//! unchanged across `tree_v1.json`, `tree_v2.json`, and any future tree.
//!
//! Output is a **complete copy of the input tree**, not a patch — written
//! to a separate `-draft` path, never overwriting the real tree file.
//! Reviewing means diffing the draft against the source tree (every
//! change is exactly one new key per `LocalizedText` object) and manually
//! merging the reviewed strings into the real file — this pipeline never
//! writes directly to a file `core-domain`'s `include_str!` loads.

use serde_json::Value;

use super::client::ClaudeClient;
use super::locales::find_locale;
use super::translate_text;

pub async fn translate_tree(
    client: &ClaudeClient,
    source_path: &std::path::Path,
    target_locale_code: &str,
) -> Result<Value, String> {
    let locale = find_locale(target_locale_code)
        .ok_or_else(|| format!("\"{target_locale_code}\" is not a tracked target locale"))?;

    let source_text = std::fs::read_to_string(source_path).map_err(|e| format!("failed to read {source_path:?}: {e}"))?;
    let mut tree: Value =
        serde_json::from_str(&source_text).map_err(|e| format!("failed to parse {source_path:?} as JSON: {e}"))?;

    let mut skipped_count = 0usize;
    let mut paths = Vec::new();
    collect_localized_text_paths(&tree, locale.code, &mut Vec::new(), &mut paths, &mut skipped_count);

    let mut translated_count = 0usize;
    for path in &paths {
        let Some(node) = get_mut_by_path(&mut tree, path) else { continue };
        let Some(english) = node.get("en").and_then(Value::as_str).map(str::to_string) else { continue };

        let translated = translate_text(client, locale.english_name, &english)
            .await
            .map_err(|e| format!("failed to translate \"{english}\": {e}"))?;

        if let Value::Object(map) = node {
            map.insert(locale.code.to_string(), Value::String(translated));
        }
        translated_count += 1;
    }

    eprintln!(
        "translated {translated_count} LocalizedText field(s) into \"{}\"; {skipped_count} already had that locale and were left as-is",
        locale.code
    );

    Ok(tree)
}

#[derive(Clone)]
enum PathSegment {
    Key(String),
    Index(usize),
}

/// Depth-first walk collecting the path to every untranslated
/// `LocalizedText`-shaped node. Synchronous and read-only by design: it
/// only ever needs `&Value`, so it runs to completion (building the full
/// list of paths) before `translate_tree` above does any `&mut`
/// borrow/await — sidestepping the usual "recursive async fn needs
/// boxing" problem entirely, since none of the actual awaiting happens
/// inside recursion.
fn collect_localized_text_paths(
    value: &Value,
    target_locale_code: &str,
    path: &mut Vec<PathSegment>,
    out: &mut Vec<Vec<PathSegment>>,
    skipped_count: &mut usize,
) {
    let is_localized_text = matches!(value, Value::Object(map) if map.get("en").is_some_and(Value::is_string));

    if is_localized_text {
        let already_has_target = matches!(value, Value::Object(map) if map.contains_key(target_locale_code));
        if already_has_target {
            *skipped_count += 1;
        } else {
            out.push(path.clone());
        }
        // A LocalizedText object's own values are plain strings, not
        // nested translatable structure — no need to recurse further into
        // this particular object either way.
        return;
    }

    match value {
        Value::Object(map) => {
            for (key, child) in map.iter() {
                path.push(PathSegment::Key(key.clone()));
                collect_localized_text_paths(child, target_locale_code, path, out, skipped_count);
                path.pop();
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                path.push(PathSegment::Index(index));
                collect_localized_text_paths(item, target_locale_code, path, out, skipped_count);
                path.pop();
            }
        }
        _ => {}
    }
}

/// Re-resolves `path` against `root` fresh each call — a safe alternative
/// to caching raw pointers/`&mut` references from the read-only collection
/// pass above, at the cost of re-walking from the root once per
/// translated node (negligible: decision trees here are tens of KB, and
/// this runs once per pipeline invocation, not in a hot loop).
fn get_mut_by_path<'a>(root: &'a mut Value, path: &[PathSegment]) -> Option<&'a mut Value> {
    let mut current = root;
    for segment in path {
        current = match (segment, current) {
            (PathSegment::Key(key), Value::Object(map)) => map.get_mut(key)?,
            (PathSegment::Index(index), Value::Array(items)) => items.get_mut(*index)?,
            _ => return None,
        };
    }
    Some(current)
}
