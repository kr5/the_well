//! Shared, channel-agnostic plumbing for VoteAssist India's messaging/
//! voice adapters (`bot-telegram`, `bot-whatsapp`, `ivr-gateway`). Per
//! docs/PRD-V2-RUST-PLATFORM.md Section 6.7, `core-domain` stays zero-I/O
//! and channel-specific — this crate is the "thin shared crate" that
//! section allows for the parts of a `ChannelAdapter`-style architecture
//! that aren't decision logic but also shouldn't be copy-pasted three
//! times: rendering a decision node into locale-resolved plain data, the
//! ephemeral session store bot/IVR channels need (Section 8.7), and the
//! MCC broadcast gate every proactive send path must check (R-MCC-1).
//! Each adapter crate owns only its own channel-native rendering/parsing
//! and transport (teloxide, Meta Cloud API HTTP calls, Exotel webhooks).

pub mod mcc_gate;
pub mod render;
pub mod session_store;

pub use mcc_gate::{
    check_broadcast_allowed, is_any_mcc_active, is_mcc_active_for_state, BroadcastDecision,
};
pub use render::{
    render_node, RenderableDeepLink, RenderableNode, RenderableOption, RenderableQuestion,
    RenderableTerminal,
};
pub use session_store::{SessionStore, DEFAULT_SESSION_TTL};
