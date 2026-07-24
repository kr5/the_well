//! `/search` — "Find my status": roll search guidance + KB search
//! combined, per docs/05-information-architecture.md Section 1.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

use crate::server_fns::search_kb;

#[component]
pub fn SearchPage() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let submitted_query = RwSignal::new(String::new());

    let results = Resource::new(
        move || submitted_query.get(),
        |q| async move {
            if q.trim().is_empty() {
                Ok(Vec::new())
            } else {
                search_kb(q).await
            }
        },
    );

    view! {
        <Title text="Find my status — VoteAssist India"/>
        <section class="search-page">
            <h1>"Find my status"</h1>

            <div class="roll-search-guidance">
                <h2>"Already on the electoral roll?"</h2>
                <p>
                    "To check whether you're already registered, search the electoral roll "
                    "directly on the official ECI service — VoteAssist India does not host a "
                    "copy of the electoral roll and cannot look this up for you."
                </p>
                <a href="https://voters.eci.gov.in" rel="noopener noreferrer" class="deep-link-cta">
                    "Search the electoral roll on voters.eci.gov.in"
                </a>
            </div>

            <div class="kb-search">
                <h2>"Search VoteAssist's knowledge base"</h2>
                <form on:submit=move |ev| {
                    ev.prevent_default();
                    submitted_query.set(query.get());
                }>
                    <label for="kb-search-input">"Search forms, FAQs, and glossary terms"</label>
                    <input
                        id="kb-search-input"
                        type="search"
                        prop:value=move || query.get()
                        on:input=move |ev| query.set(event_target_value(&ev))
                    />
                    <button type="submit">"Search"</button>
                </form>

                <Suspense fallback=|| view! { <p>"Searching..."</p> }>
                    {move || results.get().map(|result| match result {
                        Ok(entries) if entries.is_empty() && !submitted_query.get().is_empty() => {
                            view! { <p>"No knowledge-base entries matched that search."</p> }.into_any()
                        }
                        Ok(entries) => view! {
                            <ul class="search-results">
                                {entries.into_iter().map(|entry| view! {
                                    <li>
                                        <A href=format!("/learn/{}", entry.id)>{entry.title.clone()}</A>
                                        <p class="search-result-summary">{entry.summary.clone()}</p>
                                    </li>
                                }).collect_view()}
                            </ul>
                        }.into_any(),
                        Err(_) => view! { <p role="alert">"Search failed — please try again."</p> }.into_any(),
                    })}
                </Suspense>
            </div>
        </section>
    }
}
