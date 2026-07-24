#!/usr/bin/env bash
# Stops every service scripts/dev-up.sh started, by PID file.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." &>/dev/null && pwd)"
PID_DIR="$REPO_ROOT/.dev-pids"

if [ ! -d "$PID_DIR" ]; then
  echo "No $PID_DIR directory found — nothing to stop."
  exit 0
fi

for pid_file in "$PID_DIR"/*.pid; do
  [ -e "$pid_file" ] || continue
  name="$(basename "$pid_file" .pid)"
  pid="$(cat "$pid_file")"
  if kill -0 "$pid" 2>/dev/null; then
    echo "Stopping $name (pid $pid) ..."
    kill "$pid"
  else
    echo "$name (pid $pid) was already stopped."
  fi
  rm -f "$pid_file"
done
