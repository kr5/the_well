//! `voteassist-jobs`: the standalone scheduled-worker binary.
//!
//! Runs four maintenance jobs on independent interval-based schedules
//! (see the constants below), each in its own `tokio` task, sharing one
//! Postgres connection pool. See `src/lib.rs`'s module doc for why this
//! uses a hand-rolled `tokio::time::interval` scheduler rather than
//! `apalis`/`apalis-cron`.
//!
//! Configuration is via environment variables only (no config file, no
//! CLI flags), matching `crates/api`'s `main.rs` convention:
//! - `DATABASE_URL` (required): Postgres connection string.

use std::future::Future;
use std::time::Duration as StdDuration;

use analytics::AnalyticsClient;
use jobs::{
    run_analytics_maintenance_job, run_link_check_job, run_reverification_digest_job,
    run_translation_completeness_job,
};
use sqlx::postgres::PgPoolOptions;
use tokio::signal;
use tokio::sync::watch;
use tokio::time::{interval, MissedTickBehavior};
use tracing_subscriber::EnvFilter;

/// Nightly: docs/SECURITY-AND-SRE-OPERATIONS.md Section 3 specifies this
/// job runs low-frequency against government-operated domains.
const LINK_CHECK_INTERVAL: StdDuration = StdDuration::from_secs(24 * 60 * 60);

/// Daily: cheap, pure-SQL scan against `knowledge_entries`; running it
/// daily (rather than only every 180 days, the threshold itself) means an
/// entry crosses into "overdue" within a day of actually becoming overdue.
const REVERIFICATION_DIGEST_INTERVAL: StdDuration = StdDuration::from_secs(24 * 60 * 60);

/// Daily: feeds the Translation Management dashboard (PRD v2 admin page
/// 5), which does not need finer-grained freshness than once a day.
const TRANSLATION_COMPLETENESS_INTERVAL: StdDuration = StdDuration::from_secs(24 * 60 * 60);

/// Hourly: matches the hour-bucketed rollup grain docs/PRD-V2-RUST-PLATFORM.md
/// Section 12 specifies for `analytics_hourly_rollup`; also carries the
/// daily rollup and retention purge, both idempotent no-ops when not yet
/// due, so folding them into the same hourly tick is safe and avoids a
/// fifth scheduler task.
const ANALYTICS_MAINTENANCE_INTERVAL: StdDuration = StdDuration::from_secs(60 * 60);

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set, e.g. postgres://user:pass@host:5432/voteassist");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to Postgres");

    let analytics_client = AnalyticsClient::new(pool.clone());

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let link_check_pool = pool.clone();
    let link_check_task = tokio::spawn(run_scheduled(
        "link_checker",
        LINK_CHECK_INTERVAL,
        shutdown_rx.clone(),
        move || {
            let pool = link_check_pool.clone();
            async move {
                match run_link_check_job(&pool).await {
                    Ok(summary) => tracing::info!(
                        checked = summary.checked,
                        unhealthy = summary.unhealthy,
                        persisted = summary.persisted,
                        skipped_no_db_row = summary.skipped_no_db_row,
                        "link check job completed"
                    ),
                    Err(error) => tracing::error!(%error, "link check job failed"),
                }
            }
        },
    ));

    let reverification_pool = pool.clone();
    let reverification_task = tokio::spawn(run_scheduled(
        "reverification_digest",
        REVERIFICATION_DIGEST_INTERVAL,
        shutdown_rx.clone(),
        move || {
            let pool = reverification_pool.clone();
            async move {
                match run_reverification_digest_job(&pool).await {
                    Ok(flagged) => tracing::info!(flagged, "reverification digest job completed"),
                    Err(error) => tracing::error!(%error, "reverification digest job failed"),
                }
            }
        },
    ));

    let translation_pool = pool.clone();
    let translation_task = tokio::spawn(run_scheduled(
        "translation_completeness",
        TRANSLATION_COMPLETENESS_INTERVAL,
        shutdown_rx.clone(),
        move || {
            let pool = translation_pool.clone();
            async move {
                match run_translation_completeness_job(&pool).await {
                    Ok(()) => tracing::info!("translation completeness job completed"),
                    Err(error) => tracing::error!(%error, "translation completeness job failed"),
                }
            }
        },
    ));

    let analytics_maintenance_task = tokio::spawn(run_scheduled(
        "analytics_maintenance",
        ANALYTICS_MAINTENANCE_INTERVAL,
        shutdown_rx.clone(),
        move || {
            let analytics_client = analytics_client.clone();
            async move {
                match run_analytics_maintenance_job(&analytics_client).await {
                    Ok(summary) => tracing::info!(
                        hourly_rows = summary.hourly_rows,
                        daily_rows = summary.daily_rows,
                        purged_events = summary.purged_events,
                        purged_link_checks = summary.purged_link_checks,
                        "analytics maintenance job completed"
                    ),
                    Err(error) => tracing::error!(%error, "analytics maintenance job failed"),
                }
            }
        },
    ));

    tracing::info!("voteassist-jobs started: 4 scheduled workers running");

    shutdown_signal().await;
    tracing::info!("shutdown signal received, waiting for in-flight job runs to finish");
    let _ = shutdown_tx.send(true);

    let _ = tokio::join!(
        link_check_task,
        reverification_task,
        translation_task,
        analytics_maintenance_task,
    );

    pool.close().await;
    tracing::info!("voteassist-jobs stopped cleanly");
}

/// Drives one job on a fixed interval until `shutdown` reports `true`.
/// `job` is called fresh on every tick — it must be cheap to construct
/// (typically just cloning a `PgPool`/`AnalyticsClient` handle) since the
/// actual work happens in the future it returns.
async fn run_scheduled<F, Fut>(
    name: &'static str,
    period: StdDuration,
    mut shutdown: watch::Receiver<bool>,
    job: F,
) where
    F: Fn() -> Fut,
    Fut: Future<Output = ()>,
{
    let mut ticker = interval(period);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                tracing::info!(job = name, "starting scheduled job run");
                job().await;
            }
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    tracing::info!(job = name, "stopping scheduler");
                    break;
                }
            }
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
