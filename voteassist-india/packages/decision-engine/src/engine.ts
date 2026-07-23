import {
  Answer,
  DecisionTree,
  EngineState,
  QuestionNode,
  isQuestion,
  isTerminal,
} from "./types";

export class UnknownNodeError extends Error {
  constructor(nodeId: string, treeId: string) {
    super(`Node "${nodeId}" does not exist in tree "${treeId}"`);
    this.name = "UnknownNodeError";
  }
}

export class InvalidAnswerError extends Error {
  constructor(value: string, nodeId: string) {
    super(`"${value}" is not a valid option for question "${nodeId}"`);
    this.name = "InvalidAnswerError";
  }
}

export class TerminalReachedError extends Error {
  constructor(nodeId: string) {
    super(`Node "${nodeId}" is a terminal outcome; there are no more questions to answer`);
    this.name = "TerminalReachedError";
  }
}

function getNode(tree: DecisionTree, nodeId: string) {
  const node = tree.nodes[nodeId];
  if (!node) throw new UnknownNodeError(nodeId, tree.id);
  return node;
}

export function createSession(tree: DecisionTree): EngineState {
  // Fails fast if the tree was never validated.
  getNode(tree, tree.startNodeId);
  return { treeId: tree.id, currentNodeId: tree.startNodeId, history: [] };
}

export function getCurrentNode(tree: DecisionTree, state: EngineState) {
  return getNode(tree, state.currentNodeId);
}

export function answer(tree: DecisionTree, state: EngineState, value: string): EngineState {
  const node = getCurrentNode(tree, state);
  if (isTerminal(node)) throw new TerminalReachedError(node.id);

  const question = node as QuestionNode;
  const option = question.options.find((o) => o.value === value);
  if (!option) throw new InvalidAnswerError(value, question.id);

  const historyEntry: Answer = { questionId: question.id, value };
  return {
    treeId: state.treeId,
    currentNodeId: option.next,
    history: [...state.history, historyEntry],
  };
}

export function isSessionComplete(tree: DecisionTree, state: EngineState): boolean {
  return isTerminal(getCurrentNode(tree, state));
}

/**
 * Rough progress indicator for a progress bar: how many questions answered
 * versus the longest question-only path from the start node. Not exact
 * (the tree is not balanced) but monotonically increases to 1 at a terminal.
 */
export function estimateProgress(tree: DecisionTree, state: EngineState): number {
  const longest = longestQuestionPath(tree, tree.startNodeId, new Set());
  if (longest === 0) return 1;
  return Math.min(1, state.history.length / longest);
}

function longestQuestionPath(tree: DecisionTree, nodeId: string, visiting: Set<string>): number {
  const node = getNode(tree, nodeId);
  if (isTerminal(node)) return 0;
  if (visiting.has(nodeId)) {
    throw new Error(`Cycle detected at node "${nodeId}" in tree "${tree.id}"`);
  }
  visiting.add(nodeId);
  let max = 0;
  for (const option of node.options) {
    max = Math.max(max, 1 + longestQuestionPath(tree, option.next, visiting));
  }
  visiting.delete(nodeId);
  return max;
}

export interface TreeValidationIssue {
  nodeId: string;
  message: string;
}

/**
 * Structural validation every tree must pass before being shipped:
 *  - every option points at a node that exists
 *  - every terminal node carries at least one citation and one deep link
 *  - the graph is acyclic (guarantees every session eventually terminates)
 *  - every non-start node is reachable from the start node (no dead content)
 */
export function validateTree(tree: DecisionTree): TreeValidationIssue[] {
  const issues: TreeValidationIssue[] = [];

  if (!tree.nodes[tree.startNodeId]) {
    issues.push({ nodeId: tree.startNodeId, message: "startNodeId does not exist in nodes" });
    return issues;
  }

  for (const node of Object.values(tree.nodes)) {
    if (isQuestion(node)) {
      if (node.options.length === 0) {
        issues.push({ nodeId: node.id, message: "question node has no options" });
      }
      for (const option of node.options) {
        if (!tree.nodes[option.next]) {
          issues.push({
            nodeId: node.id,
            message: `option "${option.value}" points to missing node "${option.next}"`,
          });
        }
      }
    } else {
      if (node.citations.length === 0) {
        issues.push({ nodeId: node.id, message: "terminal node has no citations" });
      }
      if (node.deepLinks.length === 0) {
        issues.push({ nodeId: node.id, message: "terminal node has no official deep link" });
      }
    }
  }

  // Cycle detection (DFS over question nodes only; terminals have no out-edges).
  const WHITE = 0, GRAY = 1, BLACK = 2;
  const color = new Map<string, number>();
  const dfs = (nodeId: string) => {
    const node = tree.nodes[nodeId];
    if (!node) return; // already reported above
    color.set(nodeId, GRAY);
    if (isQuestion(node)) {
      for (const option of node.options) {
        const nextColor = color.get(option.next) ?? WHITE;
        if (nextColor === GRAY) {
          issues.push({ nodeId, message: `cycle detected via option "${option.value}" -> "${option.next}"` });
        } else if (nextColor === WHITE) {
          dfs(option.next);
        }
      }
    }
    color.set(nodeId, BLACK);
  };
  dfs(tree.startNodeId);

  // Reachability (informational: unreachable nodes are dead content).
  const reachable = new Set(color.keys());
  for (const nodeId of Object.keys(tree.nodes)) {
    if (!reachable.has(nodeId)) {
      issues.push({ nodeId, message: "node is unreachable from startNodeId" });
    }
  }

  return issues;
}
