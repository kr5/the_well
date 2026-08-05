//! Locale metadata and UI-chrome string dictionary for VoteAssist India's
//! 15-language priority set (English plus 14 Indian languages: Hindi,
//! Bengali, Tamil, Telugu, Marathi, Gujarati, Kannada, Malayalam, Punjabi,
//! Urdu, Odia, Assamese, Maithili, Nepali — see [`locale::LOCALES`]).
//!
//! ## Scope: this crate is UI chrome, not content
//!
//! This crate owns exactly two things: (1) locale metadata — script, text
//! direction, font stack, BCP-47 negotiation — and (2) the flat key/value
//! dictionary behind every button label, nav item, and generic error
//! message in the app (`strings/<code>.json`). It does **not** own
//! knowledge-base entries, decision-tree copy, citations, or any other
//! legal/procedural content — that's `crates/kb-content` and
//! `crates/core-domain`, which carry their own per-language
//! `review_status`/`last_verified_date` because a mistranslated button
//! label and a mistranslated legal claim are different classes of risk
//! (see `docs/11-multilingual-strategy.md` Section 4). **A locale being
//! [`locale::LocaleStatus::Shipped`] here says nothing about whether the
//! knowledge base is translated for it** — those ship independently, and
//! a caller that needs to gate a whole citizen journey on KB coverage
//! must check `kb-content`, not this crate.
//!
//! ## wasm32 safety
//!
//! `crates/web-app` (Leptos) compiles this crate into its client-side
//! `wasm32-unknown-unknown` bundle as well as its server binary, so this
//! crate has zero I/O: no `tokio`, no `sqlx`, no filesystem or network
//! access at runtime. All content is embedded at compile time via
//! [`include_str!`] — see `strings` module docs for the full pattern,
//! which mirrors `crates/kb-content`'s `entries.rs` exactly.
//!
//! ## Fallback is not optional
//!
//! Every lookup this crate exposes — [`strings::t`] for a UI string,
//! [`locale::resolve`] for a locale tag — degrades to English rather than
//! panicking or handing back an empty/raw value. This is a
//! voter-guidance tool used by real citizens mid-task; a panic or a
//! broken label is a worse outcome than English text some readers won't
//! fully understand.

pub mod locale;
pub mod strings;

pub use locale::{
    all_locales, default_locale, resolve, shipped_locales, Direction, Locale, LocaleStatus, Script,
};
pub use strings::{t, UiStrings, UI_STRING_KEYS};
