-- Forms metadata, and aggregate geographic/jurisdictional reference data.
-- NEVER per-elector data — see docs/PRD-V2-RUST-PLATFORM.md Section 3's
-- explicit non-goal against scraping/mirroring the electoral roll.

CREATE TABLE forms (
    code TEXT PRIMARY KEY, -- 'form-6', 'form-6a', 'form-7', 'form-8', 'form-2', 'form-12d'
    title TEXT NOT NULL,
    summary TEXT NOT NULL,
    superseded_forms TEXT[] NOT NULL DEFAULT '{}' -- e.g. form-8 lists {'form-8a','form-001'}
);

INSERT INTO forms (code, title, summary, superseded_forms) VALUES
    ('form-6', 'New Elector Registration',
     'First-time registration as an elector, including a first registration triggered by moving to a new place of ordinary residence.',
     '{}'),
    ('form-6a', 'Overseas (NRI) Elector Registration',
     'Registration of Indian citizens living abroad, retaining Indian citizenship, against the address in their passport.',
     '{}'),
    ('form-7', 'Objection to Inclusion / Claim for Deletion',
     'Objecting to a name being included in the electoral roll, or seeking deletion of an entry.',
     '{}'),
    ('form-8', 'Shifting of Residence / Correction / EPIC Replacement / PwD Marking',
     'Single consolidated form (since the Registration of Electors (Amendment) Rules 2022) covering shifting of residence within or across Assembly Constituencies, correction of entries, EPIC replacement, and PwD marking.',
     ARRAY['form-8a', 'form-001']),
    ('form-2', 'Service Elector Registration',
     'Registration of members of the Armed Forces of the Union (or Army-Act-applicable forces) as service electors.',
     '{}'),
    ('form-12d', 'Postal Ballot / Home Voting Request',
     'Election-specific request for postal ballot or home voting, available to PwD electors and electors aged 85+, submitted within 5 days of that election''s notification.',
     '{}');

CREATE TABLE states (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    is_union_territory BOOLEAN NOT NULL DEFAULT false,
    iso_3166_2 TEXT -- e.g. 'IN-KA' for Karnataka, where known
);

CREATE TABLE assembly_constituencies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    state_id UUID NOT NULL REFERENCES states (id),
    number INTEGER NOT NULL,
    name TEXT NOT NULL,
    UNIQUE (state_id, number)
);

CREATE TABLE parliamentary_constituencies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    state_id UUID NOT NULL REFERENCES states (id),
    number INTEGER NOT NULL,
    name TEXT NOT NULL,
    UNIQUE (state_id, number)
);

CREATE INDEX idx_assembly_constituencies_state ON assembly_constituencies (state_id);
CREATE INDEX idx_parliamentary_constituencies_state ON parliamentary_constituencies (state_id);
CREATE INDEX idx_assembly_constituencies_name_trgm ON assembly_constituencies (name);
CREATE INDEX idx_parliamentary_constituencies_name_trgm ON parliamentary_constituencies (name);

COMMENT ON TABLE states IS
    'Reference data only, sourced from public ECI notifications / '
    'data.gov.in aggregate datasets. Seed data is intentionally NOT '
    'included in this migration (28 states + 8 UTs, each requiring a '
    'verified source citation per PRD v2 Section 10) — see the xtask '
    'seed-dev-db command and the per-state content-verification checklist '
    'in PRD v3 Section V3.5 / Section V8 EPIC 22.';

-- ---------------------------------------------------------------------------
-- Election jurisdiction model (docs/PRD-V3-COMPREHENSIVE-EXPANSION.md
-- Section V3): which body administers which election type, and per-state
-- State Election Commission reference data for Panchayat/Municipal
-- elections, which ECI has no jurisdiction over.
-- ---------------------------------------------------------------------------

CREATE TABLE election_type_jurisdiction (
    election_type election_type PRIMARY KEY,
    administering_body administering_body NOT NULL,
    is_direct_citizen_vote BOOLEAN NOT NULL,
    voteassist_scope voteassist_scope NOT NULL
);

INSERT INTO election_type_jurisdiction (election_type, administering_body, is_direct_citizen_vote, voteassist_scope) VALUES
    ('lok_sabha', 'eci', true, 'full_guidance'),
    ('rajya_sabha', 'eci', false, 'informational_only'),
    ('legislative_assembly', 'eci', true, 'full_guidance'),
    ('legislative_council', 'eci', false, 'informational_only'),
    ('president', 'eci', false, 'informational_only'),
    ('vice_president', 'eci', false, 'informational_only'),
    ('by_election', 'eci', true, 'full_guidance'),
    ('panchayat_gram', 'state_election_commission', true, 'jurisdiction_routing_only'),
    ('panchayat_block', 'state_election_commission', true, 'jurisdiction_routing_only'),
    ('panchayat_zilla', 'state_election_commission', true, 'jurisdiction_routing_only'),
    ('municipal_corporation', 'state_election_commission', true, 'jurisdiction_routing_only'),
    ('municipal_council', 'state_election_commission', true, 'jurisdiction_routing_only'),
    ('nagar_panchayat', 'state_election_commission', true, 'jurisdiction_routing_only');

COMMENT ON TABLE election_type_jurisdiction IS
    'Reference data reflecting constitutional fact (Article 324 for ECI, '
    'Article 243K/243ZA for State Election Commissions) — changes only on '
    'a constitutional amendment or fresh ruling, an exceptionally rare '
    'event. Kept as data, not a hardcoded Rust match statement, per PRD v3 '
    'Section V3.3, so the mapping is auditable and admin-visible.';

CREATE TABLE state_election_commissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    state_id UUID NOT NULL REFERENCES states (id),
    official_name TEXT NOT NULL,
    portal_url TEXT NOT NULL,
    -- Whether this state's SEC maintains a distinct local-body electoral
    -- roll from the ECI/ERONET chain, or derives it from the ECI roll.
    -- Genuinely varies by state law; NULL until individually verified —
    -- see PRD v3 Section V3.5's per-state verification discipline.
    roll_derivation_note TEXT,
    roll_derivation_verified_date DATE,
    last_verified_date DATE NOT NULL,
    CONSTRAINT uq_sec_per_state UNIQUE (state_id)
);

CREATE INDEX idx_state_election_commissions_state ON state_election_commissions (state_id);
