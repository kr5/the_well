//! `voteassist-bot-whatsapp`: the standalone WhatsApp webhook server.
//!
//! Configuration is via environment variables, matching `crates/api`'s
//! convention:
//! - `WHATSAPP_PHONE_NUMBER_ID` (required): the Cloud API phone number id.
//! - `WHATSAPP_ACCESS_TOKEN` (required): a system-user access token with
//!   `whatsapp_business_messaging` permission.
//! - `WHATSAPP_VERIFY_TOKEN` (required): an operator-chosen string,
//!   configured to match in Meta's App Dashboard webhook subscription.
//! - `WHATSAPP_APP_SECRET` (required): used to verify
//!   `X-Hub-Signature-256` on every inbound webhook delivery.
//! - `WHATSAPP_GRAPH_API_VERSION` (optional, default
//!   `client::DEFAULT_GRAPH_API_VERSION`): Meta's Graph API version path
//!   segment, e.g. `v21.0`.
//! - `VOTEASSIST_WHATSAPP_ADDR` (optional, default `0.0.0.0:8081`).

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use bot_whatsapp::client::{WhatsAppClient, DEFAULT_GRAPH_API_VERSION};
use bot_whatsapp::state::{AppState, ConversationIndex};
use bot_whatsapp::webhook;
use channel_core::SessionStore;
use core_domain::vote_assist_tree_v2;
use tokio::signal;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let phone_number_id =
        env::var("WHATSAPP_PHONE_NUMBER_ID").expect("WHATSAPP_PHONE_NUMBER_ID must be set");
    let access_token = env::var("WHATSAPP_ACCESS_TOKEN").expect("WHATSAPP_ACCESS_TOKEN must be set");
    let verify_token = env::var("WHATSAPP_VERIFY_TOKEN")
        .expect("WHATSAPP_VERIFY_TOKEN must be set (must match Meta App Dashboard's webhook subscription)");
    let app_secret = env::var("WHATSAPP_APP_SECRET")
        .expect("WHATSAPP_APP_SECRET must be set (verifies X-Hub-Signature-256 on inbound webhooks)");
    let graph_api_version =
        env::var("WHATSAPP_GRAPH_API_VERSION").unwrap_or_else(|_| DEFAULT_GRAPH_API_VERSION.to_string());

    let client = Arc::new(WhatsAppClient::new(phone_number_id, access_token, graph_api_version));
    let tree = Arc::new(vote_assist_tree_v2());

    let state = AppState {
        client,
        tree,
        sessions: SessionStore::default(),
        conversations: ConversationIndex::default(),
        verify_token: verify_token.into(),
        app_secret: app_secret.into(),
    };

    let addr: SocketAddr = env::var("VOTEASSIST_WHATSAPP_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8081".to_string())
        .parse()
        .expect("VOTEASSIST_WHATSAPP_ADDR must be a valid socket address, e.g. 0.0.0.0:8081");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind WhatsApp webhook listener");
    tracing::info!(%addr, "voteassist-bot-whatsapp listening");

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
