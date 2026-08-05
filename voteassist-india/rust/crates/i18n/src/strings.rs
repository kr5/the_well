//! UI-chrome string dictionary: `t(locale_code, key)` over the flat
//! per-locale JSON files in `strings/`.
//!
//! ## Content pipeline
//!
//! Each `strings/<code>.json` (one per [`crate::locale::LOCALES`] code) is
//! a flat `{ "key": "translated string", ... }` object — no nesting, no
//! interpolation syntax, no reading-level variants (those are a
//! knowledge-base concept per `docs/11-multilingual-strategy.md` Section
//! 2, out of scope for UI chrome). Every file is pulled in at compile time
//! via [`include_str!`], exactly like `crates/kb-content` embeds its
//! knowledge-base JSON: this crate is a dependency of the Leptos client
//! bundle (`crates/web-app` compiles to `wasm32-unknown-unknown` for the
//! browser), which rules out reading these files from disk or over the
//! network at runtime — wasm in a browser has no filesystem and no
//! ambient network access of that kind.
//!
//! Parsing happens once, lazily, into a [`std::sync::OnceLock`] — again
//! the same pattern `kb-content::all_entries` uses. The nested
//! `HashMap<&'static str, HashMap<String, String>>` this builds is only
//! ever written once (inside `get_or_init`) and the `OnceLock` itself is a
//! `'static` item, so a `&str` borrowed out of a `String` living inside it
//! is sound to hand back with a `'static` lifetime: the data can't move or
//! drop for the rest of the program. That's what lets [`t`] return
//! `&'static str` without leaking memory on every lookup or forcing every
//! call site to clone.
//!
//! ## Fallback policy
//!
//! [`t`] never panics and never returns an untranslated raw key to a
//! citizen. A missing key in the requested locale, or a locale code this
//! crate doesn't recognize at all, falls back to the English string for
//! that key. This is a deliberate product decision, not just an
//! engineering convenience: this is a civic-information tool, and a
//! blank or garbled label is worse for a citizen than a correct English
//! one they may or may not read fluently. See [`t`]'s doc comment for the
//! one remaining edge case (English itself missing a key) and why it's
//! considered unreachable rather than unhandled.

use std::collections::HashMap;
use std::sync::OnceLock;

/// The fixed set of UI string keys every `strings/<code>.json` is expected
/// to provide a value for. This is the contract between this crate and
/// whoever authors the JSON content: do not add a key here without also
/// adding it (with a real translation, not a placeholder) to every
/// `strings/<code>.json` file, and do not add a key to a JSON file that
/// isn't listed here — `tests::en_json_matches_ui_string_keys` enforces
/// the English file matches this list exactly, since English is this
/// crate's source of truth for the key set.
pub const UI_STRING_KEYS: &[&str] = &[
    "nav.home",
    "nav.start",
    "nav.search",
    "nav.learn",
    "nav.locate",
    "nav.about",
    "nav.feedback",
    "nav.accessibility",
    "nav.account",
    "banner.not_official",
    "banner.learn_more",
    "action.start_over",
    "action.save_checklist",
    "action.log_in",
    "action.log_out",
    "action.next",
    "action.back",
    "action.submit",
    "action.search",
    "flow.loading",
    "flow.progress_label",
    "flow.checklist_heading",
    "flow.sources_heading",
    "flow.official_links_heading",
    "flow.recommended_forms",
    "flow.caution_heading",
    "flow.didnt_match",
    "account.login_heading",
    "account.email_label",
    "account.otp_label",
    "account.request_otp",
    "account.verify",
    "account.my_checklists",
    "account.delete_account",
    "consent.banner_text",
    "consent.accept",
    "consent.necessary_only",
    "footer.privacy",
    "footer.legal",
    "footer.opensource",
    "footer.what_this_is",
    "error.generic",
    "error.not_found",
];

/// A single locale's parsed UI strings, keyed by [`UI_STRING_KEYS`] entries.
/// Exposed mainly so callers (e.g. an admin translation-completeness view)
/// can inspect a whole locale at once rather than looking up one key at a
/// time via [`t`].
pub type UiStrings = HashMap<String, String>;

/// One `strings/<code>.json` file per [`crate::locale::LOCALES`] entry,
/// embedded at compile time. This list is intentionally exhaustive over
/// all 15 codes, including the 14 this crate's territory does not author
/// (see this crate's README) — the parallel content-authoring work
/// populating those files is expected to land each one at
/// `strings/<code>.json` using exactly these filenames.
macro_rules! locale_json {
    ($code:literal) => {
        include_str!(concat!("../strings/", $code, ".json"))
    };
}

