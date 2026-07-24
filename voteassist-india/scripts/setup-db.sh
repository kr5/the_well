#!/usr/bin/env bash
# Runs every migration in rust/migrations/ against $DATABASE_URL, in order.
# Idempotent: sqlx tracks applied migrations in its own `_sqlx_migrations`
# table, so re-running this after new migrations land only applies the
# new ones.
#
# Requires sqlx-cli: cargo install sqlx-cli --no-default-features --features postgres,rustls
set -euo pipefail

if [ -z "${DATABASE_URL:-}" ]; then
  echo "error: DATABASE_URL must be set, e.g. postgres://user:pass@host:5432/voteassist" >&2
  exit 1
fi

if ! command -v sqlx >/dev/null 2>&1; then
  echo "error: sqlx-cli is not installed. Run:" >&2
  echo "  cargo install sqlx-cli --no-default-features --features postgres,rustls" >&2
  exit 1
fi

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." &>/dev/null && pwd)"

echo "Running migrations from $REPO_ROOT/rust/migrations against $DATABASE_URL ..."
sqlx migrate run --source "$REPO_ROOT/rust/migrations"
echo "Done."
