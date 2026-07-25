//! The cookie/tracking consent banner — a deliberate, disclosed exception
//! to docs/PRD-V2-RUST-PLATFORM.md Section 6.6's "only two islands"
//! architecture (question/answer widget, language switcher): consent
//! gating is a legal requirement (DPDP Act 2023 Rules 2025, and a
//! GDPR-style consent-before-non-essential-cookies posture for any visitor
//! reached from outside India if this is embedded in a broader system),
//! not a product nicety that can be deferred to stay within an earlier
//! design budget.
//!
//! What actually needs consent, concretely:
//! - **Strictly necessary** (never gated): the accessibility-preference
//!   `localStorage` keys (`pages::accessibility`) and, if a citizen
//!   creates an account, the `va_account_session` cookie
//!   (`accounts::ACCOUNT_SESSION_COOKIE_NAME`) — both exist only to
//!   deliver a feature the citizen explicitly asked for, which is the
//!   normal legal basis for "necessary" cookies.
//! - **Analytics** (gated on consent): `analytics_events` recording.
//!   `server_fns.rs`'s `submit_answer`/`start_walkthrough`/`search_kb`/
//!   `get_kb_entry` all check `ANALYTICS_CONSENT_COOKIE_NAME` server-side
//!   before calling `crates/analytics::record_event` — see that file's
//!   `analytics_consent_given` helper.
//!
//! Persisted the same way as the accessibility preferences (`localStorage`
//! + a tiny vanilla-JS file, `public/cookie-consent.js`), not a third
//! Leptos-hydrated island, for the same reasoning `pages::accessibility`
//! documents: a persisted, cross-page, pre-hydration preference doesn't
//! need wasm to work, and keeping it out of the wasm bundle keeps the
//! bundle smaller (a real 2G-bandwidth concern this product takes
//! seriously). If a logged-in citizen has an account, `record_consent`
//! (`server_fns_accounts.rs`) additionally durably logs the decision to
//! `consent_artifacts` the next time they're authenticated — the banner
//! itself works identically for anonymous and logged-in visitors.
//!
//! `localStorage` alone isn't readable by a `#[server]` function (it never
//! leaves the browser), so `public/cookie-consent.js` ALSO mirrors the
//! choice into a plain, non-`HttpOnly` cookie (`ANALYTICS_CONSENT_COOKIE_NAME`)
//! specifically so server-side code can check it. That cookie carries no
//! identity and isn't itself an analytics mechanism — it only says
//! "yes/no" to whether *other* recording is allowed to run, which is why
//! setting it is itself "strictly necessary" and not gated on its own
//! consent.

use leptos::prelude::*;

/// A plain (`document.cookie`-writable, not `HttpOnly`), non-identifying
/// cookie mirroring the `localStorage` consent choice, so server-side
/// code can check it — see this module's doc comment above.
pub const ANALYTICS_CONSENT_COOKIE_NAME: &str = "va_analytics_consent";

#[component]
pub fn CookieConsentBanner() -> impl IntoView {
    view! {
        <div id="va-cookie-consent" class="cookie-consent-banner" role="dialog" aria-label="Cookie preferences" hidden=true>
            <p>
                "We use strictly necessary cookies to run this site (like remembering an "
                "accessibility preference, or keeping you signed in if you save a checklist). "
                "With your permission, we'd also like to record anonymous, aggregate usage "
                "analytics (which question was viewed, never free text) to improve the product — "
                "see our "
                <a href="/about/privacy">"privacy policy"</a>
                " for exactly what that does and doesn't include."
            </p>
            <div class="cookie-consent-actions">
                <button type="button" onclick="window.__vaSetCookieConsent('accepted')">
                    "Accept analytics"
                </button>
                <button type="button" onclick="window.__vaSetCookieConsent('rejected')">
                    "Necessary only"
                </button>
            </div>
        </div>
        <script src="/cookie-consent.js"></script>
    }
}
