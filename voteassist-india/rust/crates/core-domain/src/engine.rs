use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::types::{Answer, DecisionNode, DecisionTree, EngineState};

/// Direct Rust port of packages/decision-engine/src/engine.ts's error classes.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EngineError {
    #[error("node \"{node_id}\" does not exist in tree \"{tree_id}\"")]
    UnknownNode { node_id: String, tree_id: String },

    #[error("\"{value}\" is not a valid option for question \"{node_id}\"")]
    InvalidAnswer { value: String, node_id: String },

    #[error("node \"{node_id}\" is a terminal outcome; there are no more questions to answer")]
    TerminalReached { node_id: String },
}

fn get_node<'a>(tree: &'a DecisionTree, node_id: &str) -> Result<&'a DecisionNode, EngineError> {
    tree.nodes
        .get(node_id)
        .ok_or_else(|| EngineError::UnknownNode {
            node_id: node_id.to_string(),
            tree_id: tree.id.clone(),
        })
}

/// Fails fast if the tree's own `startNodeId` doesn't resolve, mirroring
/// `createSession` in the TypeScript prototype.
pub fn create_session(tree: &DecisionTree) -> Result<EngineState, EngineError> {
    get_node(tree, &tree.start_node_id)?;
    Ok(EngineState {
        tree_id: tree.id.clone(),
        current_node_id: tree.start_node_id.clone(),
        history: Vec::new(),
    })
}

pub fn get_current_node<'a>(
    tree: &'a DecisionTree,
    state: &EngineState,
) -> Result<&'a DecisionNode, EngineError> {
    get_node(tree, &state.current_node_id)
}

pub fn answer(
    tree: &DecisionTree,
    state: &EngineState,
    value: &str,
) -> Result<EngineState, EngineError> {
    let node = get_current_node(tree, state)?;
    let question = match node {
        DecisionNode::Terminal(t) => {
            return Err(EngineError::TerminalReached {
                node_id: t.id.clone(),
            })
        }
        DecisionNode::Question(q) => q,
    };

    let option = question
        .options
        .iter()
        .find(|o| o.value == value)
        .ok_or_else(|| EngineError::InvalidAnswer {
            value: value.to_string(),
            node_id: question.id.clone(),
        })?;

    let mut history = state.history.clone();
    history.push(Answer {
        question_id: question.id.clone(),
        value: value.to_string(),
    });

    Ok(EngineState {
        tree_id: state.tree_id.clone(),
        current_node_id: option.next.clone(),
        history,
    })
}

pub fn is_session_complete(tree: &DecisionTree, state: &EngineState) -> Result<bool, EngineError> {
    Ok(get_current_node(tree, state)?.is_terminal())
}

/// Rough progress indicator (0.0-1.0): questions answered so far versus the
/// longest question-only path from the start node. Not exact (the tree isn't
/// balanced) but monotonically increases to 1.0 at a terminal, matching
/// `estimateProgress` in the TypeScript prototype.
pub fn estimate_progress(tree: &DecisionTree, state: &EngineState) -> Result<f64, EngineError> {
    let longest = longest_question_path(tree, &tree.start_node_id, &mut HashSet::new())?;
    if longest == 0 {
        return Ok(1.0);
    }
    Ok((state.history.len() as f64 / longest as f64).min(1.0))
}

fn longest_question_path(
    tree: &DecisionTree,
    node_id: &str,
    visiting: &mut HashSet<String>,
) -> Result<u32, EngineError> {
    let node = get_node(tree, node_id)?;
    let question = match node {
        DecisionNode::Terminal(_) => return Ok(0),
        DecisionNode::Question(q) => q,
    };

    if !visiting.insert(node_id.to_string()) {
        // A cycle exists. validate_tree() is the authority on reporting this
        // as a build-breaking issue; here we just avoid infinite recursion.
        return Ok(0);
    }

    let mut max = 0u32;
    for option in &question.options {
        let candidate = 1 + longest_question_path(tree, &option.next, visiting)?;
        max = max.max(candidate);
    }
    visiting.remove(node_id);
    Ok(max)
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TreeValidationIssue {
    pub node_id: String,
    pub message: String,
}

/// Structural validation every tree must pass before being shipped:
///  - every option points at a node that exists
///  - every terminal node carries at least one citation and one deep link
///  - the graph is acyclic (guarantees every session eventually terminates)
///  - every non-start node is reachable from the start node (no dead content)
///
/// Direct port of `validateTree` from packages/decision-engine/src/engine.ts.
pub fn validate_tree(tree: &DecisionTree) -> Vec<TreeValidationIssue> {
    let mut issues = Vec::new();

    if !tree.nodes.contains_key(&tree.start_node_id) {
        issues.push(TreeValidationIssue {
            node_id: tree.start_node_id.clone(),
            message: "startNodeId does not exist in nodes".to_string(),
        });
        return issues;
    }

    for node in tree.nodes.values() {
        match node {
            DecisionNode::Question(q) => {
                if q.options.is_empty() {
                    issues.push(TreeValidationIssue {
                        node_id: q.id.clone(),
                        message: "question node has no options".to_string(),
                    });
                }
                for option in &q.options {
                    if !tree.nodes.contains_key(&option.next) {
                        issues.push(TreeValidationIssue {
                            node_id: q.id.clone(),
                            message: format!(
                                "option \"{}\" points to missing node \"{}\"",
                                option.value, option.next
                            ),
                        });
                    }
                }
            }
            DecisionNode::Terminal(t) => {
                if t.citations.is_empty() {
                    issues.push(TreeValidationIssue {
                        node_id: t.id.clone(),
                        message: "terminal node has no citations".to_string(),
                    });
                }
                if t.deep_links.is_empty() {
                    issues.push(TreeValidationIssue {
                        node_id: t.id.clone(),
                        message: "terminal node has no official deep link".to_string(),
                    });
                }
            }
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Color {
        White,
        Gray,
        Black,
    }

    fn dfs(
        tree: &DecisionTree,
        node_id: &str,
        color: &mut HashMap<String, Color>,
        issues: &mut Vec<TreeValidationIssue>,
    ) {
        let Some(node) = tree.nodes.get(node_id) else {
            return; // already reported above
        };
        color.insert(node_id.to_string(), Color::Gray);
        if let DecisionNode::Question(q) = node {
            for option in &q.options {
                match color.get(&option.next).copied().unwrap_or(Color::White) {
                    Color::Gray => issues.push(TreeValidationIssue {
                        node_id: node_id.to_string(),
                        message: format!(
                            "cycle detected via option \"{}\" -> \"{}\"",
                            option.value, option.next
                        ),
                    }),
                    Color::White => dfs(tree, &option.next, color, issues),
                    Color::Black => {}
                }
            }
        }
        color.insert(node_id.to_string(), Color::Black);
    }

    let mut color: HashMap<String, Color> = HashMap::new();
    dfs(tree, &tree.start_node_id, &mut color, &mut issues);

    for node_id in tree.nodes.keys() {
        if !color.contains_key(node_id) {
            issues.push(TreeValidationIssue {
                node_id: node_id.clone(),
                message: "node is unreachable from startNodeId".to_string(),
            });
        }
    }

    issues
}
