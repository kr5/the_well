//! `/account/login` and `/account` — the optional public account system's
//! UI: OTP login and a dashboard of saved checklists, notification
//! preferences, and consent history. Strictly opt-in per
//! docs/PRD-V3-COMPREHENSIVE-EXPANSION.md Section V6 — nothing elsewhere
//! on this site requires an account.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

use crate::server_fns_accounts::{
    current_account, delete_account, delete_draft, list_consent_history, list_my_drafts, logout_account,
    request_otp, set_notification_prefs, verify_otp_and_login,
};

#[component]
pub fn AccountLoginPage() -> impl IntoView {
    let contact = RwSignal::new(String::new());
    let otp = RwSignal::new(String::new());
    let otp_requested = RwSignal::new(false);

    let request_action = Action::new(move |contact: &String| {
        let contact = contact.clone();
        async move { request_otp(contact, "email".to_string()).await }
    });

    let verify_action = Action::new(move |input: &(String, String)| {
        let (contact, otp) = input.clone();
        async move { verify_otp_and_login(contact, "email".to_string(), otp).await }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = request_action.value().get() {
            otp_requested.set(true);
        }
    });

    view! {
        <Title text="Log in — VoteAssist India"/>
        <section class="account-page">
            <h1>"Save your checklist"</h1>
            <p>
                "An account is entirely optional and is only needed to save a checklist across "
                "visits. We only ever store a one-way hash of your email — never the address "
                "itself, and never anything like your name, EPIC number, or Aadhaar number. See "
                <A href="/about/privacy">"our privacy policy"</A>
                "."
            </p>

            {move || request_action.value().get().and_then(|r| r.err()).map(|err| view! {
                <p class="form-error" role="alert">{err.to_string()}</p>
            })}
            {move || verify_action.value().get().and_then(|r| r.err()).map(|err| view! {
                <p class="form-error" role="alert">{err.to_string()}</p>
            })}

            <form on:submit=move |ev| {
                ev.prevent_default();
                if otp_requested.get() {
                    verify_action.dispatch((contact.get(), otp.get()));
                } else {
                    request_action.dispatch(contact.get());
                }
            }>
                <div class="form-field">
                    <label for="account-email">"Email address"</label>
                    <input id="account-email" type="email" required disabled=move || otp_requested.get()
                        prop:value=move || contact.get()
                        on:input=move |ev| contact.set(event_target_value(&ev))/>
                </div>

                {move || otp_requested.get().then(|| view! {
                    <div class="form-field">
                        <label for="account-otp">"6-digit code sent to your email"</label>
                        <input id="account-otp" type="text" inputmode="numeric" maxlength="6" required
                            prop:value=move || otp.get()
                            on:input=move |ev| otp.set(event_target_value(&ev))/>
                    </div>
                })}

                <button type="submit">{move || if otp_requested.get() { "Verify and log in" } else { "Send code" }}</button>
            </form>
        </section>
    }
}

#[component]
pub fn AccountDashboardPage() -> impl IntoView {
    let account = Resource::new(|| (), |_| async move { current_account().await });
    let drafts = Resource::new(|| (), |_| async move { list_my_drafts().await });
    let consent_history = Resource::new(|| (), |_| async move { list_consent_history().await });

    let logout_action = Action::new(|_: &()| async move { logout_account().await });
    let delete_draft_action = Action::new(|id: &String| {
        let id = id.clone();
        async move { delete_draft(id).await }
    });
    let delete_account_action = Action::new(|_: &()| async move { delete_account().await });
    let notif_action = Action::new(|opt_in: &bool| {
        let opt_in = *opt_in;
        async move { set_notification_prefs(opt_in, None).await }
    });

    Effect::new(move |_| {
        if delete_draft_action.value().get().is_some() {
            drafts.refetch();
        }
    });

    Effect::new(move |_| {
        if logout_action.value().get().is_some() || delete_account_action.value().get().is_some() {
            account.refetch();
        }
    });

    view! {
        <Title text="My account — VoteAssist India"/>
        <section class="account-page">
            <h1>"My account"</h1>

            <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                {move || account.get().map(|result| match result {
                    Ok(Some(_)) => view! {
                        <div>
                            <h2>"Saved checklists"</h2>
                            <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                                {move || drafts.get().map(|result| match result {
                                    Ok(drafts) if drafts.is_empty() => view! {
                                        <p>"You haven't saved any checklists yet. Reach a result on "
                                        <A href="/start">"the decision engine"</A>" and choose \"Save this checklist.\""</p>
                                    }.into_any(),
                                    Ok(drafts) => view! {
                                        <ul class="draft-list">
                                            {drafts.into_iter().map(|d| {
                                                let id = d.id.clone();
                                                view! {
                                                    <li>
                                                        <span>{d.label.clone().unwrap_or_else(|| "Untitled checklist".to_string())}</span>
                                                        <span class="draft-meta">
                                                            {if d.is_frozen { " (result saved)" } else { " (in progress)" }}
                                                        </span>
                                                        <button type="button" on:click=move |_| { delete_draft_action.dispatch(id.clone()); }>
                                                            "Delete"
                                                        </button>
                                                    </li>
                                                }
                                            }).collect_view()}
                                        </ul>
                                    }.into_any(),
                                    Err(e) => view! { <p role="alert">{e.to_string()}</p> }.into_any(),
                                })}
                            </Suspense>

                            <h2>"Notifications"</h2>
                            <p>
                                <label>
                                    <input type="checkbox"
                                        on:change=move |ev| { notif_action.dispatch(event_target_checked(&ev)); }/>
                                    " Notify me about election reminders for my state (optional, revocable any time)"
                                </label>
                            </p>

                            <h2>"Consent history"</h2>
                            <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                                {move || consent_history.get().map(|result| match result {
                                    Ok(entries) if entries.is_empty() => view! { <p>"No consent history yet."</p> }.into_any(),
                                    Ok(entries) => view! {
                                        <ul class="consent-history">
                                            {entries.into_iter().map(|c| view! {
                                                <li>
                                                    {c.purpose.clone()}": "
                                                    {if c.granted { "granted" } else { "revoked" }}
                                                    " at "{c.occurred_at.to_rfc3339()}
                                                </li>
                                            }).collect_view()}
                                        </ul>
                                    }.into_any(),
                                    Err(e) => view! { <p role="alert">{e.to_string()}</p> }.into_any(),
                                })}
                            </Suspense>

                            <h2>"Account actions"</h2>
                            <button type="button" on:click=move |_| { logout_action.dispatch(()); }>"Log out"</button>
                            <button type="button" class="danger-button" on:click=move |_| {
                                delete_account_action.dispatch(());
                            }>
                                "Delete my account and all saved checklists"
                            </button>
                        </div>
                    }.into_any(),
                    _ => view! {
                        <div class="account-login-prompt">
                            <p>"You're not logged in."</p>
                            <A href="/account/login">"Log in or create an account"</A>
                        </div>
                    }.into_any(),
                })}
            </Suspense>
        </section>
    }
}
