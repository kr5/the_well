use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;

use crate::server_fns::login;

#[component]
pub fn LoginPage() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let navigate = use_navigate();

    let login_action = Action::new(move |input: &(String, String)| {
        let (email, password) = input.clone();
        async move { login(email, password).await }
    });

    Effect::new(move |_| {
        if let Some(Ok(())) = login_action.value().get() {
            navigate("/", Default::default());
        }
    });

    view! {
        <Title text="Log in — VoteAssist India Admin"/>
        <div class="login-page">
            <h1>"VoteAssist India — Admin login"</h1>
            <p class="login-notice">
                "This is the admin/reviewer dashboard, not the public site. If you're a citizen "
                "looking for voter guidance, "
                <a href="/">"the public site is here"</a>
                "."
            </p>

            {move || login_action.value().get().and_then(|r| r.err()).map(|err| view! {
                <p class="login-error" role="alert">{err.to_string()}</p>
            })}

            <form on:submit=move |ev| {
                ev.prevent_default();
                login_action.dispatch((email.get(), password.get()));
            }>
                <div class="form-field">
                    <label for="login-email">"Email"</label>
                    <input
                        id="login-email"
                        type="email"
                        required
                        prop:value=move || email.get()
                        on:input=move |ev| email.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-field">
                    <label for="login-password">"Password"</label>
                    <input
                        id="login-password"
                        type="password"
                        required
                        prop:value=move || password.get()
                        on:input=move |ev| password.set(event_target_value(&ev))
                    />
                </div>
                <button type="submit">"Log in"</button>
            </form>
        </div>
    }
}
