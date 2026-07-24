-- Admin/reviewer accounts (never public users — the anonymous decision-
-- engine flow never requires an account, per docs/PRD-V2-RUST-PLATFORM.md
-- Section 4's non-goals), their sessions, and the unified audit log.

CREATE TABLE admin_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL, -- argon2, per PRD v2 Section 6.10
    role admin_role NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login_at TIMESTAMPTZ
);

CREATE INDEX idx_admin_users_role ON admin_users (role) WHERE is_active;

COMMENT ON TABLE admin_users IS
    'Admin/reviewer accounts ONLY. Distinct from user_accounts (0007), '
    'which is the unrelated, strictly-optional public convenience layer — '
    'the two must use different least-privilege Postgres roles in '
    'production (docs/SECURITY-AND-SRE-OPERATIONS.md Section 3), since a '
    'compromised public-account credential store must never be a path to '
    'admin access.';

-- Now that admin_users exists, back-fill the deferred FKs from earlier
-- migrations (they were left as bare UUID columns until this point).
ALTER TABLE knowledge_entries
    ADD CONSTRAINT fk_knowledge_entries_created_by FOREIGN KEY (created_by) REFERENCES admin_users (id);
ALTER TABLE knowledge_entry_revisions
    ADD CONSTRAINT fk_knowledge_entry_revisions_changed_by FOREIGN KEY (changed_by) REFERENCES admin_users (id);
ALTER TABLE decision_trees
    ADD CONSTRAINT fk_decision_trees_published_by FOREIGN KEY (published_by) REFERENCES admin_users (id);
ALTER TABLE decision_tree_drafts
    ADD CONSTRAINT fk_decision_tree_drafts_owner FOREIGN KEY (owner_id) REFERENCES admin_users (id);
ALTER TABLE tracked_elections
    ADD CONSTRAINT fk_tracked_elections_entered_by FOREIGN KEY (entered_by) REFERENCES admin_users (id);
ALTER TABLE mcc_windows
    ADD CONSTRAINT fk_mcc_windows_entered_by FOREIGN KEY (entered_by) REFERENCES admin_users (id),
    ADD CONSTRAINT fk_mcc_windows_closed_by FOREIGN KEY (closed_by) REFERENCES admin_users (id);

-- Admin session storage (tower-sessions backend). Deliberately separate
-- from the public decision-engine's session model, which is NOT persisted
-- server-side at all (PRD v2 Section 8) — this table exists only because
-- admin/reviewer authentication genuinely needs server-side session state.
CREATE TABLE sessions (
    id TEXT PRIMARY KEY, -- tower-sessions session id
    admin_user_id UUID REFERENCES admin_users (id) ON DELETE CASCADE,
    data BYTEA NOT NULL,
    expiry_date TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_sessions_expiry_date ON sessions (expiry_date);
CREATE INDEX idx_sessions_admin_user_id ON sessions (admin_user_id);

-- Unified, structured audit log — every content edit, tree publish, role
-- change, MCC toggle, and fringe-case resolution writes exactly one row
-- here in the same transaction as the action itself, per the uniform
-- audit-log-entry format specified in
-- docs/PRD-V3-COMPREHENSIVE-EXPANSION.md Section V7. The Audit Log Viewer
-- (PRD v2 Section 11 admin page 11) renders a single timeline from this
-- table regardless of which subsystem produced the entry.
CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id UUID REFERENCES admin_users (id),
    action TEXT NOT NULL, -- e.g. 'kb_entry.updated', 'tree.published', 'mcc_window.closed'
    target_type TEXT NOT NULL, -- e.g. 'knowledge_entries', 'decision_trees', 'mcc_windows'
    target_id TEXT NOT NULL,
    before_value JSONB, -- sparse diff: only changed fields, per PRD v3 Section V7's audit-log format
    after_value JSONB,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_audit_log_target ON audit_log (target_type, target_id);
CREATE INDEX idx_audit_log_actor ON audit_log (actor_id);
CREATE INDEX idx_audit_log_occurred_at ON audit_log (occurred_at DESC);

COMMENT ON TABLE audit_log IS
    'Retained indefinitely for compliance, access-restricted to '
    'superadmin/legal_reviewer roles in the admin UI. Exportable for '
    'external compliance review (PRD v2 admin page 13).';
