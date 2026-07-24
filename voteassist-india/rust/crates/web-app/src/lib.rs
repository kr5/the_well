//! Public citizen-facing site for VoteAssist India. Leptos SSR by
//! default (plain semantic HTML, works with JS disabled, fast on 2G, per
//! docs/PRD-V2-RUST-PLATFORM.md Section 6.6); only the decision-tree
//! question/answer widget and the language switcher hydrate as
//! client-side islands.

pub mod app;
pub mod components;
pub mod locale;
pub mod pages;
pub mod render;
// NOT gated behind `ssr`: the `#[server]` macro itself splits each
// function into a real server-side body (under `ssr`) vs. a client-side
// stub that performs a `fetch` (under `hydrate`/wasm). Gating the whole
// module to `ssr` would leave the client build with no way to call these
// functions at all.
pub mod server_fns;
pub mod server_fns_accounts;

// Server-only: argon2/sqlx/rand/lettre don't compile for (and have no
// reason to exist in) the wasm/hydrate client build. See this module's
// own doc comment for how server_fns_accounts.rs still calls into it
// safely.
#[cfg(feature = "ssr")]
pub mod accounts;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
