//! Pure, zero-I/O decision-engine domain logic for VoteAssist India.
//!
//! This crate has no knowledge of HTTP, databases, or any specific channel
//! (web/Telegram/WhatsApp/IVR) — see `docs/PRD-V2-RUST-PLATFORM.md` Section
//! 6.7 for why that separation is load-bearing. It is a direct, faithful
//! port of the TypeScript prototype at `packages/decision-engine`, which
//! remains the executable specification for this crate during migration
//! (Section 6.17).

pub mod engine;
pub mod tree;
pub mod types;

pub use engine::{
    answer, create_session, estimate_progress, get_current_node, is_session_complete,
    validate_tree, EngineError, TreeValidationIssue,
};
pub use tree::vote_assist_tree_v1;
pub use types::{
    Answer, Citation, DecisionNode, DecisionTree, DeepLink, EngineState, LocalizedText,
    QuestionNode, QuestionOption, TerminalNode,
};
