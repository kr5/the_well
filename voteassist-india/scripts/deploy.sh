#!/usr/bin/env bash
# Builds every service's release binary/bundle and restarts its systemd
# unit.
#
# Disclosed assumption: this assumes a "systemd-managed single binary"
# deployment — docs/SECURITY-AND-SRE-OPERATIONS.md's own phrase for this
# project's deployment philosophy — one systemd unit per service, named
# voteassist-<service>, running the compiled binary directly. Nothing in
# this repository defines those unit files yet (no Dockerfile or
# docker-compose.yml exists either) — if your environment actually
# deploys via containers or an orchestrator instead, the restart loop at
# the bottom of this script is the one part to swap out; the build step
# above it is deployment-model-agnostic either way. scripts/tail-logs.sh
# shares this same assumption for its --systemd mode.
#
# Needs permission to restart the named units (run as root, or as a user
# a polkit/sudoers rule allows to `systemctl restart voteassist-*`) —
# this script does not attempt to elevate privileges itself.
#
# Usage: scripts/deploy.sh [service ...]
#   No arguments: builds and restarts every service.
#   One or more of: api, web-app, admin-app, jobs, bot-telegram,
#   bot-whatsapp, ivr-gateway — builds and restarts only those.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." &>/dev/null && pwd)"
CARGO_MANIFEST="$REPO_ROOT/rust/Cargo.toml"

ALL_SERVICES=(api web-app admin-app jobs bot-telegram bot-whatsapp ivr-gateway)

if [ "$#" -gt 0 ]; then
  SERVICES=("$@")
else
  SERVICES=("${ALL_SERVICES[@]}")
fi

for name in "${SERVICES[@]}"; do
  known=false
  for candidate in "${ALL_SERVICES[@]}"; do
    if [ "$name" = "$candidate" ]; then
      known=true
      break
    fi
  done
  if [ "$known" = false ]; then
    echo "error: unknown service \"$name\" — expected one of: ${ALL_SERVICES[*]}" >&2
    exit 1
  fi
done

echo "== Building: ${SERVICES[*]} =="
for name in "${SERVICES[@]}"; do
  echo "-- $name"
  case "$name" in
    web-app | admin-app)
      # cargo-leptos builds both the server binary and the wasm/client
      # bundle in one pass; plain `cargo build` only builds the former.
      (cd "$REPO_ROOT/rust/crates/$name" && cargo leptos build --release)
      ;;
    *)
      cargo build --manifest-path "$CARGO_MANIFEST" -p "$name" --release
      ;;
  esac
done

echo
echo "== Restarting systemd units =="
if ! command -v systemctl &>/dev/null; then
  echo "systemctl not found on this host — build succeeded, but restart the services yourself" >&2
  echo "(see this script's header comment if your deployment isn't systemd-based)." >&2
  exit 1
fi

for name in "${SERVICES[@]}"; do
  unit="voteassist-$name"
  echo "-- $unit"
  systemctl restart "$unit"
done

echo
echo "Done. Run scripts/smoke-test.sh to verify, and scripts/tail-logs.sh --systemd to watch logs."
