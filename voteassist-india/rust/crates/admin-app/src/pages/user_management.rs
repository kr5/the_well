//! User & Role Management — PRD v2 Section 11, admin page 10.
//! Superadmin-only: list admin accounts, create new ones, toggle active
//! status, and change roles. Until this page existed, the only ways to
//! do any of this were `scripts/seed-superadmin.sh` / `cargo run -p xtask
//! -- create-admin` and direct SQL — this page and the CLI now both call
//! the same server functions' underlying logic (mirrored validation, not
//! a shared code path, since the CLI runs standalone against
//! `DATABASE_URL` with no HTTP server involved).

use leptos::prelude::*;
use leptos_meta::Title;

use crate::server_fns::{create_admin_user, list_admin_users, set_admin_user_active, set_admin_user_role, AdminUserView};

const ROLES: &[&str] =
    &["contributor", "reviewer", "legal_reviewer", "translator", "analytics_viewer", "superadmin"];

#[component]
pub fn UserManagementPage() -> impl IntoView {
    let users = Resource::new(|| (), |_| async move { list_admin_users().await });

    let new_email = RwSignal::new(String::new());
    let new_password = RwSignal::new(String::new());
    let new_role = RwSignal::new(ROLES[0].to_string());

    let create_action = Action::new(move |input: &(String, String, String)| {
        let (email, password, role) = input.clone();
        async move { create_admin_user(email, password, role).await }
    });
    let toggle_active_action = Action::new(|input: &(String, bool)| {
        let (id, is_active) = input.clone();
        async move { set_admin_user_active(id, is_active).await }
    });
    let role_action = Action::new(|input: &(String, String)| {
        let (id, role) = input.clone();
        async move { set_admin_user_role(id, role).await }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = create_action.value().get() {
            new_email.set(String::new());
            new_password.set(String::new());
            users.refetch();
        }
    });
    Effect::new(move |_| {
        if toggle_active_action.value().get().is_some() {
            users.refetch();
        }
    });
    Effect::new(move |_| {
        if role_action.value().get().is_some() {
            users.refetch();
        }
    });

    view! {
        <Title text="User & role management — VoteAssist India Admin"/>
        <h1>"User & role management"</h1>
        <p>"Superadmin-only. Every change here writes an audit_log row, same as any other admin action."</p>

        <h2>"Create admin account"</h2>
        <form class="inline-form" on:submit=move |ev| {
            ev.prevent_default();
            create_action.dispatch((new_email.get(), new_password.get(), new_role.get()));
        }>
            <div class="form-field">
                <label for="new-admin-email">"Email"</label>
                <input id="new-admin-email" type="email" required
                    prop:value=move || new_email.get()
                    on:input=move |ev| new_email.set(event_target_value(&ev))/>
            </div>
            <div class="form-field">
                <label for="new-admin-password">"Password (min 12 characters)"</label>
                <input id="new-admin-password" type="password" required minlength="12"
                    prop:value=move || new_password.get()
                    on:input=move |ev| new_password.set(event_target_value(&ev))/>
            </div>
            <div class="form-field">
                <label for="new-admin-role">"Role"</label>
                <select id="new-admin-role" prop:value=move || new_role.get()
                    on:change=move |ev| new_role.set(event_target_value(&ev))>
                    {ROLES.iter().map(|r| view! { <option value=*r>{*r}</option> }).collect_view()}
                </select>
            </div>
            <button type="submit">"Create"</button>
        </form>
        {move || create_action.value().get().and_then(|r| r.err()).map(|err| view! {
            <p class="form-error" role="alert">{err.to_string()}</p>
        })}

        <h2>"Admin accounts"</h2>
        {move || toggle_active_action.value().get().and_then(|r| r.err()).map(|err| view! {
            <p class="form-error" role="alert">{err.to_string()}</p>
        })}
        {move || role_action.value().get().and_then(|r| r.err()).map(|err| view! {
            <p class="form-error" role="alert">{err.to_string()}</p>
        })}
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || users.get().map(|result| match result {
                Ok(rows) => view! {
                    <table class="admin-table">
                        <thead>
                            <tr><th>"Email"</th><th>"Role"</th><th>"Active"</th><th>"Created"</th><th>"Last login"</th></tr>
                        </thead>
                        <tbody>
                            {rows.into_iter().map(|user: AdminUserView| {
                                let id_for_role = user.id.clone();
                                let id_for_toggle = user.id.clone();
                                let is_active = user.is_active;
                                view! {
                                    <tr>
                                        <td>{user.email.clone()}</td>
                                        <td>
                                            <select prop:value=user.role.clone()
                                                on:change=move |ev| { role_action.dispatch((id_for_role.clone(), event_target_value(&ev))); }>
                                                {ROLES.iter().map(|r| view! { <option value=*r>{*r}</option> }).collect_view()}
                                            </select>
                                        </td>
                                        <td>
                                            <button type="button" on:click=move |_| {
                                                toggle_active_action.dispatch((id_for_toggle.clone(), !is_active));
                                            }>
                                                {if is_active { "Deactivate" } else { "Reactivate" }}
                                            </button>
                                        </td>
                                        <td>{user.created_at.format("%Y-%m-%d").to_string()}</td>
                                        <td>{user.last_login_at.map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string()).unwrap_or_else(|| "never".to_string())}</td>
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
