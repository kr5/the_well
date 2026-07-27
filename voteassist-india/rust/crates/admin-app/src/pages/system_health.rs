//! System Health — a single "is everything up" view, not one of PRD v2
//! Section 11's original 13 numbered admin pages but a natural pairing
//! with the Analytics Dashboard and `/metrics` Prometheus scraping this
//! workspace already has: those answer "how is the product being used"
//! and "what's the request-rate/latency trend," this answers "is each
//! service's process actually responding right now," a question a
//! `/metrics` time series doesn't answer at a glance.
//!
//! Pings every service's `/healthz` (same address/env-var map as
//! `scripts/health-check.sh` — see `server_fns::check_service_health`)
//! and separately surfaces `bot_channel_config`'s status
//! (`pages::bot_channels` owns actually changing it; this page is
//! read-only and just cross-links there).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

use crate::server_fns::{check_service_health, list_bot_channels};

#[component]
pub fn SystemHealthPage() -> impl IntoView {
    let services = Resource::new(|| (), |_| async move { check_service_health().await });
    let channels = Resource::new(|| (), |_| async move { list_bot_channels().await });

    view! {
        <Title text="System health — VoteAssist India Admin"/>
        <h1>"System health"</h1>
        <p>
            "Liveness only (\"is the process responding\"), not the request-rate/latency trends "
            "Prometheus scrapes from each service's /metrics endpoint — see "
            <code>"docs/SECURITY-AND-SRE-OPERATIONS.md"</code>" for real alerting on those."
        </p>

        <h2>"Services"</h2>
        <Suspense fallback=|| view! { <p>"Checking..."</p> }>
            {move || services.get().map(|result| match result {
                Ok(rows) => view! {
                    <table class="admin-table">
                        <thead><tr><th>"Service"</th><th>"Address"</th><th>"Status"</th><th>"Detail"</th></tr></thead>
                        <tbody>
                            {rows.into_iter().map(|row| view! {
                                <tr>
                                    <td>{row.name.clone()}</td>
                                    <td>{row.address.clone()}</td>
                                    <td>
                                        <span class=format!("status-badge {}", if row.healthy { "status-verified" } else { "status-needs_reverification" })>
                                            {if row.healthy { "up" } else { "down" }}
                                        </span>
                                    </td>
                                    <td>{row.detail.clone()}</td>
                                </tr>
                            }).collect_view()}
                        </tbody>
                    </table>
                }.into_any(),
                Err(e) => view! { <p role="alert">{e.to_string()}</p> }.into_any(),
            })}
        </Suspense>

        <h2>"Bot channels"</h2>
        <p>"Read-only here — toggle these from " <A href="/bot-channels">"Bot Channel Management"</A>"."</p>
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || channels.get().map(|result| match result {
                Ok(rows) => view! {
                    <table class="admin-table">
                        <thead><tr><th>"Channel"</th><th>"Enabled"</th><th>"Webhook configured"</th></tr></thead>
                        <tbody>
                            {rows.into_iter().map(|row| view! {
                                <tr>
                                    <td>{row.display_name.clone()}</td>
                                    <td>{if row.is_enabled { "yes" } else { "no" }}</td>
                                    <td>{if row.webhook_configured { "yes" } else { "no" }}</td>
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
