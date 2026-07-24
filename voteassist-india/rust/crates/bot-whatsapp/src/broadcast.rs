//! The MCC-gated, 24-hour-window-aware proactive-broadcast send path, per
//! R-MCC-1 (docs/PRD-V2-RUST-PLATFORM.md Section 2.6, item 1) and E8.F2.T3.
//! Every bot-initiated push (as opposed to a reply to a citizen's own
//! in-progress conversation) must: (1) check the shared MCC gate, and (2)
//! respect Meta's customer-service window — a free-form message is only
//! allowed within 24 hours of the citizen's last inbound message; outside
//! that window, only a pre-approved template may be sent. This module
//! never silently substitutes one for the other: a template's Meta-
//! approved wording and a free-form message's wording are not
//! interchangeable content, so callers must supply both and let this
//! function pick correctly rather than guessing which one to write.
//!
//! Scope note, matching `bot-telegram::broadcast`'s: this is the gate-
//! compliant send primitive, ready for the admin-triggered broadcast
//! feature (PRD v2 admin page 9) to call once it exists.

use serde_json::Value;
use sqlx::PgPool;

use crate::client::{WhatsAppApiError, WhatsAppClient};
use crate::state::ConversationIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BroadcastOutcome {
    SentFreeForm,
    SentTemplate,
    SuppressedByMcc,
}

#[derive(Debug, thiserror::Error)]
pub enum BroadcastError {
    #[error("MCC gate check failed: {0}")]
    Gate(#[from] sqlx::Error),
    #[error("WhatsApp send failed: {0}")]
    Send(#[from] WhatsAppApiError),
}

pub struct ProactiveMessage<'a> {
    pub wa_id: &'a str,
    pub state_name: Option<&'a str>,
    pub free_form_text: &'a str,
    pub template_name: &'a str,
    pub template_language_code: &'a str,
    pub template_components: Vec<Value>,
}

/// Sends a proactive message, choosing free-form text if `message.wa_id`
/// is within Meta's 24-hour customer-service window, or the supplied
/// template otherwise — unless MCC-suppressed, which is checked first
/// regardless of window state.
pub async fn send_proactive_message(
    client: &WhatsAppClient,
    pool: &PgPool,
    conversations: &ConversationIndex,
    message: ProactiveMessage<'_>,
) -> Result<BroadcastOutcome, BroadcastError> {
    let decision = channel_core::check_broadcast_allowed(pool, message.state_name).await?;
    if decision == channel_core::BroadcastDecision::SuppressedByMcc {
        tracing::info!(
            wa_id = message.wa_id,
            state_name = message.state_name,
            "proactive WhatsApp broadcast suppressed: MCC window active"
        );
        return Ok(BroadcastOutcome::SuppressedByMcc);
    }

    if conversations.within_free_form_window(message.wa_id).await {
        client.send_text(message.wa_id, message.free_form_text).await?;
        Ok(BroadcastOutcome::SentFreeForm)
    } else {
        client
            .send_template(
                message.wa_id,
                message.template_name,
                message.template_language_code,
                message.template_components,
            )
            .await?;
        Ok(BroadcastOutcome::SentTemplate)
    }
}
