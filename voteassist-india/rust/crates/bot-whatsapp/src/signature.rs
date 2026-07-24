//! Verifies the `X-Hub-Signature-256` header Meta attaches to every
//! webhook delivery, per Meta's documented webhook-security requirement:
//! an HMAC-SHA256 of the raw request body, keyed with the WhatsApp app
//! secret, encoded as `sha256=<hex>`. Rejecting unsigned/mis-signed
//! deliveries is what stops any third party who discovers this crate's
//! webhook URL from injecting fake inbound messages — a real security
//! requirement for a public HTTP endpoint, not optional hardening.

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SignatureError {
    #[error("missing X-Hub-Signature-256 header")]
    Missing,
    #[error("X-Hub-Signature-256 header is not in the expected \"sha256=<hex>\" format")]
    MalformedHeader,
    #[error("X-Hub-Signature-256 hex payload could not be decoded")]
    InvalidHex,
    #[error("signature does not match the computed HMAC for this payload")]
    Mismatch,
}

/// Verifies `header_value` (the raw `X-Hub-Signature-256` header, if
/// present) against an HMAC-SHA256 of `raw_body` keyed with `app_secret`.
/// Uses `Mac::verify_slice`, which compares in constant time — timing
/// side-channels are a real concern for signature checks, not a
/// theoretical one, since an attacker who can measure response latency
/// could otherwise recover the signature byte-by-byte.
pub fn verify_signature(
    header_value: Option<&str>,
    raw_body: &[u8],
    app_secret: &str,
) -> Result<(), SignatureError> {
    let header_value = header_value.ok_or(SignatureError::Missing)?;
    let hex_digest = header_value
        .strip_prefix("sha256=")
        .ok_or(SignatureError::MalformedHeader)?;

    let expected_bytes = hex::decode(hex_digest).map_err(|_| SignatureError::InvalidHex)?;

    let mut mac = HmacSha256::new_from_slice(app_secret.as_bytes())
        .expect("HMAC accepts a key of any length");
    mac.update(raw_body);

    mac.verify_slice(&expected_bytes).map_err(|_| SignatureError::Mismatch)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sign(body: &[u8], secret: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let digest = mac.finalize().into_bytes();
        format!("sha256={}", hex::encode(digest))
    }

    #[test]
    fn accepts_a_correctly_signed_body() {
        let body = br#"{"entry":[]}"#;
        let secret = "test-app-secret";
        let header = sign(body, secret);
        assert!(verify_signature(Some(&header), body, secret).is_ok());
    }

    #[test]
    fn rejects_a_body_signed_with_the_wrong_secret() {
        let body = br#"{"entry":[]}"#;
        let header = sign(body, "wrong-secret");
        assert_eq!(
            verify_signature(Some(&header), body, "test-app-secret"),
            Err(SignatureError::Mismatch)
        );
    }

    #[test]
    fn rejects_a_tampered_body() {
        let body = br#"{"entry":[]}"#;
        let secret = "test-app-secret";
        let header = sign(body, secret);
        let tampered = br#"{"entry":[{"injected":true}]}"#;
        assert_eq!(
            verify_signature(Some(&header), tampered, secret),
            Err(SignatureError::Mismatch)
        );
    }

    #[test]
    fn rejects_a_missing_header() {
        assert_eq!(verify_signature(None, b"{}", "secret"), Err(SignatureError::Missing));
    }

    #[test]
    fn rejects_a_malformed_header() {
        assert_eq!(
            verify_signature(Some("not-the-right-format"), b"{}", "secret"),
            Err(SignatureError::MalformedHeader)
        );
    }
}
