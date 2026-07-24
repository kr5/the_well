//! Admin/reviewer dashboard for VoteAssist India. A separate Leptos SSR
//! deploy target from `crates/web-app` for blast-radius containment
//! (docs/PRD-V2-RUST-PLATFORM.md Section 11): an XSS, auth bug, or outage
//! here cannot touch the anonymous public decision-engine experience, and
//! vice versa.

pub mod app;
pub mod components;
pub mod pages;
pub mod server_fns;

// Server-only: argon2/sqlx/rand don't compile for (and have no reason to
// exist in) the wasm/hydrate client build. See this module's own doc
// comment for how `server_fns.rs` still calls into it safely.
#[cfg(feature = "ssr")]
pub mod auth;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
