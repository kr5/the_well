//! Feedback & Grievance Triage — PRD v2 Section 11, admin page 7.
//! Lists the `feedback` table (written by `web-app`'s `/feedback` page),
//! filterable by triage status, with an inline status/notes editor per
//! row and a "Convert to fringe case" button
//! (`server_fns::convert_feedback_to_fringe_case`) that files a tracked
//! `fringe_case` row from it, cross-referenced on `pages::tree_editor`.

use leptos::prelude::*;
use leptos_meta::Title;

use crate::server_fns::{convert_feedback_to_fringe_case, list_feedback, update_feedback_status, FeedbackView};

const STATUSES: &[&str] = &["new", "triaged", "resolved", "wontfix"];

#[component]
pub fn FeedbackTriagePage() -> impl IntoView {
    let status_filter = RwSignal::new("new".to_string());
    let feedback = Resource::new(move || status_filter.get(), |filter| async move { list_feedback(Some(filter)).await });

    let save_action = Action::new(move |input: &(String, String, String)| {
        let (id, status, notes) = input.clone();
        async move { update_feedback_status(id, status, (!notes.trim().is_empty()).then_some(notes)).await }
    });

    let convert_action = Action::new(|feedback_id: &String| {
        let feedback_id = feedback_id.clone();
        async move { convert_feedback_to_fringe_case(feedback_id).await }
    });

    Effect::new(move |_| {
        if save_action.value().get().is_some() {
            feedback.refetch();
        }
    });
    Effect::new(move |_| {
        if convert_action.value().get().is_some() {
            feedback.refetch();
        }
    });

    view! {
        <Title text="Feedback triage — VoteAssist India Admin"/>
        <h1>"Feedback & grievance triage"</h1>
        <p>
            "Feedback is minimal-PII by design (docs/06-legal-compliance-review.md): "
            "only an optional contact email identifies the sender. Contact emails on "
            "resolved/wontfix rows are swept after 180 days by the Data Export & "
            "Retention page's retention job, per migration 0008's documented policy."
        </p>

        <div class="form-field inline-form">
            <label for="feedback-status-filter">"Filter by status"</label>
            <select id="feedback-status-filter" prop:value=move || status_filter.get()
                on:change=move |ev| status_filter.set(event_target_value(&ev))>
                <option value="all">"All"</option>
                {STATUSES.iter().map(|s| view! { <option value=*s>{*s}</option> }).collect_view()}
            </select>
        </div>

        {move || save_action.value().get().and_then(|r| r.err()).map(|err| view! {
            <p class="form-error" role="alert">{err.to_string()}</p>
        })}
        {move || convert_action.value().get().and_then(|r| r.err()).map(|err| view! {
            <p class="form-error" role="alert">{err.to_string()}</p>
        })}

        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || feedback.get().map(|result| match result {
                Ok(rows) if rows.is_empty() => view! { <p>"No feedback in this status."</p> }.into_any(),
                Ok(rows) => view! {
                    <div class="feedback-list">
                        {rows.into_iter().map(|row| view! {
                            <FeedbackRow row=row save_action=save_action convert_action=convert_action/>
                        }).collect_view()}
                    </div>
                }.into_any(),
                Err(e) => view! { <p role="alert">{e.to_string()}</p> }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn FeedbackRow(
    row: FeedbackView,
    save_action: Action<(String, String, String), Result<(), ServerFnError>>,
    convert_action: Action<String, Result<(), ServerFnError>>,
) -> impl IntoView {
    let id = row.id.clone();
    let id_for_convert = row.id.clone();
    let status = RwSignal::new(row.status.clone());
    let notes = RwSignal::new(row.internal_notes.clone().unwrap_or_default());

    view! {
        <article class="feedback-card">
            <header>
                <span class=format!("status-badge status-{}", row.status)>{row.status.clone()}</span>
                <span class="feedback-category">{row.category.clone()}</span>
                <time datetime=row.created_at.to_rfc3339()>{row.created_at.format("%Y-%m-%d %H:%M UTC").to_string()}</time>
            </header>
            <p class="feedback-message">{row.message.clone()}</p>
            {row.kb_entry_id.clone().map(|kb_id| view! { <p><small>"Linked KB entry: " {kb_id}</small></p> })}
            {row.contact_email.clone().map(|email| view! { <p><small>"Contact: " {email}</small></p> })}

            <div class="form-field">
                <label>"Status"</label>
                <select prop:value=move || status.get() on:change=move |ev| status.set(event_target_value(&ev))>
                    {STATUSES.iter().map(|s| view! { <option value=*s>{*s}</option> }).collect_view()}
                </select>
            </div>
            <div class="form-field">
                <label>"Internal notes (not shown to the citizen who submitted this)"</label>
                <textarea rows="2" prop:value=move || notes.get() on:input=move |ev| notes.set(event_target_value(&ev))></textarea>
            </div>
            <div class="feedback-actions">
                <button type="button" on:click=move |_| {
                    save_action.dispatch((id.clone(), status.get(), notes.get()));
                }>"Save"</button>
                <button type="button" on:click=move |_| {
                    convert_action.dispatch(id_for_convert.clone());
                }>
                    "Convert to fringe case"
                </button>
            </div>
        </article>
    }
}
