#!/usr/bin/env bash
# Starts every plain-`cargo run` service (api, jobs, bot-telegram,
# bot-whatsapp, ivr-gateway) in the background for local development.
# web-app and admin-app are NOT started here — both are built/served by
# `cargo-leptos` (a separate toolchain compiling two targets per app), not
# plain `cargo run`; start them yourself in separate terminals:
#   (cd rust/crates/web-app && cargo leptos serve)
#   (cd rust/crates/admin-app && cargo leptos serve)
#
# Requires DATABASE_URL to already point at a migrated database (run
# scripts/setup-db.sh first) and whatever channel-specific env vars each
# service's own README documents (TELOXIDE_TOKEN, WHATSAPP_*, etc.) —
# services that need a credential they don't have will fail to start,
# which is expected and not this script's job to route around.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." &>/dev/null && pwd)"
CARGO_MANIFEST="$REPO_ROOT/rust/Cargo.toml"
PID_DIR="$REPO_ROOT/.dev-pids"
mkdir -p "$PID_DIR"

if [ -z "${DATABASE_URL:-}" ]; then
  echo "error: DATABASE_URL must be set" >&2
  exit 1
fi

start() {
  local name="$1"
  local bin="$2"
  echo "Starting $name ..."
  (cargo run --manifest-path "$CARGO_MANIFEST" -p "$name" --bin "$bin" &> "$PID_DIR/$name.log" & echo $! > "$PID_DIR/$name.pid")
}

start api voteassist-api
start jobs voteassist-jobs

if [ -n "${TELOXIDE_TOKEN:-}" ]; then
  start bot-telegram voteassist-bot-telegram
else
  echo "Skipping bot-telegram: TELOXIDE_TOKEN not set"
fi

if [ -n "${WHATSAPP_ACCESS_TOKEN:-}" ]; then
  start bot-whatsapp voteassist-bot-whatsapp
else
  echo "Skipping bot-whatsapp: WHATSAPP_ACCESS_TOKEN not set"
fi

if [ -n "${IVR_WEBHOOK_SHARED_SECRET:-}" ]; then
  start ivr-gateway voteassist-ivr-gateway
else
  echo "Skipping ivr-gateway: IVR_WEBHOOK_SHARED_SECRET not set"
fi

echo
echo "Started. Logs: $PID_DIR/<service>.log — PIDs: $PID_DIR/<service>.pid"
echo "Stop everything with: scripts/dev-down.sh"
