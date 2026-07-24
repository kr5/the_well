//! MCC / Election-Period Control Panel — PRD v2 Section 11, admin page 8.
//! Implements declaring/closing an MCC window per state — the single
//! source of truth `channel_core::check_broadcast_allowed` (used by
//! `bot-telegram`/`bot-whatsapp`'s broadcast gates) reads from. Not
//! implemented: the per-channel broadcast kill-switch UI (independent of
//! MCC status) — `bot_channel_config`'s schema exists
//! (`migrations/0011_bot_channel_config.sql`) but this page doesn't read
//! or write it yet; a real, disclosed follow-up.

use leptos::prelude::*;
use leptos_meta::Title;

use crate::server_fns::{close_mcc_window, list_mcc_windows, open_mcc_window};

#[component]
pub fn MccPanelPage() -> impl IntoView {
    let windows = Resource::new(|| (), |_| async move { list_mcc_windows().await });

    let state_name = RwSignal::new(String::new());
    let window_start = RwSignal::new(String::new());

    let open_action = Action::new(move |input: &(String, String)| {
        let (state_name, window_start) = input.clone();
        async move {
            let parsed = window_start
                .parse()
                .map_err(|_| ServerFnError::ServerError("invalid date".to_string()))?;
            open_mcc_window(state_name, parsed).await
        }
    });

    let close_action = Action::new(|id: &String| {
        let id = id.clone();
        async move { close_mcc_window(id).await }
    });

    Effect::new(move |_| {
        if open_action.value().get().is_some() || close_action.value().get().is_some() {
            windows.refetch();
        }
    });

    view! {
        <Title text="MCC control panel — VoteAssist India Admin"/>
        <h1>"MCC / Election-Period Control Panel"</h1>
        <p>
            "Declaring an active window here immediately suppresses proactive broadcasts for "
            "that state on Telegram and WhatsApp (R-MCC-1) — the bot adapters check this table "
            "before every proactive send, not a separately-maintained flag."
        </p>

        <form class="inline-form" on:submit=move |ev| {
            ev.prevent_default();
            open_action.dispatch((state_name.get(), window_start.get()));
        }>
            <div class="form-field">
                <label for="mcc-state">"State name (must match the states table exactly)"</label>
                <input id="mcc-state" type="text" required
                    prop:value=move || state_name.get()
                    on:input=move |ev| state_name.set(event_target_value(&ev))/>
            </div>
            <div class="form-field">
                <label for="mcc-window-start">"Window start (schedule-announced date)"</label>
                <input id="mcc-window-start" type="date" required
                    prop:value=move || window_start.get()
                    on:input=move |ev| window_start.set(event_target_value(&ev))/>
            </div>
            <button type="submit">"Open MCC window"</button>
        </form>

        {move || open_action.value().get().and_then(|r| r.err()).map(|err| view! {
            <p class="form-error" role="alert">{err.to_string()}</p>
        })}

        <Suspense fallback=|| view! { <p>"Loading..."</p> }>
            {move || windows.get().map(|result| match result {
                Ok(windows) => view! {
                    <table class="admin-table">
                        <thead><tr><th>"State"</th><th>"Start"</th><th>"End"</th><th>"Active"</th><th></th></tr></thead>
                        <tbody>
                            {windows.into_iter().map(|w| {
                                let id = w.id.clone();
                                view! {
                                    <tr>
                                        <td>{w.state_name.clone()}</td>
                                        <td>{w.window_start.to_string()}</td>
                                        <td>{w.window_end.map(|d| d.to_string()).unwrap_or_else(|| "—".to_string())}</td>
                                        <td>{if w.is_active { "Active" } else { "Closed" }}</td>
                                        <td>
                                            {w.is_active.then(|| view! {
                                                <button type="button" on:click=move |_| { close_action.dispatch(id.clone()); }>
                                                    "Close"
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
    }
}
