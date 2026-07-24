//! `/locate` — a deep-link hub, not a self-hosted locator database, per
//! docs/05-information-architecture.md Section 6: "MVP explicitly does
//! not host its own polling-station database."

use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn LocatePage() -> impl IntoView {
    view! {
        <Title text="Locate — VoteAssist India"/>
        <section class="locate-page">
            <h1>"Locate"</h1>
            <p>
                "VoteAssist India doesn't host its own copy of polling-station or officer "
                "location data — these official ECI services always have the current, "
                "authoritative answer."
            </p>

            <div class="locate-card">
                <h2>"Find my polling station"</h2>
                <p>"Look up your assigned polling station by EPIC number or details."</p>
                <a href="https://voters.eci.gov.in" rel="noopener noreferrer" class="deep-link-cta">
                    "Find my polling station on voters.eci.gov.in"
                </a>
            </div>

            <div class="locate-card">
                <h2>"Find my ERO / BLO"</h2>
                <p>"Book a call with your Booth Level Officer, or find your Electoral Registration Officer."</p>
                <a href="https://voters.eci.gov.in" rel="noopener noreferrer" class="deep-link-cta">
                    "Voter Helpline app — Book-a-Call with BLO"
                </a>
            </div>

            <div class="locate-card">
                <h2>"Find my CEO's office"</h2>
                <p>"Each state and union territory has a Chief Electoral Officer with a dedicated portal."</p>
                <a href="https://ceo.eci.gov.in" rel="noopener noreferrer" class="deep-link-cta">
                    "State CEO portal directory"
                </a>
            </div>

            <p class="locate-helpline">
                "You can also call the National Voter Helpline at "
                <a href="tel:1800111950">"1800-11-1950"</a>
                "."
            </p>
        </section>
    }
}
