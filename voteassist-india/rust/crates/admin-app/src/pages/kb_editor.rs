//! Knowledge Base Content Editor — PRD v2 Section 11, admin page 3.
//! Implements list/filter, create/edit form, and the review-status
//! state-machine field. Not implemented from that page's full spec: the
//! diff view (the underlying `knowledge_entry_revisions` data this would
//! read from is written correctly by `save_kb_entry`, but no UI reads it
//! back yet) and the source-citation sub-editor with a live link-check
//! button (`crates/jobs::link_checker` exists and writes
//! `link_check_results`, but this page doesn't call it on demand or
//! render that table yet) — both disclosed, real follow-ups.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};

use crate::server_fns::{get_kb_entry_admin, list_kb_entries_admin, save_kb_entry, AdminKbEntryDetail};

#[component]
pub fn KbListPage() -> impl IntoView {
    let entries = Resource::new(|| (), |_| async move { list_kb_entries_admin().await });

    view! {
        <Title text="Knowledge base — VoteAssist India Admin"/>
        <div class="page-header">
            <h1>"Knowledge base"</h1>
            <A href="/kb/new" attr:class="cta-primary">"New entry"</A>
        </div>
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || entries.get().map(|result| match result {
                Ok(entries) => view! {
                    <table class="admin-table">
                        <thead>
                            <tr>
                                <th>"Title"</th><th>"Topic"</th><th>"Status"</th>
                                <th>"Last verified"</th><th>"Version"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {entries.into_iter().map(|entry| view! {
                                <tr>
                                    <td><A href=format!("/kb/{}", entry.id)>{entry.title.clone()}</A></td>
                                    <td>{entry.topic.clone()}</td>
                                    <td><span class=format!("status-badge status-{}", entry.review_status)>
                                        {entry.review_status.clone()}
                                    </span></td>
                                    <td>{entry.last_verified_date.to_string()}</td>
                                    <td>{entry.version}</td>
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

#[component]
pub fn KbEditorNewPage() -> impl IntoView {
    view! {
        <Title text="New entry — Knowledge base — VoteAssist India Admin"/>
        <h1>"New knowledge-base entry"</h1>
        <KbEntryForm entry=None/>
    }
}

#[component]
pub fn KbEditorPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").unwrap_or_default();
    let entry = Resource::new(id, |id| async move { get_kb_entry_admin(id).await });

    view! {
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || entry.get().map(|result| match result {
                Ok(Some(entry)) => view! {
                    <Title text=format!("Edit {} — Knowledge base — VoteAssist India Admin", entry.id.clone())/>
                    <h1>"Edit " {entry.id.clone()}</h1>
                    <KbEntryForm entry=Some(entry)/>
                }.into_any(),
                Ok(None) => view! { <p role="alert">"Entry not found."</p> }.into_any(),
                Err(e) => view! { <p role="alert">{e.to_string()}</p> }.into_any(),
            })}
        </Suspense>
    }
}

const TOPICS: &[&str] = &[
    "registration", "correction", "shifting-of-residence", "deletion-objection", "epic",
    "ordinary-residence", "nri-voter", "service-voter", "pwd-voter", "qualifying-dates",
    "grievance", "polling-station", "roll-search", "glossary",
    "local-body-elections", "documents-alternatives", "transgender-elector",
];
const SOURCE_TYPES: &[&str] =
    &["eci_official", "state_ceo", "gazette_law", "sveep", "pib_release", "community_pending_verification"];
const REVIEW_STATUSES: &[&str] = &["draft", "in_review", "verified", "needs_reverification"];

#[component]
fn KbEntryForm(entry: Option<AdminKbEntryDetail>) -> impl IntoView {
    let navigate = use_navigate();
    let is_new = entry.is_none();

    let id = RwSignal::new(entry.as_ref().map(|e| e.id.clone()).unwrap_or_default());
    let topic = RwSignal::new(entry.as_ref().map(|e| e.topic.clone()).unwrap_or_else(|| TOPICS[0].to_string()));
    let title = RwSignal::new(entry.as_ref().map(|e| e.title.clone()).unwrap_or_default());
    let summary = RwSignal::new(entry.as_ref().map(|e| e.summary.clone()).unwrap_or_default());
    let body = RwSignal::new(entry.as_ref().map(|e| e.body.clone()).unwrap_or_default());
    let source_type =
        RwSignal::new(entry.as_ref().map(|e| e.source_type.clone()).unwrap_or_else(|| SOURCE_TYPES[0].to_string()));
    let last_verified_date = RwSignal::new(
        entry.as_ref().map(|e| e.last_verified_date.to_string()).unwrap_or_default(),
    );
    let review_status =
        RwSignal::new(entry.as_ref().map(|e| e.review_status.clone()).unwrap_or_else(|| REVIEW_STATUSES[0].to_string()));
    let caution = RwSignal::new(entry.as_ref().and_then(|e| e.caution.clone()).unwrap_or_default());
    let change_summary = RwSignal::new(String::new());

    let save_action = Action::new(move |input: &(AdminKbEntryDetail, String)| {
        let (entry, change_summary) = input.clone();
        async move { save_kb_entry(entry, change_summary).await }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = save_action.value().get() {
            navigate("/kb", Default::default());
        }
    });

    view! {
        <form on:submit=move |ev| {
            ev.prevent_default();
            let parsed_date = last_verified_date.get().parse().unwrap_or_else(|_| {
                chrono::Utc::now().date_naive()
            });
            let entry = AdminKbEntryDetail {
                id: id.get(),
                topic: topic.get(),
                title: title.get(),
                summary: summary.get(),
                body: body.get(),
                source_type: source_type.get(),
                last_verified_date: parsed_date,
                review_status: review_status.get(),
                caution: (!caution.get().is_empty()).then(|| caution.get()),
            };
            save_action.dispatch((entry, change_summary.get()));
        }>
            {move || save_action.value().get().and_then(|r| r.err()).map(|err| view! {
                <p class="form-error" role="alert">{err.to_string()}</p>
            })}

            <div class="form-field">
                <label for="kb-id">"Entry ID (lowercase, hyphens only, e.g. form-6)"</label>
                <input id="kb-id" type="text" required disabled=!is_new
                    prop:value=move || id.get() on:input=move |ev| id.set(event_target_value(&ev))/>
            </div>

            <div class="form-field">
                <label for="kb-topic">"Topic"</label>
                <select id="kb-topic" prop:value=move || topic.get() on:change=move |ev| topic.set(event_target_value(&ev))>
                    {TOPICS.iter().map(|t| view! { <option value=*t>{*t}</option> }).collect_view()}
                </select>
            </div>

            <div class="form-field">
                <label for="kb-title">"Title"</label>
                <input id="kb-title" type="text" required
                    prop:value=move || title.get() on:input=move |ev| title.set(event_target_value(&ev))/>
            </div>

            <div class="form-field">
                <label for="kb-summary">"Summary (max 500 characters)"</label>
                <textarea id="kb-summary" rows="3" maxlength="500" required
                    prop:value=move || summary.get() on:input=move |ev| summary.set(event_target_value(&ev))></textarea>
            </div>

            <div class="form-field">
                <label for="kb-body">"Body (Markdown)"</label>
                <textarea id="kb-body" rows="10"
                    prop:value=move || body.get() on:input=move |ev| body.set(event_target_value(&ev))></textarea>
            </div>

            <div class="form-field">
                <label for="kb-source-type">"Source type"</label>
                <select id="kb-source-type" prop:value=move || source_type.get()
                    on:change=move |ev| source_type.set(event_target_value(&ev))>
                    {SOURCE_TYPES.iter().map(|t| view! { <option value=*t>{*t}</option> }).collect_view()}
                </select>
            </div>

            <div class="form-field">
                <label for="kb-last-verified">"Last verified date"</label>
                <input id="kb-last-verified" type="date" required
                    prop:value=move || last_verified_date.get()
                    on:input=move |ev| last_verified_date.set(event_target_value(&ev))/>
            </div>

            <div class="form-field">
                <label for="kb-review-status">"Review status"</label>
                <select id="kb-review-status" prop:value=move || review_status.get()
                    on:change=move |ev| review_status.set(event_target_value(&ev))>
                    {REVIEW_STATUSES.iter().map(|s| view! { <option value=*s>{*s}</option> }).collect_view()}
                </select>
            </div>

            <div class="form-field">
                <label for="kb-caution">"Caution note (optional)"</label>
                <textarea id="kb-caution" rows="2"
                    prop:value=move || caution.get() on:input=move |ev| caution.set(event_target_value(&ev))></textarea>
            </div>

            <div class="form-field">
                <label for="kb-change-summary">"What changed and why (for the revision history)"</label>
                <input id="kb-change-summary" type="text" required
                    prop:value=move || change_summary.get()
                    on:input=move |ev| change_summary.set(event_target_value(&ev))/>
            </div>

            <button type="submit">"Save"</button>
        </form>
    }
}
