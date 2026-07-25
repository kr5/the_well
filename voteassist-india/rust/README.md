# VoteAssist India — Rust workspace

Real implementation of the architecture specified in
`../docs/PRD-V2-RUST-PLATFORM.md`. Two build generations are mixed in this
workspace, and this README says which is which:

- **Compiled and tested locally** (`core-domain`, `kb-content`, `api`, in
  their original MVP-Rust-v1 form): every claim below about test counts,
  clippy, and the live-server curl flow was actually run and verified.
- **Written since, not locally compiled** (`migrations/`, `jurisdiction`,
  the `core-domain` v2 tree, `analytics`, `jobs`, `channel-core`,
  `bot-telegram`, `bot-whatsapp`, `ivr-gateway`, `web-app`): built under an explicit
  constraint not to run/build code on the authoring machine (limited local
  resources; the target is a server build). Every module was still
  written as real, working logic — no stub functions, no `todo!()`s — and
  checked carefully by hand (schema cross-references against the actual
  migration SQL, dependency/type signatures cross-checked against
  upstream docs), but none of it has been through `cargo build`,
  `cargo test`, or `cargo clippy` yet. Treat first-build errors on these
  crates as expected integration work, not evidence the design is wrong —
  see each crate's own doc comments for disclosed, specific risk areas
  (e.g. the exact `teloxide`/Exotel API surface pinned from documentation
  rather than compiler feedback).

## What's implemented

