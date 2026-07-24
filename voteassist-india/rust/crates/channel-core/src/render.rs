//! Channel-agnostic rendering of a `core_domain::DecisionNode` into plain,
//! locale-resolved data. Every adapter (`bot-telegram`, `bot-whatsapp`,
//! `ivr-gateway`) converts this into its own native message format —
//! inline keyboard, WhatsApp interactive list, or TTS/DTMF prompt. Per
//! docs/PRD-V2-RUST-PLATFORM.md Section 6.7: "None of these adapters
//! contain decision logic — they only translate."

use core_domain::{DecisionNode, DeepLink, QuestionOption, TerminalNode};

#[derive(Debug, Clone)]
pub enum RenderableNode {
    Question(RenderableQuestion),
    Terminal(RenderableTerminal),
}

#[derive(Debug, Clone)]
pub struct RenderableQuestion {
    pub node_id: String,
    pub prompt: String,
    pub help_text: Option<String>,
    pub options: Vec<RenderableOption>,
}

#[derive(Debug, Clone)]
pub struct RenderableOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct RenderableTerminal {
    pub node_id: String,
    pub outcome_title: String,
    pub outcome_description: String,
    pub recommended_forms: Vec<String>,
    pub checklist: Vec<String>,
    /// `kb-content` entry ids; adapters that can render rich citations
    /// (web, and to a lesser extent Telegram/WhatsApp) resolve these via
    /// `kb_content::require_entry`. `ivr-gateway` has no natural way to
    /// read out a citation list over a phone call, so it renders these as
    /// a spoken "for the official source, see the SMS we're sending you"
    /// handoff instead (see that crate's module docs).
    pub citation_ids: Vec<String>,
    pub deep_links: Vec<RenderableDeepLink>,
    pub caution: String,
}

#[derive(Debug, Clone)]
pub struct RenderableDeepLink {
    pub label: String,
    pub url: String,
}

/// Resolves every `LocalizedText` field in `node` to `locale` (falling
/// back to English per `LocalizedText::pick`). This function never
/// decides "what's next" — that's still `core_domain::engine::answer`'s
/// job; this only flattens display data.
pub fn render_node(node: &DecisionNode, locale: &str) -> RenderableNode {
    match node {
        DecisionNode::Question(q) => RenderableNode::Question(RenderableQuestion {
            node_id: q.id.clone(),
            prompt: q.prompt.pick(locale).to_string(),
            help_text: q.help_text.as_ref().map(|h| h.pick(locale).to_string()),
            options: q.options.iter().map(|o| render_option(o, locale)).collect(),
        }),
        DecisionNode::Terminal(t) => RenderableNode::Terminal(render_terminal(t, locale)),
    }
}

fn render_option(option: &QuestionOption, locale: &str) -> RenderableOption {
    RenderableOption {
        value: option.value.clone(),
        label: option.label.pick(locale).to_string(),
    }
}

fn render_terminal(terminal: &TerminalNode, locale: &str) -> RenderableTerminal {
    RenderableTerminal {
        node_id: terminal.id.clone(),
        outcome_title: terminal.outcome_title.pick(locale).to_string(),
        outcome_description: terminal.outcome_description.pick(locale).to_string(),
        recommended_forms: terminal.recommended_forms.clone(),
        checklist: terminal.checklist.iter().map(|c| c.pick(locale).to_string()).collect(),
        citation_ids: terminal.citations.iter().map(|c| c.knowledge_base_id.clone()).collect(),
        deep_links: terminal.deep_links.iter().map(|d| render_deep_link(d, locale)).collect(),
        caution: terminal.caution.pick(locale).to_string(),
    }
}

fn render_deep_link(deep_link: &DeepLink, locale: &str) -> RenderableDeepLink {
    RenderableDeepLink {
        label: deep_link.label.pick(locale).to_string(),
        url: deep_link.url.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_domain::vote_assist_tree_v2;

    #[test]
    fn renders_the_v2_tree_start_node_as_a_question() {
        let tree = vote_assist_tree_v2();
        let start = tree.nodes.get(&tree.start_node_id).unwrap();
        match render_node(start, "en") {
            RenderableNode::Question(q) => {
                assert_eq!(q.node_id, tree.start_node_id);
                assert!(!q.options.is_empty());
                assert!(!q.prompt.is_empty());
            }
            RenderableNode::Terminal(_) => panic!("start node should not be terminal"),
        }
    }

    #[test]
    fn unknown_locale_falls_back_to_english() {
        let tree = vote_assist_tree_v2();
        let start = tree.nodes.get(&tree.start_node_id).unwrap();
        let en = render_node(start, "en");
        let fallback = render_node(start, "zz-not-a-real-locale");
        match (en, fallback) {
            (RenderableNode::Question(a), RenderableNode::Question(b)) => {
                assert_eq!(a.prompt, b.prompt);
            }
            _ => panic!("expected both renders to be questions"),
        }
    }
}
