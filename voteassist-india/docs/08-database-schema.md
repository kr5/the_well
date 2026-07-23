# Database Schema (PostgreSQL, proposed)

This schema supports the MVP web app. It is deliberately minimal: no
official identity documents are ever stored, no political data is ever
collected, and personally identifying fields are optional and isolated
from analytics data. See `16-security-threat-model.md` for the associated
data classification table and `18-analytics-plan.md` for retention limits.

## 1. Design Principles

1. **No document custody.** There is no table, column, or blob storage
   path in this schema for uploaded ID documents, photos of documents, or
   scans. If a future feature genuinely requires temporary document
   handling, it must be added as a separate, explicitly consent-gated
   subsystem with field-level encryption and a mandatory TTL — not bolted
   onto these tables.
2. **Pseudonymous by default.** `users` (only needed for optional features
   like saved language preference across devices, or feedback follow-up)
   never stores name, EPIC number, Aadhaar number, or address. An email is
   optional and used only for the specific feature that required it
   (e.g., "email me this checklist").
3. **Analytics tables carry no PII.** `decision_sessions` and
   `decision_answers` are keyed by a random session UUID with no join path
   back to `users` unless the user explicitly opted into an account
   feature, and even then, decision content itself (answers) should not
   include free text.
4. **Everything cites something.** `knowledge_base_entries`, `forms`, and
   decision-tree terminal outcomes all foreign-key to `citations`.

## 2. Schema

