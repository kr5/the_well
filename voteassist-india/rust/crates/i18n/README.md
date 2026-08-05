# `i18n`

Locale metadata (BCP-47 code, English/native name, script, text direction,
CSS font stack, shipped/planned status) and the UI-chrome string
dictionary for VoteAssist India's 15-language priority set:

English, Hindi, Bengali, Tamil, Telugu, Marathi, Gujarati, Kannada,
Malayalam, Punjabi, Urdu, Odia, Assamese, Maithili, Nepali.

This is a deliberately narrower set than the full 22-language Eighth
Schedule roadmap tracked in `packages/i18n/src/locales.ts` and
`docs/11-multilingual-strategy.md` — see [`src/locale.rs`](src/locale.rs)'s
module doc comment for exactly how the two relate.

## wasm32-safe by design

`crates/web-app` (the Leptos SSR app) depends on this crate for both its
server binary and its client-side `wasm32-unknown-unknown` bundle. That
means **no `tokio`, no `sqlx`, no filesystem or network access at
runtime** — every string this crate serves is embedded into the binary at
compile time via `include_str!`, the same pattern `crates/kb-content` uses
for knowledge-base content. See `Cargo.toml`: the only dependencies are
`serde`/`serde_json`, both of which work fine compiled to wasm.

## UI chrome vs. content — read this before touching `status`

This crate's `LocaleStatus::Shipped`/`Planned` flag describes **only**
whether a locale's UI-chrome strings (`strings/<code>.json` — button
labels, nav items, generic errors) are translated and human-reviewed for
clarity. It says **nothing** about whether the knowledge base or
decision-tree content — the actual procedural/legal guidance — is
translated for that locale. Those ship on their own, separately-gated
review process (`docs/11-multilingual-strategy.md` Section 4), tracked in
`crates/kb-content`, not here.

Concretely: a locale can legitimately be `Shipped` in this crate (its
buttons say the right thing in the right script) while every knowledge-base
entry a citizen would actually read is still English-only. Do not treat
"the language switcher offers Punjabi" as "Punjabi speakers get full
Punjabi guidance" — check `kb-content`'s per-locale coverage for that.

As of this writing, only `en` and `hi` are `Shipped` here, matching
`packages/i18n/src/locales.ts`'s MVP scope. The other 13 are `Planned`:
their `strings/<code>.json` files exist (or will) with the full key set,
but haven't cleared the human-review step yet.

## Public API

- `locale::LOCALES` / `all_locales()` — the full 15-entry table.
- `locale::shipped_locales()` — only the locales a citizen-facing language
  switcher should actually offer.
- `locale::resolve(tag)` — turns a raw BCP-47-ish tag (`"hi-IN"`,
  `"pa_IN"`, `"xx-unknown"`) into the best-matching `&'static Locale`,
  always falling back to English rather than failing.
- `Locale::css_font_stack()` / `Script::css_font_stack()` — a CSS
  `font-family` value for the locale's script (Noto Sans/Nastaliq first,
  then an OS-bundled fallback or two, then a generic keyword).
- `Direction::as_attr()` — `"ltr"` / `"rtl"` for the HTML `dir` attribute.
  Urdu is the only `Rtl` locale in this table.
- `strings::t(locale_code, key)` — looks up a UI string, falling back to
  English on a missing locale or missing key. Never panics, never returns
  a blank string.
- `strings::UI_STRING_KEYS` — the fixed 43-entry key contract every
  `strings/<code>.json` file must fully cover.

## Adding a 16th language

1. Add an entry to `locale::LOCALES` in `src/locale.rs`: BCP-47 code,
   English name, **native name in native script** (don't guess — get it
   from a fluent speaker or a source like CLDR), `Script` (add a new
   `Script` variant first if the language uses one not already listed),
   `Direction` (RTL if it's Urdu/Sindhi/Kashmiri-like; LTR otherwise), and
   `LocaleStatus::Planned` (never `Shipped` on day one).
2. Add `strings/<code>.json` with all 43 keys from `UI_STRING_KEYS`,
   translated. `tests::en_json_matches_ui_string_keys` only enforces this
   for `en` (the source-of-truth file this crate owns), but every other
   locale file is expected to match the same key set — a partially-keyed
   file just means `t()` silently falls back to English for the missing
   keys, which is safe but not actually shipping that language's UI.
3. Add the new `("<code>", locale_json!("<code>"))` line to `RAW_STRINGS`
   in `src/strings.rs` — this is the one place the crate wires a new
   locale's JSON file into the compiled binary.
4. Get the translation human-reviewed for clarity
   (`docs/11-multilingual-strategy.md` Section 4.1), *then* flip
   `LocaleStatus::Planned` to `Shipped` — not before. That flip is a
   promise to citizens, not a code milestone.
5. Separately, and on its own timeline: translate and review the
   knowledge base / decision-tree content for that language in
   `crates/kb-content`. That's a bigger, legally-sensitive effort and is
   not blocked by, or a blocker for, this crate's UI-chrome work.
