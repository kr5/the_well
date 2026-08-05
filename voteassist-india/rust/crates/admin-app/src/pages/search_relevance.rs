//! Search Relevance — not one of PRD v2 Section 11's original 13 numbered
//! admin pages, but a debugging view over `kb_content::search_entries_ranked`
//! (the new ranked, field-weighted, alias-aware in-process search that
//! replaced the original naive `.contains()`-across-fields MVP search —
//! see that module's doc comment for the full scoring formula). Before
//! this page, the only way to see *why* one KB entry outranked another for
//! a given query — or to notice a near-tie, or to see the effect of the
//! engine's relative 15%-of-top-score cutoff — was to read `search.rs`'s
//! own unit tests. This page runs the same ranked search a citizen-facing
//! surface would, but shows the score, not just the final ordering.
//!
//! ## Why scores and a "% of top" column, not just a ranked list
//!
//! `kb_content::search`'s module doc documents a `RELATIVE_CUTOFF` of 15%:
//! any result scoring below 15% of the top-scoring result for the same
//! query is dropped before it ever reaches a caller. That cutoff is
//! invisible in a plain ranked list — every row still just looks like "the
//! next result." Showing each result's score as a fraction of the top
//! score makes the cutoff's effect legible: every row here is
//! (by construction) `>= 15%` of the top, and a reviewer tuning content or
//! wondering "why didn't X show up" can see exactly how close it came.
//! Scores are shown to 4 decimal places (not rounded to 1-2) so
//! near-ties — two entries a reviewer might reasonably expect to compete
//! for first place — are visible rather than collapsed into an apparent
//! tie by display rounding.
//!
//! ## The two distinct empty states
//!
//! An empty results table means one of two very different things, and
//! this page is deliberately built so a reviewer never has to guess which:
//! either no query has been submitted yet (nothing to distinguish from —
//! the "query was empty" state, shown before the first search or after
//! clearing the field without re-submitting), or a query WAS submitted and
//! the search engine itself found nothing scoring above its relevance
//! threshold (a real, informative negative result about the KB's
//! vocabulary coverage, not a UI glitch). See `SearchResultsView` below for
//! how the distinction is tracked.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

use crate::server_fns::{search_relevance_debug, SearchRelevanceRow};

#[component]
pub fn SearchRelevancePage() -> impl IntoView {
    let query_input = RwSignal::new(String::new());
    // The most recently *submitted* query, distinct from `query_input`
    // (which tracks every keystroke) — the search only re-runs on submit,
    // not on every keystroke, and this signal is what tells the empty-state
    // renderer below whether "no rows" means "nothing submitted yet" or
    // "a real query came back empty."
    let submitted_query = RwSignal::new(String::new());

    let results = Resource::new(
        move || submitted_query.get(),
        |q| async move {
            if q.trim().is_empty() {
                // Never round-trips an empty query to the server — an
                // empty/whitespace-only query is meaningless input, not a
                // valid search to run (mirrors `search_entries_ranked`'s
                // own "empty query returns nothing" behaviour, just
                // short-circuited client-side so the "query was empty"
                // state doesn't wait on a server round trip either).
                Ok(Vec::new())
            } else {
                search_relevance_debug(q).await
            }
        },
    );

    view! {
        <Title text="Search relevance — VoteAssist India Admin"/>
        <h1>"Search relevance"</h1>
        <p>
            "Debugging view over " <code>"kb_content::search_entries_ranked"</code>
            " — the ranked, field-weighted, alias-aware knowledge-base search. Scores are shown to "
            "4 decimal places so near-ties are visible, and \"% of top\" shows how close each result "
            "came to the top-scoring match for the same query — every row here necessarily scores at "
            "least 15% of the top result, since the search engine itself drops anything below that "
            "before this page ever sees it."
        </p>

        <form
            class="inline-form"
            on:submit=move |ev| {
                ev.prevent_default();
                submitted_query.set(query_input.get());
            }
        >
            <div class="form-field">
                <label for="search-relevance-query">"Query"</label>
                <input id="search-relevance-query" type="text"
                    placeholder="e.g. \"voter id\", \"change my address\", \"form 6\""
                    prop:value=move || query_input.get()
                    on:input=move |ev| query_input.set(event_target_value(&ev))/>
            </div>
            <button type="submit">"Search"</button>
        </form>

        <Suspense fallback=|| view! { <p>"Searching..."</p> }>
            {move || {
                let submitted = submitted_query.get();
                results.get().map(|result| match result {
                    Ok(rows) => view! { <ResultsOrEmptyState submitted_query=submitted.clone() rows=rows/> }.into_any(),
                    Err(e) => view! { <p role="alert">{e.to_string()}</p> }.into_any(),
                })
            }}
        </Suspense>
    }
}

#[component]
fn ResultsOrEmptyState(submitted_query: String, rows: Vec<SearchRelevanceRow>) -> impl IntoView {
    if submitted_query.trim().is_empty() {
        view! { <p>"Enter a query above and press Search."</p> }.into_any()
    } else if rows.is_empty() {
        view! {
            <p>
                "The search ran for \"" {submitted_query} "\" and matched nothing scoring above the "
                "engine's relative 15%-of-top-score cutoff — this is a real result (no sufficiently "
                "relevant entry exists for this query), not an empty/unsubmitted query."
            </p>
        }.into_any()
    } else {
        view! {
            <table class="admin-table">
                <thead>
                    <tr>
                        <th>"Rank"</th><th>"Score"</th><th>"% of top"</th>
                        <th>"Entry"</th><th>"Title"</th><th>"Topic"</th><th>"Status"</th>
                    </tr>
                </thead>
                <tbody>
                    {rows.into_iter().map(|row| view! {
                        <tr>
                            <td>{row.rank}</td>
                            <td>{format!("{:.4}", row.score)}</td>
                            <td>{format!("{:.1}%", row.ratio_of_top * 100.0)}</td>
                            <td><A href=format!("/kb/{}", row.entry_id)>{row.entry_id.clone()}</A></td>
                            <td>{row.title.clone()}</td>
                            <td>{row.topic.clone()}</td>
                            <td>
                                <span class=format!("status-badge status-{}", row.review_status)>
                                    {row.review_status.clone()}
                                </span>
                            </td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
        }.into_any()
    }
}
