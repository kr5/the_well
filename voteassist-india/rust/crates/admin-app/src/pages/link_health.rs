//! Citation & Link Health — PRD v2 Section 11, admin page 6. Reads
//! `link_check_results` (written nightly by `jobs::link_checker`) joined
//! against every cited source, and adds a live "check now" button that
//! writes a fresh row into the same table using the same courteously-
//! identified HTTP client the nightly job uses. Not implemented from that
//! page's full spec: a citation-editor "check this link" button embedded
//! directly in the Knowledge Base Content Editor (page 3) — this is a
//! standalone page instead, cross-referenced by `entry_id`.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;

use crate::server_fns::{check_link_now, list_link_health, LinkHealthRow};

#[component]
pub fn LinkHealthPage() -> impl IntoView {
    let show_broken_only = RwSignal::new(false);
    let links = Resource::new(|| (), |_| async move { list_link_health().await });

    let check_action = Action::new(|source_id: &String| {
        let source_id = source_id.clone();
        async move { check_link_now(source_id).await }
    });

    Effect::new(move |_| {
        if check_action.value().get().is_some() {
            links.refetch();
        }
    });

    view! {
        <Title text="Citation & link health — VoteAssist India Admin"/>
        <h1>"Citation & link health"</h1>
        <p>
            "Every row is a source URL cited by a knowledge-base entry. The nightly job "
            "(crates/jobs::link_checker) rate-limits itself against every host and identifies "
            "itself with a distinct User-Agent — \"Check now\" reuses that same client and "
            "writes into the same history, it does not run a second, differently-behaved crawler."
        </p>

        <div class="form-field inline-form">
            <label>
                <input type="checkbox" prop:checked=move || show_broken_only.get()
                    on:change=move |ev| show_broken_only.set(event_target_checked(&ev))/>
                " Show only broken or never-checked links"
            </label>
        </div>

        {move || check_action.value().get().and_then(|r| r.err()).map(|err| view! {
            <p class="form-error" role="alert">{err.to_string()}</p>
        })}

        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || links.get().map(|result| match result {
                Ok(rows) => {
                    let filtered: Vec<LinkHealthRow> = rows.into_iter()
                        .filter(|row| !show_broken_only.get() || !is_healthy(row))
                        .collect();
                    view! {
                        <table class="admin-table">
                            <thead>
                                <tr><th>"KB entry"</th><th>"Source"</th><th>"URL"</th><th>"Status"</th><th>"Last checked"</th><th></th></tr>
                            </thead>
                            <tbody>
                                {filtered.into_iter().map(|row| {
                                    let source_id = row.source_id.clone();
                                    let healthy = is_healthy(&row);
                                    view! {
                                        <tr>
                                            <td><A href=format!("/kb/{}", row.entry_id)>{row.entry_id.clone()}</A></td>
                                            <td>{row.source_title.clone()}</td>
                                            <td><a href=row.url.clone() target="_blank" rel="noopener noreferrer">{row.url.clone()}</a></td>
                                            <td>
                                                <span class=format!("status-badge {}", if healthy { "status-verified" } else { "status-needs_reverification" })>
                                                    {row.http_status.map(|s| s.to_string())
                                                        .or_else(|| row.error_message.clone())
                                                        .unwrap_or_else(|| "never checked".to_string())}
                                                </span>
                                            </td>
                                            <td>{row.checked_at.map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string()).unwrap_or_else(|| "—".to_string())}</td>
                                            <td>
                                                <button type="button" on:click=move |_| { check_action.dispatch(source_id.clone()); }>
                                                    "Check now"
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    }.into_any()
                }
                Err(e) => view! { <p role="alert">{e.to_string()}</p> }.into_any(),
            })}
        </Suspense>
    }
}

fn is_healthy(row: &LinkHealthRow) -> bool {
    matches!(row.http_status, Some(status) if (200..400).contains(&status))
}
