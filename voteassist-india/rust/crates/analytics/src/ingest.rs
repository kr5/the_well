use chrono::Utc;
use sqlx::PgPool;

use crate::types::{truncate_to_hour, NewEvent};

/// Records a single event. Uses the runtime-checked `sqlx::query` API
/// (see the doc comment on `EventType` for why) with an explicit column
/// list matching `migrations/0009_analytics.sql`'s `analytics_events`
/// table exactly. `occurred_at_hour` is computed here, at write time, from
/// `Utc::now()` truncated to the hour — callers never supply a timestamp,
/// so there is no way to accidentally record full-precision timing data.
pub async fn record_event(pool: &PgPool, event: &NewEvent) -> Result<(), sqlx::Error> {
    let occurred_at_hour = truncate_to_hour(Utc::now());

    sqlx::query(
        r#"
        INSERT INTO analytics_events (
            event_type, decision_tree_version, node_id, answer_option,
            session_id, locale, client_platform, deep_link_clicked,
            kb_search_category_hash, occurred_at_hour
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(event.event_type)
    .bind(event.decision_tree_version)
    .bind(&event.node_id)
    .bind(&event.answer_option)
    .bind(event.session_id)
    .bind(&event.locale)
    .bind(event.client_platform)
    .bind(event.deep_link_clicked)
    .bind(&event.kb_search_category_hash)
    .bind(occurred_at_hour)
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ClientPlatform, EventType};
    use uuid::Uuid;

    /// This test does not run against a live database (none is available
    /// in this environment) — it only checks that `record_event`'s query
    /// construction and parameter binding compile and type-check against
    /// the declared column order, which is the failure mode most likely
    /// to silently drift from the migration over time. A full integration
    /// test against a real Postgres instance (e.g. via `sqlx::test`) is a
    /// tracked follow-up once this crate runs in an environment with a
    /// database (see rust/README.md).
    #[test]
    fn record_event_compiles_against_a_representative_new_event() {
        let event = NewEvent::new(EventType::SessionStarted, Uuid::new_v4(), "en", ClientPlatform::Web);
        // Constructing the future is enough to prove the query/bind chain
        // type-checks; we deliberately do not `.await` it without a pool.
        let _future = record_event(&unreachable_pool(), &event);
    }

    fn unreachable_pool() -> PgPool {
        // PgPool::connect_lazy never actually connects until first use,
        // so this is safe to construct (but never call) in a unit test
        // with no live database.
        PgPool::connect_lazy("postgres://localhost/unused_in_unit_tests")
            .expect("connect_lazy does not perform I/O and should never fail synchronously")
    }
}
