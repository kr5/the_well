//! `voteassist-bot-telegram`: the standalone Telegram bot binary.
//!
//! Configuration is via environment variables, matching `crates/api` and
//! `crates/jobs`'s convention:
//! - `TELOXIDE_TOKEN` (required, read by `Bot::from_env()`): the bot
//!   token from @BotFather.
//!
//! Deliberately does NOT connect to Postgres — the only reason this crate
//! would need a database is the MCC-gated proactive-broadcast send path
//! (`broadcast::send_proactive_announcement`), and no trigger for that
//! exists yet (see that module's doc comment). Reply-only operation, which
//! is everything this binary does today, needs no database.
//!
//! - `BOT_TELEGRAM_HEALTH_ADDR` (optional, default `0.0.0.0:9091`): like
//!   `crates/jobs`, this binary's real work (long-polling Telegram) isn't
//!   HTTP, but docs/SECURITY-AND-SRE-OPERATIONS.md expects every service
//!   to expose `/healthz`/`/metrics` regardless.

use std::sync::Arc;
use std::time::Duration;

use axum::routing::get;
use axum::Router;
use bot_telegram::handlers::{handle_answer_callback, handle_command, handle_stray_callback};
use bot_telegram::state::State;
use channel_core::SessionStore;
use core_domain::vote_assist_tree_v2;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;
use tracing_subscriber::EnvFilter;

/// How often to sweep expired walkthrough sessions out of memory. Distinct
/// from `channel_core::DEFAULT_SESSION_TTL` (the per-session expiry
/// itself) — this is just the housekeeping cadence.
const SESSION_SWEEP_INTERVAL: Duration = Duration::from_secs(5 * 60);

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let bot = Bot::from_env();
    let tree = Arc::new(vote_assist_tree_v2());
    let sessions = SessionStore::default();

    let metric_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install the Prometheus metrics recorder");
    tokio::spawn(run_metrics_upkeep(metric_handle.clone()));
    tokio::spawn(serve_health_and_metrics(metric_handle));

    let sweep_sessions = sessions.clone();
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(SESSION_SWEEP_INTERVAL);
        loop {
            ticker.tick().await;
            let removed = sweep_sessions.sweep_expired().await;
            if removed > 0 {
                tracing::info!(removed, "swept expired Telegram walkthrough sessions");
            }
        }
    });

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<State>, State>()
                .endpoint(handle_command),
        )
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, InMemStorage<State>, State>()
                .branch(
                    dptree::case![State::Walkthrough { session_token }]
                        .endpoint(handle_answer_callback),
                )
                .branch(dptree::case![State::Idle].endpoint(handle_stray_callback)),
        );

    tracing::info!("voteassist-bot-telegram starting (long polling)");

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![InMemStorage::<State>::new(), tree, sessions])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    tracing::info!("voteassist-bot-telegram stopped");
}

/// See `crates/jobs::main::serve_health_and_metrics`'s doc comment — same
/// reasoning, different default port so both binaries can run on one host.
async fn serve_health_and_metrics(metric_handle: PrometheusHandle) {
    let addr = std::env::var("BOT_TELEGRAM_HEALTH_ADDR").unwrap_or_else(|_| "0.0.0.0:9091".to_string());

    let app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/metrics", get(move || async move { metric_handle.render() }));

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(error) => {
            tracing::error!(%error, %addr, "failed to bind bot-telegram health/metrics listener");
            return;
        }
    };

    tracing::info!(%addr, "voteassist-bot-telegram health/metrics endpoint listening");
    if let Err(error) = axum::serve(listener, app).await {
        tracing::error!(%error, "bot-telegram health/metrics server exited unexpectedly");
    }
}

async fn run_metrics_upkeep(metric_handle: PrometheusHandle) {
    let mut ticker = tokio::time::interval(Duration::from_secs(5));
    loop {
        ticker.tick().await;
        metric_handle.run_upkeep();
    }
}
