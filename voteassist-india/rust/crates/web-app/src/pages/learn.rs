//! Knowledge base browser (`/learn/...`). Per
//! docs/05-information-architecture.md Section 5.
//!
//! `LearnFaqPage` filters on `KnowledgeEntry::is_faq`
//! (`knowledge-base/schema/entry.schema.json`) now that the field exists
//! — but tagging *which* real entries are FAQs is a content-curation
//! decision for a reviewer to make (via `admin-app`'s KB editor), not
//! something this migration retroactively guesses at. Until at least one
//! entry is tagged, this page falls back to the full curated list framed
//! as commonly-asked questions, exactly as before — every entry shown
//! either way is real, cited content; only the "FAQ" categorization
//! narrows once real tags exist.

use kb_content::Topic;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::server_fns::{get_kb_entry, list_entries_by_topic, list_kb_entries, record_kb_entry_viewed};

#[component]
pub fn LearnIndexPage() -> impl IntoView {
    let entries = Resource::new(|| (), |_| async move { list_kb_entries().await });

    view! {
        <Title text="Learn — VoteAssist India"/>
        <section class="learn-index">
            <h1>"Learn"</h1>
            <p>
                "Plain-language explainers for every form and process VoteAssist India covers, "
                "each with a citation back to the official source it was drawn from."
            </p>

            <nav class="learn-sections" aria-label="Knowledge base sections">
                <A href="/learn/forms">"Forms"</A>
                <A href="/learn/faq">"FAQ"</A>
                <A href="/learn/glossary">"Glossary"</A>
            </nav>

            <h2>"All entries"</h2>
            <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                {move || entries.get().map(|result| entry_list_or_error(result))}
            </Suspense>
        </section>
    }
}

#[component]
pub fn LearnFormsPage() -> impl IntoView {
    let entries = Resource::new(|| (), |_| async move { list_kb_entries().await });

    view! {
        <Title text="Forms — Learn — VoteAssist India"/>
        <section class="learn-forms">
            <h1>"Forms"</h1>
            <p>"Which ECI form applies to which situation, explained in plain language."</p>
            <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                {move || entries.get().map(|result| match result {
                    Ok(entries) => {
                        let mut with_forms: Vec<_> = entries
                            .into_iter()
                            .filter(|e| !e.related_forms.is_empty())
                            .collect();
                        with_forms.sort_by(|a, b| a.title.cmp(&b.title));
                        entry_list_view(with_forms)
                    }
                    Err(_) => view! { <p role="alert">"Could not load the knowledge base."</p> }.into_any(),
                })}
            </Suspense>
        </section>
    }
}

#[component]
pub fn LearnFaqPage() -> impl IntoView {
    let entries = Resource::new(|| (), |_| async move { list_kb_entries().await });

    view! {
        <Title text="FAQ — Learn — VoteAssist India"/>
        <section class="learn-faq">
            <h1>"Frequently asked questions"</h1>
            <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                {move || entries.get().map(|result| entry_list_or_error(result.map(only_faq_tagged_or_all)))}
            </Suspense>
        </section>
    }
}

/// Narrows to `is_faq`-tagged entries once at least one exists; falls
/// back to the full list otherwise (see this module's doc comment).
fn only_faq_tagged_or_all(entries: Vec<kb_content::KnowledgeEntry>) -> Vec<kb_content::KnowledgeEntry> {
    let tagged: Vec<_> = entries.iter().filter(|e| e.is_faq).cloned().collect();
    if tagged.is_empty() {
        entries
    } else {
        tagged
    }
}

#[component]
pub fn LearnGlossaryPage() -> impl IntoView {
    let entries = Resource::new(|| (), |_| async move { list_entries_by_topic(Topic::Glossary).await });

    view! {
        <Title text="Glossary — Learn — VoteAssist India"/>
        <section class="learn-glossary">
            <h1>"Glossary"</h1>
            <p>"AC, PC, EPIC, BLO, ERO, DEO, CEO, SVEEP, and other terms you'll see across this site."</p>
            <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                {move || entries.get().map(|result| entry_list_or_error(result))}
            </Suspense>
        </section>
    }
}

#[component]
pub fn LearnEntryPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("slug").unwrap_or_default();

    let entry = Resource::new(slug, |slug| async move {
        let result = get_kb_entry(slug.clone()).await;
        if let Ok(Some(_)) = &result {
            // Best-effort — a citizen reading this page must never be
            // blocked or broken by an analytics-recording failure.
            let _ = record_kb_entry_viewed(slug).await;
        }
        result
    });

    view! {
        <section class="learn-entry">
            <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                {move || entry.get().map(|result| match result {
                    Ok(Some(detail)) => {
                        let entry = detail.entry;
                        view! {
                        <article>
                            <Title text=format!("{} — Learn — VoteAssist India", entry.title.clone())/>
                            <h1>{entry.title.clone()}</h1>
                            <p class="entry-summary">{entry.summary.clone()}</p>
                            <div class="entry-body" inner_html=detail.body_html.clone()></div>

                            <dl class="entry-metadata">
                                <dt>"Source type"</dt>
                                <dd>{format!("{:?}", entry.source_type)}</dd>
                                <dt>"Last verified"</dt>
                                <dd>{entry.last_verified_date.clone()}</dd>
                            </dl>

                            {(!entry.sources.is_empty()).then(|| view! {
                                <div class="entry-sources">
                                    <h2>"Sources"</h2>
                                    <ul>
                                        {entry.sources.iter().cloned().map(|source| view! {
                                            <li>
                                                <a href=source.url.clone() rel="noopener noreferrer">{source.title.clone()}</a>
                                            </li>
                                        }).collect_view()}
                                    </ul>
                                </div>
                            })}

                            {entry.caution.clone().map(|caution| view! {
                                <p class="caution-note">{caution}</p>
                            })}
                        </article>
                    }.into_any()},
                    Ok(None) => view! {
                        <p role="alert">"We couldn't find that knowledge-base entry."</p>
                        <A href="/learn">"Back to Learn"</A>
                    }.into_any(),
                    Err(_) => view! { <p role="alert">"Could not load this entry."</p> }.into_any(),
                })}
            </Suspense>
        </section>
    }
}

fn entry_list_or_error(result: Result<Vec<kb_content::KnowledgeEntry>, ServerFnError>) -> impl IntoView {
    match result {
        Ok(entries) => entry_list_view(entries).into_any(),
        Err(_) => view! { <p role="alert">"Could not load the knowledge base."</p> }.into_any(),
    }
}

fn entry_list_view(entries: Vec<kb_content::KnowledgeEntry>) -> impl IntoView {
    if entries.is_empty() {
        return view! { <p>"No entries yet."</p> }.into_any();
    }

    view! {
        <ul class="entry-list">
            {entries.into_iter().map(|entry| view! {
                <li>
                    <A href=format!("/learn/{}", entry.id)>{entry.title.clone()}</A>
                    <p class="entry-list-summary">{entry.summary.clone()}</p>
                    <span class="entry-list-verified">"Last verified " {entry.last_verified_date.clone()}</span>
                </li>
            }).collect_view()}
        </ul>
    }
    .into_any()
}
