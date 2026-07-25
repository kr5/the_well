//! Translation Management — PRD v2 Section 11, admin page 5. Reads the
//! completeness dashboard `translation_status` feeds (`jobs`' nightly
//! computation, or "recompute now" here) — see this table's own module
//! doc for the disclosed "row count, not a strict per-entry match"
//! caveat.
//!
//! Not implemented: a per-string translation workbench (editing
//! individual translated strings inline). That workflow already exists,
//! just not inside this admin UI — `rust/crates/xtask`'s
//! `translate-kb-entry`/`translate-tree` commands produce draft
//! translations (via Claude Haiku or NVIDIA NIM's Nemotron models, see
//! `xtask/src/translate/client.rs`), and `docs/20-translation-task-tracker.md`
//! is the actual, disclosed review-and-merge workflow a human reviewer
//! follows today. Building a live in-browser string editor on top of that
//! is a real, separable follow-up, not something this page silently
//! promises and doesn't do.

use leptos::prelude::*;
use leptos_meta::Title;

use crate::server_fns::{list_translation_status, recompute_translation_status, TranslationStatusRow};

#[component]
pub fn TranslationManagementPage() -> impl IntoView {
    let status = Resource::new(|| (), |_| async move { list_translation_status().await });
    let recompute_action = Action::new(|_: &()| async move { recompute_translation_status().await });

    Effect::new(move |_| {
        if recompute_action.value().get().is_some() {
            status.refetch();
        }
    });

    view! {
        <Title text="Translation management — VoteAssist India Admin"/>
        <h1>"Translation management"</h1>
        <p>
            "Machine-translation drafts (Claude Haiku or NVIDIA NIM's free-tier Nemotron models — "
            "see rust/crates/xtask) are never auto-published. Every row below counts KB entries "
            "already reviewed and merged into knowledge_entries with language set to that locale — "
            "not drafts sitting in translation-drafts/, which this dashboard cannot see by design "
            "(a draft isn't real content until a human has reviewed it)."
        </p>
        <p>
            "Full workflow: " <code>"docs/20-translation-task-tracker.md"</code>
        </p>

        <button type="button" on:click=move |_| { recompute_action.dispatch(()); }>
            "Recompute now"
        </button>
        {move || recompute_action.value().get().and_then(|r| r.err()).map(|err| view! {
            <p class="form-error" role="alert">{err.to_string()}</p>
        })}

        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || status.get().map(|result| match result {
                Ok(rows) if rows.is_empty() => view! {
                    <p>"No completeness data yet — click \"Recompute now\" or wait for the nightly job."</p>
                }.into_any(),
                Ok(rows) => view! {
                    <table class="admin-table">
                        <thead>
                            <tr><th>"Locale"</th><th>"Content type"</th><th>"Total"</th><th>"Translated"</th><th>"Reviewed"</th><th>"Computed"</th></tr>
                        </thead>
                        <tbody>
                            {rows.into_iter().map(|row: TranslationStatusRow| view! {
                                <tr>
                                    <td>{row.locale.clone()}</td>
                                    <td>{row.content_type.clone()}</td>
                                    <td>{row.total_items}</td>
                                    <td>{row.translated_items}</td>
                                    <td>{row.reviewed_items}</td>
                                    <td>{row.computed_at.format("%Y-%m-%d %H:%M UTC").to_string()}</td>
                                </tr>
                            }).collect_view()}
                        </tbody>
                    </table>
                }.into_any(),
                Err(e) => view! { <p role="alert">{e.to_string()}</p> }.into_any(),
            })}
        </Suspense>
    }
}
