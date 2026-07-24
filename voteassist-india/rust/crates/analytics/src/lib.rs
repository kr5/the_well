//! Privacy-preserving analytics event ingestion and rollup for VoteAssist
//! India. See docs/PRD-V2-RUST-PLATFORM.md Section 12 and
//! docs/PRD-V3-COMPREHENSIVE-EXPANSION.md Section V7 for the full policy
//! this crate implements. Non-negotiables, enforced by types rather than
//! left to convention: no cookies, no cross-site tracking, no free-text
//! search queries stored, no IP-to-identity linkage, hour-bucketed
//! timestamps only, 30-day raw retention with indefinite anonymized
//! rollups.

pub mod ingest;
pub mod purge;
pub mod rollup;
pub mod types;

pub use ingest::record_event;
pub use purge::{purge_expired_link_check_results, purge_expired_raw_events};
pub use rollup::{node_answer_counts, rollup_day, rollup_hour, NodeDropOff};
pub use types::{ClientPlatform, EventType, NewEvent};

use sqlx::PgPool;

/// Thin wrapper bundling a `PgPool` with this crate's operations, for
/// callers (`crates/api`, `crates/jobs`) that want a single handle rather
/// than passing `&PgPool` to every free function individually.
#[derive(Clone)]
pub struct AnalyticsClient {
    pool: PgPool,
}

impl AnalyticsClient {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record(&self, event: &NewEvent) -> Result<(), sqlx::Error> {
        record_event(&self.pool, event).await
    }

    pub async fn rollup_hour(&self, bucket_hour: chrono::DateTime<chrono::Utc>) -> Result<u64, sqlx::Error> {
        rollup_hour(&self.pool, bucket_hour).await
    }

    pub async fn rollup_day(&self, bucket_day: chrono::NaiveDate) -> Result<u64, sqlx::Error> {
        rollup_day(&self.pool, bucket_day).await
    }

    pub async fn purge_expired(&self) -> Result<(u64, u64), sqlx::Error> {
        let events = purge_expired_raw_events(&self.pool).await?;
        let links = purge_expired_link_check_results(&self.pool).await?;
        Ok((events, links))
    }
}
