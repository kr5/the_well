# Multilingual Strategy

## 1. Target Language Set

VoteAssist India's long-term goal is coverage of English plus all
languages listed in the Eighth Schedule to the Constitution of India:

1. Assamese
2. Bengali
3. Bodo
4. Dogri
5. Gujarati
6. Hindi
7. Kannada
8. Kashmiri
9. Konkani
10. Maithili
11. Malayalam
12. Manipuri
13. Marathi
14. Nepali
15. Odia
16. Punjabi
17. Sanskrit
18. Santali
19. Sindhi
20. Tamil
21. Telugu
22. Urdu

Plus English (not part of the Eighth Schedule but the effective default/
interlingua for content authoring and review).

This is a v3-horizon goal (see `19-roadmap.md`). MVP ships English and
Hindi only, both fully human-reviewed. No other language is exposed in the
production language switcher until it passes the review gate described in
section 4.

## 2. "Easy/Simple" Reading-Level Variants

Independent of language, selected high-traffic KB entries and decision-tree
terminal outcomes should offer an "Easy" reading-level variant (see
`reading_level_variant` in `09-knowledge-base-schema.md`): shorter
sentences, more everyday vocabulary, more inline explanation of terms like
"AC" or "EPIC," and typically more visual/checklist structure vs. prose.
Easy variants go through the same citation/review requirements as standard
variants — simplifying language must never simplify away accuracy.

## 3. Voice Mode

Roadmap item (see `19-roadmap.md`, v3): text-to-speech playback of KB
content and decision-engine questions, and eventually voice input for
answering questions, aimed at users with low literacy or visual
impairment, and as a foundation for a future IVR channel
(`13-technical-architecture.md`). MVP does not include voice mode;
building the i18n content pipeline (section 5) in a structured,
per-string-keyed way now is what makes voice mode and IVR additive later
rather than requiring a content rearchitecture.

## 4. Translation and Review Workflow

Two categories of text are treated very differently:

### 4.1 UI strings (non-legal, non-procedural)

Examples: button labels, navigation labels, generic empty-states, error
messages.

- May use machine translation as a first draft.
- Must have human review before publishing to a new language, but the bar
  is fluency/clarity, not legal accuracy (since there's no legal claim in
  a button label).
- Tracked via a translation memory system so repeated strings (e.g.,
  "Continue," "Back," "Language") are translated once and reused
  consistently.

### 4.2 Legal / procedural content (KB entries, decision-tree node text,
terminal outcomes, citations, disclaimers)

- **Never machine-translated-and-shipped without human review.** Machine
  translation may be used only as a drafting aid for a human reviewer who
  is fluent in the target language and who cross-checks the translated
  claim against the same official source cited in the English/Hindi
  original — the translation is a re-expression of the same verified
  claim, not a new claim.
- Every translated legal/procedural entry gets its own `review_status`
  and `last_verified_date` (per `09-knowledge-base-schema.md`) — a
  translation is not "published" merely because the source-language
  original is published.
- A translation whose source entry is updated (new `version`) is
  automatically flagged `needs_reverification` until a reviewer confirms
  the translation still matches.

## 5. i18n Key and Content Architecture

> **Rust-platform update.** The authoritative implementation is now
> `rust/crates/i18n`, not `packages/i18n` (which remains the TypeScript
> prototype's version — see `rust/README.md` on the two build
> generations). The Rust crate carries strictly more than the TS one:
> per-locale **script** and **text direction** (Urdu is right-to-left,
> and the web app sets `dir="rtl"` from this rather than hardcoding a
> language list), and a per-script **CSS font stack**, because Indic
> scripts do not render at all on many systems without an explicitly
> named font family. The crate is deliberately wasm32-safe and zero-I/O
> (strings are `include_str!`'d at compile time, same discipline as
> `kb-content`), so the Leptos client build can use it directly.
>
> **Fallback is to English, never to a raw key.** A missing translation
> renders the English string. For a civic-information tool, an English
> label a reader can at least ask someone about beats a blank or a
> `nav.start`-looking token.
>
> **UI chrome and knowledge-base content ship independently.** A locale
> having complete UI strings does NOT mean its KB entries are
> translated or reviewed — the crate tracks those as separate states,
> and the language switcher gates on the stricter one (see §6.4 below).

