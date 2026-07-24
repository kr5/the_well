# Operational scripts

Shell wrappers for tasks that don't belong inside any single running
service. Every script is self-contained (checks its own required env vars,
fails loudly rather than guessing) — see each script's own header comment
for exactly what it does and why.

| Script | Purpose |
|---|---|
| `setup-db.sh` | Runs every migration in `rust/migrations/` against `$DATABASE_URL` (via `sqlx-cli`). |
| `seed-superadmin.sh <email>` | Bootstraps the first `admin-app` superadmin account (interactive password prompt) — `admin-app` has no self-serve signup by design. |
| `purge-expired-sessions.sh` | Sweeps expired admin/account sessions and OTP challenges; intended for a cron entry (nothing in `crates/jobs` does this yet — see `crates/xtask`'s module doc). |
| `dev-up.sh` / `dev-down.sh` | Starts/stops the plain-`cargo run` services (api, jobs, bot-telegram, bot-whatsapp, ivr-gateway) for local development. `web-app`/`admin-app` use `cargo-leptos serve` instead — see `dev-up.sh`'s header for why they're not started here. |
| `health-check.sh` | Curls every service's `/healthz`. A starting point for an external uptime monitor, not a substitute for the `/metrics` Prometheus scraping documented in `docs/SECURITY-AND-SRE-OPERATIONS.md`. |
| `backup-db.sh [full\|diff\|incr]` | Wraps `pgbackrest backup` per the project's chosen backup tool (`docs/SECURITY-AND-SRE-OPERATIONS.md` Section 5). Requires an already-configured pgBackRest stanza — this script doesn't set one up. |
| `restore-db-drill.sh <dir>` | Restores the latest pgBackRest backup into a scratch directory, for the quarterly restore-drill requirement ("a backup that has never been restored is not a verified backup"). Never touches a live data directory. |

## `xtask`

`rust/crates/xtask` is the companion Rust CLI several of these scripts
call into (password hashing needs `argon2`, which a shell script can't do
natively): `cargo run -p xtask -- hash-password <pw>`,
`create-admin <email> <password> <role>`, `purge-expired-sessions`.
