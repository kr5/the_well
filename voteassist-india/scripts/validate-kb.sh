#!/usr/bin/env bash
# Validates every knowledge-base/sources/*.json file against
# knowledge-base/schema/entry.schema.json — the same check
# rust/crates/kb-content's own schema_conformance.rs test performs at
# compile time against its fixed, include_str!'d file list, but this
# script discovers files at runtime, so it also catches a brand-new file
# (e.g. right after promoting a translation draft out of
# translation-drafts/) before it's wired into kb-content's source list.
#
# Run this before every content change lands, and as a pre-merge check
# for anything touching knowledge-base/.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." &>/dev/null && pwd)"

cargo run --manifest-path "$REPO_ROOT/rust/Cargo.toml" -p xtask --release --quiet -- \
  validate-kb "$REPO_ROOT/knowledge-base/sources" "$REPO_ROOT/knowledge-base/schema/entry.schema.json"
