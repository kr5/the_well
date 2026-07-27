#!/usr/bin/env bash
# Wraps `xtask purge-expired-sessions`. `crates/jobs` now also sweeps
# expired admin/account sessions and OTP challenges on its own daily
# schedule (`run_session_cleanup_job`) — this script remains for
# on-demand runs (e.g. right after handling a DPDP data-subject request,
# or from a cron entry as a second, independent safety net), sharing the
# exact same statements rather than a second implementation.
set -euo pipefail

if [ -z "${DATABASE_URL:-}" ]; then
  echo "error: DATABASE_URL must be set" >&2
  exit 1
fi

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." &>/dev/null && pwd)"

cargo run --manifest-path "$REPO_ROOT/rust/Cargo.toml" -p xtask --release -- purge-expired-sessions
