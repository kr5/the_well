//! OTP generation, hashing, and verification for the optional public
//! account system (`account_otp_challenges`,
//! `migrations/0012_account_otp_and_sessions.sql`).

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use rand::Rng;

pub const OTP_LENGTH: usize = 6;
/// Per this migration's comment: "a 10-minute expiry, enforced at the
/// application layer."
pub const OTP_LIFETIME_MINUTES: i64 = 10;
/// Independent of expiry — caps brute-force guesses at a fixed code.
pub const MAX_OTP_ATTEMPTS: i32 = 5;

pub fn generate_otp() -> String {
    let mut rng = rand::thread_rng();
    (0..OTP_LENGTH).map(|_| rng.gen_range(0..10).to_string()).collect()
}

pub fn hash_otp(otp: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default().hash_password(otp.as_bytes(), &salt).map(|hash| hash.to_string())
}

pub fn verify_otp(stored_hash: &str, attempt: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(stored_hash) else {
        return false;
    };
    Argon2::default().verify_password(attempt.as_bytes(), &parsed).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_otp_produces_the_configured_length_of_digits() {
        let otp = generate_otp();
        assert_eq!(otp.len(), OTP_LENGTH);
        assert!(otp.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn hash_and_verify_round_trip() {
        let otp = generate_otp();
        let hash = hash_otp(&otp).unwrap();
        assert!(verify_otp(&hash, &otp));
        assert!(!verify_otp(&hash, "000000"));
    }
}
