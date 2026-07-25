#!/usr/bin/env bash
# Runs the full machine-translation pipeline (rust/crates/xtask's
# translate-kb-entry/translate-tree, via Claude Haiku) for ONE target
# language: every knowledge-base entry, plus both decision trees.
#
# Output is written under translation-drafts/<locale>/ — nothing here
# touches knowledge-base/sources/ or rust/crates/core-domain/src/*.json,
# per xtask's own module doc: every output is a draft a human reviews and
# manually promotes/merges, never something this pipeline auto-publishes.
#
# Usage: scripts/translate-language.sh <locale-code>
#   e.g.: scripts/translate-language.sh ta
#
# Requires either ANTHROPIC_API_KEY (Claude Haiku) or NVIDIA_API_KEY
# (NVIDIA NIM's free-tier Nemotron models) — set TRANSLATION_PROVIDER to
# force one over the other if both happen to be set. See
# docs/20-translation-task-tracker.md for the full checklist this feeds
# into, and rust/crates/xtask/src/translate/client.rs for backend details.
set -euo pipefail

if [ -z "${ANTHROPIC_API_KEY:-}" ] && [ -z "${NVIDIA_API_KEY:-}" ]; then
  echo "error: set ANTHROPIC_API_KEY or NVIDIA_API_KEY" >&2
  exit 1
fi

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <locale-code>" >&2
  exit 1
fi

LOCALE="$1"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." &>/dev/null && pwd)"
CARGO_MANIFEST="$REPO_ROOT/rust/Cargo.toml"
OUTPUT_DIR="$REPO_ROOT/translation-drafts/$LOCALE"
mkdir -p "$OUTPUT_DIR"

xtask() {
  cargo run --manifest-path "$CARGO_MANIFEST" -p xtask --release --quiet -- "$@"
}

echo "== Knowledge base entries -> $LOCALE =="
for entry_path in "$REPO_ROOT"/knowledge-base/sources/*.json; do
  entry_id="$(basename "$entry_path" .json)"
  output_path="$OUTPUT_DIR/$entry_id-$LOCALE.json"
  echo "-- $entry_id"
  xtask translate-kb-entry "$entry_path" "$LOCALE" "$output_path"
done

echo
echo "== Decision trees -> $LOCALE =="
xtask translate-tree "$REPO_ROOT/rust/crates/core-domain/src/tree_v1.json" "$LOCALE" "$OUTPUT_DIR/tree_v1-$LOCALE-draft.json"
xtask translate-tree "$REPO_ROOT/rust/crates/core-domain/src/tree_v2.json" "$LOCALE" "$OUTPUT_DIR/tree_v2-$LOCALE-draft.json"

echo
echo "Done. Review everything under $OUTPUT_DIR before promoting any of it — see docs/20-translation-task-tracker.md."
