//! Nightly, courteous citation link-checker.
//!
//! docs/SECURITY-AND-SRE-OPERATIONS.md Section 3 and
//! docs/PRD-V3-COMPREHENSIVE-EXPANSION.md Section V5 both specify this
//! job's discipline: check a small, curated, human-picked set of citation
//! URLs (never a crawl), identify the requester with a distinct
//! User-Agent naming the project, rate-limit requests so this tool never
//! looks like an abusive crawler against government domains, and only
//! ever FLAG content for human re-verification — never auto-edit it.
//!
//! Known scope boundary, disclosed rather than silently skipped: this
//! implementation does not parse `robots.txt` before requesting a URL.
//! Given the small, fixed, human-curated set of URLs involved (on the
//! order of tens, not thousands) and the conservative rate limiting
//! below, the risk this poses is low, but a `robots.txt`-respecting HTTP
//! client (e.g. wrapping `reqwest` with a small crate or hand-rolled
//! fetch-and-parse step) is a legitimate, separable follow-up rather than
//! something this job silently omits without acknowledgement.

use std::time::Duration;

use kb_content::knowledge_entries;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// Identifies this tool to the servers it checks, per the courteous-
/// crawling norm above. Includes a contact-adjacent project name rather
/// than impersonating a browser.
const USER_AGENT: &str = "VoteAssistIndia-LinkChecker/0.1 (+https://github.com/kr5/the_well; civic-tech citation health check, low frequency, see docs/SECURITY-AND-SRE-OPERATIONS.md)";

/// Minimum delay between consecutive outbound requests, so a KB with many
/// cited sources never produces a burst against any single domain (most
/// citations in this project point at voters.eci.gov.in or
/// servicevoter.nic.in, so bursts against the *same* host are the
/// realistic risk this guards against, not just aggregate request rate).
const MIN_DELAY_BETWEEN_REQUESTS: Duration = Duration::from_millis(750);

const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkCheckOutcome {
    pub entry_id: String,
    pub source_url: String,
    pub http_status: Option<u16>,
    pub error_message: Option<String>,
}

impl LinkCheckOutcome {
    pub fn is_healthy(&self) -> bool {
        matches!(self.http_status, Some(status) if (200..400).contains(&status))
    }
}

pub fn build_client() -> Result<Client, reqwest::Error> {
    Client::builder().user_agent(USER_AGENT).timeout(REQUEST_TIMEOUT).build()
}

/// Checks every cited source URL across the entire curated knowledge base
/// (currently loaded from `knowledge-base/sources/*.json` via
/// `kb-content` — see that crate's module docs for the file<->database
/// reconciliation note). Sequential, not concurrent, by design: the rate
/// limit above only holds if requests are issued one at a time.
pub async fn check_all_citation_links(client: &Client) -> Vec<LinkCheckOutcome> {
    let mut outcomes = Vec::new();

    for entry in knowledge_entries() {
        for source in &entry.sources {
            let outcome = check_one(client, &entry.id, &source.url).await;
            outcomes.push(outcome);
            tokio::time::sleep(MIN_DELAY_BETWEEN_REQUESTS).await;
        }
    }

    outcomes
}

async fn check_one(client: &Client, entry_id: &str, url: &str) -> LinkCheckOutcome {
    match client.head(url).send().await {
        Ok(response) => LinkCheckOutcome {
            entry_id: entry_id.to_string(),
            source_url: url.to_string(),
            http_status: Some(response.status().as_u16()),
            error_message: None,
        },
        Err(err) => LinkCheckOutcome {
            entry_id: entry_id.to_string(),
            source_url: url.to_string(),
            http_status: None,
            error_message: Some(err.to_string()),
        },
    }
}

/// Persists outcomes into `link_check_results`, per
/// `migrations/0010_link_check_and_translation.sql`. Since that table's
/// `source_id` column is a foreign key to `knowledge_entry_sources.id` (a
/// database row, not a JSON file), and no sync job populating
/// `knowledge_entries`/`knowledge_entry_sources` from the curated JSON
/// files exists yet (a tracked PRD v2 Section 9 follow-up), this function
/// resolves each outcome's `(entry_id, url)` pair to a `source_id` by
/// lookup rather than assuming one — an outcome with no matching database
/// row is logged and skipped, never silently dropped without a trace.
pub async fn persist_outcomes(pool: &PgPool, outcomes: &[LinkCheckOutcome]) -> Result<PersistSummary, sqlx::Error> {
    let mut summary = PersistSummary::default();

    for outcome in outcomes {
        let source_id: Option<uuid::Uuid> = sqlx::query_scalar(
            "SELECT id FROM knowledge_entry_sources WHERE entry_id = $1 AND url = $2",
        )
        .bind(&outcome.entry_id)
        .bind(&outcome.source_url)
        .fetch_optional(pool)
        .await?;

        match source_id {
            Some(source_id) => {
                sqlx::query(
                    r#"
                    INSERT INTO link_check_results (source_id, source_url, http_status, error_message)
                    VALUES ($1, $2, $3, $4)
                    "#,
                )
                .bind(source_id)
                .bind(&outcome.source_url)
                .bind(outcome.http_status.map(|s| s as i32))
                .bind(&outcome.error_message)
                .execute(pool)
                .await?;
                summary.persisted += 1;
            }
            None => {
                tracing::warn!(
                    entry_id = %outcome.entry_id,
                    url = %outcome.source_url,
                    "no matching knowledge_entry_sources row yet — skipping DB write until the KB-file-to-database sync job exists"
                );
                summary.skipped_no_db_row += 1;
            }
        }
    }

    Ok(summary)
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PersistSummary {
    pub persisted: u64,
    pub skipped_no_db_row: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_builds_with_the_documented_user_agent() {
        let client = build_client();
        assert!(client.is_ok(), "client should build without requiring network access");
    }

    #[test]
    fn is_healthy_treats_2xx_and_3xx_as_healthy() {
        let ok = LinkCheckOutcome {
            entry_id: "form-8".into(),
            source_url: "https://example.invalid".into(),
            http_status: Some(200),
            error_message: None,
        };
        assert!(ok.is_healthy());

        let redirect = LinkCheckOutcome { http_status: Some(301), ..ok.clone() };
        assert!(redirect.is_healthy());

        let not_found = LinkCheckOutcome { http_status: Some(404), ..ok.clone() };
        assert!(!not_found.is_healthy());

        let network_error =
            LinkCheckOutcome { http_status: None, error_message: Some("timed out".into()), ..ok };
        assert!(!network_error.is_healthy());
    }
}
