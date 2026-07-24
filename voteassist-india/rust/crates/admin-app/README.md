# admin-app

Admin/reviewer dashboard for VoteAssist India — a separate Leptos SSR
deploy target from `crates/web-app`, per docs/PRD-V2-RUST-PLATFORM.md
Section 11 and EPIC 7. Written without local compilation (see the
workspace-root `rust/README.md` for what that means here).

## Implemented (this pass)

- **Auth**: email/password login, argon2 password hashing, a hand-rolled
  Postgres-backed session (see `src/auth/session.rs` for why this isn't
  built on the `tower-sessions` crate the PRD names), HttpOnly/Secure/
  SameSite=Strict cookie.
- **RBAC**: `require_admin`/`require_role` enforced at the top of every
  server function that touches real data (`src/server_fns.rs`) — the six
  roles from PRD v2 Section 11 (`contributor`, `reviewer`,
  `legal_reviewer`, `translator`, `analytics_viewer`, `superadmin`).
- **Dashboard** (admin page 2): live counts (feedback awaiting triage,
  entries needing re-verification, verified entries, active MCC windows).
- **Knowledge Base Content Editor** (admin page 3): list/filter, create/
  edit form, review-status state machine. Every save writes an
  append-only `knowledge_entry_revisions` snapshot and an `audit_log` row
  in the same transaction.
- **MCC / Election-Period Control Panel** (admin page 8): open/close an
  MCC window per state — the exact `mcc_windows` table
  `channel_core::check_broadcast_allowed` (used by `bot-telegram`'s and
  `bot-whatsapp`'s proactive-broadcast gates) reads from. Gated to
  `legal_reviewer`/`superadmin`.
- **Audit Log Viewer** (admin page 11): reads the single unified
  `audit_log` table every other admin action writes to.

## Not implemented (disclosed, not silently skipped)

From PRD v2 Section 11's 13 admin pages, these are documented but not
built in this pass:

- **Decision Tree Visual Editor** (page 4) — node-graph editing,
  `core-domain::validate_tree` reuse, publish/rollback flow.
- **Translation Management** (page 5) — per-locale completeness dashboard
  (the `jobs` crate's `translation_completeness` job already populates
  `translation_status`; no UI reads it yet), translation workbench.
- **Citation & Link Health** (page 6) — the `jobs` crate's `link_checker`
  already populates `link_check_results`; no UI reads it yet, and the KB
  editor's source-citation sub-editor has no live "check this link now"
  button.
- **Feedback & Grievance Triage** (page 7) — the `feedback` table is
  written to by `web-app`'s `/feedback` page; no admin UI to triage it yet.
- **Bot Channel Management** (page 9) — Telegram/WhatsApp config status,
  the WhatsApp template-message review workflow. `migrations/0011` already
  defines the schema this would read from.
- **User & Role Management** (page 10) — creating/deactivating
  `admin_users` rows and changing roles currently has no UI (do it via
  direct SQL, with `hash_password` from `src/auth/password.rs`, until
  this exists).
- **Analytics Dashboard** (page 12) — reading the `analytics_hourly`/
  `analytics_daily` rollup tables the `analytics`/`jobs` crates already
  populate.
- **Data Export & Retention Tools** (page 13).

Each of these depends on data-producing pieces that already exist
elsewhere in this workspace (jobs, analytics, migrations) — building the
admin UI to read/act on them is the remaining work, not a missing backend.
