// Cookie/tracking consent banner logic. See
// src/components/cookie_consent.rs's module doc for the full reasoning
// (why this is vanilla JS rather than a third Leptos island, what
// "necessary" vs "analytics" actually covers on this site today).
(function () {
  var STORAGE_KEY = "va-cookie-consent";

  function currentConsent() {
    return localStorage.getItem(STORAGE_KEY);
  }

  // Exposed globally so any future analytics-recording code (wherever it
  // ends up calling crates/analytics::record_event) can check this before
  // firing, without needing to re-read localStorage itself.
  window.__vaAnalyticsConsent = currentConsent() === "accepted";

  window.__vaSetCookieConsent = function (value) {
    localStorage.setItem(STORAGE_KEY, value);
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
