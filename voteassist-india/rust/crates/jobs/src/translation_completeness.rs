//! Translation-completeness reporting, populating `translation_status`
//! (`migrations/0010_link_check_and_translation.sql`) for the admin
//! Translation Management dashboard (PRD v2 admin page 5).
//!
//! Known schema limitation, disclosed rather than hidden: `knowledge_entries.id`
//! is a primary key (e.g. `"form-8"`), so today's schema cannot hold two
//! rows representing the SAME topic in two different languages under the
//! same id — a Hindi variant of the `form-8` entry would need either a
//! distinct id (e.g. a `-hi` suffix convention) or a schema change adding
//! a `base_entry_id` column grouping language variants together. Neither
//! exists yet. This module computes the coarser, still-useful metric that
//! DOES work against today's schema: per-language row counts and
//! verification status, which is accurate as a rough completeness signal
//! but is not a strict 1:1 "this English entry has/hasn't been translated"
//! correlation. Tightening this (via a `base_entry_id` migration) is a
//! tracked follow-up, not something this module pretends is already solved.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LocaleCompleteness {
    pub locale: String,
    pub total_items: i64,
    pub reviewed_items: i64,
}

/// Computes per-locale KB completeness (see the module-level caveat
/// above) and writes one `translation_status` row per locale for this
/// computation run.
pub async fn compute_kb_translation_status(pool: &PgPool, locales: &[&str]) -> Result<(), sqlx::Error> {
    for &locale in locales {
        let row: (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) AS total,
                COUNT(*) FILTER (WHERE review_status = 'verified') AS reviewed
            FROM knowledge_entries
            WHERE language = $1
            "#,
        )
        .bind(locale)
        .fetch_one(pool)
        .await?;

        let (total, reviewed) = row;

        sqlx::query(
            r#"
            INSERT INTO translation_status (locale, content_type, total_items, translated_items, reviewed_items)
            VALUES ($1, 'kb_entry', $2, $2, $3)
            "#,
        )
        .bind(locale)
        .bind(total)
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
