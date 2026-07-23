# Design System Principles

## 1. Visual Language: Neutral, Calm, Non-Partisan

### 1.1 Color

VoteAssist India deliberately avoids any palette that could be read as
aligned with a political party or movement in the Indian context. In
particular:

- **Saffron/orange** and **green** are avoided as primary or dominant
  brand colors — both carry strong political-party and national-symbol
  associations in India (and green/saffron together read as the national
  flag, which additionally raises the impersonation concerns in
  `06-legal-compliance-review.md`). They may appear only in small,
  incidental, non-branding contexts if ever required for a genuinely
  neutral purpose (e.g., a status "success" green used consistently with
  standard UI conventions, at low saturation, never as a large field of
  color).
- **Primary palette: civic blue/teal.** Blue reads broadly as
  institutional-neutral, calm, and trustworthy across Indian civic and
  government digital services generally (without being a specific party's
  signature color), and teal/blue-green variants soften the tone to avoid
  feeling like a bank or a police uniform. Recommended direction: a
  desaturated blue-teal (e.g., a mid-tone teal around #0F6E7A to #1B7F8C
  for primary actions, paired with a deep slate-blue for text/headers) —
  exact tokens to be finalized by a designer, but the hue family (blue
  through teal, avoiding pure cyan-tech or navy-corporate extremes) is the
  fixed constraint.
- Neutral grays for structure; a single, desaturated warm accent (e.g.,
  amber, used sparingly for warnings/cautions only, not branding) is
  acceptable since amber/warning-yellow does not carry the same
  party-coded association in this context.
- No party symbols, no imagery resembling ballot symbols (lotus, hand,
  broom, etc. — the standard ECI-allotted party symbols), not even in
  illustrative icons.

### 1.2 Typography

- Latin-script UI: a humanist sans-serif with strong readability at small
  sizes and good hinting (e.g., a font in the Inter/Noto Sans family
  class); avoid anything with a "techy"/futuristic feel that undercuts the
  calm, institutional-neutral tone.
- **Indic scripts**: use the Noto Sans family for each required script
  (Noto Sans Devanagari, Noto Sans Bengali, Noto Sans Tamil, Noto Sans
  Gujarati, Noto Sans Gurmukhi, Noto Sans Kannada, Noto Sans Malayalam,
  Noto Sans Oriya, Noto Sans Telugu, Noto Sans Ol Chiki for Santali, etc.)
  as the default cross-script-consistent choice, since Noto is explicitly
  designed for broad script coverage and pairs visually across languages
  on the same page (e.g., a language switcher showing multiple script
  names side by side). See `11-multilingual-strategy.md` for the full
  language list and rollout order.
- Minimum body text size 16px equivalent; line height >=1.5 for body text
  to support readability and the dyslexia-friendly mode (see
  `12-accessibility-spec.md`).
- No all-caps for body or long labels (harder to read, and can read as
  shouting in a civic-guidance context).

### 1.3 Tone Elements

- Iconography: simple, geometric, outline-style icons; no photographic
  imagery of crowds, rallies, or campaign-adjacent scenes; no ballot-box
  or voting-symbol iconography that could be misread as ballot-symbol
  imagery.
- Motion: minimal, purposeful (progress transitions, gentle fade-ins);
  respects reduced-motion preference (see `12-accessibility-spec.md`).

## 2. Component Inventory

| Component | Purpose | Key states / notes |
|---|---|---|
| Non-affiliation banner | Persistent statement that this is not an official ECI site | Minimizable but never fully dismissible; always one tap from full text |
| Question card | Presents one decision-tree question with answer options | Supports single-select buttons/cards; optional "why we ask" expander; keyboard/screen-reader accessible (see `12-accessibility-spec.md`) |
| Progress indicator | Shows position in the current flow | Segmented/qualitative, not a misleading "step X of Y" count (tree length varies by path) |
| Result checklist | Displays terminal outcome: recommended form, documents, deep-link | Checkable list items; prominent single primary deep-link CTA; citation badges attached to each claim |
| Citation badge | Small inline marker linking a specific claim to its source | Two visual variants: "Official source" (linked, with source name) vs. "Community guidance — pending verification" (visually distinct, e.g., dashed border/different icon) |
| Language switcher | Switch UI + content language | Shows only fully-reviewed languages in production; each option labeled in its own script and in English |
| Accessibility toolbar | Quick access to text size, contrast, motion, dyslexia-font toggles | Persistent, collapsible; settings persist locally (no account required) |
| Deep-link CTA | Sends the user to an official ECI/CEO resource | Always states destination explicitly (e.g., "Continue on voters.eci.gov.in") — never a bare "Continue" that obscures leaving the site |
| Caution/verify banner | Attached to terminal outcomes to remind users rules/portals change | Consistent boilerplate text (see `04-decision-tree-spec.md` section 3) |
| Feedback widget | Inline "was this helpful / report an issue" | Scoped to the specific KB entry or terminal outcome ID |

## 3. Tone of Voice Guidelines

- **Plain language target**: aim for roughly a lower-secondary reading
  level in English (comparable to a plain-language/Grade 8 equivalent) as
  a working target, recognizing exact reading-level scoring for Hindi and
  other Indic languages requires separate, language-appropriate methods
  (do not assume an English readability formula transfers directly — flag
  for each language's review process in `11-multilingual-strategy.md`).
- No unexplained jargon: every ECI/legal term used in user-facing copy
  (AC, PC, EPIC, BLO, ERO, DEO, CEO, SVEEP, "ordinary residence,"
  "qualifying date," etc.) must be either explained inline on first use or
  linked to the glossary (`/learn/glossary`).
- Second person, direct address ("you"), active voice, short sentences.
- Never alarmist, never urgency-manufacturing beyond genuinely
  time-sensitive facts (e.g., the real Form 12D 5-day window) — no dark
  patterns, no fake scarcity, no manufactured countdowns.
- Never political, never persuasive about voting itself beyond stating
  factual deadlines and processes — see `01-prd.md` non-goals and
  `06-legal-compliance-review.md` section 4.
- Consistent first-person plural avoided in favor of clear third-person
  framing about "ECI," "your BLO," etc., to avoid ever implying VoteAssist
  itself is the authority taking action.
