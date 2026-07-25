# Translation Task Tracker

Tracks machine-translation-draft progress per Eighth Schedule language, per
`docs/11-multilingual-strategy.md`'s locale list (`packages/i18n/src/locales.ts`).
English and Hindi are already fully authored/reviewed and shipped — this
tracker is for the 22 remaining "planned" locales.

## Pipeline

`scripts/translate-language.sh <locale-code>` runs the full pipeline for
one language: every `knowledge-base/sources/*.json` entry, plus both
`tree_v1.json`/`tree_v2.json`, translated via either Claude Haiku or
NVIDIA NIM's free-tier Nemotron models
(`rust/crates/xtask/src/translate/`; see `client.rs` for how the backend
is picked). Everything it produces is a **draft**, written to
`translation-drafts/<locale>/` — never to `knowledge-base/sources/` or
`rust/crates/core-domain/src/*.json` directly. See
`rust/crates/xtask/src/translate/mod.rs`'s module doc for why: nothing in
this pipeline auto-publishes, matching the project-wide
"MT-assist-as-draft-only" rule (PRD v2 Section 11 admin page 5).

Which backend actually ran a given draft is not currently recorded
anywhere the reviewer sees — the review process in Step 2 below (verifying
every procedural claim against a citation) applies identically regardless
of which model drafted the wording, since MT output is never trusted on
its own merits either way.

## Workflow, per language

1. Run `scripts/translate-language.sh <code>` (needs `ANTHROPIC_API_KEY` or
   `NVIDIA_API_KEY` — see `rust/crates/xtask/src/translate/client.rs`).
2. A qualified reviewer checks every KB entry draft against the same
   citation-verification process as any other content change
   (`docs/06-legal-compliance-review.md` Section 7) — machine translation
   does not exempt a change from that process, it only drafts the starting
   text.
3. For KB entries: move the reviewed file from `translation-drafts/<code>/`
   into `knowledge-base/sources/`, and flip `reviewStatus` from `"draft"`
   to `"in_review"` or `"verified"` per the normal state machine.
4. For the decision trees: diff `translation-drafts/<code>/tree_v{1,2}-<code>-draft.json`
   against the real `tree_v{1,2}.json` (every change is exactly one new
   locale key per `LocalizedText` object) and manually merge the reviewed
   strings into the real file.
5. Once **both** a language's KB entries and decision-tree text are
   reviewed and merged, flip that language's `status` from `"planned"` to
   `"shipped"` in `packages/i18n/src/locales.ts` — per that file's own
   comment, that's the single gate controlling whether the language
   switcher (`crates/web-app`) exposes it publicly. Do not flip it early:
   a half-translated language showing up in the switcher is worse than
   the language not appearing yet.

## Status

| Locale | Language | KB entries drafted | Trees drafted | Reviewed & merged | Shipped |
|---|---|---|---|---|---|
| en | English | n/a (source) | n/a (source) | yes | **yes** |
| hi | Hindi | n/a (authored directly) | yes (embedded) | yes | **yes** |
| bn | Bengali | not started | not started | not started | no |
| ta | Tamil | not started | not started | not started | no |
| te | Telugu | not started | not started | not started | no |
| mr | Marathi | not started | not started | not started | no |
| gu | Gujarati | not started | not started | not started | no |
| kn | Kannada | not started | not started | not started | no |
| ml | Malayalam | not started | not started | not started | no |
| pa | Punjabi | not started | not started | not started | no |
| ur | Urdu | not started | not started | not started | no |
| or | Odia | not started | not started | not started | no |
| as | Assamese | not started | not started | not started | no |
| kok | Konkani | not started | not started | not started | no |
| mni | Manipuri | not started | not started | not started | no |
| doi | Dogri | not started | not started | not started | no |
| brx | Bodo | not started | not started | not started | no |
| sat | Santali | not started | not started | not started | no |
| mai | Maithili | not started | not started | not started | no |
| sd | Sindhi | not started | not started | not started | no |
| ne | Nepali | not started | not started | not started | no |
| ks | Kashmiri | not started | not started | not started | no |
| sa | Sanskrit | not started | not started | not started | no |

Update this table as each language moves through the workflow above —
it's the single source of truth for "what's actually been reviewed," not
just "what's been machine-drafted."

## Prioritization note

Not a strict order, but a reasonable starting point given speaker
population and ECI's own multi-language rollout priority: Bengali, Telugu,
Marathi, Tamil, Urdu, Gujarati, Kannada, Odia, Malayalam, Punjabi cover the
large majority of non-Hindi-non-English speakers and are a sensible batch
to run through the pipeline (and, more importantly, find a reviewer for)
before the remaining, smaller-population languages.
