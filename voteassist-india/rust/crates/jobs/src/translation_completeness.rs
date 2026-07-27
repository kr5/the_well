//! Translation-completeness reporting, populating `translation_status`
//! (`migrations/0010_link_check_and_translation.sql`) for the admin
//! Translation Management dashboard (PRD v2 admin page 5).
//!
//! `migrations/0013_kb_entry_grouping_and_faq.sql` added
//! `knowledge_entries.translation_group_id` specifically to fix this
//! module's previous limitation (every language variant of the same
//! content shares one group id; `COALESCE(translation_group_id, id)`
//! treats an ungrouped row as its own group of one, so every existing
//! entry — none of which sets this column yet — still counts correctly).
//! `total_items` is now "how many distinct English-rooted groups exist,"
//! and `translated_items`/`reviewed_items` count how many of those groups
//! have a matching/verified row in the target locale — a real 1:1
//! per-topic correlation, not a same-language row-count approximation.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LocaleCompleteness {
    pub locale: String,
    pub total_items: i64,
    pub reviewed_items: i64,
}

/// Computes per-locale KB completeness against the English-rooted set of
/// translation groups and writes one `translation_status` row per locale
/// for this computation run.
pub async fn compute_kb_translation_status(pool: &PgPool, locales: &[&str]) -> Result<(), sqlx::Error> {
    for &locale in locales {
        let row: (i64, i64, i64) = sqlx::query_as(
            r#"
            WITH groups AS (
                SELECT DISTINCT COALESCE(translation_group_id, id) AS group_id
                FROM knowledge_entries
                WHERE language = 'en'
            )
            SELECT
                COUNT(*) AS total,
                COUNT(*) FILTER (WHERE EXISTS (
                    SELECT 1 FROM knowledge_entries variant
                    WHERE COALESCE(variant.translation_group_id, variant.id) = groups.group_id
                      AND variant.language = $1
                )) AS translated,
                COUNT(*) FILTER (WHERE EXISTS (
                    SELECT 1 FROM knowledge_entries variant
                    WHERE COALESCE(variant.translation_group_id, variant.id) = groups.group_id
                      AND variant.language = $1
                      AND variant.review_status = 'verified'
                )) AS reviewed
            FROM groups
            "#,
        )
        .bind(locale)
        .fetch_one(pool)
        .await?;

        let (total, translated, reviewed) = row;

        sqlx::query(
            r#"
            INSERT INTO translation_status (locale, content_type, total_items, translated_items, reviewed_items)
            VALUES ($1, 'kb_entry', $2, $3, $4)
            "#,
        )
        .bind(locale)
        .bind(total)
        .bind(translated)
        .bind(reviewed)
        .execute(pool)
        .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_completeness_struct_round_trips_via_serde() {
        let c = LocaleCompleteness { locale: "hi".into(), total_items: 11, reviewed_items: 0 };
        let json = serde_json::to_string(&c).unwrap();
        let back: LocaleCompleteness = serde_json::from_str(&json).unwrap();
        assert_eq!(back.locale, "hi");
        assert_eq!(back.total_items, 11);
    }
}
