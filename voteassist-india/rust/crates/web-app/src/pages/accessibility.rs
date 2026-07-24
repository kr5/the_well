//! `/accessibility` — per docs/12-accessibility-spec.md Section 2.3-2.6:
//! text size, dyslexia-friendly font, reduced motion, high contrast.
//!
//! Implemented via `public/accessibility-prefs.js`, a small vanilla-JS
//! file (not a third Leptos island — docs/PRD-V2-RUST-PLATFORM.md Section
//! 6.6 caps this app at exactly two: the question/answer widget and the
//! language switcher), loaded once in `app.rs`'s shell and referenced here
//! only via plain `onchange="..."` HTML attribute strings (not Leptos
//! `on:change` closures) — these controls work independent of wasm
//! hydration. This is an honest exception to "works with JS disabled":
//! the OS-level `prefers-reduced-motion`/`prefers-contrast` media queries
//! this site's CSS respects by default are the true no-JS baseline
//! (docs/12-accessibility-spec.md Section 2.6); the in-app override
//! toggles below are a progressive-enhancement layer on top, which by
//! nature (a persisted, cross-page, user-adjustable preference) needs
//! *some* client-side mechanism.

use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn AccessibilityPage() -> impl IntoView {
    view! {
        <Title text="Accessibility — VoteAssist India"/>
        <section class="accessibility-page">
            <h1>"Accessibility"</h1>
            <p>
                "VoteAssist India targets WCAG 2.2 Level AA across the whole product, not just "
                "marketing pages. If something here doesn't work with your assistive technology, "
                "please "
                <a href="/feedback">"tell us"</a>
                "."
            </p>

            <div class="a11y-toolbar" role="group" aria-label="Accessibility preferences">
                <fieldset>
                    <legend>"Text size"</legend>
                    <label>
                        <input type="radio" name="va-text-size" value="normal" checked=true
                            onchange="window.__vaSetPref('va-text-size', this.value)"/>
                        " Normal"
                    </label>
                    <label>
                        <input type="radio" name="va-text-size" value="large"
                            onchange="window.__vaSetPref('va-text-size', this.value)"/>
                        " Large"
                    </label>
                    <label>
                        <input type="radio" name="va-text-size" value="larger"
                            onchange="window.__vaSetPref('va-text-size', this.value)"/>
                        " Larger"
                    </label>
                </fieldset>

                <fieldset>
                    <legend>"Font"</legend>
                    <label>
                        <input type="checkbox"
                            onchange="window.__vaSetPref('va-font', this.checked ? 'dyslexic' : '')"/>
                        " Use a dyslexia-friendly font"
                    </label>
                </fieldset>

                <fieldset>
                    <legend>"Contrast"</legend>
                    <label>
                        <input type="checkbox"
                            onchange="window.__vaSetPref('va-contrast', this.checked ? 'high' : '')"/>
                        " High contrast"
                    </label>
                </fieldset>

                <fieldset>
                    <legend>"Motion"</legend>
                    <label>
                        <input type="checkbox"
                            onchange="window.__vaSetPref('va-motion', this.checked ? 'reduced' : '')"/>
                        " Reduce motion (in addition to your browser/OS setting, which we always respect)"
                    </label>
                </fieldset>
            </div>

            <h2>"Using a screen reader"</h2>
            <ul>
                <li>"The decision-engine question widget uses proper fieldset/legend grouping, and announces progress via a polite live region."</li>
                <li>"The non-affiliation banner and verification-caution text are real, readable content — never decorative-only."</li>
                <li>"Citation badges state their verification status in their accessible name, not just via color."</li>
            </ul>
        </section>
    }
}
