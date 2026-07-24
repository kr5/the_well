-- Optional, strictly opt-in public accounts and saved decision-tree
-- drafts. docs/PRD-V3-COMPREHENSIVE-EXPANSION.md Section V6.
--
-- Design test repeated here as a comment because it is the single most
-- important constraint on this table: does the anonymous, no-account path
-- still work exactly as it did without this migration? It must. Nothing
-- in the rest of the schema (knowledge_entries, decision_trees, the public
-- API) references user_accounts — the anonymous flow is structurally
-- incapable of depending on it.

CREATE TABLE user_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Store a salted hash of the contact identifier, not the plaintext
    -- email/phone (OTP delivery still needs the real address transiently,
    -- handled at the application layer, never persisted here in plaintext).
    contact_identifier_hash TEXT NOT NULL UNIQUE,
    contact_channel TEXT NOT NULL CHECK (contact_channel IN ('email', 'phone')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login_at TIMESTAMPTZ,
    notifications_opt_in BOOLEAN NOT NULL DEFAULT false,
    notification_state_interest UUID REFERENCES states (id) -- only meaningful if notifications_opt_in
);

COMMENT ON TABLE user_accounts IS
    'Exhaustive list of what this table may ever hold: a login credential, '
    'saved draft/session state, and notification preferences. NEVER '
    'Aadhaar/EPIC numbers, uploaded documents, or any sensitive-category '
    'data (PRD v3 Section V6.2). Deletion (DELETE FROM user_accounts) is '
    'single-action, immediate, and cascades to every saved_drafts and '
    'consent_artifacts row via ON DELETE CASCADE below — no soft-delete, '
    'no grace period in the live system (PRD v3 Section V6.4).';

CREATE TABLE saved_drafts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES user_accounts (id) ON DELETE CASCADE,
    label TEXT, -- user-chosen, optional, free text — never validated against real identity
    decision_tree_version INTEGER NOT NULL,
    answer_history JSONB NOT NULL, -- ordered [{questionId, value}], same shape as core_domain::Answer[]
    -- If the session reached a terminal, freeze the result AS IT WAS SHOWN
    -- at save time, so a later KB edit doesn't silently rewrite a saved
    -- checklist out from under a user (PRD v3 Section V6.3).
    frozen_terminal_snapshot JSONB,
    frozen_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER trg_saved_drafts_updated_at
    BEFORE UPDATE ON saved_drafts
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE INDEX idx_saved_drafts_account_id ON saved_drafts (account_id);

COMMENT ON COLUMN saved_drafts.answer_history IS
    'Along with frozen_terminal_snapshot, the most sensitive data this '
    'product stores — it can reveal a user''s living situation (hostel vs. '
    'family address, disability status, NRI/service-voter status). This '
    'column must be encrypted at rest at the column level (not just disk-'
    'level), a v1-roadmap hardening item, precisely because a breach here '
    'discloses "someone with this email chose the student-hostel path," '
    'never "citizen X, EPIC number Y, is disabled" (PRD v3 Section V6.6) — '
    'the account/entry separation is what keeps that true.';

-- Consent-artifact log: a DEPA/consent-manager-inspired (not integrated)
-- record of every consent grant/revocation, itself exportable/deletable
-- like everything else an account holds. PRD v3 Section V7.
CREATE TABLE consent_artifacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES user_accounts (id) ON DELETE CASCADE,
    purpose TEXT NOT NULL, -- e.g. 'election_reminders_karnataka'
    granted BOOLEAN NOT NULL, -- true = granted at this timestamp, false = revoked
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_consent_artifacts_account_id ON consent_artifacts (account_id);

COMMENT ON TABLE consent_artifacts IS
    'Append-only per account (revoking consent inserts a granted=false '
    'row rather than deleting the granted=true row) so the account '
    'settings page can render an honest history of what was consented to '
    'and when, not just current state.';
