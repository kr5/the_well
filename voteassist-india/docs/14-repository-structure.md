# Repository Structure

> **NOTE**: VoteAssist India currently lives temporarily inside the
> `the_well` repository, under the top-level `voteassist-india/` folder,
> purely as a scaffolding convenience. `the_well` is an unrelated ML
> dataset project; this location is not permanent. VoteAssist India will
> be extracted into its own dedicated repository before any public
> release, preview deployment, or external contribution is solicited.
> Nothing here should be taken as an indication that VoteAssist India is
> part of, derived from, or affiliated with the_well project.

## 1. Folder Tree (as of this writing)

```
voteassist-india/
├── apps/
│   └── web/                       Next.js web application (apps/web/app router)
│       ├── app/
│       └── public/
├── packages/
│   ├── decision-engine/           Pure, versioned decision-tree logic (see 04-decision-tree-spec.md, 13-technical-architecture.md)
│   │   ├── src/
│   │   ├── test/
│   │   ├── package.json
│   │   ├── tsconfig.json
│   │   └── vitest.config.ts
│   ├── knowledge/                 Knowledge base domain logic (see 09-knowledge-base-schema.md)
│   │   ├── src/
│   │   ├── package.json
│   │   └── tsconfig.json
│   ├── i18n/                      UI string catalogs / locale handling (see 11-multilingual-strategy.md)
│   │   ├── src/
│   │   │   └── locales/
│   │   ├── package.json
│   │   └── tsconfig.json
│   └── ui/                        Shared accessible component library (see 07-design-system.md)
│       └── src/
├── knowledge-base/                Curated KB source content, versioned as data
│   ├── schema/
│   │   └── entry.schema.json      Machine-readable copy of 09-knowledge-base-schema.md
│   └── sources/                   One file per topic (form-6, form-6a, form-7, form-8,
│                                   e-epic, qualifying-dates, ordinary-residence-student,
│                                   pwd-home-voting, service-voter, helpline-grievance,
│                                   roll-search-polling-station, ...)
├── openapi/                       Formal OpenAPI definition (voteassist-api.yaml, see 10-api-specification.md)
├── docs/                          This documentation suite
├── package.json                   Workspace root
├── pnpm-workspace.yaml            pnpm workspace definition (apps/*, packages/*)
├── tsconfig.base.json             Shared TypeScript config
└── pnpm-lock.yaml
```

Not yet present, anticipated as the build progresses (see `19-roadmap.md`
and `10-api-specification.md`):

```
packages/
├── forms/                         Form metadata/helpers (Form 6/6A/7/8/12D) - referenced in 13-technical-architecture.md, not yet scaffolded
├── search/                        Full-text KB search - referenced in 10-api-specification.md, not yet scaffolded
└── api/                           HTTP API layer - referenced in 10-api-specification.md, not yet scaffolded
```

## 2. Rationale for This Layout

- **pnpm workspace monorepo** (`pnpm-workspace.yaml` scoping `apps/*` and
  `packages/*`) so that `apps/web` and any future channel app can share
  `packages/decision-engine`, `packages/knowledge`, `packages/i18n`, and
  `packages/ui` without publishing to a registry during development.
- **`packages/decision-engine` is intentionally isolated** with its own
  `package.json`, `tsconfig.json`, and `vitest.config.ts` — it has no
  dependency on Next.js, React, or any HTTP framework, consistent with the
  "pure, versioned, testable" architectural commitment in
  `13-technical-architecture.md`.
- **`knowledge-base/` is data, not code.** Curated source content (one
  file per topic under `knowledge-base/sources/`, validated against
  `knowledge-base/schema/entry.schema.json`) is kept separate from
  `packages/knowledge`, which contains the *logic* for loading, validating,
  and querying that data. This split lets content contributors (who may
  not be engineers) work in `knowledge-base/` via the review process in
  `06-legal-compliance-review.md` without touching application code.
- **`openapi/` is a single source of truth for the wire contract**,
  separate from the prose description in `10-api-specification.md` — the
  prose explains intent and rationale; the YAML is the enforceable
  contract consumed by codegen/validation tooling.
- **`docs/` (this folder) is the product/eng working-doc suite**, separate
  from any future user-facing `/learn` content — these are internal
  planning documents, not published knowledge base entries.

## 3. Migration Plan (When Extracted to Its Own Repo)

1. `git filter-repo` (or equivalent) to extract `voteassist-india/` history
   with paths rewritten to repo root.
2. Re-point CI (`15-cicd.md`) at the new repo's default branch.
3. Update `GOVERNANCE.md`/`CONTRIBUTING.md` (to be authored) to reflect the
   new repo location before inviting external contributors.
4. Until extraction happens, anything referencing "the repo" in these docs
   means `voteassist-india/` within `the_well`, not `the_well` itself.
