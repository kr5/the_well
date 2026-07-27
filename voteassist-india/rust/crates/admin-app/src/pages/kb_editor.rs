//! Knowledge Base Content Editor — PRD v2 Section 11, admin page 3.
//! Implements list/filter, create/edit form, the review-status
//! state-machine field, a revision-history diff view
//! (`list_kb_entry_revisions`), and an inline citation-health section
//! reusing `list_link_health_for_entry`/`check_link_now` from
//! `pages::link_health` rather than a second implementation. Not
//! implemented from that page's full spec: a full source-citation
//! sub-editor (adding/removing/reordering `knowledge_entry_sources` rows
//! inline) — sources are still edited elsewhere; this page only shows
//! their link-health status.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};

use crate::server_fns::{
    check_link_now, get_kb_entry_admin, list_kb_entries_admin, list_kb_entry_revisions, list_link_health_for_entry,
    save_kb_entry, AdminKbEntryDetail,
};

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
                Ok(Some(entry)) => {
                    let entry_id = entry.id.clone();
                    view! {
                        <Title text=format!("Edit {} — Knowledge base — VoteAssist India Admin", entry.id.clone())/>
                        <h1>"Edit " {entry.id.clone()}</h1>
                        <KbEntryForm entry=Some(entry)/>
                        <EntryLinkHealth entry_id=entry_id.clone()/>
                        <RevisionHistory entry_id=entry_id/>
                    }.into_any()
                }
                Ok(None) => view! { <p role="alert">"Entry not found."</p> }.into_any(),
                Err(e) => view! { <p role="alert">{e.to_string()}</p> }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn EntryLinkHealth(entry_id: String) -> impl IntoView {
    let sources = Resource::new(
        {
            let entry_id = entry_id.clone();
            move || entry_id.clone()
        },
        |entry_id| async move { list_link_health_for_entry(entry_id).await },
    );

    let check_action = Action::new(|source_id: &String| {
        let source_id = source_id.clone();
        async move { check_link_now(source_id).await }
    });
    Effect::new(move |_| {
        if check_action.value().get().is_some() {
            sources.refetch();
        }
    });

    view! {
        <h2>"Citation link health"</h2>
        {move || check_action.value().get().and_then(|r| r.err()).map(|err| view! {
            <p class="form-error" role="alert">{err.to_string()}</p>
        })}
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || sources.get().map(|result| match result {
                Ok(rows) if rows.is_empty() => view! { <p>"No cited sources on this entry."</p> }.into_any(),
                Ok(rows) => view! {
                    <table class="admin-table">
                        <thead><tr><th>"Source"</th><th>"Status"</th><th>"Last checked"</th><th></th></tr></thead>
                        <tbody>
                            {rows.into_iter().map(|row| {
                                let source_id = row.source_id.clone();
                                let healthy = matches!(row.http_status, Some(s) if (200..400).contains(&s));
                                view! {
                                    <tr>
                                        <td><a href=row.url.clone() target="_blank" rel="noopener noreferrer">{row.source_title.clone()}</a></td>
                                        <td>
                                            <span class=format!("status-badge {}", if healthy { "status-verified" } else { "status-needs_reverification" })>
                                                {row.http_status.map(|s| s.to_string()).or_else(|| row.error_message.clone()).unwrap_or_else(|| "never checked".to_string())}
                                            </span>
                                        </td>
                                        <td>{row.checked_at.map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string()).unwrap_or_else(|| "—".to_string())}</td>
                                        <td>
                                            <button type="button" on:click=move |_| { check_action.dispatch(source_id.clone()); }>
                                                "Check now"
                                            </button>
                                        </td>
                                    </tr>
                                }
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
fn RevisionHistory(entry_id: String) -> impl IntoView {
    let revisions = Resource::new(move || entry_id.clone(), |entry_id| async move { list_kb_entry_revisions(entry_id).await });

    view! {
        <h2>"Revision history"</h2>
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || revisions.get().map(|result| match result {
                Ok(rows) if rows.is_empty() => view! { <p>"No revisions yet."</p> }.into_any(),
                Ok(rows) => view! {
                    <table class="admin-table">
                        <thead><tr><th>"Version"</th><th>"Changed fields"</th><th>"Status at time"</th><th>"Changed by"</th><th>"Summary"</th><th>"When"</th></tr></thead>
                        <tbody>
                            {rows.into_iter().map(|rev| view! {
                                <tr>
                                    <td>{rev.version}</td>
                                    <td>{if rev.changed_fields.is_empty() { "—".to_string() } else { rev.changed_fields.join(", ") }}</td>
                                    <td><span class=format!("status-badge status-{}", rev.review_status_at_time)>{rev.review_status_at_time.clone()}</span></td>
                                    <td>{rev.changed_by_email.clone().unwrap_or_else(|| "—".to_string())}</td>
                                    <td>{rev.change_summary.clone().unwrap_or_default()}</td>
                                    <td>{rev.changed_at.format("%Y-%m-%d %H:%M UTC").to_string()}</td>
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
    let translation_group_id =
        RwSignal::new(entry.as_ref().and_then(|e| e.translation_group_id.clone()).unwrap_or_default());
    let is_faq = RwSignal::new(entry.as_ref().map(|e| e.is_faq).unwrap_or(false));
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
                translation_group_id: (!translation_group_id.get().is_empty()).then(|| translation_group_id.get()),
                is_faq: is_faq.get(),
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
                <label for="kb-translation-group">
                    "Translation group ID (optional — shared by every language variant of this content, "
                    "e.g. both \"form-6\" and \"form-6-hi\" set this to \"form-6\")"
                </label>
                <input id="kb-translation-group" type="text"
                    prop:value=move || translation_group_id.get()
                    on:input=move |ev| translation_group_id.set(event_target_value(&ev))/>
            </div>

            <div class="form-field">
                <label>
                    <input type="checkbox" prop:checked=move || is_faq.get()
                        on:change=move |ev| is_faq.set(event_target_checked(&ev))/>
                    " Show on the /learn/faq page"
                </label>
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
