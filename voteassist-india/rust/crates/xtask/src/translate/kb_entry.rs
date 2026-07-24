//! Translates one English `knowledge-base/sources/*.json` entry into a
//! draft entry for another locale.
//!
//! `knowledge-base/schema/entry.schema.json` has no concept of "this
//! entry is a translated variant of that other entry" — each file is a
//! single, complete, single-language entry (see
//! `crates/jobs::translation_completeness`'s module doc for this exact,
//! previously-disclosed limitation). This command works within that
//! reality rather than inventing schema it doesn't have: it writes a
//! **new, separate** entry file with an id suffixed by locale (e.g.
//! `form-6` -> `form-6-hi`), `language` set to the target locale, and
//! `reviewStatus` forced to `"draft"` regardless of the source entry's
//! status — a human reviewer promotes it (moves it into
//! `knowledge-base/sources/` proper and flips `reviewStatus`) after
//! checking it, exactly like any other content change per
//! docs/06-legal-compliance-review.md Section 7's review process.

use serde_json::Value;

use super::client::ClaudeClient;
use super::locales::find_locale;
use super::translate_text;

pub async fn translate_kb_entry(
    client: &ClaudeClient,
    source_path: &std::path::Path,
    target_locale_code: &str,
) -> Result<Value, String> {
    let locale = find_locale(target_locale_code)
        .ok_or_else(|| format!("\"{target_locale_code}\" is not a tracked target locale"))?;

    let source_text = std::fs::read_to_string(source_path).map_err(|e| format!("failed to read {source_path:?}: {e}"))?;
    let mut entry: Value =
        serde_json::from_str(&source_text).map_err(|e| format!("failed to parse {source_path:?} as JSON: {e}"))?;

    let original_id = entry["id"].as_str().ok_or("entry has no \"id\" field")?.to_string();

    for field in ["title", "summary", "body"] {
        let Some(source_value) = entry[field].as_str().map(str::to_string) else {
            continue;
        };
        let translated = translate_text(client, locale.english_name, &source_value)
            .await
            .map_err(|e| format!("failed to translate \"{field}\": {e}"))?;
        entry[field] = Value::String(translated);
    }

    if let Some(caution) = entry["caution"].as_str().map(str::to_string) {
        let translated =
            translate_text(client, locale.english_name, &caution).await.map_err(|e| format!("failed to translate \"caution\": {e}"))?;
        entry["caution"] = Value::String(translated);
    }

    entry["id"] = Value::String(format!("{original_id}-{}", locale.code));
    entry["language"] = Value::String(locale.code.to_string());
    entry["reviewStatus"] = Value::String("draft".to_string());

    Ok(entry)
}
