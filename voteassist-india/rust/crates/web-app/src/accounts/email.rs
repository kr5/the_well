//! SMTP-based OTP email delivery via `lettre`. Chosen over a
//! provider-specific HTTP API (SendGrid/SES/Resend) because plain SMTP
//! needs no additional vendor lock-in and works against literally any
//! mail provider (including a self-hosted relay) — a reasonable default
//! for a nonprofit civic-tech project without a settled email-vendor
//! contract (docs/PRD-V2-RUST-PLATFORM.md Section 5's "single low-cost
//! binary deployment for a nonprofit budget" reasoning applies here too).
//!
//! Phone/SMS OTP delivery has no chosen vendor integration in this pass
//! (Exotel, already used for `ivr-gateway`, plausibly also offers SMS,
//! but that API surface was not independently verified here — see
//! `ivr-gateway::webhook`'s module doc for this project's established
//! practice of disclosing exactly this kind of gap rather than guessing
//! at an unverified third-party schema). `request_otp` in
//! `server_fns_accounts.rs` accepts `channel = "phone"` at the type level
//! but returns a clear "not yet available" error for it today.

use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("email delivery is not configured on this deployment (SMTP_HOST/SMTP_USERNAME/SMTP_PASSWORD/SMTP_FROM_ADDRESS)")]
    NotConfigured,
    #[error("invalid email address: {0}")]
    InvalidAddress(String),
    #[error("failed to send email: {0}")]
    SendFailed(String),
}

pub struct EmailConfig {
    pub host: String,
    /// Defaults to 465 (implicit TLS) — `lettre::AsyncSmtpTransport::relay`'s
    /// own default — overridable via `SMTP_PORT` for providers expecting
    /// STARTTLS on 587 or another port.
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
}

impl EmailConfig {
    pub fn from_env() -> Result<Self, EmailError> {
        let host = std::env::var("SMTP_HOST").map_err(|_| EmailError::NotConfigured)?;
        let username = std::env::var("SMTP_USERNAME").map_err(|_| EmailError::NotConfigured)?;
        let password = std::env::var("SMTP_PASSWORD").map_err(|_| EmailError::NotConfigured)?;
        let from_address = std::env::var("SMTP_FROM_ADDRESS").map_err(|_| EmailError::NotConfigured)?;
        let port = std::env::var("SMTP_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(465);
        Ok(Self { host, port, username, password, from_address })
    }
}

pub async fn send_otp_email(to: &str, otp: &str) -> Result<(), EmailError> {
    let config = EmailConfig::from_env()?;

    let from: Mailbox = config.from_address.parse().map_err(|_| EmailError::InvalidAddress(config.from_address.clone()))?;
    let to_mailbox: Mailbox = to.parse().map_err(|_| EmailError::InvalidAddress(to.to_string()))?;

    let body = format!(
        "Your VoteAssist India verification code is: {otp}\n\n\
         This code expires in {minutes} minutes. If you didn't request this, you can ignore this email — \
         no account will be created without the code.\n\n\
         VoteAssist India is an independent, non-official guidance tool. It is not the Election \
         Commission of India.",
        minutes = super::otp::OTP_LIFETIME_MINUTES
    );

    let email = Message::builder()
        .from(from)
        .to(to_mailbox)
        .subject("Your VoteAssist India verification code")
        .body(body)
        .map_err(|e| EmailError::SendFailed(e.to_string()))?;

    let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
        .map_err(|e| EmailError::SendFailed(e.to_string()))?
        .port(config.port)
        .credentials(Credentials::new(config.username.clone(), config.password.clone()))
        .build();

    transport.send(email).await.map_err(|e| EmailError::SendFailed(e.to_string()))?;

    Ok(())
}
