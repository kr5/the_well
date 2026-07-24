//! Audit Log Viewer — PRD v2 Section 11, admin page 11. Reads the single
//! unified `audit_log` table every content edit, tree publish, role
//! change, and MCC toggle writes to.

use leptos::prelude::*;
use leptos_meta::Title;

use crate::server_fns::list_audit_log;

#[component]
pub fn AuditLogPage() -> impl IntoView {
    let entries = Resource::new(|| (), |_| async move { list_audit_log(200).await });

    view! {
        <Title text="Audit log — VoteAssist India Admin"/>
        <h1>"Audit log"</h1>
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || entries.get().map(|result| match result {
                Ok(entries) => view! {
                    <table class="admin-table">
                        <thead>
                            <tr><th>"When"</th><th>"Who"</th><th>"Action"</th><th>"Target"</th></tr>
                        </thead>
                        <tbody>
                            {entries.into_iter().map(|entry| view! {
                                <tr>
                                    <td>{entry.occurred_at.to_rfc3339()}</td>
                                    <td>{entry.actor_email.clone().unwrap_or_else(|| "system".to_string())}</td>
                                    <td>{entry.action.clone()}</td>
                                    <td>{entry.target_type.clone()}" / "{entry.target_id.clone()}</td>
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
