//! Root shell, root `App` component, and the full route table — the
//! "sitemap for the VoteAssist India web app" from
//! docs/05-information-architecture.md, ported one-to-one.

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::components::banner::NotOfficialBanner;
use crate::components::footer::SiteFooter;
use crate::components::header::SiteHeader;
use crate::locale::provide_locale_context;
use crate::pages::about::{AboutLegal, AboutOpenSource, AboutPrivacy, AboutWhatThisIs};
use crate::pages::accessibility::AccessibilityPage;
use crate::pages::feedback::FeedbackPage;
use crate::pages::home::HomePage;
use crate::pages::learn::{LearnEntryPage, LearnFaqPage, LearnFormsPage, LearnGlossaryPage, LearnIndexPage};
use crate::pages::locate::LocatePage;
use crate::pages::not_found::NotFoundPage;
use crate::pages::search::SearchPage;
use crate::pages::start::StartPage;

/// The outermost HTML document. Rendered once per request on the server;
/// `<HydrationScripts>` is what loads and boots the wasm bundle for the
/// two interactive islands (question/answer widget, language switcher) —
/// everything else on the page works without it.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="robots" content="index, follow"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
                // Applies any previously saved accessibility preference
                // (see pages::accessibility, public/accessibility-prefs.js)
                // before first paint, on every page — independent of wasm
                // hydration. A plain `src` script, not inlined through the
                // view! macro, deliberately: unlike the framework's own
                // `<HydrationScripts>`/`<AutoReload>`, this is not a case
                // this pass could verify the templating macro round-trips
                // raw `<script>` text content through unescaped.
                <script src="/accessibility-prefs.js"></script>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_locale_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/voteassist-web-app.css"/>
        <Title text="VoteAssist India — voter guidance, not a government portal"/>

        <Router>
            // Non-negotiable per docs/06-legal-compliance-review.md
            // Section 1 and docs/16-security-threat-model.md's
            // phishing-lookalike risk entry: rendered outside <Routes/>
            // so it appears on every page, including the 404 fallback,
            // and is part of the initial server-rendered HTML (readable
            // by screen readers and visible with JS disabled).
            <a href="#main-content" class="skip-link">"Skip to main content"</a>
            <NotOfficialBanner/>
            <SiteHeader/>
            <main id="main-content">
                <Routes fallback=NotFoundPage>
                    <Route path=path!("/") view=HomePage/>
                    <Route path=path!("/start") view=StartPage/>
                    <Route path=path!("/search") view=SearchPage/>
                    <Route path=path!("/learn") view=LearnIndexPage/>
                    <Route path=path!("/learn/forms") view=LearnFormsPage/>
                    <Route path=path!("/learn/faq") view=LearnFaqPage/>
                    <Route path=path!("/learn/glossary") view=LearnGlossaryPage/>
                    <Route path=path!("/learn/:slug") view=LearnEntryPage/>
                    <Route path=path!("/locate") view=LocatePage/>
                    <Route path=path!("/about/what-this-is") view=AboutWhatThisIs/>
                    <Route path=path!("/about/legal") view=AboutLegal/>
                    <Route path=path!("/about/privacy") view=AboutPrivacy/>
                    <Route path=path!("/about/open-source") view=AboutOpenSource/>
                    <Route path=path!("/feedback") view=FeedbackPage/>
                    <Route path=path!("/accessibility") view=AccessibilityPage/>
                </Routes>
            </main>
            <SiteFooter/>
        </Router>
    }
}
