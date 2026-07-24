//! The decision-tree question/answer island — the one piece of this site
//! that genuinely needs client-side interactivity without a full page
//! reload between questions (docs/PRD-V2-RUST-PLATFORM.md Section 6.6).
//!
//! State lives entirely in this component's reactive signals; nothing is
//! persisted server-side (see `server_fns.rs`'s module doc). Each answer
//! sends the current `EngineState` back to `submit_answer`, which
//! re-evaluates via `core-domain` and returns the next state plus that
//! node's rendered display data for the current locale.
//!
//! Kept as a single component (rather than split into typed helper
//! functions taking `Action<I, O>`/signal parameters) to avoid spelling
//! out generic type paths this pass could not verify by compiling —
//! everything here relies only on type inference at each closure's call
//! site, the same pattern used throughout the rest of this component.

use core_domain::EngineState;
use leptos::prelude::*;

use crate::locale::use_locale;
use crate::render::RenderableNode;
use crate::server_fns::{get_kb_entry, start_walkthrough, submit_answer, WalkthroughView};
use crate::server_fns_accounts::{current_account, save_draft};

#[component]
pub fn QuestionFlow() -> impl IntoView {
    let locale = use_locale().0;
    let view_state: RwSignal<Option<WalkthroughView>> = RwSignal::new(None);
    let load_error: RwSignal<Option<String>> = RwSignal::new(None);

    let start_action = Action::new(move |_: &()| {
        let locale = locale.get_untracked();
        async move { start_walkthrough(locale).await }
    });

    let answer_action = Action::new(move |input: &(EngineState, String)| {
        let (state, value) = input.clone();
        let locale = locale.get_untracked();
        async move { submit_answer(state, value, locale).await }
    });

    // Kick off a session the first time this island mounts.
    Effect::new(move |_| {
        if view_state.get_untracked().is_none() {
            start_action.dispatch(());
        }
    });

    Effect::new(move |_| {
        if let Some(result) = start_action.value().get() {
            match result {
                Ok(v) => {
                    load_error.set(None);
                    view_state.set(Some(v));
                }
                Err(e) => load_error.set(Some(e.to_string())),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(result) = answer_action.value().get() {
            match result {
                Ok(v) => {
                    load_error.set(None);
                    view_state.set(Some(v));
                }
                Err(e) => load_error.set(Some(e.to_string())),
            }
        }
    });

    let restart = move |_| {
        view_state.set(None);
        start_action.dispatch(());
    };

    // Citation ids for the *current* terminal outcome (empty otherwise);
    // resolved into full citation cards via a `Resource` that re-fetches
    // whenever the set of ids changes (i.e. whenever a new terminal
    // outcome is reached).
    let citation_ids = move || match view_state.get() {
        Some(v) => match v.node {
            RenderableNode::Terminal(t) => t.citation_ids,
            RenderableNode::Question(_) => Vec::new(),
        },
        None => Vec::new(),
    };
    let citation_entries = Resource::new(citation_ids, |ids| async move {
        let mut resolved = Vec::new();
        for id in ids {
            if let Ok(Some(entry)) = get_kb_entry(id).await {
                resolved.push(entry);
            }
        }
        resolved
    });

    // "Save this checklist" — only meaningful once logged in (saved
    // drafts require an account, per migration 0007's design), so this
    // section renders a login prompt instead of a broken save button for
    // an anonymous visitor.
    let account = Resource::new(|| (), |_| async move { current_account().await });
    let draft_label = RwSignal::new(String::new());
    let draft_saved = RwSignal::new(false);
    let save_draft_action = Action::new(move |input: &(EngineState, crate::render::RenderableTerminal, String)| {
        let (state, terminal, label) = input.clone();
        async move {
            let label = (!label.trim().is_empty()).then_some(label);
            save_draft(label, state, Some(terminal)).await
        }
    });

    Effect::new(move |_| {
        if let Some(Ok(_)) = save_draft_action.value().get() {
            draft_saved.set(true);
        }
    });

    view! {
        <div class="question-flow">
            {move || {
                load_error.get().map(|msg| view! {
                    <p class="error-banner" role="alert">
                        "Something went wrong loading the next step: " {msg}
                        " Please try "
                        <button type="button" on:click=restart>"starting over"</button>
                        "."
                    </p>
                })
            }}

            {move || match view_state.get() {
                None => view! { <p aria-live="polite">"Loading your first question..."</p> }.into_any(),

                Some(v) if !v.is_complete => {
                    let RenderableNode::Question(question) = v.node else {
                        unreachable!("is_complete is false, so this node is a question");
                    };
                    let state = v.state;
                    let progress_pct = (v.progress * 100.0).round().clamp(0.0, 100.0) as i32;

                    view! {
                        <fieldset class="question-card">
                            <legend>{question.prompt.clone()}</legend>
                            {question.help_text.clone().map(|help| view! { <p class="help-text">{help}</p> })}
                            <div class="progress-indicator" aria-live="polite">
                                <div class="progress-bar" style:width=format!("{progress_pct}%")></div>
                                <span class="progress-label">
                                    {format!("About {progress_pct}% of a typical path")}
                                </span>
                            </div>
                            <div class="options" role="group">
                                {question.options.into_iter().map(|option| {
                                    let state = state.clone();
                                    let value = option.value.clone();
                                    view! {
                                        <button
                                            type="button"
                                            class="option-button"
                                            on:click=move |_| {
                                                answer_action.dispatch((state.clone(), value.clone()));
                                            }
                                        >
                                            {option.label}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </fieldset>
                    }.into_any()
                }

                Some(v) => {
                    let RenderableNode::Terminal(terminal) = v.node else {
                        unreachable!("is_complete is true, so this node is a terminal outcome");
                    };
                    let state = v.state;

                    view! {
                        <div class="terminal-outcome">
                            <h2>{terminal.outcome_title.clone()}</h2>
                            <p>{terminal.outcome_description.clone()}</p>

                            {(!terminal.recommended_forms.is_empty()).then(|| view! {
                                <p class="recommended-forms">
                                    <strong>"Recommended form(s): "</strong>
                                    {terminal.recommended_forms.join(", ")}
                                </p>
                            })}

                            {(!terminal.checklist.is_empty()).then(|| view! {
                                <div class="checklist">
                                    <h3>"Checklist"</h3>
                                    <ul>
                                        {terminal.checklist.iter().cloned().map(|item| view! {
                                            <li><label><input type="checkbox"/> {item}</label></li>
                                        }).collect_view()}
                                    </ul>
                                </div>
                            })}

                            {(!terminal.deep_links.is_empty()).then(|| view! {
                                <div class="official-links">
                                    <h3>"Official next steps"</h3>
                                    <ul>
                                        {terminal.deep_links.iter().cloned().map(|link| view! {
                                            <li>
                                                <a href=link.url.clone() class="deep-link-cta" rel="noopener noreferrer">
                                                    {link.label}
                                                </a>
                                            </li>
                                        }).collect_view()}
                                    </ul>
                                </div>
                            })}

                            <div class="citations">
                                <h3>"Sources"</h3>
                                <Suspense fallback=|| view! { <p>"Loading sources..."</p> }>
                                    {move || citation_entries.get().map(|entries| view! {
                                        <ul>
                                            {entries.into_iter().map(|detail| view! {
                                                <li class="citation-card">
                                                    <a href=format!("/learn/{}", detail.entry.id)>{detail.entry.title.clone()}</a>
                                                    <span class="citation-meta">
                                                        " — last verified " {detail.entry.last_verified_date.clone()}
                                                    </span>
                                                </li>
                                            }).collect_view()}
                                        </ul>
                                    })}
                                </Suspense>
                            </div>

                            {(!terminal.caution.is_empty()).then(|| view! {
                                <p class="caution-note">{terminal.caution.clone()}</p>
                            })}

                            <p class="not-official-reminder">
                                {crate::components::banner::NOT_OFFICIAL_BANNER_TEXT}
                            </p>

                            <p><a href="/feedback">"This didn't match my situation"</a></p>

                            <div class="save-checklist">
                                <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                                    {move || {
                                        let state = state.clone();
                                        let terminal = terminal.clone();
                                        account.get().map(move |result| match result {
                                            Ok(Some(_)) if draft_saved.get() => view! {
                                                <p class="draft-saved-notice">
                                                    "Saved. Find it under "
                                                    <a href="/account">"My account"</a>
                                                    "."
                                                </p>
                                            }.into_any(),
                                            Ok(Some(_)) => view! {
                                                <div class="save-checklist-form">
                                                    <label for="draft-label">"Save this checklist (optional name)"</label>
                                                    <input id="draft-label" type="text"
                                                        prop:value=move || draft_label.get()
                                                        on:input=move |ev| draft_label.set(event_target_value(&ev))/>
                                                    <button type="button" on:click={
                                                        let state = state.clone();
                                                        let terminal = terminal.clone();
                                                        move |_| {
                                                            save_draft_action.dispatch((state.clone(), terminal.clone(), draft_label.get()));
                                                        }
                                                    }>
                                                        "Save this checklist"
                                                    </button>
                                                </div>
                                            }.into_any(),
                                            _ => view! {
                                                <p>
                                                    <a href="/account/login">"Log in"</a>
                                                    " to save this checklist and come back to it later."
                                                </p>
                                            }.into_any(),
                                        })
                                    }}
                                </Suspense>
                            </div>

                            <button type="button" class="restart-button" on:click=restart>"Start over"</button>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
