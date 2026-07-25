//! A single reactive locale preference, shared via Leptos context between
//! the `LanguageSwitcher` island and the question-flow island — the only
//! two pieces of this site with genuine bilingual content today (the
//! decision tree's `LocalizedText` fields; see `render.rs`). Static page
//! chrome and knowledge-base entries are English-only in this pass (the
//! KB's `language` field is `"en"` on every existing entry) — switching
//! the language here does not retranslate the rest of the site, and this
//! module doesn't pretend otherwise.
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
//! compiler.

use leptos::prelude::*;

pub const DEFAULT_LOCALE: &str = "en";
pub const SUPPORTED_LOCALES: &[&str] = &["en", "hi"];
pub const LOCALE_COOKIE_NAME: &str = "va_locale";

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
        .filter(|value| SUPPORTED_LOCALES.contains(&value.as_str())))
}

/// Persists a locale choice — called by `LanguageSwitcher` alongside
/// (not instead of) setting the in-memory signal, so switching feels
/// instant while the choice also survives a hard reload.
#[server]
pub async fn set_locale_preference(locale: String) -> Result<(), ServerFnError> {
    if !SUPPORTED_LOCALES.contains(&locale.as_str()) {
        return Err(ServerFnError::ServerError(format!("\"{locale}\" is not a supported locale")));
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
