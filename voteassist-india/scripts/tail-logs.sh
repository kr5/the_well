#!/usr/bin/env bash
# Tails every service's logs. Two modes:
#   - Local dev (default): tails .dev-pids/<service>.log, written by
#     scripts/dev-up.sh's background `cargo run` redirection.
#   - Production (--systemd): tails journalctl for each service's
#     systemd unit instead — this shares scripts/deploy.sh's own
#     disclosed "systemd-managed single binary" deployment assumption
#     (see that script's header comment); adjust both scripts together
#     if your environment actually uses something else (containers, an
#     orchestrator).
set -uo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." &>/dev/null && pwd)"
PID_DIR="$REPO_ROOT/.dev-pids"

# web-app/admin-app are excluded from --systemd's default unit-name guess
# below on purpose: scripts/dev-up.sh doesn't start them either (they run
# under cargo-leptos, not a plain binary) — pass explicit unit names as
# extra arguments after --systemd if your deployment does run them as
# systemd services, e.g.: tail-logs.sh --systemd voteassist-web-app
SERVICES=(api jobs bot-telegram bot-whatsapp ivr-gateway)

if [ "${1:-}" = "--systemd" ]; then
  shift
  journalctl_args=()
  for name in "${SERVICES[@]}"; do
    journalctl_args+=(-u "voteassist-$name")
  done
  for extra_unit in "$@"; do
    journalctl_args+=(-u "$extra_unit")
  done
  echo "Tailing journalctl for: ${journalctl_args[*]}"
  exec journalctl "${journalctl_args[@]}" -f
fi

echo "Tailing local dev logs in $PID_DIR (scripts/dev-up.sh's output) — pass --systemd for a production deployment instead."
log_files=()
for name in "${SERVICES[@]}"; do
  if [ -f "$PID_DIR/$name.log" ]; then
    log_files+=("$PID_DIR/$name.log")
  fi
done

if [ "${#log_files[@]}" -eq 0 ]; then
  echo "No dev logs found in $PID_DIR — is scripts/dev-up.sh running?" >&2
  exit 1
fi

exec tail -f "${log_files[@]}"
