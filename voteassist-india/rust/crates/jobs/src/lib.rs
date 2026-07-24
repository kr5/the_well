//! Crate root for `voteassist-jobs`: the scheduled background-worker
//! binary that runs VoteAssist India's maintenance jobs outside the
//! request/response path of `crates/api`. See
//! docs/PRD-V2-RUST-PLATFORM.md Section 6.8 for the job list and
//! docs/SECURITY-AND-SRE-OPERATIONS.md Section 3 for the operational
//! posture (rate limits, alerting, what each job is and isn't allowed to
//! do).
//!
//! This crate deliberately schedules jobs with a plain
//! `tokio::time::interval` loop per job (see `src/main.rs`) rather than
//! pulling in `apalis`/`apalis-cron` as docs/PRD-V2-RUST-PLATFORM.md
//! Section 6.8 originally proposed. That crate's exact API surface
//! (builder types, trait bounds for a `Job` impl, the cron-schedule
//! adapter) cannot be verified without a local `cargo build`, which is
//! off-limits for this development environment (code is written here and
//! compiled/run later on a server). A hand-rolled interval scheduler is a
//! legitimate, if less feature-rich, production pattern — it has no
//! retry/backoff or persistent job queue, which a future migration to a
//! real job-queue crate (apalis or otherwise) should add once it can be
//! compiled and verified. That gap is tracked, not hidden.

pub mod link_checker;
pub mod reverification_digest;
pub mod translation_completeness;

pub use link_checker::{build_client, check_all_citation_links, persist_outcomes, LinkCheckOutcome, PersistSummary};
pub use reverification_digest::{
    find_overdue_entries, flag_overdue_entries, EntryNeedingReverification,
    DEFAULT_REVERIFICATION_THRESHOLD_DAYS,
};
pub use translation_completeness::{compute_kb_translation_status, LocaleCompleteness};

use analytics::AnalyticsClient;
use chrono::{DateTime, Duration, Timelike, Utc};
use sqlx::PgPool;

/// All Eighth Schedule locale codes VoteAssist India tracks translation
/// completeness for, mirroring `packages/i18n/src/locales.ts`'s
/// `supportedLocales` list. Only `en` and `hi` are `"shipped"` there today;
/// the rest are included here too so the Translation Management dashboard
/// (PRD v2 admin page 5) can show real "0 of N reviewed" rows for
/// not-yet-started languages instead of omitting them entirely.
pub const TRACKED_LOCALES: &[&str] = &[
    "en", "hi", "bn", "ta", "te", "mr", "gu", "kn", "ml", "pa", "ur", "or", "as", "kok", "mni",
    "doi", "brx", "sat", "mai", "sd", "ne", "ks", "sa",
];

/// Runs the nightly citation link-checker end to end: fetch every cited
/// URL in the curated knowledge base, persist the outcomes, and return a
/// summary for logging/alerting. Split out from `main.rs`'s scheduling
/// loop so it has a single, directly testable entry point.
pub async fn run_link_check_job(pool: &PgPool) -> Result<LinkCheckJobSummary, sqlx::Error> {
    let client = build_client().expect("reqwest client with a static, valid user agent must build");
    let outcomes = check_all_citation_links(&client).await;
    let unhealthy = outcomes.iter().filter(|o| !o.is_healthy()).count() as u64;
    let persisted = persist_outcomes(pool, &outcomes).await?;

    Ok(LinkCheckJobSummary {
        checked: outcomes.len() as u64,
        unhealthy,
        persisted: persisted.persisted,
        skipped_no_db_row: persisted.skipped_no_db_row,
    })
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LinkCheckJobSummary {
    pub checked: u64,
    pub unhealthy: u64,
    pub persisted: u64,
    pub skipped_no_db_row: u64,
}

/// Runs the KB re-verification digest with the documented default
/// threshold (180 days, see `reverification_digest::DEFAULT_REVERIFICATION_THRESHOLD_DAYS`)
/// and returns how many entries were newly flagged `needs_reverification`.
pub async fn run_reverification_digest_job(pool: &PgPool) -> Result<u64, sqlx::Error> {
    flag_overdue_entries(pool, DEFAULT_REVERIFICATION_THRESHOLD_DAYS).await
}

/// Runs the translation-completeness report across every tracked locale
/// (see `TRACKED_LOCALES` above), writing one `translation_status` row per
/// locale for this run.
pub async fn run_translation_completeness_job(pool: &PgPool) -> Result<(), sqlx::Error> {
    compute_kb_translation_status(pool, TRACKED_LOCALES).await
}

/// Runs the hourly analytics rollup for the just-completed UTC hour (i.e.
/// the previous whole hour relative to when this job fires, so the source
/// hour's raw events have all landed before it's summarized) plus the
/// day-level rollup for "yesterday" once a day, and the raw-event/
/// link-check retention purge. Bundling these three into one function
/// mirrors how `main.rs` schedules them together at a coarser cadence than
/// the link-checker/digest/translation jobs above, since they're cheap,
/// pure-SQL aggregations rather than outbound-network jobs.
pub async fn run_analytics_maintenance_job(
    analytics: &AnalyticsClient,
) -> Result<AnalyticsMaintenanceSummary, sqlx::Error> {
    let now = Utc::now();
    let current_hour_start = {
        let truncated = now
            .date_naive()
            .and_hms_opt(now.hour(), 0, 0)
            .expect("now.hour() is always a valid hour-of-day");
        DateTime::<Utc>::from_naive_utc_and_offset(truncated, Utc)
    };
    let previous_hour = current_hour_start - Duration::hours(1);
    let hourly_rows = analytics.rollup_hour(previous_hour).await?;

    let yesterday = (now - Duration::days(1)).date_naive();
    let daily_rows = analytics.rollup_day(yesterday).await?;

    let (purged_events, purged_link_checks) = analytics.purge_expired().await?;

    Ok(AnalyticsMaintenanceSummary {
        hourly_rows,
        daily_rows,
        purged_events,
        purged_link_checks,
    })
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AnalyticsMaintenanceSummary {
    pub hourly_rows: u64,
    pub daily_rows: u64,
    pub purged_events: u64,
    pub purged_link_checks: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracked_locales_includes_both_shipped_languages() {
        assert!(TRACKED_LOCALES.contains(&"en"));
        assert!(TRACKED_LOCALES.contains(&"hi"));
    }

    #[test]
    fn tracked_locales_has_no_duplicates() {
        let mut sorted = TRACKED_LOCALES.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), TRACKED_LOCALES.len(), "TRACKED_LOCALES must not contain duplicate codes");
    }
}
