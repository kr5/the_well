use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::language_switcher::LanguageSwitcher;

#[component]
pub fn SiteHeader() -> impl IntoView {
    view! {
        <header class="site-header">
            <div class="site-header-brand">
                <A href="/">
                    <span class="brand-name">"VoteAssist India"</span>
                </A>
            </div>
            <nav aria-label="Primary" class="site-nav">
                <A href="/start">"Start"</A>
                <A href="/search">"Find my status"</A>
                <A href="/learn">"Learn"</A>
                <A href="/locate">"Locate"</A>
                <A href="/about/what-this-is">"About"</A>
            </nav>
            <div class="site-header-tools">
                <LanguageSwitcher/>
                <A href="/accessibility">"Accessibility"</A>
            </div>
        </header>
    }
}
