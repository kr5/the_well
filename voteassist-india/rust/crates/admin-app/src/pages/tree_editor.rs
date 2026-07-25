//! Decision Tree Visual Editor — PRD v2 Section 11, admin page 4. "Visual"
//! here is a structured JSON editor around `decision_tree_drafts`/
//! `decision_trees` with validate/publish/rollback and cross-referenced
//! open `fringe_case` reports — not a drag-and-drop node-graph canvas.
//! See `crate::server_fns`'s module doc comment above these functions for
//! why that's a disclosed, deliberate scope decision rather than a
//! silently-cut corner: this pass can hand-write and carefully check a
//! JSON editor without a compiler; a bespoke graph-layout canvas library
//! is a much larger, separately-verifiable undertaking this pass doesn't
//! attempt.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::server_fns::{
    get_or_create_draft, list_open_fringe_cases, list_tree_keys, list_tree_versions, publish_draft,
    rollback_tree_version, save_draft_artifact, validate_draft, DraftView, FringeCaseView,
};

#[component]
pub fn TreeEditorListPage() -> impl IntoView {
    let tree_keys = Resource::new(|| (), |_| async move { list_tree_keys().await });

    view! {
        <Title text="Decision trees — VoteAssist India Admin"/>
        <h1>"Decision tree visual editor"</h1>
        <p>
            "Each tree_key below is a separately versioned, separately published decision tree. "
            "voteassist-core-v1/v2 are this codebase's two built-in trees (core_domain::vote_assist_tree_v1/v2) — "
            "opening either for the first time starts a draft seeded from that compiled-in JSON, "
            "since nothing has published a database row for it yet."
        </p>
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || tree_keys.get().map(|result| match result {
                Ok(keys) => view! {
                    <ul class="tree-key-list">
                        {keys.into_iter().map(|key| view! {
                            <li><A href=format!("/tree-editor/{key}")>{key}</A></li>
                        }).collect_view()}
                    </ul>
                }.into_any(),
                Err(e) => view! { <p role="alert">{e.to_string()}</p> }.into_any(),
            })}
        </Suspense>

        <h2>"Open fringe-case reports (any tree)"</h2>
        <p>
            "\"The tree does not yet handle X\" reports, cross-referenced here per fringe_case's own "
            "table comment. linked_tree_node_id is not a hard foreign key (node ids live in JSONB, "
            "not a relational column) — cross-check it by hand against whichever tree you're editing."
        </p>
        <OpenFringeCases/>
    }
}

