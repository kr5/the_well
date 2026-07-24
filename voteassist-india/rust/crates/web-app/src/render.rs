//! Renders a `core_domain::DecisionNode` into plain, locale-resolved,
//! serializable data the browser can render without needing the full
//! `DecisionTree` (which never crosses the wire — only the current node's
//! display data and the opaque `EngineState` do).
//!
//! This intentionally duplicates `channel_core::render`'s small rendering
//! function rather than depending on that crate directly: `channel-core`
//! pulls in `sqlx`/`tokio` (for its MCC gate and session store), neither
//! of which targets `wasm32-unknown-unknown` — and this module's types
//! are shared between `web-app`'s `ssr` (native) and `hydrate` (wasm)
//! builds via server-function signatures, so a `wasm32`-incompatible
//! transitive dependency here would break the client build entirely. The
//! duplicated logic is intentionally small (~40 lines); a future refactor
//! splitting `channel-core` into a wasm-safe rendering crate and a
//! server-only session/gate crate would let this module go away, but that
//! split is out of scope for this pass.

use core_domain::{DecisionNode, DeepLink, QuestionOption, TerminalNode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderableNode {
    Question(RenderableQuestion),
    Terminal(RenderableTerminal),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderableQuestion {
    pub node_id: String,
    pub prompt: String,
    pub help_text: Option<String>,
    pub options: Vec<RenderableOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderableOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderableTerminal {
    pub node_id: String,
    pub outcome_title: String,
    pub outcome_description: String,
    pub recommended_forms: Vec<String>,
    pub checklist: Vec<String>,
    /// `kb-content` entry ids — resolved into full citation cards
    /// client-side by fetching `get_kb_entry` for each id (see
    /// `pages::start`'s terminal-outcome rendering).
    pub citation_ids: Vec<String>,
    pub deep_links: Vec<RenderableDeepLink>,
    pub caution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderableDeepLink {
    pub label: String,
    pub url: String,
}

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
