//! The Axum webhook receiver Meta calls: `GET /webhook` for the one-time
//! subscription-verification handshake, `POST /webhook` for every
//! inbound message/status delivery. No decision logic lives here beyond
//! routing an inbound reply to `core_domain::answer` — rendering is
//! `render_whatsapp`'s job, per docs/PRD-V2-RUST-PLATFORM.md Section 6.7.

use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use channel_core::{render_node, RenderableNode};
use core_domain::EngineState;
use serde::Deserialize;

use crate::client::WhatsAppApiError;
use crate::render_whatsapp::{render_terminal, send_question};
use crate::signature::verify_signature;
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/webhook", get(verify_webhook).post(receive_webhook))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct VerifyQuery {
    #[serde(rename = "hub.mode")]
    mode: String,
    #[serde(rename = "hub.verify_token")]
    verify_token: String,
    #[serde(rename = "hub.challenge")]
    challenge: String,
}

async fn verify_webhook(State(state): State<AppState>, Query(query): Query<VerifyQuery>) -> impl IntoResponse {
    if query.mode == "subscribe" && query.verify_token == *state.verify_token {
        (StatusCode::OK, query.challenge).into_response()
    } else {
        tracing::warn!("WhatsApp webhook verification attempt failed (mode/token mismatch)");
        StatusCode::FORBIDDEN.into_response()
    }
}

