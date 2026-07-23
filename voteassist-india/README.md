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

## What's here (MVP scope)

This first pass delivers a foundation, not the full vision described in
`docs/`. Concretely, working today:

- A **decision-engine** (`packages/decision-engine`) — a versioned,
  data-driven, fully-tested decision tree covering the highest-frequency
  scenarios: first-time registration, students choosing hostel vs. native
  address, moving house, corrections, lost/damaged EPIC, e-EPIC download,
  NRI (overseas) electors, service voters (armed forces), PwD marking, and
  reporting incorrect/duplicate entries.
- A **curated, cited knowledge base** (`knowledge-base/`) — every terminal
  outcome in the decision tree links to real ECI/SVEEP/PIB sources.
- A **web app MVP** (`apps/web`, Next.js) — the decision flow end-to-end in
  English and Hindi, plus a knowledge-base browser, with a persistent
  "we are not the ECI" banner and official deep-links opening in a new tab.
- The full **documentation suite** (`docs/01` through `docs/19` plus
  `docs/citations.md`) — PRD, personas, journeys, legal/compliance review,
  architecture, security threat model, test plan, roadmap, etc.

Not yet built (see `docs/19-roadmap.md`): the other 20 Eighth Schedule
languages, WhatsApp/Telegram/IVR channels, the `packages/api` REST layer
(the OpenAPI contract in `openapi/voteassist-api.yaml` describes the
intended shape), state-by-state legal review, and full WCAG AA certification.

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