- `packages/i18n` owns UI string keys (namespaced, e.g.,
  `start.question.residence_type.label`), following standard i18n key
  conventions; no hardcoded user-facing strings anywhere in
  `apps/web`/`packages/ui`.
- Legal/procedural content (KB entries, decision-tree copy) is NOT stored
  as i18n string-table keys — it's stored as full `knowledge_base_entries`
  records per language (see `09-knowledge-base-schema.md`), because it
  needs its own citation, versioning, and review-status metadata that a
  flat translation-key/value pair can't carry.
- Automated i18n completeness tests (see `17-test-plan.md`) check that
  every UI string key has a value in every *exposed* language (a language
  not yet exposed in the switcher is allowed to be incomplete, but a
  language that IS exposed must be 100% complete for UI strings — partial
  UI translation is worse than none, since it looks broken/unfinished).

## 6. Community Contribution Process (for languages beyond English/Hindi)

1. A contributor (community translator, potentially with domain
   knowledge in a given state/language) proposes a translation of a KB
   entry or UI string batch via a PR, following `CONTRIBUTING.md`
   guidance (repo governance doc, tracked as a follow-up per
   `06-legal-compliance-review.md`).
2. UI strings: reviewed by any fluent speaker for clarity; merged once
   i18n completeness tests pass.
3. Legal/procedural content: reviewed by (a) a fluent speaker AND (b) the
   Compliance Owner or delegate, who confirms the translation doesn't
   drift from the cited source's meaning; only then does `review_status`
   move to `published` for that language variant.
4. A language only appears in the production language switcher once it
   has a defined minimum coverage threshold of published (not draft)
   legal/procedural entries for the MVP scenario set in
   `04-decision-tree-spec.md` — partial coverage stays behind a
   "preview/beta languages" flag, not in the default switcher, to avoid
   presenting incomplete guidance as if it were complete.

## 7. MVP Statement

MVP ships English and Hindi, both fully reviewed for all decision-tree
scenarios and KB entries in scope for `04-decision-tree-spec.md`. All other
Eighth Schedule languages are roadmap items (see `19-roadmap.md`) with an
open, documented contribution path from day one, so community translation
work can begin well before each language is exposed to end users.

### 7.1 Current state (Rust platform)

The picture is now more granular than "English and Hindi only," and the
distinction matters because the two layers gate differently:

| Layer | Coverage today | Gate to ship a language |
|---|---|---|
| **UI chrome** (`rust/crates/i18n`) | 15 languages: en, hi, bn, ta, te, mr, gu, kn, ml, pa, ur, or, as, mai, ne | All 40 string keys present; any fluent speaker can review — this is navigation and button copy, not legal text |
| **Knowledge-base entries + decision-tree text** | en, hi | Citation-verified per `06-legal-compliance-review.md` §7, by a fluent speaker **and** the Compliance Owner |

Translated UI chrome alone does **not** put a language in the public
switcher. That gate is the content layer, and it is deliberately the
stricter one: a citizen who picks their language and then reads
procedural guidance in English has been failed less badly than one who
reads confidently-worded machine-translated procedure that nobody
verified.

The machine-translation pipeline for the content layer
(`rust/crates/xtask`'s `translate-kb-entry`/`translate-tree`, backed by
either Claude Haiku or NVIDIA NIM's Nemotron models) produces **drafts
only**, written to `translation-drafts/` — never directly into
`knowledge-base/sources/`. See `20-translation-task-tracker.md` for the
per-language review checklist that turns a draft into shipped content.
