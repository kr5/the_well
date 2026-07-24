//! Deterministic, keyed hashing for `user_accounts.contact_identifier_hash`
//! and `account_otp_challenges.contact_identifier_hash` — a "blind index"
//! over an email/phone number.
//!
//! This deliberately does NOT use `argon2` (used elsewhere in this crate
//! for the OTP code itself, and in `admin-app` for passwords): argon2
//! salts every call randomly by design, so hashing the same email twice
//! produces two different outputs — correct for a password you only ever
//! verify against one stored hash, wrong for a column with a `UNIQUE`
//! constraint that must be looked up by equality
//! (`WHERE contact_identifier_hash = $1`). HMAC-SHA256 keyed with a
//! server-only secret is deterministic (same input, same output) while
//! still being infeasible to reverse or rainbow-table without the key —
//! the standard technique for an indexed, lookup-by-hash PII column.

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, thiserror::Error)]
pub enum HashingError {
    #[error("ACCOUNT_CONTACT_HASH_PEPPER must be set to a non-empty secret")]
    MissingPepper,
}

/// Normalizes an email (lowercase, trimmed) or phone number (digits only,
/// per E.164-ish convention) before hashing, so `User@Example.com` and
/// `user@example.com` hash identically — otherwise the same person could
/// accidentally create two accounts differing only in input casing.
pub fn normalize_contact(contact: &str, channel: &str) -> String {
    if channel == "email" {
        contact.trim().to_lowercase()
    } else {
        contact.chars().filter(|c| c.is_ascii_digit() || *c == '+').collect()
    }
}

/// Pure, testable core: given an explicit pepper, hash `normalized_contact`.
/// Split from `hash_contact_identifier` (which reads the pepper from the
/// environment) specifically so tests never mutate process-global env
/// state — `std::env::set_var` in parallel-running tests is a real
/// flakiness source, not a theoretical one.
fn hash_with_pepper(pepper: &str, normalized_contact: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(pepper.as_bytes()).expect("HMAC accepts a key of any length");
    mac.update(normalized_contact.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub fn hash_contact_identifier(normalized_contact: &str) -> Result<String, HashingError> {
    let pepper = std::env::var("ACCOUNT_CONTACT_HASH_PEPPER").map_err(|_| HashingError::MissingPepper)?;
    if pepper.trim().is_empty() {
        return Err(HashingError::MissingPepper);
    }
    Ok(hash_with_pepper(&pepper, normalized_contact))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_contact_lowercases_email() {
        assert_eq!(normalize_contact("User@Example.com", "email"), "user@example.com");
    }

    #[test]
    fn normalize_contact_strips_non_digits_from_phone() {
        assert_eq!(normalize_contact("+91 90000 00000", "phone"), "+919000000000");
    }

    #[test]
    fn hash_with_pepper_is_deterministic_given_the_same_pepper() {
        let a = hash_with_pepper("test-pepper", "user@example.com");
        let b = hash_with_pepper("test-pepper", "user@example.com");
        assert_eq!(a, b);
    }

    #[test]
    fn hash_with_pepper_differs_for_different_contacts() {
        let a = hash_with_pepper("test-pepper", "user@example.com");
        let b = hash_with_pepper("test-pepper", "other@example.com");
        assert_ne!(a, b);
    }

    #[test]
    fn hash_with_pepper_differs_for_different_peppers() {
        let a = hash_with_pepper("pepper-one", "user@example.com");
        let b = hash_with_pepper("pepper-two", "user@example.com");
        assert_ne!(a, b);
    }
}
