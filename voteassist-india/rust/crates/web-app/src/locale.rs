//! A single reactive locale preference, shared via Leptos context between
//! the `LanguageSwitcher` island and the question-flow island — the only
//! two pieces of this site with genuine bilingual content today (the
//! decision tree's `LocalizedText` fields; see `render.rs`). Static page
//! chrome and knowledge-base entries are English-only in this pass (the
//! KB's `language` field is `"en"` on every existing entry) — switching
//! the language here does not retranslate the rest of the site, and this
//! module doesn't pretend otherwise.
//!
//! ## Delegating to `i18n`
//!
//! Locale *identity* — which codes exist, their native names, script,
//! direction, and shipped/planned status — is owned entirely by the
//! `i18n` crate now (`crates/i18n/src/locale.rs`), not duplicated here.
//! This module used to hardcode a 2-element `SUPPORTED_LOCALES: &[&str] =
//! &["en", "hi"]`; that's gone. Validating a locale code (in both server
//! functions below) now means checking it against `i18n::all_locales()` —
//! all 15 codes the crate knows about, `Shipped` and `Planned` alike, not
//! just the ones a citizen-facing switcher currently renders. That's a
//! deliberate choice: a stored `va_locale` cookie or an explicit
//! `set_locale_preference` call is a *narrower, more deliberate* action
//! than "which locale does the language switcher show" — e.g. a community
//! translator previewing a `Planned` locale's UI strings (see `i18n`'s
//! `resolve()` doc comment for exactly this scenario) is a legitimate
//! caller this validation shouldn't reject. `LanguageSwitcher` itself is
//! what narrows to `i18n::shipped_locales()` for what it actually offers
//! — see that component's doc comment.
//!
//! Persisted via `LOCALE_COOKIE_NAME`, read back once per page load
//! (`app.rs`'s root `App` component calls `get_locale_preference` on
//! mount and applies it to the signal below) rather than a full SSR-time
//! seed of the signal's initial value: Leptos's root component tree is
//! set up synchronously, and reading a request cookie needs an async
//! `leptos_axum::extract` call, so this trades a real (if small and
//! disclosed) tradeoff — a brief flash of `DEFAULT_LOCALE` before the
//! stored preference applies — for not inventing an unverified deeper
//! SSR-context-injection mechanism this pass can't check against a
//! compiler. See `app.rs`'s module doc for how this same constraint plays
//! out for the `<html>` shell's `lang`/`dir` attributes specifically.

use leptos::prelude::*;

pub const DEFAULT_LOCALE: &str = "en";
pub const LOCALE_COOKIE_NAME: &str = "va_locale";

/// Whether `code` is one of `i18n`'s 15 known locale codes (`Shipped` or
/// `Planned`) — an exact, case-sensitive match against `Locale::code`.
/// Deliberately *not* `i18n::resolve`, which always succeeds (falling
/// back to English for anything it can't match): that's the right
/// behavior for "what locale does this browser tag probably mean," but
/// wrong here, where the whole point is to reject a value that isn't a
/// real code (`set_locale_preference` must error on a bogus locale, not
/// silently coerce it to English).
fn is_known_locale(code: &str) -> bool {
    i18n::all_locales().iter().any(|locale| locale.code == code)
}

#[derive(Clone, Copy)]
pub struct LocaleSignal(pub RwSignal<String>);

pub fn provide_locale_context() {
    provide_context(LocaleSignal(RwSignal::new(DEFAULT_LOCALE.to_string())));
}

pub fn use_locale() -> LocaleSignal {
    use_context::<LocaleSignal>().expect("LocaleSignal context must be provided by the App root component")
}

/// Reads `LOCALE_COOKIE_NAME` if present and set to a recognized locale —
/// called once from `app.rs`'s root `App` component on mount.
#[server]
pub async fn get_locale_preference() -> Result<Option<String>, ServerFnError> {
    use axum_extra::extract::cookie::CookieJar;

    let Ok(jar) = leptos_axum::extract::<CookieJar>().await else {
        return Ok(None);
    };

    Ok(jar
        .get(LOCALE_COOKIE_NAME)
        .map(|cookie| cookie.value().to_string())
        .filter(|value| is_known_locale(value)))
}

/// Persists a locale choice — called by `LanguageSwitcher` alongside
/// (not instead of) setting the in-memory signal, so switching feels
/// instant while the choice also survives a hard reload. Accepts any of
/// `i18n`'s 15 known codes (see this module's doc comment for why that's
/// wider than what the switcher itself offers), rejecting anything else.
#[server]
pub async fn set_locale_preference(locale: String) -> Result<(), ServerFnError> {
    if !is_known_locale(&locale) {
        return Err(ServerFnError::ServerError(format!("\"{locale}\" is not a recognized locale code")));
    }
    set_locale_cookie(&locale);
    Ok(())
}

#[cfg(feature = "ssr")]
fn set_locale_cookie(locale: &str) {
    use http::header::{HeaderValue, SET_COOKIE};

    if let Some(opts) = use_context::<leptos_axum::ResponseOptions>() {
        // HttpOnly/Secure/SameSite=Lax, matching this codebase's other
        // server-set cookies (accounts::session, admin-app's auth
        // session) — this value isn't sensitive, but there's no reason
        // to relax the same defaults for a cookie only server-side code
        // ever needs to read back.
        let cookie_value = format!("{LOCALE_COOKIE_NAME}={locale}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=31536000");
        if let Ok(header_value) = HeaderValue::from_str(&cookie_value) {
            opts.insert_header(SET_COOKIE, header_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `DEFAULT_LOCALE` is a plain string literal (not derived from
    /// `i18n::default_locale().code`) only because `default_locale()`
    /// isn't a `const fn`, so it can't initialize a `const` item — this
    /// guards against the two silently drifting apart.
    #[test]
    fn default_locale_matches_the_i18n_crate() {
        assert_eq!(DEFAULT_LOCALE, i18n::default_locale().code);
    }

    #[test]
    fn is_known_locale_accepts_all_15_and_rejects_garbage() {
        for locale in i18n::all_locales() {
            assert!(is_known_locale(locale.code), "{} should be known", locale.code);
        }
        assert!(!is_known_locale("xx"));
        assert!(!is_known_locale(""));
        assert!(!is_known_locale("EN")); // exact-match only, unlike i18n::resolve
    }

    /// `components::language_switcher::LanguageSwitcher` now offers all 15
    /// `i18n` locales (grouped by `LocaleStatus`, not filtered down to
    /// `Shipped`) — see that component's module doc. That makes this
    /// module's existing "accept `all_locales()`, not just
    /// `shipped_locales()`" behavior load-bearing rather than merely
    /// forward-looking: a preference the switcher can set is a preference
    /// the server must be willing to store, so `is_known_locale` (and thus
    /// `get_locale_preference`/`set_locale_preference`) must not reject a
    /// `Planned` locale. "bn" (Bengali) is `Planned` as of this writing —
    /// picked over a `Shipped` one specifically so this test would fail
    /// loudly if `is_known_locale` were ever narrowed to
    /// `i18n::shipped_locales()`.
    #[test]
    fn is_known_locale_accepts_a_planned_locale_the_switcher_now_offers() {
        let bn = i18n::all_locales().iter().find(|l| l.code == "bn").expect("bn is one of the 15 i18n locales");
        assert_eq!(bn.status, i18n::LocaleStatus::Planned, "this test assumes bn is still Planned; if it shipped, swap in another currently-Planned code");
        assert!(is_known_locale("bn"), "a Planned locale the switcher offers must round-trip through cookie validation");
    }
}
