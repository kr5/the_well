# Test Plan

## 1. Unit Tests for Decision-Engine Logic

`packages/decision-engine` is the highest-scrutiny package in the codebase
(see `13-technical-architecture.md`). Testing requirements:

- **Deterministic unit tests** for every node's transition logic (given
  node ID + answer, assert the exact next node or terminal outcome).
- **Property-based tests** (e.g., via `fast-check` or equivalent) asserting
  invariants that must hold across the *entire* tree, not just specific
  paths:
  - **Termination**: every reachable sequence of valid answers reaches a
    `terminal` node within a bounded number of steps (no cycles, no
    infinite branches).
  - **Citation completeness**: every `terminal` node's outcome payload
    includes at least one citation, OR is explicitly labeled
    `source_type: community_pending_verification` — no terminal node ships
    with neither.
  - **No orphan nodes**: every node defined in the tree is reachable from
    the root via at least one path (dead content is a maintenance smell
    and a sign of a spec/implementation drift).
  - **Answer-set validity**: every `question` node's defined answer options
    map to a valid next node/edge — no dangling edges.
  - **No sensitive-field capture**: static analysis / schema check
    confirming no node definition includes a field resembling caste,
    religion, or party-affiliation collection (defense in depth alongside
    the schema constraints in `08-database-schema.md`).
- Version-pinned fixture tests: for each shipped `decision_tree_version`,
  a frozen set of "golden path" input->output fixtures that must not
  silently change without a deliberate version bump — protects against
  accidental regressions to already-reviewed guidance.

## 2. i18n Completeness Tests

- For every language exposed in the production language switcher (MVP:
  English, Hindi), assert 100% of `packages/i18n` UI string keys have a
  non-empty value — a missing key fails the build, per
  `11-multilingual-strategy.md`.
- For KB content, assert every `published` entry in one exposed language
  has either a `published` translation in every other exposed language, or
  an explicit, intentional fallback marker — no silent English-only gaps
  in a KB section that's supposed to be fully bilingual for MVP.
- Placeholder/interpolation consistency: if a string contains variables
  (e.g., `{{qualifying_date}}`), assert every language's translation
  contains the same variable set (prevents broken interpolation in a
  translated string).

## 3. Accessibility Automated Checks

- `axe-core` (via `@axe-core/playwright` or equivalent) run against every
  major page/flow state in CI (see `15-cicd.md`): home, decision-engine
  question screen, result/checklist screen, KB browser, locate page,
  accessibility settings page.
- Zero tolerance for axe "critical" and "serious" severity violations in
  CI; "moderate"/"minor" findings tracked as issues, not automatic build
  failures, but reviewed regularly.
- Manual testing supplement (not automatable): periodic screen-reader
  walkthroughs (NVDA, VoiceOver) of the full decision-engine flow, per
  `12-accessibility-spec.md` section 4 — scheduled at least once per
  minor release, not just at launch.

## 4. Playwright End-to-End Tests for Critical Journeys

Cover, at minimum, the eight journeys in `03-user-journeys.md`, end-to-end
through the actual UI (not mocked at the component level):

1. New 18-year-old checks qualifying date, reaches Form 6 outcome.
2. Student chooses hostel address, reaches Form 6 + bonafide-certificate
   outcome.
3. Registered user reports a cross-AC move, reaches Form 8 outcome.
4. NRI first-time registration reaches Form 6A outcome.
5. PwD user marks PwD status then requests home voting (Form 8 + Form
   12D sequential outcome).
6. User reports a lost EPIC and chooses e-EPIC download vs. physical
   replacement, both branches verified.
7. User requests a name correction, reaches Form 8 (correction) outcome.
8. User reports a deceased relative still on the roll, reaches Form 7
   outcome.

Each e2e test also asserts: the non-affiliation banner is present, the
result screen shows a citation badge, and the deep-link CTA points to an
allow-listed official domain (voters.eci.gov.in, ecinet.eci.gov.in, or a
configured CEO portal / helpline reference) — never anywhere else.

## 5. Visual Regression Testing

- Snapshot-based visual regression (e.g., Playwright's built-in screenshot
  comparison, or a dedicated tool) on key components from
  `07-design-system.md`'s inventory: question card, progress indicator,
  result checklist, citation badge (both variants), language switcher,
  accessibility toolbar.
- Run against both light and dark color-scheme preferences, and at
  standard + 200% zoom, given the text-resizing requirement in
  `12-accessibility-spec.md`.
- Deliberately includes a palette-compliance check: automated or manual
  confirmation that no build introduces saffron/green-as-primary-brand
  colors or party-symbol-adjacent iconography, tying back to
  `07-design-system.md` section 1.1.

## 6. Load / Performance Testing Plan

- Baseline load test (e.g., k6 or Artillery) against `packages/api`
  session-creation and answer-submission endpoints, sized to a plausible
  spike scenario (e.g., a surge in traffic around a well-publicized
  registration deadline or qualifying date).
- Lighthouse CI budget enforcement in every PR (see `15-cicd.md`) as the
  primary continuous performance gate; dedicated load testing reserved for
  pre-major-release checkpoints and before anticipated high-traffic
  periods (e.g., ahead of a state election).
- Explicit test for graceful degradation under load: confirm that if
  `packages/api` becomes slow/unavailable, `apps/web` still renders static
  KB content and the non-affiliation/disclaimer messaging (no full
  white-screen failure mode for read-only content).

## 7. Test Ownership and Cadence

- Unit/property/i18n/accessibility tests: run on every PR (see
  `15-cicd.md`), blocking merge on failure.
- E2E and visual regression: run on every PR touching `apps/web` or
  `packages/ui`; full suite also run nightly against the deployed
  preview/staging environment to catch environment-specific drift.
- Load testing: run before each minor/major release and ahead of known
  high-traffic periods, not on every PR (too slow/costly for that
  cadence).
