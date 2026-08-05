//! Locale metadata: script, text direction, font stack, shipped/planned
//! status, and locale negotiation.
//!
//! Native names are transcribed from `packages/i18n/src/locales.ts`
//! (`supportedLocales`), which is the authoritative TypeScript-side source
//! for this data during the migration window. This table is a *subset* of
//! that file: `locales.ts` tracks all 22 Eighth Schedule languages
//! (`docs/11-multilingual-strategy.md` Section 1) as the long-term v3
//! horizon, while this crate covers only the 15 languages this project has
//! prioritized for real translation work. If `locales.ts` and this table
//! ever disagree on a code both list (native name, English name, or
//! status), `locales.ts` wins and this table should be corrected to match.

/// A writing script used by at least one of this crate's 15 priority
/// languages. Deliberately narrower than Unicode's full script list — add
/// a variant only when a language actually using it is added to
/// [`LOCALES`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Script {
    Latin,
    Devanagari,
    Bengali,
    Tamil,
    Telugu,
    Gujarati,
    Kannada,
    Malayalam,
    Gurmukhi,
    Arabic,
    Odia,
}

impl Script {
    /// A CSS `font-family` stack for this script: a Noto Sans/Nastaliq
    /// family first (the one universally-available answer — Google's Noto
    /// project explicitly aims for full script coverage), then a
    /// commonly-bundled Windows font, then a commonly-bundled macOS font,
    /// then a generic keyword.
    ///
    /// These OS-bundled names are grounded in real, shipped system fonts
    /// (e.g. Windows' "Nirmala UI" covers most Indic scripts in one family;
    /// macOS ships script-specific "Kohinoor"/"Sangam MN" families), but
    /// exact availability varies by OS version and this crate has no way
    /// to verify it live — this sandbox cannot run a browser. Treat the
    /// OS fallbacks as best-effort, not a guarantee. The one reliable fix
    /// is for the web app to actually serve Noto Sans/Nastaliq as a
    /// `@font-face` web font rather than depend on any system font being
    /// present, which `web-app` should adopt as a follow-up.
    pub const fn css_font_stack(self) -> &'static str {
        match self {
            Script::Latin => {
                r#"system-ui, -apple-system, "Segoe UI", Roboto, Arial, sans-serif"#
            }
            Script::Devanagari => {
                r#""Noto Sans Devanagari", "Nirmala UI", "Kohinoor Devanagari", sans-serif"#
            }
            Script::Bengali => {
                r#""Noto Sans Bengali", "Nirmala UI", "Kohinoor Bangla", sans-serif"#
            }
            Script::Tamil => {
                r#""Noto Sans Tamil", "Nirmala UI", "Tamil Sangam MN", sans-serif"#
            }
            Script::Telugu => {
                r#""Noto Sans Telugu", "Nirmala UI", "Kohinoor Telugu", sans-serif"#
            }
            Script::Gujarati => {
                r#""Noto Sans Gujarati", "Nirmala UI", "Gujarati Sangam MN", sans-serif"#
            }
            Script::Kannada => {
                r#""Noto Sans Kannada", "Nirmala UI", "Kannada Sangam MN", sans-serif"#
            }
            Script::Malayalam => {
                r#""Noto Sans Malayalam", "Nirmala UI", "Malayalam Sangam MN", sans-serif"#
            }
            Script::Gurmukhi => {
                r#""Noto Sans Gurmukhi", "Nirmala UI", "Gurmukhi MN", sans-serif"#
            }
            // Urdu is conventionally set in the calligraphic Nastaliq style,
            // not the plainer Naskh style most "Arabic" web fonts default
            // to, so this stack leads with the Nastaliq-specific Noto face
            // rather than a generic "Noto Sans Arabic".
            Script::Arabic => {
                r#""Noto Nastaliq Urdu", "Urdu Typesetting", "Geeza Pro", serif"#
            }
            // No common macOS system font covers Odia specifically (unlike
            // most other scripts here), so this stack has only a Windows
            // fallback ("Kalinga") before Noto/generic — a disclosed gap,
            // not an oversight.
            Script::Odia => r#""Noto Sans Oriya", "Kalinga", sans-serif"#,
        }
    }
}

