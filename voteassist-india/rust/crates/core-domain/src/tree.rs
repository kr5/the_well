use std::sync::OnceLock;

use crate::types::DecisionTree;

/// VoteAssist India — MVP decision tree (v1), ported verbatim from
/// `packages/decision-engine/src/tree.ts` (the working, tested TypeScript
/// prototype) so the Rust and TypeScript implementations stay in lockstep
/// during the migration window described in the PRD (Section 6.17).
///
/// Scope note: this v1 tree covers the highest-frequency scenarios described
/// in docs/04-decision-tree-spec.md. It is NOT yet the exhaustive v2 tree
/// specified in docs/PRD-V2-RUST-PLATFORM.md Section 10 — porting that
/// expansion is a separate, tracked follow-up (see docs/19-roadmap.md and
/// the PRD's EPIC 2, Feature 2.3).
///
/// Every terminal node MUST carry at least one citation (resolved against
/// knowledge-base/sources/*.json via the sibling `kb-content` crate) and at
/// least one official deep link — enforced by `validate_tree()` in
/// `engine.rs` and by the test suite in `tests/`.
const TREE_JSON: &str = include_str!("tree_v1.json");

pub fn vote_assist_tree_v1() -> &'static DecisionTree {
    static TREE: OnceLock<DecisionTree> = OnceLock::new();
    TREE.get_or_init(|| {
        serde_json::from_str(TREE_JSON)
            .expect("tree_v1.json must deserialize into a valid DecisionTree — this is a build-time invariant, not a runtime possibility")
    })
}
