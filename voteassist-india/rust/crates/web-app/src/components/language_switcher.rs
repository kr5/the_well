//! The language-switcher island (docs/PRD-V2-RUST-PLATFORM.md Section
//! 6.6). Per docs/05-information-architecture.md Section 7: "MVP:
//! English, Hindi (fully reviewed)." See `locale.rs`'s module doc for the
//! honest scope of what switching actually retranslates today.

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
            <button
                type="button"
                class="lang-button"
                aria-pressed=move || (locale.get() == "en").to_string()
                on:click=move |_| {
                    locale.set("en".to_string());
                    persist_action.dispatch("en".to_string());
                }
            >
                "English"
            </button>
            <button
                type="button"
                class="lang-button"
                aria-pressed=move || (locale.get() == "hi").to_string()
                on:click=move |_| {
                    locale.set("hi".to_string());
                    persist_action.dispatch("hi".to_string());
                }
            >
                "हिन्दी"
            </button>
        </div>
    }
}
