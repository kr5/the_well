//! Authentication and authorization for the admin app. This whole module
//! is server-only (argon2/sqlx/rand don't need to — and largely can't —
//! compile for the wasm/hydrate target), gated at `lib.rs`'s `pub mod
//! auth;` declaration, not here — see that module doc for why callers in
//! `server_fns.rs` must `use crate::auth::...` *inside* each `#[server]`
//! function body rather than at file scope.
//!
//! Real, server-side enforcement lives here and only here: per
//! docs/PRD-V2-RUST-PLATFORM.md Section 16, "every one of the 'cannot'
//! cells is enforced server-side... not left as a UI-only restriction."
//! Every admin server function that touches data calls `require_role`
//! first; pages that merely *hide* a button for a role the citizen (er,
//! admin) doesn't have are a UX nicety on top, never the actual gate.

pub mod guard;
pub mod password;
pub mod session;

pub use guard::{require_admin, require_role};
pub use password::{hash_password, verify_password, PasswordError};
pub use session::{create_session, destroy_session, validate_session, AdminUser};

pub const SESSION_COOKIE_NAME: &str = "va_admin_session";

/// The six RBAC roles from docs/PRD-V2-RUST-PLATFORM.md Section 11.
/// `superadmin` always passes every `require_role` check, matching that
/// section's "superadmin can do everything" rule — checked explicitly
/// below rather than requiring every call site to remember to list it.
pub const ROLE_CONTRIBUTOR: &str = "contributor";
pub const ROLE_REVIEWER: &str = "reviewer";
pub const ROLE_LEGAL_REVIEWER: &str = "legal_reviewer";
pub const ROLE_TRANSLATOR: &str = "translator";
pub const ROLE_ANALYTICS_VIEWER: &str = "analytics_viewer";
pub const ROLE_SUPERADMIN: &str = "superadmin";

pub fn role_satisfies(user_role: &str, allowed: &[&str]) -> bool {
    user_role == ROLE_SUPERADMIN || allowed.contains(&user_role)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn superadmin_satisfies_any_role_requirement() {
        assert!(role_satisfies(ROLE_SUPERADMIN, &[ROLE_LEGAL_REVIEWER]));
    }

    #[test]
    fn a_role_not_in_the_allowed_list_is_rejected() {
        assert!(!role_satisfies(ROLE_TRANSLATOR, &[ROLE_LEGAL_REVIEWER, ROLE_REVIEWER]));
    }

    #[test]
    fn a_role_in_the_allowed_list_is_accepted() {
        assert!(role_satisfies(ROLE_REVIEWER, &[ROLE_LEGAL_REVIEWER, ROLE_REVIEWER]));
    }
}
