//! Dashboard (home) page — PRD v2 Section 11, admin page 2.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

use crate::server_fns::dashboard_stats;

#[component]
pub fn DashboardPage() -> impl IntoView {
    let stats = Resource::new(|| (), |_| async move { dashboard_stats().await });

    view! {
        <Title text="Dashboard — VoteAssist India Admin"/>
        <h1>"Dashboard"</h1>
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || stats.get().map(|result| match result {
                Ok(s) => view! {
                    <div class="dashboard-tiles">
                        <A href="/kb" attr:class="dashboard-tile">
                            <span class="tile-value">{s.needs_reverification_count}</span>
                            <span class="tile-label">"Entries needing re-verification"</span>
                        </A>
                        <A href="/kb" attr:class="dashboard-tile">
                            <span class="tile-value">{s.verified_entry_count}</span>
                            <span class="tile-label">"Verified entries live"</span>
                        </A>
                        <A href="/feedback" attr:class="dashboard-tile">
                            <span class="tile-value">{s.new_feedback_count}</span>
                            <span class="tile-label">"New feedback awaiting triage"</span>
                        </A>
                        <A href="/mcc" attr:class="dashboard-tile">
                            <span class="tile-value">{s.active_mcc_window_count}</span>
                            <span class="tile-label">"Active MCC windows"</span>
                        </A>
                    </div>
                }.into_any(),
                Err(e) => view! { <p role="alert">"Could not load dashboard stats: " {e.to_string()}</p> }.into_any(),
            })}
        </Suspense>
    }
}
