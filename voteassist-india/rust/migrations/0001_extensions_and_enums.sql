-- VoteAssist India — schema migration 0001
-- Extensions and shared enum types used across later migrations.
-- See docs/PRD-V2-RUST-PLATFORM.md Section 8 and
-- docs/PRD-V3-COMPREHENSIVE-EXPANSION.md Sections V3/V6/V7/V8.

CREATE EXTENSION IF NOT EXISTS pgcrypto; -- gen_random_uuid()

-- ---------------------------------------------------------------------------
-- Knowledge base enums (mirror knowledge-base/schema/entry.schema.json,
-- kept in lockstep by rust/crates/kb-content's schema-conformance test).
-- ---------------------------------------------------------------------------

CREATE TYPE kb_topic AS ENUM (
    'registration',
    'correction',
    'shifting-of-residence',
    'deletion-objection',
    'epic',
    'ordinary-residence',
    'nri-voter',
    'service-voter',
    'pwd-voter',
    'qualifying-dates',
    'grievance',
    'polling-station',
    'roll-search',
    'glossary',
    -- New topics added for the v2/v3 decision-tree expansion
    -- (docs/PRD-V2-RUST-PLATFORM.md Section 10).
    'local-body-elections',
    'documents-alternatives',
    'transgender-elector'
);

CREATE TYPE kb_source_type AS ENUM (
    'eci_official',
    'state_ceo',
    'gazette_law',
    'sveep',
    'pib_release',
    'community_pending_verification'
);

CREATE TYPE kb_review_status AS ENUM (
    'draft',
    'in_review',
    'verified',
    'needs_reverification'
);

CREATE TYPE kb_related_form AS ENUM (
    'form-2',
    'form-6',
    'form-6a',
    'form-7',
    'form-8',
    'form-12d'
);

-- ---------------------------------------------------------------------------
-- Election/jurisdiction enums (docs/PRD-V3-COMPREHENSIVE-EXPANSION.md
-- Section V3).
-- ---------------------------------------------------------------------------

CREATE TYPE administering_body AS ENUM ('eci', 'state_election_commission');

CREATE TYPE election_type AS ENUM (
    'lok_sabha',
    'rajya_sabha',
    'legislative_assembly',
    'legislative_council',
    'president',
    'vice_president',
    'by_election',
    'panchayat_gram',
    'panchayat_block',
    'panchayat_zilla',
    'municipal_corporation',
    'municipal_council',
    'nagar_panchayat'
);

CREATE TYPE voteassist_scope AS ENUM (
    'full_guidance',
    'informational_only',
    'jurisdiction_routing_only'
);

-- ---------------------------------------------------------------------------
-- Admin RBAC enum (docs/PRD-V2-RUST-PLATFORM.md Section 11.0).
-- ---------------------------------------------------------------------------

CREATE TYPE admin_role AS ENUM (
    'contributor',
    'reviewer',
    'legal_reviewer',
    'translator',
    'analytics_viewer',
    'superadmin'
);

-- ---------------------------------------------------------------------------
-- Misc shared enums.
-- ---------------------------------------------------------------------------

CREATE TYPE client_platform AS ENUM ('web', 'telegram', 'whatsapp', 'ivr');

CREATE TYPE fringe_case_status AS ENUM (
    'new',
    'triaged',
    'added_to_tree',
    'wontfix',
    'needs_legal_review'
);

CREATE TYPE fringe_case_channel AS ENUM (
    'feedback_form',
    'admin_manual_entry',
    'support_channel',
    'analytics_signal'
);

CREATE TYPE feedback_status AS ENUM ('new', 'triaged', 'resolved', 'wontfix');

CREATE FUNCTION set_updated_at() RETURNS trigger AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION set_updated_at() IS
    'Generic BEFORE UPDATE trigger function that stamps updated_at = now(). '
    'Attached per-table in the migrations that define an updated_at column.';
