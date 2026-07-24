-- Privacy-preserving analytics event pipeline.
-- docs/PRD-V2-RUST-PLATFORM.md Section 12.
--
-- Non-negotiables encoded directly into this schema: no cookies, no
-- cross-site tracking, no IP storage (rate-limiting uses a short-TTL
-- in-memory/Redis cache, never this table), no free-text search queries,
-- no political-inclination data (there is none to collect, by design).

CREATE TYPE analytics_event_type AS ENUM (
    'session_started',
    'question_answered',
    'terminal_reached',
    'deep_link_clicked',
    'kb_search_performed',
    'kb_entry_viewed'
);

CREATE TABLE analytics_events (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    event_type analytics_event_type NOT NULL,
    decision_tree_version INTEGER,
    node_id TEXT, -- the question/terminal node this event concerns, where applicable
    answer_option TEXT, -- the OPTION VALUE chosen (e.g. 'student_hostel'), NEVER free text
    -- Ephemeral, rotated per session — no server-side link to any prior
    -- session, no persistent device fingerprint (PRD v2 Section 12).
    session_id UUID NOT NULL,
    locale TEXT NOT NULL,
    client_platform client_platform NOT NULL,
    deep_link_clicked BOOLEAN,
    -- kb_search_performed events store a hashed/bucketed query CATEGORY,
    -- never the raw query text, since a user's own search terms about
    -- their situation could themselves be identifying/sensitive.
    kb_search_category_hash TEXT,
    -- Hour-bucketed, NOT full-precision, to prevent re-identification via
    -- timing correlation (PRD v2 Section 12).
    occurred_at_hour TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_analytics_events_type_hour ON analytics_events (event_type, occurred_at_hour);
CREATE INDEX idx_analytics_events_node_id ON analytics_events (node_id) WHERE node_id IS NOT NULL;
CREATE INDEX idx_analytics_events_tree_version ON analytics_events (decision_tree_version);

COMMENT ON TABLE analytics_events IS
    'Raw events, retained 30 days post-event and then purged once rolled '
    'up into analytics_rollups_hourly/daily (a hard requirement, not an '
    'aspiration — enforced by a crates/jobs cron task). session_id here is '
    'NOT the same value as the public API''s opaque state token; it is a '
    'separate, purpose-built ephemeral identifier generated only for '
    'within-session funnel correlation and discarded with the rest of the '
    'row after 30 days.';

CREATE TABLE analytics_rollups_hourly (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    bucket_hour TIMESTAMPTZ NOT NULL,
    decision_tree_version INTEGER,
    node_id TEXT,
    event_type analytics_event_type NOT NULL,
    locale TEXT,
    client_platform client_platform,
    event_count BIGINT NOT NULL,
    UNIQUE (bucket_hour, decision_tree_version, node_id, event_type, locale, client_platform)
);

CREATE INDEX idx_analytics_rollups_hourly_bucket ON analytics_rollups_hourly (bucket_hour);

CREATE TABLE analytics_rollups_daily (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    bucket_day DATE NOT NULL,
    decision_tree_version INTEGER,
    node_id TEXT,
    event_type analytics_event_type NOT NULL,
    locale TEXT,
    client_platform client_platform,
    event_count BIGINT NOT NULL,
    UNIQUE (bucket_day, decision_tree_version, node_id, event_type, locale, client_platform)
);

CREATE INDEX idx_analytics_rollups_daily_bucket ON analytics_rollups_daily (bucket_day);

COMMENT ON TABLE analytics_rollups_hourly IS
    'Fully anonymized aggregates, retained indefinitely (no re-'
    'identification surface at this granularity) — populated by an '
    'hourly crates/jobs task, per docs/PRD-V2-RUST-PLATFORM.md Section '
    '6.8/Section 12. Retained roughly 13 months for seasonality '
    'comparison; analytics_rollups_daily is retained indefinitely as the '
    'long-horizon aggregate.';
