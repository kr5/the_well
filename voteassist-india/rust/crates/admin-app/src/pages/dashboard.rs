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
                        <div class="dashboard-tile">
                            <span class="tile-value">{s.new_feedback_count}</span>
                            <span class="tile-label">"New feedback awaiting triage"</span>
                        </div>
                        <A href="/mcc" attr:class="dashboard-tile">
                            <span class="tile-value">{s.active_mcc_window_count}</span>
                            <span class="tile-label">"Active MCC windows"</span>
                        </A>
                    </div>
                }.into_any(),
                Err(e) => view! { <p role="alert">"Could not load dashboard stats: " {e.to_string()}</p> }.into_any(),
            })}
        </Suspense>

        <h2>"Not yet built in this pass"</h2>
        <p>
            "Feedback & Grievance Triage, Bot Channel Management, User & Role Management, the "
            "Translation Management workbench, the Decision Tree Visual Editor, the Analytics "
            "Dashboard, and Data Export & Retention Tools (PRD v2 Section 11 admin pages 4, 5, "
            "7, 9, 10, 12, 13) are documented but not implemented yet — see this crate's README."
        </p>
    }
}
