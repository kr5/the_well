//! The MCC-gated proactive-broadcast send path, per E8.F1.T5
//! ("Wire the MCC kill-switch (E7.F3.T2) to pause/throttle proactive
//! Telegram broadcasts") and R-MCC-1 (docs/PRD-V2-RUST-PLATFORM.md
//! Section 2.6, item 1): every bot-initiated push — as opposed to a reply
//! to a citizen's own in-progress question — must check the shared MCC
//! gate (`channel_core::check_broadcast_allowed`) before firing.
//!
//! Scope note: this function is the gate-compliant send primitive, ready
//! for the admin-triggered broadcast feature (PRD v2 admin page 9, Bot
//! Channel Management) to call. That admin trigger — the UI/queue that
//! decides *which* citizens to message and *when* — is a v2 feature not
//! yet built, so this crate's `main.rs` does not invoke this function
//! today (and correspondingly does not require a `DATABASE_URL`/Postgres
//! connection just to run the reply-only walkthrough bot). Wiring the
//! gate now, ahead of the feature that will call it, means the broadcast
//! trigger — whenever it is built — has no path to a non-gated send.

use channel_core::BroadcastDecision;
use sqlx::PgPool;
use teloxide::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BroadcastOutcome {
    Sent,
    SuppressedByMcc,
}

/// Sends `text` to `chat_id` as a proactive (not reply-triggered) message,
/// unless an MCC window is active for `state_name` (or platform-wide, if
/// `state_name` is `None`), in which case the send is skipped and
/// `SuppressedByMcc` is returned instead of silently failing or silently
/// succeeding — callers (a future admin broadcast job) can log/report the
/// suppression rather than have it look identical to "sent".
pub async fn send_proactive_announcement(
    bot: &Bot,
    pool: &PgPool,
    chat_id: ChatId,
    state_name: Option<&str>,
    text: &str,
) -> Result<BroadcastOutcome, BroadcastError> {
    match channel_core::check_broadcast_allowed(pool, state_name).await? {
        BroadcastDecision::SuppressedByMcc => {
            tracing::info!(
                ?chat_id,
                state_name,
                "proactive Telegram broadcast suppressed: MCC window active"
            );
            Ok(BroadcastOutcome::SuppressedByMcc)
        }
        BroadcastDecision::Allowed => {
            bot.send_message(chat_id, text).await?;
            Ok(BroadcastOutcome::Sent)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BroadcastError {
    #[error("MCC gate check failed: {0}")]
    Gate(#[from] sqlx::Error),
    #[error("Telegram send failed: {0}")]
    Send(#[from] teloxide::RequestError),
}
