//! The language-switcher island (docs/PRD-V2-RUST-PLATFORM.md Section
//! 6.6). Per docs/05-information-architecture.md Section 7: "MVP:
//! English, Hindi (fully reviewed)." See `locale.rs`'s module doc for the
//! honest scope of what switching actually retranslates today.

use leptos::prelude::*;

use crate::locale::use_locale;

#[component]
pub fn LanguageSwitcher() -> impl IntoView {
    let locale = use_locale().0;

    view! {
        <div class="language-switcher" role="group" aria-label="Language">
            <button
                type="button"
                class="lang-button"
                aria-pressed=move || (locale.get() == "en").to_string()
                on:click=move |_| locale.set("en".to_string())
            >
                "English"
            </button>
            <button
                type="button"
                class="lang-button"
                aria-pressed=move || (locale.get() == "hi").to_string()
                on:click=move |_| locale.set("hi".to_string())
            >
                "हिन्दी"
            </button>
        </div>
    }
}
