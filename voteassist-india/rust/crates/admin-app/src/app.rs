//! Root shell, root `App` component, and the admin route table. Real
//! authorization is enforced per server function (`server_fns.rs`'s
//! `require_admin`/`require_role`) — the page-level gating here is a UX
//! nicety (showing a "please log in" prompt instead of a blank/broken
//! page), never the actual security boundary.

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::components::shell::AdminShell;
use crate::pages::analytics::AnalyticsDashboardPage;
use crate::pages::audit_log::AuditLogPage;
use crate::pages::bot_channels::BotChannelsPage;
use crate::pages::dashboard::DashboardPage;
use crate::pages::data_retention::DataRetentionPage;
use crate::pages::feedback::FeedbackTriagePage;
use crate::pages::kb_editor::{KbEditorNewPage, KbEditorPage, KbListPage};
use crate::pages::link_health::LinkHealthPage;
use crate::pages::login::LoginPage;
use crate::pages::mcc_panel::MccPanelPage;
use crate::pages::not_found::NotFoundPage;
use crate::pages::system_health::SystemHealthPage;
use crate::pages::translation::TranslationManagementPage;
use crate::pages::tree_editor::{TreeEditorListPage, TreeEditorPage};
use crate::pages::user_management::UserManagementPage;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="robots" content="noindex, nofollow"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
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

    view! {
        <Stylesheet id="leptos" href="/pkg/voteassist-admin-app.css"/>
        <Title text="VoteAssist India — Admin"/>

        <Router>
            <Routes fallback=NotFoundPage>
                <Route path=path!("/login") view=LoginPage/>
                <Route path=path!("/") view=|| view! { <AdminShell><DashboardPage/></AdminShell> }/>
                <Route path=path!("/kb") view=|| view! { <AdminShell><KbListPage/></AdminShell> }/>
                <Route path=path!("/kb/new") view=|| view! { <AdminShell><KbEditorNewPage/></AdminShell> }/>
                <Route path=path!("/kb/:id") view=|| view! { <AdminShell><KbEditorPage/></AdminShell> }/>
                <Route path=path!("/mcc") view=|| view! { <AdminShell><MccPanelPage/></AdminShell> }/>
                <Route path=path!("/audit-log") view=|| view! { <AdminShell><AuditLogPage/></AdminShell> }/>
                <Route path=path!("/analytics") view=|| view! { <AdminShell><AnalyticsDashboardPage/></AdminShell> }/>
                <Route path=path!("/feedback") view=|| view! { <AdminShell><FeedbackTriagePage/></AdminShell> }/>
                <Route path=path!("/link-health") view=|| view! { <AdminShell><LinkHealthPage/></AdminShell> }/>
                <Route path=path!("/bot-channels") view=|| view! { <AdminShell><BotChannelsPage/></AdminShell> }/>
                <Route path=path!("/translations") view=|| view! { <AdminShell><TranslationManagementPage/></AdminShell> }/>
                <Route path=path!("/users") view=|| view! { <AdminShell><UserManagementPage/></AdminShell> }/>
                <Route path=path!("/data-retention") view=|| view! { <AdminShell><DataRetentionPage/></AdminShell> }/>
                <Route path=path!("/tree-editor") view=|| view! { <AdminShell><TreeEditorListPage/></AdminShell> }/>
                <Route path=path!("/tree-editor/:tree_key") view=|| view! { <AdminShell><TreeEditorPage/></AdminShell> }/>
                <Route path=path!("/system-health") view=|| view! { <AdminShell><SystemHealthPage/></AdminShell> }/>
            </Routes>
        </Router>
    }
}
