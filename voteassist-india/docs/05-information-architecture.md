# Information Architecture

Sitemap for the VoteAssist India web app (`apps/web`). Structure favors a
single strong entry point (the decision engine) over a sprawling menu, on
the theory that most users arrive with one specific problem to solve.

## 1. Top-Level Sitemap

```
/ (Home)
├── /start                      Decision engine entry ("What do you need help with?")
│   └── /start/q/:nodeId         Individual question steps (client-routed, not deep-linkable mid-flow by default)
│   └── /start/result/:sessionId Result / checklist screen
├── /search                     "Find my status" — roll search guidance + KB search combined
├── /learn                      Knowledge base browser
│   ├── /learn/forms            Form-by-form explainers (Form 6, 6A, 7, 8, 12D)
│   ├── /learn/faq              FAQ entries
│   ├── /learn/glossary         Glossary (AC, PC, EPIC, BLO, ERO, DEO, CEO, SVEEP, etc.)
│   └── /learn/:slug            Individual KB entry page
├── /locate                     Polling station / ERO / BLO / CEO locator (deep-link hub)
├── /about
│   ├── /about/what-this-is     "We are not the ECI" explainer (mission + non-affiliation)
│   ├── /about/legal            Terms, disclaimers
│   ├── /about/privacy          Privacy policy
│   └── /about/open-source      Repo link, contribution info, governance
├── /feedback                   Feedback / report-an-issue form
├── /accessibility              Accessibility statement + settings shortcut
└── /lang/:code                 Language switch handler (redirects back to prior page in new language)
```

## 2. Home / Entry (`/`)

Primary components (see `07-design-system.md` for the component inventory):

- Non-affiliation banner (persistent, not dismissible past a minimized
  state) — "VoteAssist India is independent guidance, not the official ECI
  site."
- Primary call to action: "Start" → `/start` (decision engine).
- Secondary entry: "Search the knowledge base" → `/search`.
- Tertiary entry: "Find my polling station / BLO" → `/locate`.
- Language switcher (top-level, always visible).
- Accessibility toolbar toggle (always visible).

## 3. Question Flow (`/start/...`)

- One question per screen; progress indicator shows position, not a
  numeric "step X of Y" (tree length varies by path, so a numeric total
  would mislead — use a qualitative/segmented indicator instead).
- Back navigation always available; answers are held in-session, not
  persisted server-side beyond an anonymous session log used for
  aggregate analytics (see `18-analytics-plan.md`).
- Each question may have an optional "why are you asking this" expandable
  explainer.
- No question node may be skipped by deep link — mid-flow states are
  reconstructable only by replaying prior answers, to avoid encouraging
  people to share/bookmark a URL that implies a specific personal
  situation.

## 4. Results / Checklist Screen (`/start/result/:sessionId`)

Displays, per the terminal outcome contract in `04-decision-tree-spec.md`:

- Recommended form/action, in plain language first, form number second.
- Document checklist (general categories, checkable off).
- Citation badges (see `07-design-system.md`) linking to source text.
- A single prominent deep-link button to the relevant official destination
  (voters.eci.gov.in / ECINET / Voter Helpline app / CEO portal / 1950).
- The verification-caution boilerplate.
- "This didn't match my situation" feedback link → `/feedback`.
- Optional: "email/save this checklist to yourself" (no server-side
  storage of personal identifiers beyond what's needed to send the email,
  and only on explicit request — see `08-database-schema.md`).

## 5. Knowledge Base Browser (`/learn`)

- Forms section: one explainer page per form (6, 6A, 7, 8, 12D) — what
  it's for, who uses it, plain-language summary, citation.
- FAQ section: sourced from ECI/SVEEP FAQs plus common decision-engine
  feedback themes.
- Glossary: AC (Assembly Constituency), PC (Parliamentary Constituency),
  EPIC (Elector's Photo Identity Card), BLO (Booth Level Officer), ERO
  (Electoral Registration Officer), DEO (District Election Officer), CEO
  (Chief Electoral Officer), SVEEP, ECINET, NVSP (historical, superseded),
  qualifying date, ordinary residence, etc.
- Every KB entry page shows `source_type`, `source_url`, and
  `last_verified_date` per the schema in `09-knowledge-base-schema.md`.

## 6. Locate (`/locate`)

- A deep-link hub, not a self-hosted locator database in MVP: cards for
  "Find my polling station" (links to voters.eci.gov.in), "Find my ERO/
  BLO" (Voter Helpline app "Book-a-Call with BLO" + CEO portal), "Find my
  CEO's office" (per-state links; content task, not engineering task).
- MVP explicitly does not host its own polling-station database — see
  `01-prd.md` open questions on whether any legitimate ECI data feed
  exists for this.

## 7. Language Switcher

- Present in the header on every page.
- MVP: English, Hindi (fully reviewed). Additional languages appear in the
  switcher only once they pass the review gate in
  `11-multilingual-strategy.md`; partially translated languages are not
  exposed in production.

## 8. Accessibility Settings (`/accessibility`)

- Text size control, dyslexia-friendly font toggle, reduced-motion toggle,
  high-contrast toggle, screen-reader usage notes. See
  `12-accessibility-spec.md`.

## 9. About / Legal / Privacy (`/about/...`)

- `/about/what-this-is`: the core non-affiliation statement, mission,
  "who runs this," and a clear link back to voters.eci.gov.in for anything
  official.
- `/about/legal`: terms of use, liability disclaimer.
- `/about/privacy`: privacy policy per DPDP Act 2023 principles (see
  `06-legal-compliance-review.md`).
- `/about/open-source`: repository link, CONTRIBUTING/GOVERNANCE pointers,
  compliance-owner contact.

## 10. Feedback (`/feedback`)

- General feedback form and a "report inaccurate information" flow scoped
  to a specific KB entry or terminal outcome (carries an entry ID so
  reports route to the right content owner for review).
