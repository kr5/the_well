//! `voteassist-jobs`: the standalone scheduled-worker binary.
//!
//! Runs five maintenance jobs on independent interval-based schedules
//! (see the constants below), each in its own `tokio` task, sharing one
//! Postgres connection pool. See `src/lib.rs`'s module doc for why this
//! uses a hand-rolled `tokio::time::interval` scheduler rather than
//! `apalis`/`apalis-cron`.
//!
//! Configuration is via environment variables only (no config file, no
//! CLI flags), matching `crates/api`'s `main.rs` convention:
//! - `DATABASE_URL` (required): Postgres connection string.
//! - `JOBS_HEALTH_ADDR` (optional, default `0.0.0.0:9090`): this binary
//!   has no HTTP traffic of its own (it's a scheduler, not a web
//!   service), but docs/SECURITY-AND-SRE-OPERATIONS.md's monitoring
//!   posture expects every service to expose `/healthz` and `/metrics`
//!   regardless — see `serve_health_and_metrics` below.

use std::future::Future;
use std::time::Duration as StdDuration;

use analytics::AnalyticsClient;
use axum::routing::get;
use axum::Router;
use jobs::{
    run_analytics_maintenance_job, run_link_check_job, run_reverification_digest_job,
    run_session_cleanup_job, run_translation_completeness_job,
};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
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

/// Daily: expired sessions/OTP challenges are low-urgency cleanup (their
/// own expiry already makes them unusable; this just reclaims storage),
/// matching the reverification-digest/translation-completeness cadence.
const SESSION_CLEANUP_INTERVAL: StdDuration = StdDuration::from_secs(24 * 60 * 60);

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

    let metric_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install the Prometheus metrics recorder");
    tokio::spawn(run_metrics_upkeep(metric_handle.clone()));
    tokio::spawn(serve_health_and_metrics(metric_handle));

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

    let session_cleanup_pool = pool.clone();
    let session_cleanup_task = tokio::spawn(run_scheduled(
        "session_cleanup",
        SESSION_CLEANUP_INTERVAL,
        shutdown_rx.clone(),
        move || {
            let pool = session_cleanup_pool.clone();
            async move {
                match run_session_cleanup_job(&pool).await {
                    Ok(summary) => tracing::info!(
                        admin_sessions = summary.admin_sessions,
                        account_sessions = summary.account_sessions,
                        otp_challenges = summary.otp_challenges,
                        "session cleanup job completed"
                    ),
                    Err(error) => tracing::error!(%error, "session cleanup job failed"),
                }
            }
        },
    ));

    tracing::info!("voteassist-jobs started: 5 scheduled workers running");

    shutdown_signal().await;
    tracing::info!("shutdown signal received, waiting for in-flight job runs to finish");
    let _ = shutdown_tx.send(true);

    let _ = tokio::join!(
        link_check_task,
        reverification_task,
        translation_task,
        analytics_maintenance_task,
        session_cleanup_task,
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
                metrics::counter!("voteassist_job_runs_total", "job" => name).increment(1);
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

/// Every service in this workspace exposes `/healthz` (liveness) and
/// `/metrics` (Prometheus scrape target), per
/// docs/SECURITY-AND-SRE-OPERATIONS.md — including this one, even though
/// it's a scheduler with no other HTTP surface. `voteassist_job_runs_total`
/// (a per-job counter, incremented in `run_scheduled` above) is
/// deliberately the only custom metric recorded in this pass: splitting
/// it into success/failure would need each job closure's return type
/// threaded back through `run_scheduled`'s generic `Fn() -> Fut` — a real,
/// disclosed follow-up, not silently skipped.
async fn serve_health_and_metrics(metric_handle: PrometheusHandle) {
    let addr = std::env::var("JOBS_HEALTH_ADDR").unwrap_or_else(|_| "0.0.0.0:9090".to_string());

    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/metrics", get(move || async move { metric_handle.render() }));

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(%error, %addr, "failed to bind jobs health/metrics listener");
            return;
        }
    };

    tracing::info!(%addr, "voteassist-jobs health/metrics endpoint listening");
    if let Err(error) = axum::serve(listener, app).await {
        tracing::error!(%error, "jobs health/metrics server exited unexpectedly");
    }
}

/// `metrics-exporter-prometheus`'s own docs: callers of `install_recorder`
/// (rather than the all-in-one `install()`) are "responsible for keeping
/// a handle to the recorder and calling `run_upkeep` at a regular
/// interval" — this drives time-based bookkeeping (histogram bucket
/// decay, etc.) that would otherwise never run.
async fn run_metrics_upkeep(metric_handle: PrometheusHandle) {
    let mut ticker = interval(StdDuration::from_secs(5));
    loop {
        ticker.tick().await;
        metric_handle.run_upkeep();
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
