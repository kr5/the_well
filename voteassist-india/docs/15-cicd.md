# CI/CD Plan

## 1. Goals

- Every pull request is automatically checked for correctness (lint, type
  safety, unit/property tests), accessibility, and performance budget
  before it can merge.
- Decision-engine changes get extra scrutiny (property-based tests per
  `17-test-plan.md`) since they encode the product's core legal/procedural
  claims.
  Content changes (KB entries) are checked for schema validity and citation
  presence in the same pipeline.
- Dependencies are kept current via automated PRs, reviewed by a human, not
  auto-merged blindly given the compliance-sensitive nature of the product.

## 2. Pipeline Stages (GitHub Actions)

| Stage | Trigger | What it does |
|---|---|---|
| Lint | Every PR | ESLint/Prettier across `apps/` and `packages/` |
| Typecheck | Every PR | `tsc --noEmit` per package, using `tsconfig.base.json` |
| Unit + property tests | Every PR | `vitest` run across all packages, including `packages/decision-engine`'s property-based path/citation tests |
| KB schema validation | Every PR touching `knowledge-base/**` | Validates every `knowledge-base/sources/*.json` file against `knowledge-base/schema/entry.schema.json` |
| i18n completeness | Every PR touching `packages/i18n/**` or `knowledge-base/**` | Fails if any exposed language is missing a UI string key or a published-language KB entry falls out of sync (see `11-multilingual-strategy.md`) |
| Accessibility (axe) | Every PR touching `apps/web/**` or `packages/ui/**` | Automated axe-core scan against key pages/components |
| Playwright e2e | Every PR touching `apps/web/**` | Critical-journey tests (see `17-test-plan.md`) against a preview build |
| Lighthouse CI | Every PR touching `apps/web/**` | Performance/accessibility/best-practices/SEO budget; target score >95 on each category, build fails below threshold |
| Preview deployment | Every PR | Deploys `apps/web` to an ephemeral preview URL for manual review |
| Dependency updates | Scheduled (weekly) | Renovate or Dependabot opens PRs for dependency bumps, routed through the same checks above |

## 3. Example Workflow YAML

```yaml
# .github/workflows/ci.yml
name: CI

on:
  pull_request:
  push:
    branches: [main]

jobs:
  lint-typecheck-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm -r lint
      - run: pnpm -r typecheck
      - run: pnpm -r test -- --coverage

  kb-schema-validation:
    runs-on: ubuntu-latest
    needs: lint-typecheck-test
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm --filter @voteassist/knowledge run validate:kb-sources

  i18n-completeness:
    runs-on: ubuntu-latest
    needs: lint-typecheck-test
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm --filter @voteassist/i18n run check:completeness

  build-and-preview:
    runs-on: ubuntu-latest
    needs: lint-typecheck-test
    outputs:
      preview_url: ${{ steps.deploy.outputs.preview_url }}
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm --filter @voteassist/web build
      - name: Deploy preview
        id: deploy
        run: echo "preview_url=https://pr-${{ github.event.number }}.preview.example.org" >> "$GITHUB_OUTPUT"
        # Replace with actual hosting provider's deploy action (e.g. Vercel/Netlify/Cloudflare Pages)

  e2e:
    runs-on: ubuntu-latest
    needs: build-and-preview
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm exec playwright install --with-deps
      - run: pnpm --filter @voteassist/web test:e2e
        env:
          E2E_BASE_URL: ${{ needs.build-and-preview.outputs.preview_url }}

  accessibility:
    runs-on: ubuntu-latest
    needs: build-and-preview
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm --filter @voteassist/web test:axe
        env:
          E2E_BASE_URL: ${{ needs.build-and-preview.outputs.preview_url }}

  lighthouse:
    runs-on: ubuntu-latest
    needs: build-and-preview
    steps:
      - uses: actions/checkout@v4
      - uses: treosh/lighthouse-ci-action@v11
        with:
          urls: |
            ${{ needs.build-and-preview.outputs.preview_url }}
            ${{ needs.build-and-preview.outputs.preview_url }}/start
            ${{ needs.build-and-preview.outputs.preview_url }}/learn
          budgetPath: ./lighthouse-budget.json
          uploadArtifacts: true
          temporaryPublicStorage: true
```

Example Lighthouse budget (`lighthouse-budget.json`, referenced above):

```json
[
  {
    "path": "/*",
    "resourceCounts": [
      { "resourceType": "script", "budget": 20 },
      { "resourceType": "third-party", "budget": 0 }
    ]
  }
]
```

with a companion assertion (via `lighthouserc.json` or the action's
`assert` step) requiring performance, accessibility, best-practices, and
SEO category scores each >= 0.95, failing the build otherwise.

## 4. Dependency Updates

- Renovate (preferred over Dependabot for its grouping/scheduling
  flexibility, though either is acceptable) configured to:
  - Group minor/patch updates weekly to reduce PR noise.
  - Open major-version updates as separate, individually-reviewed PRs.
  - Never auto-merge without CI passing; given the compliance sensitivity
    of this product, human review of dependency PRs is required even
    when CI is green, at least for `apps/web` and `packages/api` once it
    exists.

## 5. Branch Protection

- `main` requires: all CI checks passing, at least one human review
  approval, and (for any PR touching `knowledge-base/**` or
  decision-tree terminal-node content) sign-off consistent with the
  content review process in `06-legal-compliance-review.md`.
