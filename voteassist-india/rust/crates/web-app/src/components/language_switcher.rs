//! The language-switcher island (docs/PRD-V2-RUST-PLATFORM.md Section
//! 6.6). See `locale.rs`'s module doc for the honest scope of what
//! switching actually retranslates today.
//!
//! ## Which locales this renders, and why
//!
//! This iterates `i18n::shipped_locales()`, not `i18n::all_locales()`.
//! That's not a new policy invented here — it's the one the `i18n` crate
//! itself documents and enforces: `LocaleStatus::Shipped` means a
//! locale's `strings/<code>.json` has actually been through human review
//! (see `i18n::locale`'s and this crate's README's `LocaleStatus` docs),
//! and `i18n::shipped_locales()`'s own doc comment says outright that a
//! citizen-facing switcher "should iterate over" exactly this list —
//! offering a `Planned` locale here would let someone pick a language
//! whose buttons and labels are still English, which the `i18n` crate
//! calls "confusing at best." As of this writing that's still just
//! English and Hindi (`i18n`'s README, "As of this writing, only `en` and
//! `hi` are `Shipped`"); this component doesn't hardcode that number
//! though — it will pick up the 13 other languages automatically, in
//! whatever order `i18n::LOCALES` lists them, the moment each one's
//! `LocaleStatus` flips to `Shipped` upstream. No code change needed here
//! for that.
//!
//! ## Why a `<select>` instead of one `<button>` per locale
//! Fifteen (eventually) inline buttons doesn't fit this site's mobile-
//! first, low-bandwidth-device layout; a native `<select>` scales to
//! that count for free and is a real, standard form control — a citizen
//! can open it, arrow-key through every option, and choose one using
//! only native browser behavior, no JavaScript required to *operate* the
//! control itself (unlike a hand-rolled JS dropdown widget, which would
//! need JS just to open/close).
//!
//! That said, be honest about what's *not* JS-independent here: like the
//! two-button version this replaces, *acting* on a choice — flipping the
//! visible locale and persisting the cookie — still goes through this
//! component's `on:change` handler, which only runs once the wasm island
//! hydrates. A citizen with JavaScript disabled can see and operate the
//! dropdown, but picking an option is currently a no-op for them. Making
//! that round trip work with no JS at all would mean wrapping this in a
//! real `<form>` (e.g. `leptos_router`'s `ActionForm`) that POSTs to
//! `set_locale_preference`'s server-function endpoint and redirects back
//! to the current page — a bigger change than this pass attempts, since
//! it touches request/redirect behavior this pass has no compiler to
//! verify against. Disclosed here rather than left silently unclear.

use leptos::prelude::*;

use crate::locale::{set_locale_preference, use_locale};

#[component]
pub fn LanguageSwitcher() -> impl IntoView {
    let locale = use_locale().0;
    // Fire-and-forget: persisting the choice must never block or break
    // switching the in-memory signal, which is what the citizen actually
    // sees happen immediately.
    let persist_action = Action::new(|new_locale: &String| {
        let new_locale = new_locale.clone();
        async move {
            let _ = set_locale_preference(new_locale).await;
        }
    });

    view! {
        <div class="language-switcher" role="group" aria-label="Language">
            // Not sourced from `i18n::t()`: the 43-key UI_STRING_KEYS
            // contract (crates/i18n/src/strings.rs) has no generic
            // "language switcher" label — its keys are all `nav.*`,
            // `banner.*`, `action.*`, `flow.*`, `account.*`, `consent.*`,
            // `footer.*`, `error.*`. Adding one would mean changing that
            // contract, which lives in `crates/i18n` — outside this
            // pass's file territory. Left as a plain English string
            // rather than inventing a key unilaterally.
            <select
                class="lang-select"
                aria-label="Language"
                prop:value=move || locale.get()
                on:change=move |ev| {
                    let new_locale = event_target_value(&ev);
                    locale.set(new_locale.clone());
                    persist_action.dispatch(new_locale);
                }
            >
                {i18n::shipped_locales()
                    .into_iter()
                    .map(|l| {
                        // Visible label is `native_name` — a Bengali
                        // speaker looks for "বাংলা", not "Bengali"
                        // (`i18n::Locale::native_name`'s own doc comment).
                        view! { <option value=l.code>{l.native_name}</option> }
                    })
                    .collect_view()}
            </select>
        </div>
    }
}
