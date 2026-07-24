-- Knowledge base: curated content, its citations, and its full audit trail.
-- docs/PRD-V2-RUST-PLATFORM.md Section 8/9.

CREATE TABLE knowledge_entries (
    id TEXT PRIMARY KEY CHECK (id ~ '^[a-z0-9-]+$'),
    topic kb_topic NOT NULL,
    title TEXT NOT NULL,
    summary TEXT NOT NULL CHECK (char_length(summary) <= 500),
    body TEXT NOT NULL DEFAULT '',
    applicable_states TEXT[] NOT NULL DEFAULT ARRAY['all'],
    source_type kb_source_type NOT NULL,
    last_verified_date DATE NOT NULL,
    version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1),
    related_forms kb_related_form[] NOT NULL DEFAULT '{}',
    related_entities TEXT[] NOT NULL DEFAULT '{}',
    language TEXT NOT NULL DEFAULT 'en',
    review_status kb_review_status NOT NULL DEFAULT 'draft',
    caution TEXT,
    created_by UUID, -- FK to admin_users, added in 0006 once that table exists
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER trg_knowledge_entries_updated_at
    BEFORE UPDATE ON knowledge_entries
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_knowledge_entries_topic ON knowledge_entries (topic);
CREATE INDEX idx_knowledge_entries_review_status ON knowledge_entries (review_status);
CREATE INDEX idx_knowledge_entries_last_verified_date ON knowledge_entries (last_verified_date);
CREATE INDEX idx_knowledge_entries_applicable_states ON knowledge_entries USING gin (applicable_states);
CREATE INDEX idx_knowledge_entries_related_entities ON knowledge_entries USING gin (related_entities);

COMMENT ON TABLE knowledge_entries IS
    'Curated, cited knowledge-base entries. This is the database-of-record '
    'mirror of knowledge-base/sources/*.json — see kb-content crate for the '
    'file<->row reconciliation strategy during the content-authoring '
    'migration described in PRD v2 Section 9.';
COMMENT ON COLUMN knowledge_entries.last_verified_date IS
    'Drives the nightly re-verification-due digest job (crates/jobs): once '
    'this ages past the configured threshold (default 180 days), the row is '
    'auto-flagged review_status = needs_reverification.';

CREATE TABLE knowledge_entry_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entry_id TEXT NOT NULL REFERENCES knowledge_entries (id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    url TEXT NOT NULL,
    publisher TEXT,
    display_order INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_knowledge_entry_sources_entry_id ON knowledge_entry_sources (entry_id);

-- Append-only audit trail: every save to knowledge_entries creates a row
-- here first (in the same transaction), never mutated afterward. This is
-- the backbone the admin Content Editor's diff view and rollback feature
-- read from (PRD v2 Section 11 admin page 3).
CREATE TABLE knowledge_entry_revisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entry_id TEXT NOT NULL REFERENCES knowledge_entries (id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    snapshot JSONB NOT NULL, -- full entry state at this version, incl. sources
    review_status_at_time kb_review_status NOT NULL,
    changed_by UUID, -- FK to admin_users, added in 0006
    change_summary TEXT,
    changed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (entry_id, version)
);

CREATE INDEX idx_knowledge_entry_revisions_entry_id ON knowledge_entry_revisions (entry_id, version DESC);

COMMENT ON TABLE knowledge_entry_revisions IS
    'Append-only. Retained indefinitely — this is content history, not '
    'personal data, so DPDP retention-minimization limits do not apply the '
    'same way they do to analytics_events or feedback (PRD v3 Section V9).';
