// Cookie/tracking consent banner logic. See
// src/components/cookie_consent.rs's module doc for the full reasoning
// (why this is vanilla JS rather than a third Leptos island, what
// "necessary" vs "analytics" actually covers on this site today).
(function () {
  var STORAGE_KEY = "va-cookie-consent";
  // Must match ANALYTICS_CONSENT_COOKIE_NAME in
  // src/components/cookie_consent.rs exactly.
  var COOKIE_NAME = "va_analytics_consent";
  var COOKIE_MAX_AGE_SECONDS = 60 * 60 * 24 * 365; // ~1 year

  function currentConsent() {
    return localStorage.getItem(STORAGE_KEY);
  }

  // localStorage never leaves the browser, so a #[server] function can't
  // read it — mirroring the choice into a plain cookie is what actually
  // lets server-side code (crates/web-app's analytics_consent_given) gate
  // on it. No "Secure" flag here: this cookie carries no identity or
  // secret, and forcing Secure would silently fail to write during local
  // (non-HTTPS) development.
  function writeConsentCookie(value) {
    document.cookie =
      COOKIE_NAME + "=" + value + "; Path=/; Max-Age=" + COOKIE_MAX_AGE_SECONDS + "; SameSite=Lax";
  }

  // Exposed globally so any future client-side code can check this
  // without needing to re-read localStorage itself.
  window.__vaAnalyticsConsent = currentConsent() === "accepted";

  window.__vaSetCookieConsent = function (value) {
    localStorage.setItem(STORAGE_KEY, value);
    writeConsentCookie(value);
    window.__vaAnalyticsConsent = value === "accepted";
    var banner = document.getElementById("va-cookie-consent");
    if (banner) {
      banner.setAttribute("hidden", "hidden");
    }
  };

  document.addEventListener("DOMContentLoaded", function () {
    var banner = document.getElementById("va-cookie-consent");
    if (banner && !currentConsent()) {
      banner.removeAttribute("hidden");
    }
  });
})();
