-- User feedback and the fringe-case registry (a decision-tree COVERAGE
-- gap tracker, distinct from ordinary feedback triage).
-- docs/PRD-V3-COMPREHENSIVE-EXPANSION.md Section V5.

CREATE TABLE feedback (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category TEXT NOT NULL,
    message TEXT NOT NULL,
    kb_entry_id TEXT REFERENCES knowledge_entries (id),
    decision_session_tree_version INTEGER, -- informational only; no session id is ever persisted (PRD v2 Section 8)
    contact_email TEXT, -- optional; only stored if the user provided it
    status feedback_status NOT NULL DEFAULT 'new',
    internal_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ
);

CREATE INDEX idx_feedback_status ON feedback (status);
CREATE INDEX idx_feedback_kb_entry_id ON feedback (kb_entry_id);

COMMENT ON TABLE feedback IS
    'Minimal PII by design: contact_email is optional and the only '
    'identifying field. Retention target: 180 days post-resolution for '
    'contact_email specifically (docs/SECURITY-AND-SRE-OPERATIONS.md '
    'Section 13 / PRD v3 Section V6.4''s pattern) — message/category '
    'persist as anonymized product-feedback history.';

CREATE TABLE fringe_case (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    description TEXT NOT NULL,
    reporter_channel fringe_case_channel NOT NULL,
    linked_feedback_id UUID REFERENCES feedback (id),
    status fringe_case_status NOT NULL DEFAULT 'new',
    linked_kb_entry_id TEXT REFERENCES knowledge_entries (id),
    linked_tree_node_id TEXT, -- node id within the active decision_trees artifact; not a hard FK since nodes live in JSONB
    resolution_note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ
);

CREATE INDEX idx_fringe_case_status ON fringe_case (status);
CREATE INDEX idx_fringe_case_linked_tree_node_id ON fringe_case (linked_tree_node_id);

COMMENT ON TABLE fringe_case IS
    'Tracks "the tree does not yet handle X" reports as a visible backlog, '
    'distinct from the general feedback inbox above. The Decision Tree '
    'Visual Editor (PRD v2 admin page 4) surfaces open fringe_case rows '
    'inline on the specific node they reference, per the cross-referencing '
    'requirement in PRD v3 Section V5 ("deeply coupled" admin UI).';