/// Text direction. Modeled as an enum rather than an `is_rtl: bool` so call
/// sites read as intent ("set this locale's direction") rather than a
/// double-negative-prone boolean, and so a third direction (there isn't
/// one in any living script, but vertical scripts exist elsewhere in
/// Unicode) would be a straightforward variant addition rather than a
/// breaking signature change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Ltr,
    Rtl,
}

impl Direction {
    /// The literal value for an HTML `dir` attribute, e.g.
    /// `<html dir=..(locale.direction.as_attr())..>`.
    pub const fn as_attr(self) -> &'static str {
        match self {
            Direction::Ltr => "ltr",
            Direction::Rtl => "rtl",
        }
    }
}

/// Whether a locale's content is genuinely ready for citizens to use, or
/// merely reserved in this table for future work.
///
/// This is deliberately a *single* flag scoped to what this crate owns:
/// UI chrome strings (`strings/<code>.json`). It says nothing about
/// whether the knowledge base or decision-tree content is translated for
/// that locale — those ship independently, on their own review gate
/// (`docs/11-multilingual-strategy.md` Section 4). A locale can
/// legitimately be `Shipped` here (its buttons and labels are translated
/// and reviewed) while its knowledge-base entries are still English-only;
/// callers that need to gate on KB coverage must consult `kb-content`'s
/// per-locale data, not this flag. Do not flip a locale to `Shipped` here
/// unless its `strings/<code>.json` has actually been through the UI-string
/// review step described in that doc — a language switcher option is a
/// promise that the labels behind it were checked by a fluent speaker, not
/// just that a file with the right keys exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocaleStatus {
    Shipped,
    Planned,
}

/// One supported (or reserved-for-future) locale's metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Locale {
    /// BCP-47 language subtag, e.g. `"hi"`. Matches the `code` values in
    /// `packages/i18n/src/locales.ts` and the `language` column used
    /// throughout the Postgres schema (`08-database-schema.md`).
    pub code: &'static str,
    pub english_name: &'static str,
    /// The language's own name for itself, in its own script — sourced
    /// from `packages/i18n/src/locales.ts`'s `nativeName`, which is what a
    /// language switcher should render as the visible label (a Hindi
    /// speaker looks for "हिन्दी", not "Hindi").
    pub native_name: &'static str,
    pub script: Script,
    pub direction: Direction,
    pub status: LocaleStatus,
}

impl Locale {
    /// Convenience passthrough to `self.script.css_font_stack()` for
    /// callers that only have a `Locale` in hand (the common case in a
    /// render layer that's already resolved the visitor's locale).
    pub const fn css_font_stack(&self) -> &'static str {
        self.script.css_font_stack()
    }
}

