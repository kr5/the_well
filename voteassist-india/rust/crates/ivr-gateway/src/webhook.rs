//! The Exotel Passthru-applet webhook: `GET /exotel/passthru`.
//!
//! Built against Exotel's publicly documented Passthru-applet contract —
//! a GET request carrying call details as query parameters, with Sync
//! mode expecting an HTTP response back within the call flow — per
//! support.exotel.com as of this crate's authorship. This is NOT built
//! against Exotel's exact schema for reconfiguring a "programmable
//! Gather" applet's prompt/digit-count from a webhook response, which
//! requires a live Exotel developer account/dashboard flow to confirm
//! precisely and could not be verified in this environment.
//!
//! This is exactly the "interface defined, not fully wired" scope
//! docs/PRD-V2-RUST-PLATFORM.md Section 6.7 designates for MVP-Rust-v1:
//! `PassthruResponse` below is this scaffold's own documented contract,
//! meant to be wired into whichever Say/Gather applet configuration the
//! operator's Exotel dashboard flow uses, and to be tightened against
//! Exotel's real schema when this crate's v2/v3 full implementation
//! (E8.F3) is built against a live account.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use axum_prometheus::PrometheusMetricLayer;
use channel_core::{render_node, RenderableNode, RenderableTerminal};
use serde::{Deserialize, Serialize};

use crate::render_ivr::{map_digit_to_option, render_spoken_prompt};
use crate::state::AppState;

/// Liveness only — see `bot_whatsapp::webhook::healthz`'s doc comment for
/// why this deliberately doesn't probe Exotel's own reachability.
async fn healthz() -> &'static str {
    "ok"
}

pub fn router(state: AppState) -> Router {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    Router::new()
        .route("/exotel/passthru", get(handle_passthru))
        .route("/healthz", get(healthz))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(prometheus_layer)
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct PassthruQuery {
    #[serde(rename = "CallSid")]
    call_sid: String,
    #[serde(rename = "CallFrom", default)]
    call_from: Option<String>,
    #[serde(rename = "Digits", default)]
    digits: Option<String>,
    #[serde(default)]
    secret: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PassthruResponse {
    /// Text for the current Say/Gather applet to speak via TTS.
    pub prompt_text: String,
    /// Locale tag Exotel's Indic TTS should render `prompt_text` in
    /// (only "en"/"hi" carry real content today, matching this project's
    /// shipped-locale set).
    pub prompt_language: &'static str,
    /// How many DTMF digits the next Gather step should collect (always
    /// 1 today — see `render_ivr::MAX_DTMF_OPTIONS`).
    pub expected_digits: u8,
    /// True once the call has reached a terminal outcome and no further
    /// Gather step is expected — the dashboard flow should route to a
    /// closing/hangup applet when this is true.
    pub is_terminal: bool,
}

async fn handle_passthru(
    State(state): State<AppState>,
    Query(query): Query<PassthruQuery>,
) -> impl IntoResponse {
    if let Some(expected) = &state.webhook_shared_secret {
        if query.secret.as_deref() != Some(expected.as_ref()) {
            tracing::warn!(call_sid = %query.call_sid, "rejected Exotel Passthru request: missing or wrong shared secret");
            return StatusCode::FORBIDDEN.into_response();
        }
    }

    let current_state = match state.sessions.get(&query.call_sid).await {
        Some(existing) => existing,
        None => core_domain::create_session(&state.tree).expect(
            "vote_assist_tree_v2's startNodeId is validated by tests/tree_v2_structural.rs at build time",
        ),
    };

    let current_node = core_domain::get_current_node(&state.tree, &current_state)
        .expect("current_state.current_node_id always resolves for a state this crate produced itself");

    let next_state = match (
        query.digits.as_deref().filter(|d| !d.is_empty()),
        render_node(current_node, "en"),
    ) {
        (Some(digits), RenderableNode::Question(question)) => match map_digit_to_option(&question, digits) {
            Some(value) => match core_domain::answer(&state.tree, &current_state, value) {
                Ok(next) => next,
                Err(err) => {
                    tracing::warn!(error = %err, call_sid = %query.call_sid, "invalid DTMF-derived answer");
                    current_state
                }
            },
            None => {
                tracing::warn!(digits, call_sid = %query.call_sid, "DTMF digits did not map to any offered option");
                current_state
            }
        },
        _ => current_state,
    };

    let next_node = core_domain::get_current_node(&state.tree, &next_state)
        .expect("next_state.current_node_id always resolves for a state this crate just produced");

    let response = match render_node(next_node, "en") {
        RenderableNode::Question(question) => {
            state.sessions.put(query.call_sid.clone(), next_state).await;
            PassthruResponse {
                prompt_text: render_spoken_prompt(&question),
                prompt_language: "en",
                expected_digits: 1,
                is_terminal: false,
            }
        }
        RenderableNode::Terminal(terminal) => {
            state.sessions.remove(&query.call_sid).await;
            handle_terminal_handoff(&state, query.call_from.as_deref(), &terminal).await;
            PassthruResponse {
                prompt_text: spoken_terminal_summary(&terminal),
                prompt_language: "en",
                expected_digits: 0,
                is_terminal: true,
            }
        }
    };

    Json(response).into_response()
}

fn spoken_terminal_summary(terminal: &RenderableTerminal) -> String {
    let mut spoken = terminal.outcome_title.clone();
    spoken.push_str(". ");
    spoken.push_str(&terminal.outcome_description);
    if !terminal.recommended_forms.is_empty() {
        spoken.push_str(". Recommended form: ");
        spoken.push_str(&terminal.recommended_forms.join(", "));
    }
    spoken.push_str(
        ". We are sending the full details, including the official link, by WhatsApp or SMS to this number.",
    );
    spoken
}

/// E8.F3.T4: "Implement terminal-outcome rendering as a spoken summary +
/// SMS/WhatsApp follow-up with the deep link (a phone call can't 'click' a
/// link — design the handoff explicitly)." Reuses `bot-whatsapp`'s
/// already-built Cloud API client rather than introducing a second
/// messaging integration; if no `WhatsAppClient` is configured for this
/// deployment (or Exotel didn't supply `CallFrom`), the handoff is
/// skipped and logged — never silently claimed as done.
async fn handle_terminal_handoff(state: &AppState, caller_number: Option<&str>, terminal: &RenderableTerminal) {
    let (Some(client), Some(caller_number)) = (state.whatsapp_handoff.as_ref(), caller_number) else {
        tracing::info!(
            whatsapp_configured = state.whatsapp_handoff.is_some(),
            caller_number_present = caller_number.is_some(),
            "skipping post-call WhatsApp handoff"
        );
        return;
    };

    let message = bot_whatsapp::render_whatsapp::render_terminal(terminal);
    let normalized = normalize_to_whatsapp_format(caller_number);

    if let Err(err) = client.send_text(&normalized, &message).await {
        tracing::error!(error = %err, "failed to send post-call WhatsApp follow-up");
    }
}

/// Best-effort normalization only: Exotel's exact `CallFrom` format
/// (whether it includes a leading `+`, a `0` trunk prefix, etc.) was not
/// confirmed against a live account in this environment — see this
/// module's top-level doc comment. Strips a leading `+` since Meta's
/// Cloud API expects a bare country-code-prefixed number.
fn normalize_to_whatsapp_format(exotel_number: &str) -> String {
    exotel_number.trim_start_matches('+').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_a_leading_plus() {
        assert_eq!(normalize_to_whatsapp_format("+919000000000"), "919000000000");
        assert_eq!(normalize_to_whatsapp_format("919000000000"), "919000000000");
    }
}
