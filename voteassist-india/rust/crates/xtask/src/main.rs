//! `xtask`: small operational CLI for tasks that don't belong inside any
//! single running service — most importantly, creating the very first
//! `admin-app` account. `admin-app` deliberately has no self-serve signup
//! (see its README's "User & Role Management" gap): the very first
//! superadmin has to come from somewhere, and a CLI run once against
//! production Postgres, rather than a web form, is the standard
//! deliberately-out-of-band way to bootstrap that.
//!
//! Usage:
//! ```text
//! xtask hash-password <password>
//! xtask create-admin <email> <password> <role>
//! xtask purge-expired-sessions
//! xtask translate-kb-entry <path/to/entry.json> <locale> <output-path>
//! xtask translate-tree <path/to/tree.json> <locale> <output-path>
//! ```
//!
//! `<role>` must be one of the six values `admin_role` accepts:
//! contributor, reviewer, legal_reviewer, translator, analytics_viewer,
//! superadmin (see `migrations/0001_extensions_and_enums.sql`).
//!
//! The two `translate-*` commands need `ANTHROPIC_API_KEY` set — see
//! `translate/client.rs`.

mod translate;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::Argon2;
use sqlx::postgres::PgPoolOptions;

const VALID_ROLES: &[&str] =
    &["contributor", "reviewer", "legal_reviewer", "translator", "analytics_viewer", "superadmin"];

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let result = match args.get(1).map(String::as_str) {
        Some("hash-password") => hash_password_command(&args),
        Some("create-admin") => create_admin_command(&args).await,
        Some("purge-expired-sessions") => purge_expired_sessions_command().await,
        Some("translate-kb-entry") => translate_kb_entry_command(&args).await,
        Some("translate-tree") => translate_tree_command(&args).await,
        _ => {
            print_usage();
            std::process::exit(2);
        }
    };

    if let Err(message) = result {
        eprintln!("error: {message}");
        std::process::exit(1);
    }
}

fn print_usage() {
    eprintln!(
        "Usage:\n  \
         xtask hash-password <password>\n  \
         xtask create-admin <email> <password> <role>\n  \
         xtask purge-expired-sessions\n  \
         xtask translate-kb-entry <path/to/entry.json> <locale> <output-path>\n  \
         xtask translate-tree <path/to/tree.json> <locale> <output-path>\n\n\
         <role> is one of: {}\n\
         <locale> is one of the codes in crates/xtask/src/translate/locales.rs",
        VALID_ROLES.join(", ")
    );
}

async fn translate_kb_entry_command(args: &[String]) -> Result<(), String> {
    let usage = "usage: xtask translate-kb-entry <path/to/entry.json> <locale> <output-path>";
    let source_path = args.get(2).ok_or(usage)?;
    let locale = args.get(3).ok_or(usage)?;
    let output_path = args.get(4).ok_or(usage)?;

    let client = translate::client::ClaudeClient::from_env().map_err(|e| e.to_string())?;
    let translated = translate::kb_entry::translate_kb_entry(&client, std::path::Path::new(source_path), locale)
        .await
        .map_err(|e| e.to_string())?;

    write_pretty_json(output_path, &translated)?;
    println!("Wrote draft translation to {output_path}. This is a DRAFT — review before moving it into knowledge-base/sources/ and publishing.");
    Ok(())
}

async fn translate_tree_command(args: &[String]) -> Result<(), String> {
    let usage = "usage: xtask translate-tree <path/to/tree.json> <locale> <output-path>";
    let source_path = args.get(2).ok_or(usage)?;
    let locale = args.get(3).ok_or(usage)?;
    let output_path = args.get(4).ok_or(usage)?;

    let client = translate::client::ClaudeClient::from_env().map_err(|e| e.to_string())?;
    let translated = translate::tree::translate_tree(&client, std::path::Path::new(source_path), locale)
        .await
        .map_err(|e| e.to_string())?;

    write_pretty_json(output_path, &translated)?;
    println!("Wrote draft translation to {output_path}. This is a DRAFT — diff it against the source tree and manually merge reviewed strings before they ship.");
    Ok(())
}

fn write_pretty_json(path: &str, value: &serde_json::Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(value).map_err(|e| format!("failed to serialize output: {e}"))?;
    std::fs::write(path, text).map_err(|e| format!("failed to write {path}: {e}"))
}

fn hash_password_command(args: &[String]) -> Result<(), String> {
    let password = args.get(2).ok_or("usage: xtask hash-password <password>")?;
    let hash = hash_password(password)?;
    println!("{hash}");
    Ok(())
}

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| format!("failed to hash password: {e}"))
}

async fn create_admin_command(args: &[String]) -> Result<(), String> {
    let email = args.get(2).ok_or("usage: xtask create-admin <email> <password> <role>")?;
    let password = args.get(3).ok_or("usage: xtask create-admin <email> <password> <role>")?;
    let role = args.get(4).ok_or("usage: xtask create-admin <email> <password> <role>")?;

    if !VALID_ROLES.contains(&role.as_str()) {
        return Err(format!("\"{role}\" is not a valid role — must be one of: {}", VALID_ROLES.join(", ")));
    }

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set, e.g. postgres://user:pass@host:5432/voteassist".to_string())?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .map_err(|e| format!("failed to connect to Postgres: {e}"))?;

    let password_hash = hash_password(password)?;

    let id: String = sqlx::query_scalar(
        r#"
        INSERT INTO admin_users (email, password_hash, role)
        VALUES ($1, $2, $3::admin_role)
        RETURNING id::text
        "#,
    )
    .bind(email)
    .bind(&password_hash)
    .bind(role.as_str())
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("failed to create admin user (does this email already exist?): {e}"))?;

    println!("Created {role} account {email} (id {id}). Log in at /login on the admin-app deployment.");
    Ok(())
}

/// `migrations/0012_account_otp_and_sessions.sql`'s
/// `account_otp_challenges`/`account_sessions` and `migrations/0006`'s
/// admin `sessions` table all accumulate expired rows over time — nothing
/// in `crates/jobs` sweeps them yet (a disclosed gap, since that crate's
/// four jobs are specifically the ones named in
/// docs/PRD-V2-RUST-PLATFORM.md Section 6.8, not a general-purpose
/// cleanup worker). Safe to run repeatedly (e.g. from a cron entry) —
/// every statement is a plain `DELETE ... WHERE expires_at < now()`.
async fn purge_expired_sessions_command() -> Result<(), String> {
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set, e.g. postgres://user:pass@host:5432/voteassist".to_string())?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .map_err(|e| format!("failed to connect to Postgres: {e}"))?;

    let admin_sessions = sqlx::query("DELETE FROM sessions WHERE expiry_date < now()")
        .execute(&pool)
        .await
        .map_err(|e| format!("failed to purge admin sessions: {e}"))?
        .rows_affected();

    let account_sessions = sqlx::query("DELETE FROM account_sessions WHERE expiry_date < now()")
        .execute(&pool)
        .await
        .map_err(|e| format!("failed to purge account sessions: {e}"))?
        .rows_affected();

    let otp_challenges = sqlx::query("DELETE FROM account_otp_challenges WHERE expires_at < now()")
        .execute(&pool)
        .await
        .map_err(|e| format!("failed to purge OTP challenges: {e}"))?
        .rows_affected();

    println!(
        "Purged {admin_sessions} expired admin session(s), {account_sessions} expired account session(s), \
         {otp_challenges} expired OTP challenge(s)."
    );
    Ok(())
}
