# Analytics Plan

## 1. Principles

- **Aggregate only.** No analytics feature reports or stores individual-
  level behavioral profiles. All reporting is over counts/rates aggregated
  across sessions, never a per-user timeline.
- **No political inference, ever, under any framing.** Not "which
  terminal outcomes correlate with which regions" if that could function
  as a proxy for political/demographic profiling beyond what's needed for
  UX improvement; regional aggregation is limited to what's needed to
  confirm the product works well across contexts (e.g., "are tribal/remote
  personas' scenarios completing successfully"), not to build any kind of
  targeting capability.
- **No cross-site tracking, no ad pixels, no third-party analytics SDKs**
  that phone home to an ad-tech vendor. VoteAssist India's analytics stack
  must be self-hosted or first-party-only (e.g., a first-party event
  pipeline into the product's own PostgreSQL analytics tables, or a
  privacy-respecting self-hosted tool), never a vendor that builds
  cross-site profiles.
- **Cookieless by default.** The anonymous `decision_sessions.id` (a
  random UUID generated per session, see `08-database-schema.md`) is the
  only session-correlation mechanism, held client-side only for the
  duration of the session, not set as a long-lived tracking cookie and not
  synced to any third party.
- **Data retention limits, enforced, not aspirational.** Raw
  `decision_answers` and `deep_link_clicks` rows are retained for a
  bounded period (recommended starting point: 90 days) sufficient to
  compute rolling metrics, after which they are deleted or rolled up into
  pre-aggregated, non-reversible summary tables. Exact retention period is
  a policy decision for the Compliance Owner (see
  `06-legal-compliance-review.md`) to set and publish in `/about/privacy`,
  not something this document finalizes unilaterally.

## 2. Metrics Tracked (all aggregate)

| Metric | Computation | Purpose |
|---|---|---|
| Task completion rate | terminal-reaching sessions / started sessions, per period | Overall flow health (see `01-prd.md`) |
| Time-to-clarity | median(`completed_at` - `started_at`) for completed sessions | Speed of resolution |
| Deep-link click-through rate | sessions with >=1 `deep_link_clicks` row / sessions reaching a terminal | Hand-off effectiveness |
| Per-node drop-off rate | sessions abandoned at node X / sessions that reached node X | Identifies confusing questions |
| Terminal outcome distribution | count of sessions per `terminal_node_id`, per period | Understand which scenarios are most common, to prioritize content maintenance |
| KB search success rate | searches followed by an entry view / total searches | Search relevance |
| Feedback volume by category | count of `feedback.category` per period | Feeds content review backlog |
| Language usage distribution | sessions per `language`, aggregate only | Prioritizes translation roadmap (`11-multilingual-strategy.md`) |

## 3. Explicitly Disallowed

- Any metric that segments users by inferred political leaning, caste,
  religion, or community.
- Any individual-level session replay or heatmap tooling that captures
  granular mouse/keystroke behavior tied to a persistent identifier.
- Any cross-device or cross-session identity stitching (no fingerprinting,
  no device IDs synced across sessions).
- Any sharing of raw or aggregate analytics data with third parties for
  purposes beyond VoteAssist India's own product improvement and public
  transparency reporting (section 5).
- Any A/B testing framework that could be used to differentially nudge
  users toward or away from registering, or toward a particular
  disposition — experimentation is limited to neutral UX clarity
  improvements (e.g., wording clarity, layout), reviewed under the same
  political-neutrality lens as any other content change.

## 4. Implementation Notes

- Event pipeline: `apps/web` emits events to `packages/api`, which writes
  to the `decision_sessions`/`decision_answers`/`deep_link_clicks` tables
  (`08-database-schema.md`); no client-side third-party analytics script
  tags at all in MVP.
- Aggregation jobs (scheduled, e.g., nightly) roll raw event tables into
  summary tables/materialized views feeding the metrics in section 2, and
  raw rows past the retention window are purged per section 1.
- Any dashboard consuming this data (internal or public, see section 5)
  reads only from aggregated summary tables, never raw per-session tables,
  as a structural safeguard against accidental re-identification.

## 5. Public Transparency Dashboard (Goal)

As a longer-term goal (tracked in `19-roadmap.md`), VoteAssist India should
publish a public, aggregate-only transparency dashboard showing metrics
like overall task completion rate, most common terminal outcomes, and
language usage distribution — both to build public trust in an
open-source civic tool and to model the kind of transparency VoteAssist
expects of the systems it's helping users navigate. This is explicitly a
goal, not an MVP commitment, and must be designed with the same
aggregate-only, no-re-identification-risk discipline as the internal
metrics above before it ships.