/// The 15-language priority set this crate covers, in the same order the
/// task brief and `packages/i18n/src/locales.ts` list them.
///
/// Notably absent from this table despite being real Eighth-Schedule /
/// widely-used RTL languages: **Sindhi** (`sd`) and **Kashmiri** (`ks`),
/// both also written right-to-left in their Perso-Arabic forms. They're
/// out of scope for this crate's 15-language priority set (they *are*
/// listed as `planned` placeholders in `packages/i18n/src/locales.ts`).
/// Whoever adds either language here later: give it `Direction::Rtl`, not
/// `Ltr` — that's the easy mistake, since most of this table is LTR.
pub const LOCALES: [Locale; 15] = [
    Locale {
        code: "en",
        english_name: "English",
        native_name: "English",
        script: Script::Latin,
        direction: Direction::Ltr,
        status: LocaleStatus::Shipped,
    },
    Locale {
        code: "hi",
        english_name: "Hindi",
        native_name: "हिन्दी",
        script: Script::Devanagari,
        direction: Direction::Ltr,
        status: LocaleStatus::Shipped,
    },
    Locale {
        code: "bn",
        english_name: "Bengali",
        native_name: "বাংলা",
        script: Script::Bengali,
        direction: Direction::Ltr,
        status: LocaleStatus::Planned,
    },
    Locale {
        code: "ta",
        english_name: "Tamil",
        native_name: "தமிழ்",
        script: Script::Tamil,
        direction: Direction::Ltr,
        status: LocaleStatus::Planned,
    },
    Locale {
        code: "te",
        english_name: "Telugu",
        native_name: "తెలుగు",
        script: Script::Telugu,
        direction: Direction::Ltr,
        status: LocaleStatus::Planned,
    },
    Locale {
        code: "mr",
        english_name: "Marathi",
        native_name: "मराठी",
        script: Script::Devanagari,
        direction: Direction::Ltr,
        status: LocaleStatus::Planned,
    },
    Locale {
        code: "gu",
        english_name: "Gujarati",
        native_name: "ગુજરાતી",
        script: Script::Gujarati,
        direction: Direction::Ltr,
        status: LocaleStatus::Planned,
    },
    Locale {
        code: "kn",
        english_name: "Kannada",
        native_name: "ಕನ್ನಡ",
        script: Script::Kannada,
        direction: Direction::Ltr,
        status: LocaleStatus::Planned,
    },
    Locale {
        code: "ml",
        english_name: "Malayalam",
        native_name: "മലയാളം",
        script: Script::Malayalam,
        direction: Direction::Ltr,
        status: LocaleStatus::Planned,
    },
    Locale {
        code: "pa",
        english_name: "Punjabi",
        native_name: "ਪੰਜਾਬੀ",
        script: Script::Gurmukhi,
        direction: Direction::Ltr,
        status: LocaleStatus::Planned,
    },
    // Urdu: the one right-to-left language in this table. See the
    // `LOCALES` doc comment above re: Sindhi/Kashmiri, the other two
    // RTL languages this project cares about but that are out of scope
    // for this 15-language set.
    Locale {
        code: "ur",
        english_name: "Urdu",
        native_name: "اردو",
        script: Script::Arabic,
        direction: Direction::Rtl,
        status: LocaleStatus::Planned,
    },
    Locale {
        code: "or",
        english_name: "Odia",
        native_name: "ଓଡ଼ିଆ",
        script: Script::Odia,
        direction: Direction::Ltr,
        status: LocaleStatus::Planned,
    },
    Locale {
        code: "as",
        english_name: "Assamese",
        native_name: "অসমীয়া",
        // Assamese is written in the Bengali-Assamese script (its own
        // print tradition adds two letters, ৰ and ৱ, that Bengali doesn't
        // use, but it is the same base script) — there's no separate
        // `Script::Assamese` variant because font selection is identical
        // to Bengali.
        script: Script::Bengali,
        direction: Direction::Ltr,
        status: LocaleStatus::Planned,
    },
    Locale {
        code: "mai",
        english_name: "Maithili",
        native_name: "मैथिली",
        // Maithili has its own historical script (Tirhuta/Mithilakshar),
        // but modern print and digital usage is overwhelmingly Devanagari
        // — there's no `Script::Tirhuta` variant because no font-stack
        // decision in this crate needs one yet.
        script: Script::Devanagari,
        direction: Direction::Ltr,
        status: LocaleStatus::Planned,
    },
    Locale {
        code: "ne",
        english_name: "Nepali",
        native_name: "नेपाली",
        script: Script::Devanagari,
        direction: Direction::Ltr,
        status: LocaleStatus::Planned,
    },
];

/// All 15 locales this crate knows about, `Shipped` and `Planned` alike.
/// Use this for admin/debug views of the full table; use
/// [`shipped_locales`] for anything a citizen-facing language switcher
/// renders.
pub fn all_locales() -> &'static [Locale] {
    &LOCALES
}

/// Only the locales whose UI-chrome strings are translated and reviewed
/// (see [`LocaleStatus`]'s doc comment for exactly what this does and
/// does not promise). This is what a citizen-facing language switcher
/// should iterate over — showing a `Planned` locale in the switcher would
/// let someone select a language whose buttons and labels are still
/// English, which is confusing at best.
pub fn shipped_locales() -> Vec<&'static Locale> {
    LOCALES.iter().filter(|l| l.status == LocaleStatus::Shipped).collect()
}

/// The fallback locale for any request this crate can't otherwise resolve.
/// English, because it's this project's interlingua for content authoring
/// and review (`docs/11-multilingual-strategy.md` Section 1) and the one
/// locale guaranteed to be `Shipped` and 100% complete.
pub fn default_locale() -> &'static Locale {
    LOCALES
        .iter()
        .find(|l| l.code == "en")
        .expect("\"en\" is always present in LOCALES — see the constant above")
}

