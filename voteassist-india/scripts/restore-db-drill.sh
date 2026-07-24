#!/usr/bin/env bash
# Restores the latest pgBackRest backup into a NEW, scratch Postgres data
# directory — never over a live one — per
# docs/SECURITY-AND-SRE-OPERATIONS.md Section 5's quarterly restore-drill
# requirement: "a backup that has never been restored is not a verified
# backup." This script only performs the restore step; verifying the
# restored data (row counts, spot-checking a few tables, confirming the
# application can connect to it) is a manual step deliberately left to
# the person running the drill, not automated away.
#
# Usage: scripts/restore-db-drill.sh <empty-target-directory>
set -euo pipefail

if ! command -v pgbackrest >/dev/null 2>&1; then
  echo "error: pgbackrest is not installed or not on PATH." >&2
  exit 1
fi

if [ -z "${PGBACKREST_STANZA:-}" ]; then
  echo "error: PGBACKREST_STANZA must be set to the configured stanza name for this environment" >&2
  exit 1
fi

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <empty-target-directory>" >&2
  exit 1
fi

TARGET_DIR="$1"

if [ -e "$TARGET_DIR" ] && [ -n "$(ls -A "$TARGET_DIR" 2>/dev/null)" ]; then
  echo "error: $TARGET_DIR already exists and is not empty — refusing to restore over it." >&2
  exit 1
fi

mkdir -p "$TARGET_DIR"

echo "Restoring the latest $PGBACKREST_STANZA backup into $TARGET_DIR ..."
pgbackrest --stanza="$PGBACKREST_STANZA" --pg1-path="$TARGET_DIR" restore

cat <<EOF

Restore complete into $TARGET_DIR.

This is a scratch data directory, not a running server yet. Next:
  1. Start a throwaway Postgres instance pointed at $TARGET_DIR (a distinct
     port from any live database — this drill must never touch production).
  2. Connect to it and spot-check: row counts on a few tables you know the
     expected order of magnitude for, and confirm crates/api can point at
     it and answer a request.
  3. Record the drill's outcome per Section 5's "100% success rate, a
     failed drill is itself the finding" policy — a failed drill is a
     SEV2 incident to track, not something to silently retry.
  4. Tear down the throwaway instance and this scratch directory once
     verification is done.
EOF