/// Deliberately takes raw `Bytes` (not a pre-parsed JSON extractor) so
/// `verify_signature` can HMAC the exact bytes Meta signed — parsing to
/// JSON first and re-serializing before signing would not reliably
/// reproduce Meta's original byte sequence (key ordering, whitespace).
async fn receive_webhook(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> StatusCode {
    let signature_header = headers.get("X-Hub-Signature-256").and_then(|v| v.to_str().ok());

    if let Err(err) = verify_signature(signature_header, &body, &state.app_secret) {
        tracing::warn!(error = %err, "rejected WhatsApp webhook delivery: signature verification failed");
        return StatusCode::UNAUTHORIZED;
    }

    let payload: WebhookPayload = match serde_json::from_slice(&body) {
        Ok(payload) => payload,
        Err(err) => {
            tracing::warn!(error = %err, "WhatsApp webhook delivered a body that isn't valid JSON");
            return StatusCode::BAD_REQUEST;
        }
    };

    // Always acknowledge 200 past this point even if downstream
    // processing errors — Meta retries/backs off aggressively on
    // non-200 responses, and a transient Cloud API send failure
    // shouldn't cause Meta to redeliver (and reprocess) the same inbound
    // message repeatedly. Errors are logged, not swallowed silently.
    if let Err(err) = process_payload(&state, &payload).await {
        tracing::error!(error = %err, "error processing WhatsApp webhook payload");
    }

    StatusCode::OK
}

#[derive(Debug, Deserialize)]
struct WebhookPayload {
    #[serde(default)]
    entry: Vec<WebhookEntry>,
}

#[derive(Debug, Deserialize)]
struct WebhookEntry {
    #[serde(default)]
    changes: Vec<WebhookChange>,
}

#[derive(Debug, Deserialize)]
struct WebhookChange {
    value: WebhookChangeValue,
}

#[derive(Debug, Deserialize)]
struct WebhookChangeValue {
    /// Absent on delivery-status callbacks (`statuses[]`), which this
    /// adapter has nothing to do with and silently ignores.
    #[serde(default)]
    messages: Vec<InboundMessage>,
}

#[derive(Debug, Deserialize)]
struct InboundMessage {
    from: String,
    id: String,
    #[serde(default)]
    interactive: Option<InboundInteractive>,
}

#[derive(Debug, Deserialize)]
struct InboundInteractive {
    #[serde(default)]
    list_reply: Option<InboundReply>,
    #[serde(default)]
    button_reply: Option<InboundReply>,
}

#[derive(Debug, Deserialize)]
struct InboundReply {
    id: String,
}

async fn process_payload(state: &AppState, payload: &WebhookPayload) -> Result<(), WhatsAppApiError> {
    for entry in &payload.entry {
        for change in &entry.changes {
            for message in &change.value.messages {
                handle_inbound_message(state, message).await?;
            }
        }
    }
    Ok(())
}

async fn handle_inbound_message(state: &AppState, message: &InboundMessage) -> Result<(), WhatsAppApiError> {
    // Best-effort read receipt; a failure here shouldn't block answering.
    let _ = state.client.mark_read(&message.id).await;

    let answer_value: Option<&str> = message
        .interactive
        .as_ref()
        .and_then(|interactive| interactive.list_reply.as_ref().or(interactive.button_reply.as_ref()))
        .map(|reply| reply.id.as_str());

    let existing_token = state.conversations.session_token_for(&message.from).await;

    match (existing_token, answer_value) {
        (Some(token), Some(value)) => continue_walkthrough(state, &message.from, &token, value).await,
        (Some(_), None) => {
            state
                .client
                .send_text(
                    &message.from,
                    "Please tap one of the options above to answer, or send any message to start over.",
                )
                .await
        }
        (None, _) => start_walkthrough(state, &message.from).await,
    }
}

/// Every message this adapter sends is rendered in English. WhatsApp's
/// Cloud API, unlike Telegram's `User.language_code`, does not expose the
/// citizen's device/app language on an inbound message or contact object
/// — there is no equivalent field to read a locale preference from
/// automatically. Capturing a locale preference here would require either
/// an explicit first-turn language-choice question (a real, buildable
/// follow-up: prepend a language-select node to the tree specifically for
/// this channel) or a stored per-`wa_id` preference; neither exists yet,
/// so this is a disclosed scope limit, not a silently-dropped feature.
const LOCALE: &str = "en";

async fn start_walkthrough(state: &AppState, wa_id: &str) -> Result<(), WhatsAppApiError> {
    let engine_state = core_domain::create_session(&state.tree)
        .expect("vote_assist_tree_v2's startNodeId is validated by tests/tree_v2_structural.rs at build time");
    let token = state.sessions.create(engine_state.clone()).await;
    state.conversations.record_inbound(wa_id, token).await;
    send_current_node(state, wa_id, &engine_state).await
}

async fn continue_walkthrough(
    state: &AppState,
    wa_id: &str,
    session_token: &str,
    value: &str,
) -> Result<(), WhatsAppApiError> {
    let Some(current_state) = state.sessions.get(session_token).await else {
        state.conversations.remove(wa_id).await;
        return start_walkthrough(state, wa_id).await;
    };

    let next_state = match core_domain::answer(&state.tree, &current_state, value) {
        Ok(next) => next,
        Err(err) => {
            tracing::warn!(error = %err, wa_id, "invalid answer submitted via WhatsApp interactive reply");
            return state
                .client
                .send_text(wa_id, "That option is no longer valid — send any message to begin again.")
                .await;
        }
    };

    let node = core_domain::get_current_node(&state.tree, &next_state).expect(
        "core_domain::answer only ever transitions to a node id that exists in the same tree",
    );

    if node.is_terminal() {
        let RenderableNode::Terminal(terminal) = render_node(node, LOCALE) else {
            unreachable!("node.is_terminal() just confirmed this node renders as a terminal");
        };
        state.client.send_text(wa_id, &render_terminal(&terminal)).await?;
        state.sessions.remove(session_token).await;
        state.conversations.remove(wa_id).await;
        Ok(())
    } else {
        state.sessions.put(session_token.to_string(), next_state.clone()).await;
        state.conversations.record_inbound(wa_id, session_token.to_string()).await;
        send_current_node(state, wa_id, &next_state).await
    }
}

async fn send_current_node(state: &AppState, wa_id: &str, engine_state: &EngineState) -> Result<(), WhatsAppApiError> {
    let node = core_domain::get_current_node(&state.tree, engine_state)
        .expect("engine_state.current_node_id always resolves for a state this crate produced itself");

    match render_node(node, LOCALE) {
        RenderableNode::Question(question) => send_question(&state.client, wa_id, &question).await,
        RenderableNode::Terminal(terminal) => state.client.send_text(wa_id, &render_terminal(&terminal)).await,
    }
}
