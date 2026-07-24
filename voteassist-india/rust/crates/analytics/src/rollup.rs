use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// Aggregates every raw `analytics_events` row within `[bucket_hour,
/// bucket_hour + 1 hour)` into `analytics_rollups_hourly`, per
/// docs/PRD-V2-RUST-PLATFORM.md Section 12's rollup/purge discipline.
/// Idempotent: re-running for an hour that already has rollup rows
/// updates them in place via `ON CONFLICT ... DO UPDATE`, matching the
/// `UNIQUE` constraint on `analytics_rollups_hourly` in
/// `migrations/0009_analytics.sql`.
///
/// Intended to be invoked hourly by `crates/jobs`'s `apalis-cron`
/// scheduler, not on a per-request basis.
pub async fn rollup_hour(pool: &PgPool, bucket_hour: DateTime<Utc>) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO analytics_rollups_hourly (
            bucket_hour, decision_tree_version, node_id, event_type, locale, client_platform, event_count
        )
        SELECT
            occurred_at_hour,
            decision_tree_version,
            node_id,
            event_type,
            locale,
            client_platform,
            COUNT(*) AS event_count
        FROM analytics_events
        WHERE occurred_at_hour = $1
        GROUP BY occurred_at_hour, decision_tree_version, node_id, event_type, locale, client_platform
        ON CONFLICT (bucket_hour, decision_tree_version, node_id, event_type, locale, client_platform)
        DO UPDATE SET event_count = EXCLUDED.event_count
        "#,
    )
    .bind(bucket_hour)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Aggregates a full day's `analytics_rollups_hourly` rows into
/// `analytics_rollups_daily`. Reads from the hourly rollup table, not raw
/// events, since by the time a daily rollup runs the corresponding raw
/// events may already have been purged (30-day retention, per PRD v2
/// Section 12) — the daily table's long-horizon retention must not depend
/// on the short-lived raw table still existing.
pub async fn rollup_day(pool: &PgPool, bucket_day: chrono::NaiveDate) -> Result<u64, sqlx::Error> {
    let day_start = bucket_day.and_hms_opt(0, 0, 0).expect("midnight is always valid").and_utc();
    let day_end = day_start + chrono::Duration::days(1);

    let result = sqlx::query(
        r#"
        INSERT INTO analytics_rollups_daily (
            bucket_day, decision_tree_version, node_id, event_type, locale, client_platform, event_count
        )
        SELECT
            $1::date,
            decision_tree_version,
            node_id,
            event_type,
            locale,
            client_platform,
            SUM(event_count) AS event_count
        FROM analytics_rollups_hourly
        WHERE bucket_hour >= $2 AND bucket_hour < $3
        GROUP BY decision_tree_version, node_id, event_type, locale, client_platform
        ON CONFLICT (bucket_day, decision_tree_version, node_id, event_type, locale, client_platform)
        DO UPDATE SET event_count = EXCLUDED.event_count
        "#,
    )
    .bind(bucket_day)
    .bind(day_start)
    .bind(day_end)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Funnel drop-off rate for a single question node within a tree version:
/// (sessions that answered this node) vs. (sessions that reached this
/// node), read from the hourly rollups within `[since, until)`. Backs the
/// admin "tree health" report (PRD v3 Section V5) — the Decision Tree
/// Visual Editor shows this inline on each node.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct NodeDropOff {
    pub node_id: String,
    pub reached_count: i64,
    pub answered_count: i64,
}

impl NodeDropOff {
    /// Fraction of sessions that reached this node but never answered it
    /// (0.0 = no drop-off, 1.0 = everyone abandoned here). `reached_count`
    /// is approximated here as `question_answered` events for the node
    /// PRIOR to this one in a session; since raw per-session sequencing
    /// isn't retained after purge, the admin dashboard query
    /// (`crates/jobs`'s tree-health report) computes reached_count
    /// separately as "answered_count of any node with this as its `next`
    /// target," which is a `core-domain`-aware join done at the call site,
    /// not inside this struct.
    pub fn drop_off_rate(&self) -> f64 {
        if self.reached_count == 0 {
            return 0.0;
        }
        let abandoned = (self.reached_count - self.answered_count).max(0);
        abandoned as f64 / self.reached_count as f64
    }
}

pub async fn node_answer_counts(
    pool: &PgPool,
    decision_tree_version: i32,
    since: DateTime<Utc>,
    until: DateTime<Utc>,
) -> Result<Vec<(String, i64)>, sqlx::Error> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT node_id, SUM(event_count)::bigint AS total
        FROM analytics_rollups_hourly
        WHERE decision_tree_version = $1
          AND event_type = 'question_answered'
          AND node_id IS NOT NULL
          AND bucket_hour >= $2 AND bucket_hour < $3
        GROUP BY node_id
        ORDER BY total DESC
        "#,
    )
    .bind(decision_tree_version)
    .bind(since)
    .bind(until)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
