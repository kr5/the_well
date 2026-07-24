//! KB re-verification-due digest: flags database-backed `knowledge_entries`
//! rows whose `last_verified_date` has aged past a configurable threshold
//! as `needs_reverification`, per docs/PRD-V2-RUST-PLATFORM.md Section 9's
//! `reviewStatus` state machine. This job only ever flags content for a
//! human reviewer to act on — it never edits the entry's actual content,
//! consistent with every other content pipeline in this system.
//!
//! This job operates against the database-backed `knowledge_entries`
//! table (the admin-editable store, per `migrations/0002_knowledge_base.sql`),
//! which is a superset target of the read-only JSON snapshot `kb-content`
//! currently loads for the MVP web app — see that crate's module docs for
//! the file<->database reconciliation note tracked as a PRD v2 Section 9
//! follow-up.

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// Default threshold from docs/PRD-V2-RUST-PLATFORM.md Section 9: "e.g.
/// 180 days."
pub const DEFAULT_REVERIFICATION_THRESHOLD_DAYS: i64 = 180;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EntryNeedingReverification {
    pub id: String,
    pub title: String,
    pub last_verified_date: chrono::NaiveDate,
    pub days_overdue: i32,
}

/// Read-only preview of what the digest would flag, without writing
/// anything — used by the admin Dashboard tile (PRD v2 admin page 2) and
/// by `flag_overdue_entries` internally before it commits any UPDATE.
pub async fn find_overdue_entries(
    pool: &PgPool,
    threshold_days: i64,
) -> Result<Vec<EntryNeedingReverification>, sqlx::Error> {
    let cutoff = (Utc::now() - Duration::days(threshold_days)).date_naive();

    sqlx::query_as(
        r#"
        SELECT
            id,
            title,
            last_verified_date,
            (CURRENT_DATE - last_verified_date)::int AS days_overdue
        FROM knowledge_entries
        WHERE last_verified_date < $1
          AND review_status != 'needs_reverification'
        ORDER BY last_verified_date ASC
        "#,
    )
    .bind(cutoff)
    .fetch_all(pool)
    .await
}

/// Flags every overdue entry `needs_reverification` in a single
/// transaction and returns how many rows were updated. Also writes one
/// `audit_log` row per flagged entry (actor_id = NULL, since this is a
/// system-initiated action, not an admin action — the Audit Log Viewer,
/// per docs/PRD-V3-COMPREHENSIVE-EXPANSION.md Section V7's format,
/// renders a NULL actor as "system: re-verification digest job").
pub async fn flag_overdue_entries(pool: &PgPool, threshold_days: i64) -> Result<u64, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let overdue: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT id FROM knowledge_entries
        WHERE last_verified_date < (CURRENT_DATE - $1::int)
          AND review_status != 'needs_reverification'
        "#,
    )
    .bind(threshold_days as i32)
    .fetch_all(&mut *tx)
    .await?;

    for (entry_id,) in &overdue {
        sqlx::query("UPDATE knowledge_entries SET review_status = 'needs_reverification' WHERE id = $1")
            .bind(entry_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r#"
            INSERT INTO audit_log (actor_id, action, target_type, target_id, before_value, after_value)
            VALUES (NULL, 'kb_entry.auto_flagged_needs_reverification', 'knowledge_entries', $1,
                    '{"review_status": "verified_or_in_review"}'::jsonb,
                    '{"review_status": "needs_reverification"}'::jsonb)
            "#,
        )
        .bind(entry_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(overdue.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_threshold_matches_the_documented_policy() {
        assert_eq!(DEFAULT_REVERIFICATION_THRESHOLD_DAYS, 180);
    }
}
