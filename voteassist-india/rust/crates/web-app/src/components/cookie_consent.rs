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
//! - **Analytics** (gated on consent): `analytics_events` recording. This
//!   site does not yet call `crates/analytics::record_event` from any
//!   page (a disclosed follow-up — see `crates/web-app`'s README) — this
//!   banner and its consent signal exist now specifically so that wiring,
//!   whenever it lands, has a real gate to check from day one rather than
//!   retrofitting consent onto already-shipping tracking.
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

use leptos::prelude::*;

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