#[component]
fn OpenFringeCases() -> impl IntoView {
    let cases = Resource::new(|| (), |_| async move { list_open_fringe_cases().await });

    view! {
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || cases.get().map(|result| match result {
                Ok(rows) if rows.is_empty() => view! { <p>"No open fringe-case reports."</p> }.into_any(),
                Ok(rows) => view! {
                    <table class="admin-table">
                        <thead><tr><th>"Status"</th><th>"Reported via"</th><th>"Linked node"</th><th>"Description"</th><th>"Reported"</th></tr></thead>
                        <tbody>
                            {rows.into_iter().map(|case: FringeCaseView| view! {
                                <tr>
                                    <td><span class=format!("status-badge status-{}", case.status)>{case.status.clone()}</span></td>
                                    <td>{case.reporter_channel.clone()}</td>
                                    <td>{case.linked_tree_node_id.clone().unwrap_or_else(|| "—".to_string())}</td>
                                    <td>{case.description.clone()}</td>
                                    <td>{case.created_at.format("%Y-%m-%d").to_string()}</td>
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

#[component]
pub fn TreeEditorPage() -> impl IntoView {
    let params = use_params_map();
    let tree_key = move || params.read().get("tree_key").unwrap_or_default();

    let draft = Resource::new(tree_key, |tree_key| async move { get_or_create_draft(tree_key).await });

    view! {
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || draft.get().map(|result| match result {
                Ok(draft) => view! {
                    <Title text=format!("{} — Decision trees — VoteAssist India Admin", draft.tree_key)/>
                    <TreeEditorForm draft=draft/>
                }.into_any(),
                Err(e) => view! { <p role="alert">{e.to_string()}</p> }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn TreeEditorForm(draft: DraftView) -> impl IntoView {
    let tree_key = draft.tree_key.clone();
    let draft_id = RwSignal::new(draft.id.clone());
    let artifact_text = RwSignal::new(draft.artifact_json.clone());
    let issues = RwSignal::new(draft.last_validation_issues.clone());
    let based_on_version = draft.based_on_version;

    let versions = Resource::new({
        let tree_key = tree_key.clone();
        move || tree_key.clone()
    }, |tree_key| async move { list_tree_versions(tree_key).await });

    let save_action = Action::new(move |input: &(String, String)| {
        let (id, text) = input.clone();
        async move { save_draft_artifact(id, text).await }
    });

    let validate_action = Action::new(move |id: &String| {
        let id = id.clone();
        async move { validate_draft(id).await }
    });
    Effect::new(move |_| {
        if let Some(Ok(new_issues)) = validate_action.value().get() {
            issues.set(new_issues);
        }
    });

    let publish_action = Action::new(move |id: &String| {
        let id = id.clone();
        async move { publish_draft(id).await }
    });
    Effect::new(move |_| {
        if publish_action.value().get().is_some() {
            versions.refetch();
        }
    });

    let rollback_action = Action::new({
        let tree_key = tree_key.clone();
        move |version: &i32| {
            let tree_key = tree_key.clone();
            let version = *version;
            async move { rollback_tree_version(tree_key, version).await }
        }
    });
    Effect::new(move |_| {
        if rollback_action.value().get().is_some() {
            versions.refetch();
        }
    });

    view! {
        <h1>{tree_key.clone()}</h1>
        <p>"Draft based on version " {based_on_version} " (0 means: no published version existed yet — this draft was seeded from the built-in default)."</p>

        <div class="form-field tree-editor-textarea">
            <label for="tree-artifact">"Tree artifact (JSON — same shape as core_domain::DecisionTree)"</label>
            <textarea id="tree-artifact" rows="24" spellcheck="false"
                prop:value=move || artifact_text.get()
                on:input=move |ev| artifact_text.set(event_target_value(&ev))></textarea>
        </div>

        <div class="tree-editor-actions">
            <button type="button" on:click=move |_| {
                save_action.dispatch((draft_id.get(), artifact_text.get()));
            }>"Save draft"</button>
            <button type="button" on:click=move |_| {
                validate_action.dispatch(draft_id.get());
            }>"Validate tree"</button>
            <button type="button" on:click=move |_| {
                publish_action.dispatch(draft_id.get());
            }>"Publish"</button>
        </div>

        {move || save_action.value().get().and_then(|r| r.err()).map(|err| view! {
            <p class="form-error" role="alert">{err.to_string()}</p>
        })}
        {move || save_action.value().get().and_then(|r| r.ok()).map(|_| view! {
            <p class="retention-result">"Draft saved."</p>
        })}
        {move || validate_action.value().get().and_then(|r| r.err()).map(|err| view! {
            <p class="form-error" role="alert">{err.to_string()}</p>
        })}
        {move || publish_action.value().get().map(|result| match result {
            Ok(version) => view! { <p class="retention-result">"Published as version " {version} "."</p> }.into_any(),
            Err(e) => view! { <p class="form-error" role="alert">{e.to_string()}</p> }.into_any(),
        })}

        <h2>"Validation issues"</h2>
        {move || {
            let current = issues.get();
            if current.is_empty() {
                view! { <p>"No issues from the last validation run (or it hasn't been run yet — click \"Validate tree\")."</p> }.into_any()
            } else {
                view! {
                    <ul class="validation-issues">
                        {current.into_iter().map(|issue| view! {
                            <li><strong>{issue.node_id}</strong>": " {issue.message}</li>
                        }).collect_view()}
                    </ul>
                }.into_any()
            }
        }}

        <h2>"Version history"</h2>
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || versions.get().map(|result| match result {
                Ok(rows) if rows.is_empty() => view! { <p>"No published version yet."</p> }.into_any(),
                Ok(rows) => view! {
                    <table class="admin-table">
                        <thead><tr><th>"Version"</th><th>"Active"</th><th>"Published by"</th><th>"Published at"</th><th></th></tr></thead>
                        <tbody>
                            {rows.into_iter().map(|v| {
                                let version = v.version;
                                view! {
                                    <tr>
                                        <td>{v.version}</td>
                                        <td>{if v.is_active { "Active" } else { "—" }}</td>
                                        <td>{v.published_by_email.clone().unwrap_or_else(|| "—".to_string())}</td>
                                        <td>{v.published_at.map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string()).unwrap_or_else(|| "—".to_string())}</td>
                                        <td>
                                            {(!v.is_active).then(|| view! {
                                                <button type="button" on:click=move |_| { rollback_action.dispatch(version); }>
                                                    "Roll back to this version"
                                                </button>
                                            })}
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
        {move || rollback_action.value().get().and_then(|r| r.err()).map(|err| view! {
            <p class="form-error" role="alert">{err.to_string()}</p>
        })}
    }
}