/// Resolves a requested locale tag to the best matching entry in
/// [`LOCALES`], in three steps:
///
/// 1. Exact, case-insensitive match against a locale's `code` (handles
///    `"hi"`, `"HI"`).
/// 2. Base-language match: the tag is split on `-`/`_` and the first
///    subtag is matched the same way (handles a full BCP-47 tag like
///    `"hi-IN"` or `"pa-Guru-IN"` a browser's `Accept-Language` or a
///    bot platform's `language_code` field commonly sends).
/// 3. Otherwise, [`default_locale`] (English) — this function never
///    returns `None` and never panics, on the same "never show a citizen
///    a broken state over an unrecognized language tag" principle as
///    [`crate::strings::t`]'s fallback.
///
/// Deliberately returns a *table entry*, including `Planned` ones — this
/// function resolves "what locale does this tag mean," not "is this
/// locale exposed in the switcher." A caller building a language switcher
/// should filter through [`shipped_locales`] itself; a caller just trying
/// to render request-language font/direction metadata for arbitrary
/// content (e.g. a bot channel echoing back a `Planned` language a
/// community translator is previewing) legitimately wants the exact
/// match even if it's not yet `Shipped`.
pub fn resolve(requested: &str) -> &'static Locale {
    let requested = requested.trim();
    if let Some(locale) = LOCALES.iter().find(|l| l.code.eq_ignore_ascii_case(requested)) {
        return locale;
    }
    // Normalize the `_` some platforms use (e.g. `hi_IN`) to BCP-47's `-`,
    // then take the first subtag — avoids depending on the newer
    // multi-pattern `str::split` API so this keeps working on an older
    // MSRV too.
    let normalized = requested.replace('_', "-");
    let base = normalized.split('-').next().unwrap_or(&normalized);
    if let Some(locale) = LOCALES.iter().find(|l| l.code.eq_ignore_ascii_case(base)) {
        return locale;
    }
    default_locale()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_locale_has_a_non_empty_native_name() {
        for locale in LOCALES.iter() {
            assert!(
                !locale.native_name.is_empty(),
                "{} is missing a native name",
                locale.code
            );
            assert!(
                !locale.english_name.is_empty(),
                "{} is missing an English name",
                locale.code
            );
        }
    }

    #[test]
    fn table_covers_exactly_the_15_priority_codes_with_no_duplicates() {
        let expected = [
            "en", "hi", "bn", "ta", "te", "mr", "gu", "kn", "ml", "pa", "ur", "or", "as", "mai",
            "ne",
        ];
        assert_eq!(LOCALES.len(), expected.len());
        for code in expected {
            let matches = LOCALES.iter().filter(|l| l.code == code).count();
            assert_eq!(matches, 1, "{code} should appear exactly once in LOCALES");
        }
    }

    #[test]
    fn resolve_handles_exact_match() {
        assert_eq!(resolve("hi").code, "hi");
        assert_eq!(resolve("HI").code, "hi");
    }

    #[test]
    fn resolve_handles_base_language_match() {
        assert_eq!(resolve("hi-IN").code, "hi");
        assert_eq!(resolve("pa-Guru-IN").code, "pa");
        assert_eq!(resolve("EN-us").code, "en");
    }

    #[test]
    fn resolve_falls_back_to_english_for_unknown_tags() {
        assert_eq!(resolve("xx-YY").code, "en");
        assert_eq!(resolve("").code, "en");
        assert_eq!(resolve("klingon").code, "en");
    }

    #[test]
    fn urdu_is_rtl_and_hindi_is_ltr() {
        let ur = resolve("ur");
        assert_eq!(ur.direction, Direction::Rtl);
        assert_eq!(ur.direction.as_attr(), "rtl");

        let hi = resolve("hi");
        assert_eq!(hi.direction, Direction::Ltr);
        assert_eq!(hi.direction.as_attr(), "ltr");
    }

    #[test]
    fn only_urdu_is_rtl_among_the_15() {
        let rtl_codes: Vec<&str> =
            LOCALES.iter().filter(|l| l.direction == Direction::Rtl).map(|l| l.code).collect();
        assert_eq!(rtl_codes, vec!["ur"]);
    }

    #[test]
    fn shipped_locales_is_exactly_english_and_hindi_for_now() {
        let shipped: Vec<&str> = shipped_locales().iter().map(|l| l.code).collect();
        assert_eq!(shipped.len(), 2);
        assert!(shipped.contains(&"en"));
        assert!(shipped.contains(&"hi"));
    }

    #[test]
    fn every_script_has_a_non_empty_font_stack() {
        for locale in LOCALES.iter() {
            assert!(!locale.css_font_stack().is_empty(), "{} has an empty font stack", locale.code);
        }
    }
}
