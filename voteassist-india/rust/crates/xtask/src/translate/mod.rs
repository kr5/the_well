//! Machine-translation pipeline for VoteAssist India's knowledge base and
//! decision-tree text, using either Claude Haiku or NVIDIA NIM's
//! free-tier Nemotron models (see `client.rs` for backend selection).
//! Every output this pipeline produces is a **draft**, written to a
//! location the running application never reads by default — matching
//! the "MT-assist-as-draft-only" discipline documented (but not, until
//! now, implemented) in `crates/jobs::translation_completeness`'s module
//! doc and PRD v2 Section 11 admin page 5's "Translation Management"
//! spec. Nothing in this module writes to a file `kb-content`'s loader or
//! `core-domain`'s tree functions read at runtime.

pub mod client;
pub mod kb_entry;
pub mod locales;
pub mod tree;

use client::{TranslationClient, TranslationError};

/// Shared system prompt for every translation call in this pipeline —
/// same wording regardless of whether the caller is translating a KB
/// entry field or a decision-tree `LocalizedText` value, since both are
/// the same genre of text (plain-language Indian electoral-registration
/// procedure copy).
fn system_prompt(target_language: &str) -> String {
    format!(
        "You are translating plain-language civic/administrative text about Indian voter \
         registration procedures from English into {target_language}. This text will be shown \
         to Indian citizens on a non-official voter-guidance website. Rules:\n\
         - Preserve ECI form numbers exactly as written (Form 6, 6A, 7, 8, 12D) — never translate \
           or renumber them.\n\
         - Preserve acronyms and proper nouns as commonly used in {target_language}-language Indian \
           civic contexts, transliterating rather than translating where that's the natural local \
           usage: ECI, EPIC, BLO, ERO, DEO, CEO, SVEEP, NRI, PwD, AC, PC, MCC.\n\
         - Keep the tone plain-language and neutral — this is procedural guidance, not legal or \
           political text; do not add emphasis, opinion, or content not present in the source.\n\
         - Preserve the paragraph/sentence structure of the source as closely as natural in \
           {target_language}.\n\
         - Output ONLY the translated text. No preamble, no explanation, no quotation marks around \
           the output, no notes about your translation choices."
    )
}

async fn translate_text(client: &TranslationClient, target_language: &str, text: &str) -> Result<String, TranslationError> {
    if text.trim().is_empty() {
        return Ok(String::new());
    }
    client.complete(&system_prompt(target_language), text).await
}
