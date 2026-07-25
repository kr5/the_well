# admin-app

Admin/reviewer dashboard for VoteAssist India — a separate Leptos SSR
deploy target from `crates/web-app`, per docs/PRD-V2-RUST-PLATFORM.md
Section 11 and EPIC 7. Written without local compilation (see the
workspace-root `rust/README.md` for what that means here).

## Implemented — all 13 admin pages from PRD v2 Section 11

- **Auth**: email/password login, argon2 password hashing, a hand-rolled
  Postgres-backed session (see `src/auth/session.rs` for why this isn't
  built on the `tower-sessions` crate the PRD names), HttpOnly/Secure/
  SameSite=Strict cookie.
- **RBAC**: `require_admin`/`require_role` enforced at the top of every
  server function that touches real data (`src/server_fns.rs`) — the six
  roles from PRD v2 Section 11 (`contributor`, `reviewer`,
  `legal_reviewer`, `translator`, `analytics_viewer`, `superadmin`).
- **Dashboard** (page 2): live counts (feedback awaiting triage, entries
  needing re-verification, verified entries, active MCC windows).
- **Knowledge Base Content Editor** (page 3): list/filter, create/edit
  form, review-status state machine. Every save writes an append-only
  `knowledge_entry_revisions` snapshot and an `audit_log` row in the same
  transaction.
- **Decision Tree Visual Editor** (page 4, `src/pages/tree_editor.rs`): a
  structured JSON editor around `decision_tree_drafts`/`decision_trees`
  — not a drag-and-drop node-graph canvas, a disclosed, deliberate
  simplification (see that page's module doc). Save draft / "Validate
  tree" (reuses `core_domain::validate_tree`, never a second
  reimplementation of the structural checks) / Publish (re-validates,
  creates a new version, flips `is_active`, gated `legal_reviewer`+) /
  version history with rollback (flips `is_active` back, never
  duplicates a row) / cross-referenced open `fringe_case` reports.
  Starts a fresh draft seeded from `core_domain::vote_assist_tree_v1`/`v2`
  when nothing has ever been published for a given `tree_key` yet.
- **Translation Management** (page 5, `src/pages/translation.rs`):
  per-locale/content-type completeness dashboard reading
  `translation_status`, plus a "recompute now" button calling the same
  `jobs::compute_kb_translation_status` the nightly job uses. The
  string-by-string translation workbench itself is `rust/crates/xtask`'s
  `translate-kb-entry`/`translate-tree` CLI (Claude Haiku or NVIDIA NIM's
  free-tier Nemotron models) plus the human review workflow in
  `docs/20-translation-task-tracker.md` — disclosed as a separate,
  already-real workflow rather than duplicated inside this admin UI.
- **Citation & Link Health** (page 6, `src/pages/link_health.rs`): every
  cited source joined to its latest `link_check_results` row, plus a
  live "check now" button reusing `jobs::build_client()`'s courteously-
  identified HTTP client.
- **Feedback & Grievance Triage** (page 7, `src/pages/feedback.rs`):
  filter-by-status list over the `feedback` table (written by
  `web-app`'s `/feedback` page), inline status/internal-notes editor.
- **MCC / Election-Period Control Panel** (page 8): open/close an MCC
  window per state — the exact `mcc_windows` table
  `channel_core::check_broadcast_allowed` (used by `bot-telegram`'s and
  `bot-whatsapp`'s proactive-broadcast gates) reads from. Gated to
  `legal_reviewer`/`superadmin`.
- **Bot Channel Management** (page 9, `src/pages/bot_channels.rs`):
  per-channel enable/disable kill switch (independent of MCC status) and
  the WhatsApp template pre-approval workflow (recording Meta's actual
  decision by hand — no Meta API integration exists in this codebase).
- **User & Role Management** (page 10, `src/pages/user_management.rs`):
  create/deactivate/reactivate admin accounts, change roles.
  Superadmin-only; refuses to let a superadmin deactivate or demote
  themselves (a cheap self-lockout guard). The same operation
  `scripts/seed-superadmin.sh`/`xtask create-admin` perform from the CLI,
  now also available without shell access.
- **Audit Log Viewer** (page 11): reads the single unified `audit_log`
  table every other admin action writes to.
- **Analytics Dashboard** (page 12): aggregate counts by event type,
  channel, and sessions-per-day, reading only the fully-anonymized
  `analytics_rollups_daily` table — no per-user drill-down exists because
  that table has no session/user column to drill into.
- **Data Export & Retention Tools** (page 13,
  `src/pages/data_retention.rs`): on-demand expired-session purge (same
  statements as `xtask purge-expired-sessions`), the `feedback.contact_email`
  180-day post-resolution sweep (`migrations/0008`'s documented policy),
  and an `audit_log` date-range export as a downloadable JSON file, for
  external compliance review.

Also added as part of a broader `rust/` update alongside the admin pages:
every service in the workspace (not just `admin-app`) now exposes
`/healthz` and `/metrics` (Prometheus) — see `rust/README.md`'s monitoring
notes — and `rust/crates/xtask` is a small CLI for bootstrapping the first
superadmin account and running machine-translation drafts.

## Disclosed scope boundaries within pages above

Nothing below is a missing backend — every one of these depends on data
this workspace already produces (jobs, analytics, migrations); what's
listed is UI/workflow this pass didn't build on top of that data:

- The Decision Tree Visual Editor is a JSON editor, not a node-graph
  canvas (page 4, above).
- Translation Management has no in-browser per-string editor — that's
  the `xtask`/docs-20 workflow instead (page 5, above).
- The Knowledge Base Content Editor (page 3) still has no diff view over
  `knowledge_entry_revisions` and no inline "check this citation link"
  button (that's `link_health.rs`'s standalone page instead).
- Bot Channel Management (page 9) reads `webhook_configured`/
  `last_health_check_*` but doesn't actively re-probe them — that's
  `bot-whatsapp`/`bot-telegram`'s own `/healthz`, scraped by Prometheus.
- Data Export & Retention Tools (page 13) has no per-citizen "export/
  delete everything about me" self-service flow — VoteAssist holds very
  little identifying data by design, so most such requests resolve via
  `web-app`'s existing self-service account deletion instead.
