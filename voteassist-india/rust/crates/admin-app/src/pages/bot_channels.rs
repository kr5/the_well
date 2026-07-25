//! Bot Channel Management — PRD v2 Section 11, admin page 9. Per-channel
//! enable/disable (independent of MCC status — the gap `mcc_panel.rs`'s
//! module doc names) and the WhatsApp template pre-approval workflow.
//! Not implemented from that page's full spec: live webhook connectivity
//! probing (`webhook_configured`/`last_health_check_*` are read here, but
//! nothing in this page actively re-checks them — that's `bot-whatsapp`/
//! `bot-telegram`'s own `/healthz`, scraped by Prometheus per
//! `rust/README.md`'s monitoring section, not re-implemented here).

use leptos::prelude::*;
use leptos_meta::Title;

use crate::server_fns::{
    create_whatsapp_template, list_bot_channels, list_whatsapp_templates, set_bot_channel_enabled,
    set_whatsapp_template_status,
};

const TEMPLATE_STATUSES: &[&str] = &["pending", "approved", "rejected"];

#[component]
pub fn BotChannelsPage() -> impl IntoView {
    let channels = Resource::new(|| (), |_| async move { list_bot_channels().await });
    let templates = Resource::new(|| (), |_| async move { list_whatsapp_templates().await });

    let toggle_action = Action::new(|input: &(String, bool)| {
        let (channel, enabled) = input.clone();
        async move { set_bot_channel_enabled(channel, enabled).await }
    });

    let template_name = RwSignal::new(String::new());
    let template_body = RwSignal::new(String::new());
    let template_locale = RwSignal::new("en".to_string());
    let create_template_action = Action::new(move |input: &(String, String, String)| {
        let (name, body, locale) = input.clone();
        async move { create_whatsapp_template(name, body, locale).await }
    });

    let status_action = Action::new(|input: &(String, String)| {
        let (id, status) = input.clone();
        async move { set_whatsapp_template_status(id, status).await }
    });

    Effect::new(move |_| {
        if toggle_action.value().get().is_some() {
            channels.refetch();
        }
    });
    Effect::new(move |_| {
        if let Some(Ok(())) = create_template_action.value().get() {
            template_name.set(String::new());
            template_body.set(String::new());
            templates.refetch();
        }
    });
    Effect::new(move |_| {
        if status_action.value().get().is_some() {
            templates.refetch();
        }
    });

    view! {
        <Title text="Bot channels — VoteAssist India Admin"/>
        <h1>"Bot channel management"</h1>

        <h2>"Channels"</h2>
        <p>
            "Disabling a channel here is independent of an MCC window (crates/channel-core's "
            "\"is this state under a Model Code of Conduct restriction right now\" check) — this "
            "is a blunt, whole-channel kill switch for operational reasons (an incident, a "
            "provider outage, a policy pause), gated to legal_reviewer/superadmin."
        </p>
        {move || toggle_action.value().get().and_then(|r| r.err()).map(|err| view! {
            <p class="form-error" role="alert">{err.to_string()}</p>
        })}
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || channels.get().map(|result| match result {
                Ok(rows) => view! {
                    <table class="admin-table">
                        <thead><tr><th>"Channel"</th><th>"Enabled"</th><th>"Webhook configured"</th><th>"Last health check"</th></tr></thead>
                        <tbody>
                            {rows.into_iter().map(|row| {
                                let channel = row.channel.clone();
                                let is_enabled = row.is_enabled;
                                view! {
                                    <tr>
                                        <td>{row.display_name.clone()}</td>
                                        <td>
                                            <label>
                                                <input type="checkbox" prop:checked=is_enabled
                                                    on:change=move |ev| {
                                                        toggle_action.dispatch((channel.clone(), event_target_checked(&ev)));
                                                    }/>
                                                " enabled"
                                            </label>
                                        </td>
                                        <td>{if row.webhook_configured { "yes" } else { "no" }}</td>
                                        <td>
                                            {match (row.last_health_check_at, row.last_health_check_ok) {
                                                (Some(t), Some(true)) => format!("OK at {}", t.format("%Y-%m-%d %H:%M UTC")),
                                                (Some(t), Some(false)) => format!("FAILING since {}", t.format("%Y-%m-%d %H:%M UTC")),
                                                _ => "never checked".to_string(),
                                            }}
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

        <h2>"WhatsApp message templates"</h2>
        <p>
            "Meta requires pre-approval for template messages sent outside a user-initiated "
            "24-hour window. This table only records that review workflow by hand — it does not "
            "call Meta's API to submit or check templates."
        </p>

        <form class="inline-form" on:submit=move |ev| {
            ev.prevent_default();
            create_template_action.dispatch((template_name.get(), template_body.get(), template_locale.get()));
        }>
            <div class="form-field">
                <label for="tpl-name">"Template name"</label>
                <input id="tpl-name" type="text" required
                    prop:value=move || template_name.get()
                    on:input=move |ev| template_name.set(event_target_value(&ev))/>
            </div>
            <div class="form-field">
                <label for="tpl-body">"Template body"</label>
                <input id="tpl-body" type="text" required
                    prop:value=move || template_body.get()
                    on:input=move |ev| template_body.set(event_target_value(&ev))/>
            </div>
            <div class="form-field">
                <label for="tpl-locale">"Locale"</label>
                <input id="tpl-locale" type="text" required
                    prop:value=move || template_locale.get()
                    on:input=move |ev| template_locale.set(event_target_value(&ev))/>
            </div>
            <button type="submit">"Submit for review"</button>
        </form>

        {move || create_template_action.value().get().and_then(|r| r.err()).map(|err| view! {
            <p class="form-error" role="alert">{err.to_string()}</p>
        })}
        {move || status_action.value().get().and_then(|r| r.err()).map(|err| view! {
            <p class="form-error" role="alert">{err.to_string()}</p>
        })}

        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || templates.get().map(|result| match result {
                Ok(rows) if rows.is_empty() => view! { <p>"No templates submitted yet."</p> }.into_any(),
                Ok(rows) => view! {
                    <table class="admin-table">
                        <thead><tr><th>"Name"</th><th>"Locale"</th><th>"Body"</th><th>"Status"</th><th>"Approved"</th></tr></thead>
                        <tbody>
                            {rows.into_iter().map(|row| {
                                let id = row.id.clone();
                                view! {
                                    <tr>
                                        <td>{row.template_name.clone()}</td>
                                        <td>{row.locale.clone()}</td>
                                        <td>{row.template_body.clone()}</td>
                                        <td>
                                            <select prop:value=row.meta_approval_status.clone()
                                                on:change=move |ev| { status_action.dispatch((id.clone(), event_target_value(&ev))); }>
                                                {TEMPLATE_STATUSES.iter().map(|s| view! { <option value=*s>{*s}</option> }).collect_view()}
                                            </select>
                                        </td>
                                        <td>{row.approved_at.map(|t| t.format("%Y-%m-%d").to_string()).unwrap_or_else(|| "—".to_string())}</td>
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
