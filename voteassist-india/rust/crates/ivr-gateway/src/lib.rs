//! Scaffolded IVR (voice/telephony) channel adapter for VoteAssist India,
//! targeting Exotel. Per docs/PRD-V2-RUST-PLATFORM.md Section 6.7: the
//! interface is defined and a real, working vertical slice (spoken
//! question-and-answer over DTMF, terminal outcome with a WhatsApp
//! deep-link handoff) exists, but this is not fully wired against a live
//! Exotel account — full implementation is v2/v3 (E8.F3). See
//! `webhook`'s module doc for exactly what is and isn't confirmed.

pub mod render_ivr;
pub mod state;
pub mod webhook;
