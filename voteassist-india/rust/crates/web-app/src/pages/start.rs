//! Decision-engine entry (`/start`). Per docs/05-information-architecture.md
//! Section 3: "client-routed, not deep-linkable mid-flow by default" — the
//! individual question steps and the result screen are all rendered
//! within this one route by `QuestionFlow`'s own reactive state, not
//! separate server routes, precisely so a URL can never be bookmarked or
//! shared mid-walkthrough in a way that implies a specific personal
//! situation.

use leptos::prelude::*;
use leptos_meta::Title;

use crate::components::question_flow::QuestionFlow;

#[component]
pub fn StartPage() -> impl IntoView {
    view! {
        <Title text="What do you need to do? — VoteAssist India"/>
        <section class="start-page">
            <h1>"What do you need help with?"</h1>
            <QuestionFlow/>
        </section>
    }
}
