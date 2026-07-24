use chrono::{Duration, Utc};
use sqlx::PgPool;

/// The hard retention window for raw `analytics_events`, per
/// docs/PRD-V2-RUST-PLATFORM.md Section 12: "raw analytics_events purged
/// after the 30-day retention window once rolled up ... this is a hard
/// requirement, not an aspiration." Raw rows are only ever deleted here —
/// never mutated, never read after purge.
pub const RAW_EVENT_RETENTION_DAYS: i64 = 30;

/// Deletes every `analytics_events` row older than
/// `RAW_EVENT_RETENTION_DAYS`. Callers (the `crates/jobs` daily purge
/// task) MUST have already run `rollup_hour` for every bucket being
/// purged — this function does not itself verify that a rollup exists
/// before deleting, since re-deriving "has this hour been rolled up"
/// safely requires the caller's own scheduling guarantees, not a query
/// this crate can make authoritative on its own.
pub async fn purge_expired_raw_events(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let cutoff = Utc::now() - Duration::days(RAW_EVENT_RETENTION_DAYS);

    let result = sqlx::query("DELETE FROM analytics_events WHERE occurred_at_hour < $1")
        .bind(cutoff)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

/// Analogous retention enforcement for `link_check_results`
/// (docs/SECURITY-AND-SRE-OPERATIONS.md / PRD v3 Section V7: retained 12
/// months, then summarized). Summarization itself (producing a compact
/// historical record before deletion) is a `crates/jobs` responsibility;
/// this function only performs the deletion half once summarization has
/// already happened, mirroring `purge_expired_raw_events`'s division of
/// responsibility.
pub const LINK_CHECK_RETENTION_DAYS: i64 = 365;

pub async fn purge_expired_link_check_results(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let cutoff = Utc::now() - Duration::days(LINK_CHECK_RETENTION_DAYS);

    let result = sqlx::query("DELETE FROM link_check_results WHERE checked_at < $1")
        .bind(cutoff)
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retention_windows_match_the_documented_policy() {
        assert_eq!(RAW_EVENT_RETENTION_DAYS, 30);
        assert_eq!(LINK_CHECK_RETENTION_DAYS, 365);
    }
}
