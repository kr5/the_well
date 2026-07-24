//! The persistent non-affiliation banner. Non-negotiable per
//! docs/06-legal-compliance-review.md Section 1 ("must appear on every
//! page, not just a buried footer") and docs/16-security-threat-model.md's
//! phishing-lookalike risk entry ("carried into the Leptos SSR templates
//! so it renders even for non-JS/low-bandwidth clients").
//!
//! Deliberately has no dismiss/minimize control: the requirement is
//! "persistent... not dismissible past a minimized state," and the
//! simplest reading that satisfies "never fully hidden" without adding a
//! third hydrated island (docs/PRD-V2-RUST-PLATFORM.md Section 6.6 caps
//! this app at exactly two: the question/answer widget and the language
//! switcher) is to render it as plain, always-visible, non-interactive
//! markup. A minimize affordance is a legitimate future enhancement, not
//! a requirement this component silently drops — see this module's own
//! doc for the reasoning.

use leptos::prelude::*;

pub const NOT_OFFICIAL_BANNER_TEXT: &str = "VoteAssist India is an independent, non-official guidance tool. It is not the Election Commission of India and cannot register you to vote or submit any application on your behalf.";

#[component]
pub fn NotOfficialBanner() -> impl IntoView {
    view! {
        <div class="not-official-banner" role="note" aria-label="Non-affiliation notice">
            <p>
                <strong>"Not an official government site. "</strong>
                {NOT_OFFICIAL_BANNER_TEXT}
                " "
                <a href="/about/what-this-is">"Learn more about who runs this."</a>
            </p>
        </div>
    }
}
