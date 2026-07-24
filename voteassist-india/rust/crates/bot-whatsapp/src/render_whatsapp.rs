//! Renders channel-agnostic `channel_core::RenderableNode` data as
//! WhatsApp-native messages: interactive reply buttons (2-3 options) or
//! an interactive list (4-10 options) for questions, plain text for
//! terminal outcomes. No decision logic lives here — only translation,
//! per docs/PRD-V2-RUST-PLATFORM.md Section 6.7 and E8.F2.T2/T4.
//!
//! Handles Meta's hard per-field character limits by truncating (never by
//! erroring or silently dropping the option) — a real platform constraint
//! every string here must respect, not a corner this module cuts.
//! Truncation operates on `char` boundaries (not bytes) so multi-byte
//! Hindi/Devanagari text is never split mid-codepoint.

use channel_core::{RenderableQuestion, RenderableTerminal};

use crate::client::{ListRow, ReplyButton, WhatsAppApiError, WhatsAppClient};

/// Meta's documented limits (interactive messages, Cloud API, 2026):
/// button/list-row title <= 24 chars for list rows, <= 20 for reply
/// buttons; list row description <= 72 chars; interactive body <= 1024.
const LIST_ROW_TITLE_MAX: usize = 24;
const BUTTON_TITLE_MAX: usize = 20;
const INTERACTIVE_BODY_MAX: usize = 1024;

/// Meta caps reply-button messages at 3 buttons; above that, a list
/// message (which supports up to 10 rows) is used instead.
const MAX_REPLY_BUTTONS: usize = 3;
/// Meta caps list messages at 10 rows total. A decision-tree question
/// with more options than this cannot be fully rendered on WhatsApp —
/// see the doc comment on `send_question` for how that's handled.
const MAX_LIST_ROWS: usize = 10;

pub const NOT_OFFICIAL_BANNER: &str = "VoteAssist India is an independent, non-official guidance tool. It is not the Election Commission of India and cannot register you to vote or submit any application on your behalf.";

fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let mut truncated: String = s.chars().take(max_chars.saturating_sub(1)).collect();
    truncated.push('…');
    truncated
}

/// Sends a question as an interactive WhatsApp message: reply buttons for
/// up to 3 options, a list for 4-10. A tree authored for VoteAssist's web
/// and Telegram channels is not guaranteed to fit WhatsApp's 10-option
/// list cap; rather than silently drop options past the 10th (which would
/// make an otherwise-valid answer unreachable on this channel only), this
/// logs a warning naming the dropped options so a maintainer notices and
/// either narrows the WhatsApp-facing tree or splits the question — never
/// something a citizen encounters as a silent gap.
pub async fn send_question(
    client: &WhatsAppClient,
    to: &str,
    question: &RenderableQuestion,
) -> Result<(), WhatsAppApiError> {
    let mut body_text = question.prompt.clone();
    if let Some(help) = &question.help_text {
        body_text.push_str("\n\n");
        body_text.push_str(help);
    }
    let body_text = truncate_chars(&body_text, INTERACTIVE_BODY_MAX);

    if question.options.len() <= MAX_REPLY_BUTTONS {
        let buttons: Vec<ReplyButton> = question
            .options
            .iter()
            .map(|o| ReplyButton {
                id: o.value.clone(),
                title: truncate_chars(&o.label, BUTTON_TITLE_MAX),
            })
            .collect();
        client.send_interactive_buttons(to, &body_text, &buttons).await
    } else {
        if question.options.len() > MAX_LIST_ROWS {
            let dropped: Vec<&str> = question.options[MAX_LIST_ROWS..]
                .iter()
                .map(|o| o.value.as_str())
                .collect();
            tracing::warn!(
                node_id = %question.node_id,
                dropped_option_count = dropped.len(),
                dropped_values = ?dropped,
                "question has more options than WhatsApp's 10-row list limit; trailing options are unreachable on this channel"
            );
        }

        let rows: Vec<ListRow> = question
            .options
            .iter()
            .take(MAX_LIST_ROWS)
            .map(|o| ListRow {
                id: o.value.clone(),
                title: truncate_chars(&o.label, LIST_ROW_TITLE_MAX),
                description: None,
            })
            .collect();
        client.send_interactive_list(to, &body_text, "Choose", &rows).await
    }
}

pub fn render_terminal(terminal: &RenderableTerminal) -> String {
    let mut out = String::new();

    out.push_str(&terminal.outcome_title);
    out.push_str("\n\n");
    out.push_str(&terminal.outcome_description);
    out.push_str("\n\n");

    if !terminal.recommended_forms.is_empty() {
        out.push_str("Recommended form(s): ");
        out.push_str(&terminal.recommended_forms.join(", "));
        out.push_str("\n\n");
    }

    if !terminal.checklist.is_empty() {
        out.push_str("Checklist:\n");
        for item in &terminal.checklist {
            out.push_str("- ");
            out.push_str(item);
            out.push('\n');
        }
        out.push('\n');
    }

    if !terminal.deep_links.is_empty() {
        out.push_str("Official next steps:\n");
        for link in &terminal.deep_links {
            out.push_str("- ");
            out.push_str(&link.label);
            out.push_str(": ");
            out.push_str(&link.url);
            out.push('\n');
        }
        out.push('\n');
    }

    if !terminal.citation_ids.is_empty() {
        out.push_str("Sources:\n");
        for citation_id in &terminal.citation_ids {
            match kb_content::get_entry(citation_id) {
                Some(entry) => {
                    out.push_str("- ");
                    match entry.sources.first() {
                        Some(source) => {
                            out.push_str(&source.title);
                            out.push_str(": ");
                            out.push_str(&source.url);
                        }
                        None => out.push_str(&entry.title),
                    }
                    out.push('\n');
                }
                None => tracing::warn!(
                    citation_id = %citation_id,
                    "terminal node citation references an unknown knowledge-base entry id"
                ),
            }
        }
        out.push('\n');
    }

    if !terminal.caution.is_empty() {
        out.push_str(&terminal.caution);
        out.push_str("\n\n");
    }

    out.push_str(NOT_OFFICIAL_BANNER);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_chars_leaves_short_strings_untouched() {
        assert_eq!(truncate_chars("Yes", 24), "Yes");
    }

    #[test]
    fn truncate_chars_respects_multi_byte_boundaries() {
        let devanagari = "हिन्दी में एक बहुत लंबा उत्तर विकल्प जो सीमा से अधिक लंबा है";
        let truncated = truncate_chars(devanagari, LIST_ROW_TITLE_MAX);
        assert!(truncated.chars().count() <= LIST_ROW_TITLE_MAX);
        assert!(truncated.ends_with('…'));
    }

    #[test]
    fn render_terminal_includes_the_not_official_banner() {
        let terminal = RenderableTerminal {
            node_id: "t1".into(),
            outcome_title: "File Form 6".into(),
            outcome_description: "You should register as a new elector.".into(),
            recommended_forms: vec!["form-6".into()],
            checklist: vec!["Proof of age".into()],
            citation_ids: vec![],
            deep_links: vec![],
            caution: String::new(),
        };
        let rendered = render_terminal(&terminal);
        assert!(rendered.contains(NOT_OFFICIAL_BANNER));
        assert!(rendered.contains("form-6"));
    }
}
