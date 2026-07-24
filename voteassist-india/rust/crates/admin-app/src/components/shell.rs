//! The authenticated admin shell: nav, current-user display, logout —
//! wraps every route except `/login`. Shows a "please log in" prompt
//! instead of the wrapped page when there's no valid session; this is a
//! UX nicety, not the security boundary (see `app.rs`'s module doc).

use leptos::prelude::*;
use leptos_router::components::A;

use crate::server_fns::{current_user, logout};

#[component]
pub fn AdminShell(children: Children) -> impl IntoView {
    let user = Resource::new(|| (), |_| async move { current_user().await });
    let logout_action = Action::new(|_: &()| async move { logout().await });

    Effect::new(move |_| {
        if logout_action.value().get().is_some() {
            user.refetch();
        }
    });

    view! {
        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || {
                let children = children();
                user.get().map(|result| match result {
                    Ok(Some(admin)) => view! {
                        <div class="admin-shell">
                            <header class="admin-header">
                                <A href="/">"VoteAssist India — Admin"</A>
                                <nav aria-label="Admin">
                                    <A href="/kb">"Knowledge base"</A>
                                    <A href="/mcc">"MCC control panel"</A>
                                    <A href="/audit-log">"Audit log"</A>
                                </nav>
                                <div class="admin-user-info">
                                    <span>{admin.email.clone()}" (" {admin.role.clone()} ")"</span>
                                    <button
                                        type="button"
                                        on:click=move |_| { logout_action.dispatch(()); }
                                    >
                                        "Log out"
                                    </button>
                                </div>
                            </header>
                            <main class="admin-main">{children}</main>
                        </div>
                    }.into_any(),
                    _ => view! {
                        <div class="admin-login-prompt">
                            <p>"You need to log in to view this page."</p>
                            <A href="/login">"Go to login"</A>
                        </div>
                    }.into_any(),
                })
            }}
        </Suspense>
    }
}
