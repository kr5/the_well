# Operational scripts

Shell wrappers for tasks that don't belong inside any single running
service. Every script is self-contained (checks its own required env vars,
fails loudly rather than guessing) — see each script's own header comment
for exactly what it does and why.

| Script | Purpose |
|---|---|
| `setup-db.sh` | Runs every migration in `rust/migrations/` against `$DATABASE_URL` (via `sqlx-cli`). |
| `seed-superadmin.sh <email>` | Bootstraps the first `admin-app` superadmin account (interactive password prompt) — `admin-app` has no self-serve signup by design. |
| `purge-expired-sessions.sh` | Sweeps expired admin/account sessions and OTP challenges on demand — `crates/jobs` also runs this same sweep on its own daily schedule; this script is for out-of-band runs. |
| `dev-up.sh` / `dev-down.sh` | Starts/stops the plain-`cargo run` services (api, jobs, bot-telegram, bot-whatsapp, ivr-gateway) for local development. `web-app`/`admin-app` use `cargo-leptos serve` instead — see `dev-up.sh`'s header for why they're not started here. |
| `health-check.sh` | Curls every service's `/healthz`. A starting point for an external uptime monitor, not a substitute for the `/metrics` Prometheus scraping documented in `docs/SECURITY-AND-SRE-OPERATIONS.md`. Also backs `admin-app`'s System Health page. |
| `smoke-test.sh` | Runs `health-check.sh`, then exercises a handful of real endpoints (start a session, search the KB, load the homepage/login page) and checks the response actually looks right — not just that the process answered. Run this right after a deploy. |
| `deploy.sh [service ...]` | Builds release binaries/bundles and restarts each service's systemd unit — see the script's own header for the disclosed "systemd-managed single binary" deployment assumption this and `tail-logs.sh --systemd` share. |
| `tail-logs.sh [--systemd]` | Tails `dev-up.sh`'s local log files by default, or `journalctl` for each service's systemd unit with `--systemd`. |
| `validate-kb.sh` | Validates every `knowledge-base/sources/*.json` file against `entry.schema.json` at runtime (via `xtask validate-kb`) — catches a new/edited file `crates/kb-content`'s compile-time `include_str!` list doesn't know about yet. Run before any content change lands. |
| `backup-db.sh [full\|diff\|incr]` | Wraps `pgbackrest backup` per the project's chosen backup tool (`docs/SECURITY-AND-SRE-OPERATIONS.md` Section 5). Requires an already-configured pgBackRest stanza — this script doesn't set one up. |
| `restore-db-drill.sh <dir>` | Restores the latest pgBackRest backup into a scratch directory, for the quarterly restore-drill requirement ("a backup that has never been restored is not a verified backup"). Never touches a live data directory. |
| `translate-language.sh <locale-code>` | Runs the machine-translation pipeline for one language via `xtask translate-kb-entry`/`translate-tree` — see `docs/20-translation-task-tracker.md`. |

## `xtask`

`rust/crates/xtask` is the companion Rust CLI several of these scripts
call into (password hashing needs `argon2`, JSON Schema validation needs
`jsonschema` — work a shell script can't do natively):
`cargo run -p xtask -- hash-password <pw>`,
`create-admin <email> <password> <role>`, `purge-expired-sessions`,
`translate-kb-entry`/`translate-tree`, `validate-kb <sources-dir> <schema-path>`.
