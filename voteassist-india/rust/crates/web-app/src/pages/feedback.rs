//! `/feedback` — general feedback and "report inaccurate information",
//! per docs/05-information-architecture.md Section 10. The only page on
//! this site that writes to Postgres — see `server_fns::submit_feedback`.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;

use crate::server_fns::submit_feedback;

#[component]
pub fn FeedbackPage() -> impl IntoView {
    // `/feedback?entry=form-8` pre-fills the "report inaccurate
    // information" flow scoped to a specific KB entry, per the IA doc.
    let query = use_query_map();
    let prefilled_entry_id = move || query.read().get("entry").unwrap_or_default();

    let category = RwSignal::new("general".to_string());
    let message = RwSignal::new(String::new());
    let kb_entry_id = RwSignal::new(String::new());
    let contact_email = RwSignal::new(String::new());

    // Pre-fill from `?entry=...` once, as a genuine side effect rather
    // than inside a view-value read closure (mutating a signal while it's
    // being read is a footgun in any fine-grained-reactive framework).
    Effect::new(move |_| {
        let prefilled = prefilled_entry_id();
        if !prefilled.is_empty() {
            kb_entry_id.set(prefilled);
        }
    });

    let submit = Action::new(move |input: &(String, String, String, String)| {
        let (category, message, kb_entry_id, contact_email) = input.clone();
        async move {
            submit_feedback(
                category,
                message,
                Some(kb_entry_id),
                Some(contact_email),
            )
            .await
        }
    });

    view! {
        <Title text="Feedback — VoteAssist India"/>
        <section class="feedback-page">
            <h1>"Feedback"</h1>
            <p class="not-a-help-desk-notice">
                "VoteAssist India is not an official ECI support desk and can't check your "
                "individual voter status or application. For that, use "
                <a href="https://voters.eci.gov.in" rel="noopener noreferrer">"voters.eci.gov.in"</a>
                " or call 1800-11-1950. This form is for feedback about VoteAssist India itself — "
                "an inaccuracy you spotted, a situation the decision engine didn't cover, or "
                "general comments."
            </p>

            {move || match submit.value().get() {
                Some(Ok(())) => view! {
                    <p class="feedback-success" role="status">
                        "Thank you — your feedback has been recorded."
                    </p>
                }.into_any(),
                Some(Err(_)) => view! {
                    <p class="feedback-error" role="alert">
                        "Something went wrong submitting your feedback. Please try again."
                    </p>
                }.into_any(),
                None => view! {}.into_any(),
            }}

            <form on:submit=move |ev| {
                ev.prevent_default();
                submit.dispatch((
                    category.get(),
                    message.get(),
                    kb_entry_id.get(),
                    contact_email.get(),
                ));
            }>
                <div class="form-field">
                    <label for="feedback-category">"What kind of feedback is this?"</label>
                    <select
                        id="feedback-category"
                        prop:value=move || category.get()
                        on:change=move |ev| category.set(event_target_value(&ev))
                    >
                        <option value="general">"General feedback"</option>
                        <option value="inaccuracy">"Report inaccurate information"</option>
                        <option value="coverage_gap">"This didn't match my situation"</option>
                    </select>
                </div>

                <div class="form-field">
                    <label for="feedback-message">"Your feedback"</label>
                    <textarea
                        id="feedback-message"
                        rows="6"
                        required
                        prop:value=move || message.get()
                        on:input=move |ev| message.set(event_target_value(&ev))
                    ></textarea>
                </div>

                <div class="form-field">
                    <label for="feedback-kb-entry">"Knowledge-base entry ID (optional, e.g. form-8)"</label>
                    <input
                        id="feedback-kb-entry"
                        type="text"
                        prop:value=move || kb_entry_id.get()
                        on:input=move |ev| kb_entry_id.set(event_target_value(&ev))
                    />
                </div>

                <div class="form-field">
                    <label for="feedback-email">"Your email (optional — only if you want a reply)"</label>
                    <input
                        id="feedback-email"
                        type="email"
                        prop:value=move || contact_email.get()
                        on:input=move |ev| contact_email.set(event_target_value(&ev))
                    />
                </div>

                <button type="submit">"Submit feedback"</button>
            </form>
        </section>
    }
}
