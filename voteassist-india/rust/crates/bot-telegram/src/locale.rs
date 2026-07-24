//! Resolves a citizen's locale from Telegram's own `User.language_code`
//! (an IETF language tag Telegram sends based on the user's app-language
//! setting) rather than asking them to run a `/language` command first —
//! one fewer step for the messaging-first personas (migrant workers,
//! elderly citizens — `02-personas.md` personas 3, 9) this channel exists
//! for. Only `en`/`hi` are shipped (`packages/i18n/src/locales.ts`); any
//! other tag falls back to English via the same rule
//! `core_domain::LocalizedText::pick` applies everywhere else.

use teloxide::types::User;

pub fn resolve_locale(user: Option<&User>) -> &'static str {
    match user.and_then(|u| u.language_code.as_deref()) {
        Some(code) if code.eq_ignore_ascii_case("hi") || code.starts_with("hi-") => "hi",
        _ => "en",
    }
}
