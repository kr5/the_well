//! Per-chat dialogue state teloxide tracks internally (keyed by `ChatId`,
//! an unavoidable detail of how Telegram delivers updates). The only value
//! ever stored here is `session_token` — the same fully-random,
//! `core_domain`-independent token `channel_core::SessionStore` issues —
//! never the chat id itself, preserving the "must not reuse the
//! platform's own persistent chat/user ID as the analytics session_id"
//! rule from docs/PRD-V2-RUST-PLATFORM.md Section 8.7.

use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::Dialogue;

#[derive(Debug, Clone, Default)]
pub enum State {
    #[default]
    Idle,
    Walkthrough {
        session_token: String,
    },
}

pub type TelegramDialogue = Dialogue<State, InMemStorage<State>>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
