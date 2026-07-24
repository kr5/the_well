-- Citation link-health tracking and translation-completeness tracking.
-- docs/PRD-V2-RUST-PLATFORM.md Section 11 admin pages 5/6; PRD v3 Section V2.

CREATE TABLE link_check_results (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    source_id UUID NOT NULL REFERENCES knowledge_entry_sources (id) ON DELETE CASCADE,
    source_url TEXT NOT NULL,
    http_status INTEGER,
    checked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    error_message TEXT -- populated on connection failure/timeout rather than a clean HTTP status
);

CREATE INDEX idx_link_check_results_source_id ON link_check_results (source_id, checked_at DESC);

COMMENT ON TABLE link_check_results IS
    'Written by the nightly courteous change-detection job (crates/jobs): '
    'HEAD-requests each cited URL, rate-limited and identified by a '
    'distinct user-agent string per docs/SECURITY-AND-SRE-OPERATIONS.md''s '
    'courteous-crawling norm — checks a small, curated, human-picked set '
    'of citation URLs, never a crawl. Retained 12 months, then summarized.';

-- Per-locale, per-content-type translation completeness, populated by the
-- translation-completeness report job rather than computed live on every
-- admin dashboard page load (PRD v3 Section V5's stated preference for
-- query speed on that dashboard).
CREATE TABLE translation_status (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    locale TEXT NOT NULL,
    content_type TEXT NOT NULL CHECK (content_type IN ('kb_entry', 'tree_node', 'ui_string')),
    total_items INTEGER NOT NULL,
    translated_items INTEGER NOT NULL,
    reviewed_items INTEGER NOT NULL,
    computed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (locale, content_type, computed_at)
);

CREATE INDEX idx_translation_status_locale ON translation_status (locale, computed_at DESC);

COMMENT ON TABLE translation_status IS
    'One row per (locale, content_type) per computation run — the admin '
    'Translation Management page (PRD v2 admin page 5) reads the latest '
    'row per (locale, content_type) for its completeness dashboard. '
    'reviewed_items is always <= translated_items <= total_items; machine-'
    'translation output that has not cleared human review (PRD v3 Section '
    'V2''s Bhashini MT-assist discipline) counts toward translated_items '
    'but never reviewed_items.';
