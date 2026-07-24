#!/usr/bin/env bash
# Wraps pgBackRest per docs/SECURITY-AND-SRE-OPERATIONS.md Section 5
# ("Backup, Disaster Recovery & Point-in-Time Restore"): pgBackRest, not
# plain pg_dump, is this project's chosen backup tool (block-level
# incremental backups + continuous WAL archiving). This script does NOT
# configure pgBackRest for you — a pgBackRest stanza (repository location,
# retention policy) is environment-specific and must already exist per
# that document's "Concrete backup policy" (full base backup daily,
# incremental/differential per that section's cadence, backups in a
# separate failure domain from the live database). This script is the
# thin, repeatable wrapper around the commands that policy actually runs.
#
# Usage: scripts/backup-db.sh [full|diff|incr]  (default: incr)
set -euo pipefail

if ! command -v pgbackrest >/dev/null 2>&1; then
  echo "error: pgbackrest is not installed or not on PATH. See docs/SECURITY-AND-SRE-OPERATIONS.md Section 5." >&2
  exit 1
fi

if [ -z "${PGBACKREST_STANZA:-}" ]; then
  echo "error: PGBACKREST_STANZA must be set to the configured stanza name for this environment" >&2
  exit 1
fi

BACKUP_TYPE="${1:-incr}"
case "$BACKUP_TYPE" in
  full|diff|incr) ;;
  *)
    echo "usage: $0 [full|diff|incr]" >&2
    exit 1
    ;;
esac

echo "Running pgbackrest --stanza=$PGBACKREST_STANZA --type=$BACKUP_TYPE backup ..."
pgbackrest --stanza="$PGBACKREST_STANZA" --type="$BACKUP_TYPE" backup

echo "Backup complete. Verify with: pgbackrest --stanza=$PGBACKREST_STANZA info"
echo "Per the quarterly-restore-drill policy (Section 5), this backup is not considered verified until it has actually been restored into a scratch environment."
