//! Telegram channel adapter for VoteAssist India. Renders the identical,
//! identically-tested `core-domain` decision engine as an inline-keyboard
//! Telegram conversation. Contains no decision logic of its own — see
//! docs/PRD-V2-RUST-PLATFORM.md Section 6.7.

pub mod broadcast;
pub mod commands;
pub mod handlers;
pub mod locale;
pub mod render_telegram;
pub mod state;
