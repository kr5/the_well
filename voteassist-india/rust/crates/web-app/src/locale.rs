//! A single reactive locale preference, shared via Leptos context between
//! the `LanguageSwitcher` island and the question-flow island — the only
//! two pieces of this site with genuine bilingual content today (the
//! decision tree's `LocalizedText` fields; see `render.rs`). Static page
//! chrome and knowledge-base entries are English-only in this pass (the
//! KB's `language` field is `"en"` on every existing entry) — switching
//! the language here does not retranslate the rest of the site, and this
//! module doesn't pretend otherwise.
//!
//! Deliberately a plain in-memory signal, not a cookie: it resets to
//! English on a hard page reload. Persisting it (a cookie read during
//! SSR) is a reasonable follow-up, not attempted here to keep this pass's
//! scope to what's confidently verifiable without a compiler.

use leptos::prelude::*;

pub const DEFAULT_LOCALE: &str = "en";

#[derive(Clone, Copy)]
pub struct LocaleSignal(pub RwSignal<String>);

pub fn provide_locale_context() {
    provide_context(LocaleSignal(RwSignal::new(DEFAULT_LOCALE.to_string())));
}

pub fn use_locale() -> LocaleSignal {
    use_context::<LocaleSignal>().expect("LocaleSignal context must be provided by the App root component")
}
