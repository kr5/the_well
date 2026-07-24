//! Analytics Dashboard — PRD v2 Section 11, admin page 12. Reads only the
//! fully-anonymized `analytics_rollups_daily` aggregate table — see
//! `server_fns::analytics_summary`'s module doc for why this page
//! structurally cannot offer a per-user drill-down.

use leptos::prelude::*;
use leptos_meta::Title;

use crate::server_fns::analytics_summary;

#[component]
pub fn AnalyticsDashboardPage() -> impl IntoView {
    let window_days = RwSignal::new(30i32);
    let summary = Resource::new(move || window_days.get(), |days| async move { analytics_summary(days).await });

    view! {
        <Title text="Analytics — VoteAssist India Admin"/>
        <div class="page-header">
            <h1>"Analytics"</h1>
            <select on:change=move |ev| {
                if let Ok(days) = event_target_value(&ev).parse::<i32>() {
                    window_days.set(days);
                }
            }>
                <option value="7">"Last 7 days"</option>
                <option value="30" selected=true>"Last 30 days"</option>
                <option value="90">"Last 90 days"</option>
            </select>
        </div>
        <p class="analytics-caveat">
            "Aggregate, anonymized counts only — no session- or user-level detail exists in "
            "this table to drill into (see docs/PRD-V2-RUST-PLATFORM.md Section 12)."
        </p>

        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || summary.get().map(|result| match result {
                Ok(summary) => view! {
                    <div class="analytics-grid">
                        <div class="analytics-card">
                            <h2>"Events by type"</h2>
                            <table class="admin-table">
                                <thead><tr><th>"Event"</th><th>"Count"</th></tr></thead>
                                <tbody>
                                    {summary.by_event_type.iter().map(|row| view! {
                                        <tr><td>{row.event_type.clone()}</td><td>{row.total_events}</td></tr>
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>

                        <div class="analytics-card">
                            <h2>"Events by channel"</h2>
                            <table class="admin-table">
                                <thead><tr><th>"Channel"</th><th>"Count"</th></tr></thead>
                                <tbody>
                                    {summary.by_platform.iter().map(|row| view! {
                                        <tr>
                                            <td>{row.client_platform.clone().unwrap_or_else(|| "unknown".to_string())}</td>
                                            <td>{row.total_events}</td>
                                        </tr>
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>

                        <div class="analytics-card analytics-card-wide">
                            <h2>"Sessions started per day"</h2>
                            <table class="admin-table">
                                <thead><tr><th>"Day"</th><th>"Sessions"</th></tr></thead>
                                <tbody>
                                    {summary.daily_sessions.iter().map(|row| view! {
                                        <tr><td>{row.bucket_day.to_string()}</td><td>{row.session_count}</td></tr>
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>
                    </div>
                }.into_any(),
                Err(e) => view! { <p role="alert">{e.to_string()}</p> }.into_any(),
            })}
        </Suspense>
    }
}
