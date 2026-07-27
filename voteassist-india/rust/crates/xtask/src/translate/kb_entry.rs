//! Translates one English `knowledge-base/sources/*.json` entry into a
//! draft entry for another locale.
//!
//! Each locale is still its own, separate, complete entry file — this
//! writes a **new, separate** entry with an id suffixed by locale (e.g.
//! `form-6` -> `form-6-hi`), `language` set to the target locale,
//! `translationGroupId` set to the source entry's own id (linking it back
//! for `crates/jobs::translation_completeness`'s per-topic coverage
//! computation — see `migrations/0013_kb_entry_grouping_and_faq.sql`),
//! and `reviewStatus` forced to `"draft"` regardless of the source
//! entry's status — a human reviewer promotes it (moves it into
//! `knowledge-base/sources/` proper and flips `reviewStatus`) after
//! checking it, exactly like any other content change per
//! docs/06-legal-compliance-review.md Section 7's review process.

use serde_json::Value;

use super::client::TranslationClient;
use super::locales::find_locale;
use super::translate_text;

pub async fn translate_kb_entry(
    client: &TranslationClient,
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

    entry["translationGroupId"] = Value::String(original_id.clone());
    entry["id"] = Value::String(format!("{original_id}-{}", locale.code));
    entry["language"] = Value::String(locale.code.to_string());
    entry["reviewStatus"] = Value::String("draft".to_string());

    Ok(entry)
}
