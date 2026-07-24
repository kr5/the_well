use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <Title text="Page not found — VoteAssist India"/>
        <section class="not-found-page">
            <h1>"Page not found"</h1>
            <p>"That page doesn't exist. Here are some places to start:"</p>
            <ul>
                <li><A href="/start">"Find out what I need to do"</A></li>
                <li><A href="/learn">"Browse the knowledge base"</A></li>
                <li><A href="/">"Home"</A></li>
            </ul>
        </section>
    }
}
