//! WhatsApp channel adapter for VoteAssist India: a thin `reqwest`
//! wrapper over the Meta WhatsApp Cloud API plus an Axum webhook
//! receiver, rendering the identical, identically-tested `core-domain`
//! decision engine as interactive list/button messages. Contains no
//! decision logic of its own — see docs/PRD-V2-RUST-PLATFORM.md Section
//! 6.7.

pub mod broadcast;
pub mod client;
pub mod render_whatsapp;
pub mod signature;
pub mod state;
pub mod webhook;
