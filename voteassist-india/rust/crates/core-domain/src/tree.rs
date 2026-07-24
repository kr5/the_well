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
const TREE_V1_JSON: &str = include_str!("tree_v1.json");

pub fn vote_assist_tree_v1() -> &'static DecisionTree {
    static TREE: OnceLock<DecisionTree> = OnceLock::new();
    TREE.get_or_init(|| {
        serde_json::from_str(TREE_V1_JSON)
            .expect("tree_v1.json must deserialize into a valid DecisionTree — this is a build-time invariant, not a runtime possibility")
    })
}

/// VoteAssist India — expanded decision tree (v2), per
/// docs/PRD-V2-RUST-PLATFORM.md Section 10 and
/// docs/PRD-V3-COMPREHENSIVE-EXPANSION.md Sections V4/V5. Extends v1's
/// 21 nodes to 33, adding: interstate/intrastate move clarification with
/// an explicit government-transfer/service-voter branch, a combined
/// shift-and-correction terminal (e.g. for a married name change after
/// moving), a shared senior (85+) postal-ballot path distinct from PwD
/// marking, a documents-alternatives path for citizens lacking Aadhaar/
/// passport/driving licence, an honest "needs local verification" path
/// for tribal/remote/urban-slum address-proof questions, a duplicate-vs-
/// wrongfully-deleted-entry distinction, a shifted-polling-station-
/// without-a-move informational terminal, and a gender-marker-update
/// path that honestly flags itself as needing further legal/content
/// research rather than inventing specifics.
///
/// v1 is retained (not replaced) as the MVP-Rust-v1 parity target against
/// the TypeScript prototype; v2 is the forward-looking default for new
/// integrations once parity is confirmed (PRD v2 Section 21).
const TREE_V2_JSON: &str = include_str!("tree_v2.json");

pub fn vote_assist_tree_v2() -> &'static DecisionTree {
    static TREE: OnceLock<DecisionTree> = OnceLock::new();
    TREE.get_or_init(|| {
        serde_json::from_str(TREE_V2_JSON)
            .expect("tree_v2.json must deserialize into a valid DecisionTree — this is a build-time invariant, not a runtime possibility")
    })
}