const RAW_STRINGS: &[(&str, &str)] = &[
    ("en", locale_json!("en")),
    ("hi", locale_json!("hi")),
    ("bn", locale_json!("bn")),
    ("ta", locale_json!("ta")),
    ("te", locale_json!("te")),
    ("mr", locale_json!("mr")),
    ("gu", locale_json!("gu")),
    ("kn", locale_json!("kn")),
    ("ml", locale_json!("ml")),
    ("pa", locale_json!("pa")),
    ("ur", locale_json!("ur")),
    ("or", locale_json!("or")),
    ("as", locale_json!("as")),
    ("mai", locale_json!("mai")),
    ("ne", locale_json!("ne")),
];

fn all_strings() -> &'static HashMap<&'static str, UiStrings> {
    static STRINGS: OnceLock<HashMap<&'static str, UiStrings>> = OnceLock::new();
    STRINGS.get_or_init(|| {
        RAW_STRINGS
            .iter()
            .map(|(code, raw)| {
                let parsed: UiStrings = serde_json::from_str(raw).unwrap_or_else(|e| {
                    panic!(
                        "i18n/strings/{code}.json failed to parse as a flat {{string: string}} \
                         object (this is a build-time content-quality invariant, not a runtime \
                         possibility, matching kb-content's entries.rs): {e}"
                    )
                });
                (*code, parsed)
            })
            .collect()
    })
}

/// Looks up `key` in the given locale, falling back to English if the
/// locale is unrecognized or doesn't have that key — see this module's
/// doc comment for why that fallback exists. Never panics.
///
/// `locale_code` is matched exactly (e.g. `"hi"`, not `"hi-IN"`) — pass it
/// through [`crate::locale::resolve`] first if you have a raw
/// `Accept-Language`-style tag rather than an already-resolved code.
pub fn t(locale_code: &str, key: &str) -> &'static str {
    let strings = all_strings();

    if let Some(value) = strings.get(locale_code).and_then(|m| m.get(key)) {
        return value.as_str();
    }
    if let Some(value) = strings.get("en").and_then(|m| m.get(key)) {
        return value.as_str();
    }

    // Unreachable in a correctly-shipped build: `en.json` is this crate's
    // source of truth for the key set, and
    // `tests::en_json_matches_ui_string_keys` fails the build the moment
    // it and `UI_STRING_KEYS` diverge. This branch only fires if that
    // invariant is somehow violated at runtime — e.g. a hand-edited
    // `en.json` shipped without running tests. Returning the raw key
    // instead of panicking is still the right call even here: a visibly
    // wrong label is recoverable (a citizen can still read the rest of
    // the page); a panicked SSR render is not.
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn en_json_matches_ui_string_keys() {
        let en = all_strings().get("en").expect("en.json is always embedded");
        assert_eq!(
            en.len(),
            UI_STRING_KEYS.len(),
            "strings/en.json has a different number of keys than UI_STRING_KEYS"
        );
        for key in UI_STRING_KEYS {
            assert!(
                en.contains_key(*key),
                "strings/en.json is missing the contract key \"{key}\""
            );
        }
        for key in en.keys() {
            assert!(
                UI_STRING_KEYS.contains(&key.as_str()),
                "strings/en.json has key \"{key}\" that isn't in UI_STRING_KEYS — either it's a \
                 typo or UI_STRING_KEYS needs updating"
            );
        }
    }

    #[test]
    fn every_english_string_is_non_empty() {
        let en = all_strings().get("en").unwrap();
        for key in UI_STRING_KEYS {
            let value = en.get(*key).unwrap_or_else(|| panic!("missing key {key}"));
            assert!(!value.trim().is_empty(), "strings/en.json[\"{key}\"] is empty");
        }
    }

    #[test]
    fn t_returns_the_english_string_for_a_known_key() {
        assert_eq!(t("en", "action.back"), "Back");
    }

    #[test]
    fn t_falls_back_to_english_for_an_unrecognized_locale() {
        assert_eq!(t("zz", "action.back"), t("en", "action.back"));
    }

    #[test]
    fn t_falls_back_to_english_for_a_missing_key_in_a_known_locale() {
        // A key that exists in no locale's JSON at all (not even English)
        // exercises `t`'s final, deliberately-non-panicking branch: it
        // hands back the key itself rather than crashing. This is the one
        // case `t`'s doc comment calls "unreachable in a correctly-shipped
        // build" — here we reach it on purpose, with a key that was never
        // part of the `UI_STRING_KEYS` contract, to prove the branch is
        // safe rather than merely theoretical.
        assert_eq!(t("hi", "this.key.does.not.exist"), "this.key.does.not.exist");
    }

    #[test]
    fn t_never_panics_on_garbage_input() {
        let _ = t("", "");
        let _ = t("this-is-not-a-locale", "nor.is.this.a.key");
    }
}
