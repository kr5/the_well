# VoteAssist India — Rust workspace

Real, compiling, tested implementation of the architecture specified in
`../docs/PRD-V2-RUST-PLATFORM.md`. This is not a stub: every crate below
has a working test suite that runs in CI-equivalent conditions locally.

## What's implemented

| Crate | What it is | Tests |
|---|---|---|
| `crates/core-domain` | Pure, zero-I/O decision-engine — a faithful Rust port of `packages/decision-engine` (the TS prototype), including the full 21-node MVP tree | 13 (unit + structural + `proptest` property tests) |
| `crates/kb-content` | Typed loader over `knowledge-base/sources/*.json`, validated against `knowledge-base/schema/entry.schema.json` with a real `jsonschema` validator (not just serde) | 11 (including full schema-conformance) |
| `crates/api` | Axum HTTP API wiring the two crates above behind a stateless, anonymous session model, with generated OpenAPI docs at `/docs` | 9 (including a full end-to-end HTTP flow test) |

**33 tests, zero clippy warnings (`-D warnings`), zero unformatted files.**

While building this, the real JSON-Schema validation in `kb-content` caught
two genuine bugs in the existing knowledge base that the TypeScript
prototype's looser typing had missed:
1. `service-voter.json` used `relatedForms: ["form-2"]`, but the schema's
   enum didn't include `form-2` — fixed by adding it to the schema (the data
   was correct; Form 2 is the real service-voter registration form).
2. `form-8.json`'s `summary` field was 498 characters against the schema's
   400-char cap — fixed by raising the cap to 500 rather than truncating a
   genuinely useful summary (it was the only entry over the old limit).

Both fixes are in `../knowledge-base/schema/entry.schema.json`.

## Running it

```bash
cd rust
cargo test --workspace          # all 33 tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo run -p api --bin voteassist-api   # starts the server on :8080
```

With the server running:

```bash
curl -s http://localhost:8080/healthz
curl -s -X POST http://localhost:8080/v1/sessions | jq
curl -s http://localhost:8080/openapi.json | jq '.paths | keys'
open http://localhost:8080/docs   # Swagger UI
```

## What's NOT yet implemented

Everything else in `docs/PRD-V2-RUST-PLATFORM.md` Section 6.2's crate list:
`web-app` (Leptos), `admin-app`, `bot-telegram`, `bot-whatsapp`,
`ivr-gateway`, `jobs`, `analytics`, and the Postgres-backed persistence
layer (`migrations/`). The `api` crate today is intentionally minimal —
stateless, in-memory tree, no database — because that's the honest MVP-
Rust-v1 slice worth building and testing correctly before adding
persistence, auth, and the remaining channels. See PRD v2 Section 21 and
PRD v3 Section V8 (EPICs 17-22) for the sequencing of what comes next.
