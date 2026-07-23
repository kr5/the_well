# VoteAssist India

> Help every eligible Indian citizen understand exactly what they need to do
> to register, correct, or manage their voter status — then send them to the
> **official** Election Commission of India (ECI) services to actually do it.

**This is not an official Election Commission of India website, and it is not
affiliated with, endorsed by, or operated by the ECI, any State CEO, or the
Government of India.** It never submits applications on a user's behalf and
never stores official identity documents by default. It is strictly
politically neutral: it explains procedures, never candidates or causes.

> **Temporary location.** This project currently lives inside the `the_well`
> repository (an unrelated ML dataset project) purely as scratch space while
> it's being scaffolded. It is intended to be extracted into its own
> repository — see `docs/14-repository-structure.md` and
> `docs/19-roadmap.md`. Nothing here depends on `the_well`'s code or data.

## Production architecture (planning stage)

The MVP below proved the product concept. The production rebuild is planned
as a **Rust** platform — see
[`docs/PRD-V2-RUST-PLATFORM.md`](docs/PRD-V2-RUST-PLATFORM.md) for the full
specification: deeper ECI/legal research, the Axum/Leptos/sqlx/Postgres
architecture and library choices, an expanded decision-engine covering every
persona in the original brief, a full admin/content-operations tool
(knowledge-base editor, decision-tree visual editor with versioning,
translation workflow, MCC election-period controls, analytics dashboard),
a privacy-preserving analytics pipeline, a security threat model, and a
complete numbered feature/task backlog. That document supersedes the
TypeScript-specific technical/architecture assumptions in `docs/13`–`docs/15`
below (v1); it does not change the mission, personas, or non-goals, which
carry forward unchanged. **This is a specification, not yet implemented** —
the working code today is the TypeScript MVP described next.

[`docs/PRD-V3-COMPREHENSIVE-EXPANSION.md`](docs/PRD-V3-COMPREHENSIVE-EXPANSION.md)
extends v2 further: it corrects an important scoping gap (Panchayat/
Municipal elections are run by separate State Election Commissions, not
the ECI — v2 implicitly assumed ECI-only), generalizes the platform to
every election type, adds a strictly-optional accounts/saved-drafts/
deletion/privacy layer that never weakens the anonymous-by-default
guarantee, makes already-registered ("existing") voters a co-equal pillar
to new-voter registration, specifies a human-in-the-loop self-improving
content pipeline and a fringe-case registry, and researches further
open-source/government platforms (Bhashini for Indic ASR/MT/TTS, DIGIT,
India Stack's consent-architecture pattern) relevant to the build.

## What's here (implemented today)

- A **real Rust implementation** (`rust/`) of the core of the PRD v2
  architecture — not a stub. `crates/core-domain` (the decision engine,
  ported faithfully from the TS prototype), `crates/kb-content` (a typed
  knowledge-base loader validated against the real JSON Schema with an
  actual `jsonschema` validator), and `crates/api` (a working Axum HTTP
  server with generated OpenAPI docs). **33 tests, zero clippy warnings.**
  See `rust/README.md` — including two real content bugs the schema
  validation caught and fixed along the way.
- Concrete operational planning: `docs/SECURITY-AND-SRE-OPERATIONS.md`
  (named tools — SOPS+age, Falco, Trivy, pgBackRest, GoAlert, Uptime Kuma,
  k6 — with real config examples, an incident-response runbook template,
  and SLOs), and `docs/assets/ux-wireframes.html` (screen-by-screen
  wireframes and interaction states for the public app and admin console,
  built from the real decision-tree content, not placeholder copy).
- The original **TypeScript MVP** (`apps/web`, `packages/*`) — a versioned,
  data-driven, fully-tested decision tree covering the highest-frequency
  scenarios: first-time registration, students choosing hostel vs. native
  address, moving house, corrections, lost/damaged EPIC, e-EPIC download,
  NRI (overseas) electors, service voters (armed forces), PwD marking, and
  reporting incorrect/duplicate entries. Retained as the working reference
  implementation during the Rust migration (see PRD v2 Section 6.17).
- A **curated, cited knowledge base** (`knowledge-base/`) — every terminal
  outcome in the decision tree links to real ECI/SVEEP/PIB sources.
- A **web app MVP** (`apps/web`, Next.js) — the decision flow end-to-end in
  English and Hindi, plus a knowledge-base browser, with a persistent
  "we are not the ECI" banner and official deep-links opening in a new tab.
- The full **documentation suite** (`docs/01` through `docs/19`,
  `docs/citations.md`, `docs/PRD-V2-RUST-PLATFORM.md`, and
  `docs/PRD-V3-COMPREHENSIVE-EXPANSION.md`) — PRD, personas, journeys,
  legal/compliance review, architecture, security threat model, test plan,
  roadmap, election-jurisdiction model, and a full feature/task backlog.

Not yet built: `web-app`/`admin-app` (Leptos), the bot channels
(Telegram/WhatsApp/IVR), the Postgres persistence layer, and full
state-by-state legal review — see `rust/README.md` and PRD v2 Section 21 /
PRD v3 Section V8 for the sequencing.

## Repository layout

```
voteassist-india/
  docs/                   PRD, personas, architecture, legal review, roadmap, ...
  knowledge-base/          Curated, cited source-of-truth content (JSON)
  openapi/                 OpenAPI contract for the future API layer
  apps/web/                Next.js MVP web app
  packages/decision-engine/ Pure, tested decision-tree engine (no UI/framework deps)
  packages/knowledge/       Typed loader over knowledge-base/
  packages/i18n/            UI string dictionaries + locale metadata
  packages/ui/              Shared, framework-light React components
```

## Running it

```bash
cd voteassist-india
pnpm install
pnpm test        # decision-engine + i18n unit/property tests
pnpm dev         # apps/web on http://localhost:3000
```

## Guardrails every contribution must respect

1. Never claim or imply official ECI/government status.
2. Never submit a form on a user's behalf.
3. Never store official identity documents by default.
4. Never take a political position, on anything.
5. Every procedural claim needs a citation or an explicit
   "pending verification" label — see `knowledge-base/schema/entry.schema.json`
   and `docs/06-legal-compliance-review.md`.

See `docs/01-prd.md` for the full mission and non-goals, and
`docs/19-roadmap.md` for what's next.
