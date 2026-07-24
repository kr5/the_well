//! `voteassist-ivr-gateway`: the standalone Exotel Passthru webhook
//! server. See `webhook`'s module doc for the scope/confidence caveats
//! around Exotel's exact integration schema.
//!
//! Configuration is via environment variables, matching the rest of this
//! workspace's binaries:
//! - `IVR_WEBHOOK_SHARED_SECRET` (optional but strongly recommended):
//!   embed as `?secret=...` in the webhook URL configured in Exotel's
//!   dashboard. If unset, the endpoint is unauthenticated — a loud
//!   startup warning says so, since Exotel's Passthru applet has no
//!   built-in request-signing scheme to fall back on.
//! - `WHATSAPP_PHONE_NUMBER_ID` / `WHATSAPP_ACCESS_TOKEN` /
//!   `WHATSAPP_GRAPH_API_VERSION` (all optional together): if
//!   `WHATSAPP_PHONE_NUMBER_ID` and `WHATSAPP_ACCESS_TOKEN` are both set,
//!   enables the E8.F3.T4 post-call WhatsApp deep-link handoff via
//!   `bot-whatsapp`'s client; otherwise that handoff is skipped and
//!   logged.
//! - `VOTEASSIST_IVR_ADDR` (optional, default `0.0.0.0:8082`).

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use bot_whatsapp::client::{WhatsAppClient, DEFAULT_GRAPH_API_VERSION};
use channel_core::SessionStore;
use core_domain::vote_assist_tree_v2;
use ivr_gateway::state::AppState;
use ivr_gateway::webhook;
use tokio::signal;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let webhook_shared_secret = env::var("IVR_WEBHOOK_SHARED_SECRET").ok().map(Arc::from);
    if webhook_shared_secret.is_none() {
        tracing::warn!(
            "IVR_WEBHOOK_SHARED_SECRET is not set — the Exotel Passthru webhook is unauthenticated. Set it and configure the matching ?secret=... in Exotel's dashboard webhook URL before going live."
        );
    }

    let whatsapp_handoff = match (env::var("WHATSAPP_PHONE_NUMBER_ID"), env::var("WHATSAPP_ACCESS_TOKEN")) {
        (Ok(phone_number_id), Ok(access_token)) => {
            let graph_api_version = env::var("WHATSAPP_GRAPH_API_VERSION")
                .unwrap_or_else(|_| DEFAULT_GRAPH_API_VERSION.to_string());
            tracing::info!("post-call WhatsApp deep-link handoff (E8.F3.T4) enabled");
            Some(Arc::new(WhatsAppClient::new(phone_number_id, access_token, graph_api_version)))
        }
        _ => {
            tracing::info!(
                "WHATSAPP_PHONE_NUMBER_ID/WHATSAPP_ACCESS_TOKEN not both set — post-call WhatsApp handoff disabled"
            );
            None
        }
    };

    let state = AppState {
        tree: Arc::new(vote_assist_tree_v2()),
        sessions: SessionStore::default(),
        whatsapp_handoff,
        webhook_shared_secret,
    };

    let addr: SocketAddr = env::var("VOTEASSIST_IVR_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8082".to_string())
        .parse()
        .expect("VOTEASSIST_IVR_ADDR must be a valid socket address, e.g. 0.0.0.0:8082");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind IVR webhook listener");
    tracing::info!(%addr, "voteassist-ivr-gateway listening");

    axum::serve(listener, webhook::router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
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
    tracing::info!("shutdown signal received, draining in-flight webhook requests");
}
