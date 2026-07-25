//! Data Export & Retention Tools — PRD v2 Section 11, admin page 13.
//! Superadmin-only. Three real, working actions, each enforcing a policy
//! already documented elsewhere in this workspace rather than inventing
//! a new one here: expired-session purging (same statements as `xtask
//! purge-expired-sessions`), the feedback `contact_email` 180-day
//! post-resolution sweep (`migrations/0008`'s documented policy), and an
//! `audit_log` date-range export for external compliance review (the
//! table's own comment names this as the intended use).
//!
//! Not implemented: a per-citizen "export/delete everything you hold
//! about me" self-service flow answering a DPDP data-subject request end
//! to end. That's deliberately out of scope for this specific admin page
//! (see `docs/06-legal-compliance-review.md`'s DPDP section) — this
//! project holds very little identifying data by design (hash-only
//! account contacts, optional feedback email), so most such requests
//! resolve via `web-app`'s existing self-service account deletion
//! (`crate::accounts::delete_account` in that crate) rather than an admin
//! tool acting on a citizen's behalf.

use leptos::prelude::*;
use leptos_meta::Title;

use crate::server_fns::{
    export_audit_log, purge_expired_sessions_now, purge_old_feedback_contact_emails,
};

#[component]
pub fn DataRetentionPage() -> impl IntoView {
    let purge_sessions_action = Action::new(|_: &()| async move { purge_expired_sessions_now().await });
    let purge_feedback_action = Action::new(|_: &()| async move { purge_old_feedback_contact_emails().await });

    let from_date = RwSignal::new(String::new());
    let to_date = RwSignal::new(String::new());
    let export_action = Action::new(move |input: &(String, String)| {
        let (from, to) = input.clone();
        async move {
            let from = from.parse().map_err(|_| ServerFnError::ServerError("invalid start date".to_string()))?;
            let to = to.parse().map_err(|_| ServerFnError::ServerError("invalid end date".to_string()))?;
            export_audit_log(from, to).await
        }
    });

    view! {
        <Title text="Data export & retention — VoteAssist India Admin"/>
        <h1>"Data export & retention tools"</h1>
        <p>"Superadmin-only. Every action below writes its own audit_log row."</p>

        <section class="retention-section">
            <h2>"Expired sessions"</h2>
            <p>"Deletes rows already past their expiry from " <code>"sessions"</code>", "
               <code>"account_sessions"</code>", and "<code>"account_otp_challenges"</code>
               " — the same sweep " <code>"scripts/purge-expired-sessions.sh"</code> " runs on a schedule."</p>
            <button type="button" on:click=move |_| { purge_sessions_action.dispatch(()); }>
                "Purge expired sessions now"
            </button>
            {move || purge_sessions_action.value().get().map(|result| match result {
                Ok(summary) => view! {
                    <p class="retention-result">
                        "Purged " {summary.admin_sessions_purged} " admin session(s), "
                        {summary.account_sessions_purged} " account session(s), "
                        {summary.otp_challenges_purged} " OTP challenge(s)."
                    </p>
                }.into_any(),
                Err(e) => view! { <p class="form-error" role="alert">{e.to_string()}</p> }.into_any(),
            })}
        </section>

        <section class="retention-section">
            <h2>"Feedback contact-email retention"</h2>
            <p>"Clears " <code>"feedback.contact_email"</code> " on rows resolved 180+ days ago, per "
               <code>"migrations/0008_feedback_and_fringe_cases.sql"</code>"'s documented policy. "
               "The feedback message/category text itself is kept as anonymized product-feedback history."</p>
            <button type="button" on:click=move |_| { purge_feedback_action.dispatch(()); }>
                "Sweep old feedback contact emails"
            </button>
            {move || purge_feedback_action.value().get().map(|result| match result {
                Ok(count) => view! { <p class="retention-result">"Cleared " {count} " contact email(s)."</p> }.into_any(),
                Err(e) => view! { <p class="form-error" role="alert">{e.to_string()}</p> }.into_any(),
            })}
        </section>

        <section class="retention-section">
            <h2>"Audit log export"</h2>
            <p>"For external compliance review. Exports every audit_log row in the selected date range as JSON."</p>
            <form class="inline-form" on:submit=move |ev| {
                ev.prevent_default();
                export_action.dispatch((from_date.get(), to_date.get()));
            }>
                <div class="form-field">
                    <label for="export-from">"From"</label>
                    <input id="export-from" type="date" required
                        prop:value=move || from_date.get()
                        on:input=move |ev| from_date.set(event_target_value(&ev))/>
                </div>
                <div class="form-field">
                    <label for="export-to">"To"</label>
                    <input id="export-to" type="date" required
                        prop:value=move || to_date.get()
                        on:input=move |ev| to_date.set(event_target_value(&ev))/>
                </div>
                <button type="submit">"Prepare export"</button>
            </form>
            {move || export_action.value().get().map(|result| match result {
                Ok(data_uri) => view! {
                    <p><a href=data_uri download="voteassist-audit-log-export.json">"Download audit-log-export.json"</a></p>
                }.into_any(),
                Err(e) => view! { <p class="form-error" role="alert">{e.to_string()}</p> }.into_any(),
            })}
        </section>
    }
}
