//! Home / entry (`/`). Per docs/05-information-architecture.md Section 2.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Title text="VoteAssist India — voter guidance, not a government portal"/>
        <section class="hero">
            <h1>"Understand what you need to do to vote in India."</h1>
            <p class="hero-subtitle">
                "We'll ask a few questions about your situation, then point you to the "
                "official Election Commission of India service to actually complete it."
            </p>
            <A href="/start" attr:class="cta-primary">"Find out what I need to do"</A>
        </section>

        <section class="secondary-entries">
            <A href="/search" attr:class="cta-secondary">
                "Search the knowledge base"
            </A>
            <A href="/locate" attr:class="cta-secondary">
                "Find my polling station / BLO"
            </A>
        </section>

        <section class="home-explainer">
            <h2>"What VoteAssist India is — and isn't"</h2>
            <p>
                "VoteAssist India is an independent, non-official guide. It doesn't register "
                "you to vote, doesn't submit any form on your behalf, and isn't affiliated with "
                "the Election Commission of India or any government body. It helps you figure "
                "out which official form or process applies to your situation, then links you "
                "directly to the real thing."
            </p>
            <A href="/about/what-this-is">"Read more about who runs this →"</A>
        </section>
    }
}
