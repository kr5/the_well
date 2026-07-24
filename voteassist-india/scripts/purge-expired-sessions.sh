#!/usr/bin/env bash
# Wraps `xtask purge-expired-sessions` — intended to be run from cron
# (e.g. hourly) since nothing in crates/jobs sweeps expired admin/account
# sessions or OTP challenges (see xtask's own module doc for why that's a
# disclosed gap rather than folded into that crate's four named jobs).
set -euo pipefail

if [ -z "${DATABASE_URL:-}" ]; then
  echo "error: DATABASE_URL must be set" >&2
  exit 1
fi

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." &>/dev/null && pwd)"

cargo run --manifest-path "$REPO_ROOT/rust/Cargo.toml" -p xtask --release -- purge-expired-sessions