| Crate | What it is | Status |
|---|---|---|
| `crates/core-domain` | Pure, zero-I/O decision engine — a faithful Rust port of `packages/decision-engine`. Ships both the original 21-node MVP tree (`tree_v1.json`) and a 33-node v2 tree (`tree_v2.json`) covering NRI/service-voter/PwD/shifting-residence flows | v1: compiled + 13 tests passing. v2 additions: written, not locally compiled |
| `crates/kb-content` | Typed loader over `knowledge-base/sources/*.json`, validated against the JSON Schema with a real `jsonschema` validator | Compiled + 11 tests passing |
| `crates/api` | Axum HTTP API wiring the above behind a stateless, anonymous session model, with generated OpenAPI docs at `/docs` | Compiled + 9 tests passing, verified live via curl |
| `crates/jurisdiction` | ECI-vs-State-Election-Commission jurisdiction model (Article 243K/243ZA) — which body administers which election type, per state | Written, not locally compiled |
| `crates/analytics` | Privacy-preserving event ingestion/rollup/purge (hour-bucketed, 30-day raw retention, no free-text/IP storage) | Written, not locally compiled |
| `crates/jobs` (`voteassist-jobs` binary) | Scheduled workers: nightly citation link-checker, 180-day KB re-verification digest, hourly analytics rollup/purge, translation-completeness report. Uses a hand-rolled `tokio::time::interval` scheduler instead of `apalis`/`apalis-cron` (see `src/lib.rs` doc comment for why) | Written, not locally compiled |
| `crates/channel-core` | Shared plumbing for the messaging/voice adapters: locale-resolved node rendering, an ephemeral TTL session store (no Redis dependency), the MCC proactive-broadcast gate (R-MCC-1) | Written, not locally compiled |
| `crates/bot-telegram` (`voteassist-bot-telegram` binary) | Telegram adapter (`teloxide`, long polling): inline-keyboard question/answer flow, locale auto-detected from `User.language_code`, MCC-gated broadcast primitive | Written, not locally compiled |
| `crates/bot-whatsapp` (`voteassist-bot-whatsapp` binary) | WhatsApp adapter: thin `reqwest` wrapper over the Meta Cloud API, Axum webhook receiver with HMAC-SHA256 signature verification, interactive button/list rendering, 24-hour customer-service-window-aware proactive send path | Written, not locally compiled |
| `crates/ivr-gateway` (`voteassist-ivr-gateway` binary) | Scaffolded Exotel voice adapter, deliberately narrower per PRD Section 6.7 ("interface defined, not fully wired" until v2/v3): DTMF question/answer over a Passthru webhook, spoken English/Hindi prompts, terminal-outcome handoff via `bot-whatsapp`'s client (a phone call can't click a link) | Written, not locally compiled — see `src/webhook.rs` for exactly what is/isn't confirmed against Exotel's real API |
| `migrations/0001`-`0012` | Full Postgres schema: KB + audit trail, decision trees, forms/geography/jurisdiction, elections calendar + MCC windows, admin/RBAC/audit log, accounts/drafts/consent, feedback/fringe cases, analytics, link-check/translation status, bot channel config, account OTP/sessions | Written, not run against a live database |
| `crates/web-app` (`voteassist-web-app` binary, built via `cargo-leptos`) | Public site: Leptos SSR + islands, full sitemap from `docs/05-information-architecture.md` (home, `/start`, `/search`, `/learn` + forms/faq/glossary/`:slug`, `/locate`, four `/about/*` pages, `/feedback` writing to Postgres, `/accessibility`, `/account` + `/account/login`). Server functions call `core-domain`/`kb-content` in-process — no separate HTTP hop to `crates/api`. Includes the optional public account system (email OTP via `lettre`/SMTP, HMAC-blind-indexed contact hash, argon2 OTP hashing), saved/frozen decision-tree "checklists," and a cookie-consent banner gating future analytics recording | Written, not locally compiled — see `src/server_fns.rs`/`src/render.rs`/`src/accounts/` module docs for the client-held-session architecture, the disclosed reason `channel-core` isn't reused here, and the DPDP/cookie-consent design |
| `crates/admin-app` (`voteassist-admin-app` binary, built via `cargo-leptos`) | Admin dashboard, a separate deploy target from `web-app`: hand-rolled Postgres-backed session auth (argon2 + a CSPRNG token — not `tower-sessions`, see `src/auth/session.rs` for why), RBAC (`require_role`, enforced per server function, not just hidden UI), Dashboard, Knowledge Base Content Editor (list/create/edit + revision history + audit log), MCC Control Panel (the same `mcc_windows` table the bot adapters' broadcast gate reads), Audit Log Viewer, Analytics Dashboard (event/channel/daily-session breakdowns from `analytics_rollups_daily`, never raw `analytics_events`) | Written, not locally compiled — see this crate's README for which of the 13 PRD v2 Section 11 admin pages are (and aren't yet) implemented |
| `crates/xtask` (`xtask` binary) | Operational CLI, not a deployed service: `hash-password`, `create-admin` (bootstraps the first superadmin — `admin-app` has no self-serve signup), `purge-expired-sessions`, and `translate-kb-entry`/`translate-tree` (machine-translation drafts via Claude Haiku or NVIDIA NIM's free-tier Nemotron models — needs `ANTHROPIC_API_KEY` or `NVIDIA_API_KEY`, see `src/translate/client.rs` for backend selection). See `../scripts/README.md` for the shell wrappers that call it, and `../docs/20-translation-task-tracker.md` for the translation workflow | Written, not locally compiled |

**Locally-verified baseline: 33 tests, zero clippy warnings (`-D warnings`), zero unformatted files** (the three original crates only — see above).

While building this, the real JSON-Schema validation in `kb-content` caught
two genuine bugs in the existing knowledge base that the TypeScript
prototype's looser typing had missed:
1. `service-voter.json` used `relatedForms: ["form-2"]`, but the schema's
   enum didn't include `form-2` — fixed by adding it to the schema (the data
   was correct; Form 2 is the real service-voter registration form).
2. `form-8.json`'s `summary` field was 498 characters against the schema's
   400-char cap — fixed by raising the cap to 500 rather than truncating a
   genuinely useful summary (it was the only entry over the old limit).

Both fixes are in `../knowledge-base/schema/entry.schema.json`.

## Running it

The first `cargo build --workspace` on a server is expected to surface
issues in the "written, not locally compiled" crates above — most likely
dependency-version pins (`teloxide`, `sqlx`'s exact point release) and any
sqlx query needing its offline `.sqlx` metadata generated against a real
database (every query in this workspace uses runtime-checked
`query()`/`query_as()`/`query_scalar()`, not the `query!` compile-time
macros, specifically so a live database is only needed at run time, not
build time — but do verify this on first build).

```bash
cd rust
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check

# Set up Postgres and run the schema (needed by anything below except
# core-domain/kb-content/api, which stay database-free by design):
sqlx migrate run --source migrations

cargo run -p api --bin voteassist-api             # HTTP API, :8080
cargo run -p jobs --bin voteassist-jobs           # scheduled workers (needs DATABASE_URL)
cargo run -p bot-telegram --bin voteassist-bot-telegram     # needs TELOXIDE_TOKEN
cargo run -p bot-whatsapp --bin voteassist-bot-whatsapp     # needs WHATSAPP_*, :8081
cargo run -p ivr-gateway --bin voteassist-ivr-gateway       # needs IVR_WEBHOOK_SHARED_SECRET, :8082

# web-app and admin-app are built/served by cargo-leptos (not plain
# `cargo run`), since each compiles two targets (the ssr binary, native;
# the hydrate lib, wasm32):
cd crates/web-app && cargo leptos serve                     # needs DATABASE_URL, :3000
cd crates/admin-app && cargo leptos serve                   # needs DATABASE_URL, :3010
```

With the API server running:

```bash
curl -s http://localhost:8080/healthz
curl -s -X POST http://localhost:8080/v1/sessions | jq
curl -s http://localhost:8080/openapi.json | jq '.paths | keys'
open http://localhost:8080/docs   # Swagger UI
```

## What's NOT yet implemented

Every crate from `docs/PRD-V2-RUST-PLATFORM.md` Section 6.2's list now has
a real, if not-yet-compiled, implementation. `crates/admin-app` covers 5
of the 13 admin pages from PRD v2 Section 11 (auth/RBAC, Dashboard, KB
Content Editor, MCC Control Panel, Audit Log Viewer, Analytics Dashboard);
the remaining 7 pages (Decision Tree Visual Editor, Translation
Management, Citation & Link Health, Feedback Triage, Bot Channel
Management, User & Role Management, Data Export & Retention) are
documented follow-ups — see `crates/admin-app/README.md` for exactly what's
missing and why each is a UI-only gap (the data these pages would read/act
on is already produced by `jobs`/`analytics`/the migrations). See PRD v2
Section 21 and PRD v3 Section V8 (EPICs 17-22) for sequencing.

## Monitoring, ops scripts, and embedding this as a module

- **Monitoring**: every axum-based service (`api`, `bot-whatsapp`,
  `ivr-gateway`, `web-app`, `admin-app`) exposes `/healthz` and `/metrics`
  (Prometheus format, via `axum-prometheus`); `jobs` and `bot-telegram`
  have no HTTP traffic of their own, so they run a small side-channel
  health/metrics server instead (`JOBS_HEALTH_ADDR`/`BOT_TELEGRAM_HEALTH_ADDR`,
  default `0.0.0.0:9090`/`0.0.0.0:9091`), via `metrics`+`metrics-exporter-prometheus`.
- **Helper scripts**: `../scripts/` — see `../scripts/README.md` for
  DB setup/seeding, dev up/down, health checks, backup/restore
  (pgBackRest), expired-session purging, and the translation pipeline.
- **Running this as a module of a larger system** (shared Postgres,
  reverse-proxy path-mounting, `crates/api`'s CORS layer, session/cookie
  scoping, and which crates are usable as plain Rust libraries): see
  `../docs/21-integration-as-a-module.md`.
