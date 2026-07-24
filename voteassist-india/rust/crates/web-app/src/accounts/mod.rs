//! The optional, strictly opt-in public account system
//! (`migrations/0007_accounts_and_drafts.sql`,
//! `migrations/0012_account_otp_and_sessions.sql`) — OTP login, saved
//! decision-tree drafts ("checklists"), and consent history. Entirely
//! server-only (argon2/sqlx/rand/lettre don't compile for, and have no
//! reason to exist in, the wasm/hydrate client build); gated at `lib.rs`'s
//! `pub mod accounts;` declaration, not here — see that module's doc for
//! why `server_fns_accounts.rs` must `use crate::accounts::...` *inside*
//! each `#[server]` function body rather than at file scope.
//!
//! The anonymous, no-account decision-engine flow (`server_fns.rs`)
//! remains structurally independent of this module — nothing in it
//! references `user_accounts`, matching migration 0007's own stated
//! design test.

pub mod email;
pub mod guard;
pub mod hashing;
pub mod otp;
pub mod session;

pub const ACCOUNT_SESSION_COOKIE_NAME: &str = "va_account_session";
