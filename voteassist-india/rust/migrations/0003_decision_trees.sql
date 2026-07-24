-- Decision trees are stored as whole, validated JSON artifacts per
-- published version (never node-by-node relational rows) — the tree is
-- validated as a complete graph (acyclic, fully cited, fully reachable via
-- core_domain::validate_tree()) and partial relational edits would let it
-- sit in an invalid state between transactions. See
-- docs/PRD-V2-RUST-PLATFORM.md Section 8.3 for the full rationale.

CREATE TABLE decision_trees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tree_key TEXT NOT NULL, -- e.g. 'voteassist-core'
    version INTEGER NOT NULL,
    artifact JSONB NOT NULL, -- the full DecisionTree, matching core_domain::DecisionTree's serde shape
    is_active BOOLEAN NOT NULL DEFAULT false,
    validated_at TIMESTAMPTZ, -- set only after core_domain::validate_tree() returned zero issues
    published_by UUID, -- FK to admin_users, added in 0006; must hold legal_reviewer or superadmin
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tree_key, version)
);

-- Exactly one active version per tree_key at any time.
CREATE UNIQUE INDEX idx_decision_trees_one_active_per_key
    ON decision_trees (tree_key)
    WHERE is_active;

CREATE INDEX idx_decision_trees_tree_key_version ON decision_trees (tree_key, version DESC);

COMMENT ON TABLE decision_trees IS
    'Immutable, published tree versions. Never UPDATE an existing row after '
    'publish (only is_active flips as a new version takes over) — a new '
    'version is always a new row, giving admin page 4''s version history/'
    'rollback view a simple append-only sequence to render.';
COMMENT ON COLUMN decision_trees.validated_at IS
    'NULL means this artifact has not passed validate_tree() and must never '
    'be flipped is_active = true; the publish endpoint enforces this at the '
    'application layer, this column just makes the invariant visible in SQL.';

-- The node-level visual editor (PRD v2 Section 11 admin page 4) needs
-- finer-grained working state than "one big JSON blob" while a
-- contributor/reviewer is mid-edit. Drafts hold in-progress, NOT YET
-- validated edits; publishing reconciles a draft into a new, validated
-- decision_trees row (never mutates a published version in place).
CREATE TABLE decision_tree_drafts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tree_key TEXT NOT NULL,
    based_on_version INTEGER NOT NULL, -- the published decision_trees.version this draft started from
    artifact JSONB NOT NULL, -- working copy, same shape as decision_trees.artifact, possibly invalid mid-edit
    last_validation_issues JSONB NOT NULL DEFAULT '[]', -- cached Vec<TreeValidationIssue> from the last "Validate tree" click
    owner_id UUID, -- FK to admin_users, added in 0006
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER trg_decision_tree_drafts_updated_at
    BEFORE UPDATE ON decision_tree_drafts
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_decision_tree_drafts_tree_key ON decision_tree_drafts (tree_key);
CREATE INDEX idx_decision_tree_drafts_owner ON decision_tree_drafts (owner_id);
