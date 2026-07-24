//! Argon2 password hashing for `admin_users.password_hash`, per
//! docs/PRD-V2-RUST-PLATFORM.md Section 6.10. Only ever called from
//! server-only code paths (account creation, login) — never exposed to
//! the client.

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("failed to hash password: {0}")]
    Hash(String),
    #[error("failed to parse stored password hash: {0}")]
    Parse(String),
}

pub fn hash_password(plaintext: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(plaintext.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| PasswordError::Hash(e.to_string()))
}

pub fn verify_password(stored_hash: &str, attempt: &str) -> Result<bool, PasswordError> {
    let parsed = PasswordHash::new(stored_hash).map_err(|e| PasswordError::Parse(e.to_string()))?;
    Ok(Argon2::default().verify_password(attempt.as_bytes(), &parsed).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hashed_password_verifies_against_the_original_and_rejects_others() {
        let hash = hash_password("correct horse battery staple").unwrap();
        assert!(verify_password(&hash, "correct horse battery staple").unwrap());
        assert!(!verify_password(&hash, "wrong password").unwrap());
    }

    #[test]
    fn two_hashes_of_the_same_password_differ() {
        // Argon2 uses a fresh random salt each time, per hash_password's
        // use of SaltString::generate — this guards against ever
        // regressing to a fixed/no-salt scheme.
        let a = hash_password("same password").unwrap();
        let b = hash_password("same password").unwrap();
        assert_ne!(a, b);
    }
}
