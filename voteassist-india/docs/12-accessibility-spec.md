# Accessibility Specification

## 1. Baseline: WCAG 2.2 Level AA

VoteAssist India's MVP accessibility baseline is **WCAG 2.2 Level AA**,
applied across the whole product (not just marketing pages).

A note on AAA: this document deliberately does **not** claim blanket WCAG
AAA compliance. AAA contains success criteria that are frequently
infeasible to satisfy universally across an entire product (for example,
some AAA contrast and sign-language criteria are appropriately treated as
per-criterion, context-dependent goals rather than an all-or-nothing
target). Where a specific AAA criterion is both feasible and valuable for
this product's mission (for example, a stronger contrast ratio, or the
sign-language video roadmap item in section 9), we adopt it explicitly and
say so — rather than asserting "AAA" as a blanket label that would not
hold up under audit. This honesty matters here specifically because the
product's credibility depends on not overclaiming, consistent with the
citation discipline in `06-legal-compliance-review.md`.

## 2. Specific MVP Requirements

### 2.1 Keyboard Navigation

- Every interactive element (question answer options, buttons, language
  switcher, accessibility toolbar, deep-link CTAs) must be reachable and
  operable via keyboard alone, in a logical tab order matching visual
  order.
- Visible focus indicators on all interactive elements (not suppressed via
  `outline: none` without an equally visible replacement).
- No keyboard traps in the decision-engine flow, including any modal
  (e.g., "why we ask this" expander).

### 2.2 Screen Reader Labeling for the Decision Tree

- Each question node is a properly labeled form region: the question text
  is the accessible name of the answer-option group (e.g., via
  `fieldset`/`legend` or `aria-labelledby`), and each answer option has a
  clear, unambiguous accessible name (not just an icon or single word
  where context is needed).
- Progress indicator changes are announced via an `aria-live="polite"`
  region so screen reader users know they've advanced, without being
  interrupted mid-navigation.
- Citation badges expose their distinction (official source vs. community-
  guidance-pending-verification) in the accessible name/description, not
  just via color/icon (see `07-design-system.md`).
- The non-affiliation banner and verification-caution boilerplate must be
  reachable and readable by screen readers, not visually-present-only
  decoration.

### 2.3 Color Contrast

- Minimum 4.5:1 contrast for normal body text, 3:1 for large text and UI
  component boundaries, per WCAG 2.2 AA 1.4.3/1.4.11.
- The chosen civic blue/teal palette (`07-design-system.md`) must be
  contrast-checked against both light and dark backgrounds before
  finalizing exact tokens; contrast requirements take precedence over
  exact hue preference if a conflict arises.
- Citation-badge and caution-banner color coding must not be the only
  differentiator (icon/text label always accompanies color, for color-
  blind users).

### 2.4 Dyslexia-Friendly Font Toggle

- Accessibility toolbar includes a toggle to switch body text to a
  dyslexia-friendly font option (e.g., a font designed/commonly recommended
  for dyslexia readability); persists via local preference storage, no
  account required.

### 2.5 Text Resizing

- All text must remain legible and layout must not break up to 200% browser
  zoom (WCAG 2.2 AA 1.4.4), using relative units (rem/em) throughout
  rather than fixed pixel text sizing.
- Accessibility toolbar additionally offers an in-app text-size stepper for
  users who prefer not to use browser zoom.

### 2.6 Reduced Motion

- Respect `prefers-reduced-motion` at the OS/browser level by default; the
  accessibility toolbar also offers an explicit in-app override in case a
  user's OS-level setting doesn't reach the browser context.
- Progress transitions, fade-ins, and any decorative animation must have a
  reduced/near-instant equivalent when this preference is active.

### 2.7 Offline / Low-Bandwidth Mode

- Given target personas in remote/tribal/low-connectivity areas (see
  `02-personas.md` persona 13), the web app should degrade gracefully on
  slow connections: aggressive asset optimization, text-first rendering,
  and a lightweight "data saver" mode that defers non-essential images/
  fonts.
- Full offline-first PWA behavior (installable, works with no connection,
  syncs when reconnected) is a v1 roadmap item, not MVP (see
  `19-roadmap.md`) — MVP's commitment is "degrades well on poor
  connections," not "works fully offline."

## 3. Roadmap Items (not MVP)

- **Voice navigation**: voice input for answering decision-engine
  questions and voice output (text-to-speech) for reading KB content and
  questions aloud — tracked alongside the multilingual voice-mode roadmap
  in `11-multilingual-strategy.md` and the future IVR channel in
  `13-technical-architecture.md`.
- **Sign-language video**: Indian Sign Language (ISL) video explainers for
  key flows and KB entries — a v3-horizon goal (see `19-roadmap.md`),
  explicitly named here as an aspirational AAA-adjacent commitment rather
  than an MVP requirement, given the production resources it requires.

## 4. Testing and Verification

- Automated accessibility testing (axe or equivalent) runs in CI on every
  PR (see `17-test-plan.md` and `15-cicd.md`).
- Automated tooling catches a meaningful subset of issues but not all —
  periodic manual testing with actual screen readers (e.g., NVDA/VoiceOver)
  and, where feasible, testing with users who have disabilities, is part
  of the pre-release checklist, not a one-time launch activity.