```sql
-- ============================================================
-- Reference / geography
-- ============================================================

CREATE TABLE states (
    id              SERIAL PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,      -- e.g. 'Maharashtra'
    code            TEXT NOT NULL UNIQUE,      -- e.g. 'MH' (ISO/ECI convention - verify canonical source)
    ceo_portal_url  TEXT,                      -- state CEO portal, nullable until content team fills in
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE constituencies (
    id              SERIAL PRIMARY KEY,
    state_id        INTEGER NOT NULL REFERENCES states(id),
    type            TEXT NOT NULL CHECK (type IN ('AC', 'PC')), -- Assembly or Parliamentary Constituency
    number          INTEGER,                   -- official constituency number, if applicable
    name            TEXT NOT NULL,
    reserved_status TEXT CHECK (reserved_status IN ('none', 'SC', 'ST')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (state_id, type, number)
);

-- ============================================================
-- Citations & knowledge base
-- ============================================================

CREATE TABLE citations (
    id              SERIAL PRIMARY KEY,
    source_type     TEXT NOT NULL CHECK (source_type IN
                       ('eci_official', 'state_ceo', 'gazette_law',
                        'sveep', 'pib', 'community_pending_verification')),
    title           TEXT NOT NULL,
    url             TEXT NOT NULL,
    published_date  DATE,
    last_checked_at TIMESTAMPTZ,               -- last time a human confirmed this URL/content still matches
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE forms (
    id                  SERIAL PRIMARY KEY,
    code                TEXT NOT NULL UNIQUE,   -- 'FORM_6', 'FORM_6A', 'FORM_7', 'FORM_8', 'FORM_12D'
    name                TEXT NOT NULL,          -- plain-language name
    official_form_number TEXT NOT NULL,         -- '6', '6A', '7', '8', '12D'
    purpose_summary     TEXT NOT NULL,
    superseded_forms    TEXT[],                 -- e.g. ARRAY['8A','001'] for Form 8
    primary_citation_id INTEGER REFERENCES citations(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE knowledge_base_entries (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug                TEXT NOT NULL UNIQUE,
    topic               TEXT NOT NULL,          -- see 09-knowledge-base-schema.md for controlled vocabulary
    title               TEXT NOT NULL,
    summary             TEXT NOT NULL,
    body_markdown       TEXT NOT NULL,
    applicable_states   INTEGER[],              -- state IDs, or NULL/empty meaning "all"
    source_type         TEXT NOT NULL CHECK (source_type IN
                           ('eci_official', 'state_ceo', 'gazette_law',
                            'sveep', 'community_pending_verification')),
    primary_citation_id INTEGER REFERENCES citations(id),
    last_verified_date  DATE NOT NULL,
    version             INTEGER NOT NULL DEFAULT 1,
    review_status       TEXT NOT NULL CHECK (review_status IN
                           ('draft', 'in_review', 'published', 'needs_reverification', 'retired')),
    language            TEXT NOT NULL DEFAULT 'en',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE knowledge_base_entry_forms (
    kb_entry_id UUID NOT NULL REFERENCES knowledge_base_entries(id) ON DELETE CASCADE,
    form_id     INTEGER NOT NULL REFERENCES forms(id),
    PRIMARY KEY (kb_entry_id, form_id)
);

-- ============================================================
-- Decision engine session logs (analytics only, no PII)
-- ============================================================

CREATE TABLE decision_tree_versions (
    id            SERIAL PRIMARY KEY,
    version_tag   TEXT NOT NULL UNIQUE,        -- e.g. 'v1.0.0', matches packages/decision-engine release
    published_at  TIMESTAMPTZ NOT NULL,
    changelog     TEXT
);

CREATE TABLE decision_sessions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tree_version_id     INTEGER NOT NULL REFERENCES decision_tree_versions(id),
    started_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at        TIMESTAMPTZ,
    terminal_node_id    TEXT,                  -- identifier of the terminal node reached, if any
    language            TEXT NOT NULL DEFAULT 'en',
    -- Deliberately no user_id FK by default; no IP storage beyond what's
    -- needed transiently for abuse prevention (see 16-security-threat-model.md);
    -- no free-text fields that could accidentally capture PII.
    client_platform     TEXT CHECK (client_platform IN ('web')), -- extend when other channels ship
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE decision_answers (
    id            BIGSERIAL PRIMARY KEY,
    session_id    UUID NOT NULL REFERENCES decision_sessions(id) ON DELETE CASCADE,
    node_id       TEXT NOT NULL,               -- decision-engine question node identifier
    answer_value  TEXT NOT NULL,               -- must be a value from a fixed enum per node, never free text
    answered_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE deep_link_clicks (
    id            BIGSERIAL PRIMARY KEY,
    session_id    UUID REFERENCES decision_sessions(id) ON DELETE SET NULL,
    target        TEXT NOT NULL,               -- e.g. 'voters.eci.gov.in', 'ecinet.eci.gov.in', 'voter_helpline_app', 'ceo_portal', '1950_helpline'
    clicked_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================
-- Users (optional, minimal, pseudonymous)
-- ============================================================

CREATE TABLE users (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email               TEXT UNIQUE,           -- optional; only set if user opts into an email-dependent feature
    preferred_language  TEXT NOT NULL DEFAULT 'en',
    accessibility_prefs  JSONB,                 -- e.g. {"text_size": "large", "reduced_motion": true}
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at          TIMESTAMPTZ            -- soft-delete to support right-to-erasure workflow with audit trail
    -- Explicitly absent, and must never be added: name, EPIC number,
    -- Aadhaar number, phone number, address, caste, religion, party
    -- affiliation, or any political-opinion field.
);

-- ============================================================
-- Feedback
-- ============================================================

CREATE TABLE feedback (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kb_entry_id         UUID REFERENCES knowledge_base_entries(id),
    decision_session_id UUID REFERENCES decision_sessions(id),
    category            TEXT NOT NULL CHECK (category IN
                           ('inaccurate_info', 'confusing', 'broken_link', 'general', 'other')),
    message             TEXT NOT NULL,
    contact_email       TEXT,                  -- optional, only if user wants a reply
    status              TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'triaged', 'resolved', 'wontfix')),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at         TIMESTAMPTZ
);

-- ============================================================
-- Indexes (representative, not exhaustive)
-- ============================================================

CREATE INDEX idx_kb_entries_topic ON knowledge_base_entries(topic);
CREATE INDEX idx_kb_entries_language ON knowledge_base_entries(language);
CREATE INDEX idx_kb_entries_review_status ON knowledge_base_entries(review_status);
CREATE INDEX idx_decision_sessions_terminal ON decision_sessions(terminal_node_id);
CREATE INDEX idx_decision_answers_session ON decision_answers(session_id);
CREATE INDEX idx_feedback_status ON feedback(status);
```

## 3. Notes on Sensitive Data Handling

- No column in this schema stores an official document, a document image,
  or a document number (EPIC number, Aadhaar number, passport number).
  This is a deliberate design constraint, not an oversight — see
  `01-prd.md` non-goals and `06-legal-compliance-review.md`.
- If a future, explicitly-scoped feature requires temporary document
  handling (e.g., client-side-only OCR assistance that never leaves the
  device, or a narrowly-scoped, separately-reviewed server-side feature),
  it must live in its own table with: field-level encryption at rest, a
  mandatory `expires_at` TTL enforced by a scheduled purge job, and a
  `consent_given_at` timestamp tied to a specific, dated consent-text
  version. No such table exists in this MVP schema.
- `decision_answers.answer_value` is constrained (at the application
  layer, and ideally via a lookup/enum table in a later revision) to the
  fixed set of options defined per node in `packages/decision-engine`, to
  prevent accidental free-text PII capture.
- Retention limits for `decision_sessions`, `decision_answers`, and
  `deep_link_clicks` are defined in `18-analytics-plan.md`, not in this
  schema file, since retention policy may change independently of table
  structure.
