use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <Title text="Page not found — VoteAssist India Admin"/>
        <h1>"Page not found"</h1>
        <A href="/">"Back to the dashboard"</A>
    }
}
