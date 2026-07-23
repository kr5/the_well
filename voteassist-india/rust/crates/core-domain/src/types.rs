use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Localized text with a guaranteed English fallback. Mirrors the TypeScript
/// prototype's `LocalizedText` (packages/decision-engine/src/types.ts) so the
/// same JSON shape (`{ "en": "...", "hi": "..." }`) round-trips identically
/// through either implementation during the migration window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalizedText {
    pub en: String,
    #[serde(flatten)]
    pub other: BTreeMap<String, String>,
}

impl LocalizedText {
    pub fn new(en: impl Into<String>) -> Self {
        Self {
            en: en.into(),
            other: BTreeMap::new(),
        }
    }

    pub fn with(mut self, locale: impl Into<String>, text: impl Into<String>) -> Self {
        self.other.insert(locale.into(), text.into());
        self
    }

    /// Resolve localized text with a guaranteed fallback to English, mirroring
    /// packages/i18n/src/index.ts's `pick()`.
    pub fn pick(&self, locale: &str) -> &str {
        if locale == "en" {
            return &self.en;
        }
        self.other
            .get(locale)
            .map(String::as_str)
            .unwrap_or(&self.en)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionOption {
    pub value: String,
    pub label: LocalizedText,
    pub next: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionNode {
    pub id: String,
    pub prompt: LocalizedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub help_text: Option<LocalizedText>,
    pub options: Vec<QuestionOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeepLink {
    pub label: LocalizedText,
    pub url: String,
}

/// References an entry in the knowledge base (resolved via the sibling
/// `kb-content` crate, not this one — `core-domain` stays zero-I/O and does
/// not know how to load or validate knowledge entries itself).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Citation {
    pub knowledge_base_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalNode {
    pub id: String,
    pub outcome_title: LocalizedText,
    pub outcome_description: LocalizedText,
    pub recommended_forms: Vec<String>,
    pub checklist: Vec<LocalizedText>,
    pub citations: Vec<Citation>,
    pub deep_links: Vec<DeepLink>,
    pub caution: LocalizedText,
}

/// A node in the decision graph: either a question with branching options,
/// or a terminal outcome. Internally tagged on `type` so the JSON shape
/// (`{ "id": ..., "type": "question", ... }`) matches the TypeScript
/// prototype's discriminated union exactly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DecisionNode {
    Question(QuestionNode),
    Terminal(TerminalNode),
}

impl DecisionNode {
    pub fn id(&self) -> &str {
        match self {
            DecisionNode::Question(q) => &q.id,
            DecisionNode::Terminal(t) => &t.id,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, DecisionNode::Terminal(_))
    }

    pub fn is_question(&self) -> bool {
        matches!(self, DecisionNode::Question(_))
    }

    pub fn as_question(&self) -> Option<&QuestionNode> {
        match self {
            DecisionNode::Question(q) => Some(q),
            DecisionNode::Terminal(_) => None,
        }
    }

    pub fn as_terminal(&self) -> Option<&TerminalNode> {
        match self {
            DecisionNode::Terminal(t) => Some(t),
            DecisionNode::Question(_) => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionTree {
    pub id: String,
    pub version: u32,
    pub start_node_id: String,
    pub nodes: BTreeMap<String, DecisionNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Answer {
    pub question_id: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineState {
    pub tree_id: String,
    pub current_node_id: String,
    pub history: Vec<Answer>,
}
