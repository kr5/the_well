-- Election calendar tracking and MCC (Model Code of Conduct) windows.
-- docs/PRD-V3-COMPREHENSIVE-EXPANSION.md Section V3.6 and PRD v2 admin
-- page 8. Admin-curated content, deliberately NOT a live feed from an
-- external API (none exists for India comparable to Democracy Works'
-- Elections API for the US, and even that ecosystem's Google Civic
-- Information Representatives-lookup endpoint was sunset April 2025 —
-- see docs/PRD-V3-COMPREHENSIVE-EXPANSION.md Section V2's caution against
-- hard external-API dependencies for this data).

CREATE TABLE tracked_elections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    election_type election_type NOT NULL,
    state_id UUID REFERENCES states (id), -- NULL for a Lok Sabha general election spanning all states
    constituency_ids UUID[] NOT NULL DEFAULT '{}', -- specific ACs/PCs for a by-election; empty for a general election
    schedule_announced_date DATE,
    nomination_start_date DATE,
    nomination_end_date DATE,
    polling_date DATE,
    counting_date DATE,
    source_notification_url TEXT NOT NULL, -- the Gazette/PIB/ECI or SEC press release this was entered from
    entered_by UUID, -- FK to admin_users, added in 0006
    entered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_verified_date DATE NOT NULL
);

CREATE INDEX idx_tracked_elections_state ON tracked_elections (state_id);
CREATE INDEX idx_tracked_elections_polling_date ON tracked_elections (polling_date);
CREATE INDEX idx_tracked_elections_type ON tracked_elections (election_type);

COMMENT ON TABLE tracked_elections IS
    'Feeds: (a) the MCC Control Panel (mcc_windows, below), (b) the public '
    '"is there an election coming up where I live" notice, (c) opt-in '
    'reminder notifications for accounts with a state of interest '
    '(PRD v3 Section V6.2). Semi-automated ingestion assist (reading ECI/'
    'SEC press-release feeds and proposing a draft row) is a v2/v3 roadmap '
    'item — it must always produce a row for human confirmation, never '
    'auto-publish, matching every other content pipeline in this system.';

-- Model Code of Conduct windows: a safety-critical control surface
-- (PRD v2 admin page 8). Deliberately a separate table from
-- tracked_elections even though they are usually 1:1, because an MCC
-- window's start can be entered the moment ECI announces a schedule while
-- the rest of tracked_elections' dates are still being filled in, and
-- because closing an MCC window is always an explicit, separately-audited
-- admin action distinct from any other election-record edit.
CREATE TABLE mcc_windows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tracked_election_id UUID REFERENCES tracked_elections (id),
    state_id UUID NOT NULL REFERENCES states (id),
    window_start DATE NOT NULL, -- typically = schedule_announced_date
    window_end DATE, -- NULL until results are declared; ECI does not publish a fixed end date upfront
    is_active BOOLEAN NOT NULL GENERATED ALWAYS AS (window_end IS NULL) STORED,
    entered_by UUID, -- FK to admin_users, added in 0006
    entered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    closed_by UUID,
    closed_at TIMESTAMPTZ
);

CREATE INDEX idx_mcc_windows_state ON mcc_windows (state_id);
CREATE INDEX idx_mcc_windows_active ON mcc_windows (is_active) WHERE is_active;

COMMENT ON TABLE mcc_windows IS
    'While is_active, proactive bot broadcasts for that state are '
    'throttled and public/bot copy for that state carries an extra '
    'neutrality disclaimer (PRD v2 admin page 8). Closing a window '
    '(setting window_end) is always an explicit human action — there is '
    'no auto-expiry, since MCC "stays in force until results are '
    'declared," which is not a fixed calendar duration ECI publishes '
    'upfront.';
