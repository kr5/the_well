//! Renders channel-agnostic `channel_core::RenderableNode` data as
//! Telegram-native messages: an inline keyboard for questions, plain text
//! for terminal outcomes. No decision logic lives here — only translation,
//! per docs/PRD-V2-RUST-PLATFORM.md Section 6.7.
//!
//! Deliberately plain text (no Markdown/HTML parse mode): knowledge-base
//! content is curated, not attacker-controlled, but Telegram's Markdown/
//! MarkdownV2 parsers reject messages containing unescaped reserved
//! characters, and correctly escaping arbitrary curated prose is a real
//! source of send-time failures for little visual benefit in a bot
//! context. Telegram auto-links bare URLs in plain-text messages, so
//! deep links remain clickable regardless.

use channel_core::{RenderableQuestion, RenderableTerminal};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub const NOT_OFFICIAL_BANNER: &str = "VoteAssist India is an independent, non-official guidance tool. It is not the Election Commission of India and cannot register you to vote or submit any application on your behalf.";

pub fn render_question(question: &RenderableQuestion) -> (String, InlineKeyboardMarkup) {
    let mut text = question.prompt.clone();
    if let Some(help) = &question.help_text {
        text.push_str("\n\n");
        text.push_str(help);
    }

    let keyboard: Vec<Vec<InlineKeyboardButton>> = question
        .options
        .iter()
        .map(|option| {
            vec![InlineKeyboardButton::callback(
                option.label.clone(),
                option.value.clone(),
            )]
        })
        .collect();

    (text, InlineKeyboardMarkup::new(keyboard))
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
    use channel_core::RenderableOption;

    #[test]
    fn render_question_produces_one_button_per_option() {
        let question = RenderableQuestion {
            node_id: "n1".into(),
            prompt: "Are you a first-time voter?".into(),
            help_text: None,
            options: vec![
                RenderableOption { value: "yes".into(), label: "Yes".into() },
                RenderableOption { value: "no".into(), label: "No".into() },
            ],
        };
        let (text, keyboard) = render_question(&question);
        assert_eq!(text, "Are you a first-time voter?");
        assert_eq!(keyboard.inline_keyboard.len(), 2);
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
