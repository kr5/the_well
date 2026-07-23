//! Public decision-engine sessions are deliberately NOT persisted server-side
//! (no database row, no Redis key) — per docs/PRD-V2-RUST-PLATFORM.md
//! Section 8's explicit recommendation to keep the anonymous, no-account
//! path as close to zero-data-footprint as possible. Instead, the entire
//! `EngineState` travels with the client as an opaque, base64-encoded token
//! that the client echoes back on every subsequent request.
//!
//! This token carries no privilege and no personal data beyond the user's
//! own in-progress answers (which the user's own browser/bot session already
//! has), so it is not cryptographically signed: tampering with it can only
//! make the client's own decision-tree navigation behave oddly, never
//! disclose or authorize anything belonging to a different user.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use core_domain::EngineState;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateTokenError {
    #[error("state token is not valid base64")]
    InvalidBase64,
    #[error("state token does not decode into valid session state")]
    InvalidJson,
}

pub fn encode(state: &EngineState) -> String {
    let json = serde_json::to_vec(state).expect("EngineState always serializes");
    URL_SAFE_NO_PAD.encode(json)
}

pub fn decode(token: &str) -> Result<EngineState, StateTokenError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|_| StateTokenError::InvalidBase64)?;
    serde_json::from_slice(&bytes).map_err(|_| StateTokenError::InvalidJson)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_domain::{create_session, vote_assist_tree_v1};

    #[test]
    fn round_trips_a_fresh_session() {
        let tree = vote_assist_tree_v1();
        let state = create_session(tree).unwrap();
        let token = encode(&state);
        let decoded = decode(&token).unwrap();
        assert_eq!(decoded.current_node_id, state.current_node_id);
        assert_eq!(decoded.history, state.history);
    }

    #[test]
    fn rejects_garbage_input() {
        assert!(decode("not-valid-base64!!!").is_err());
        assert!(
            decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"not json")).is_err()
        );
    }
}
