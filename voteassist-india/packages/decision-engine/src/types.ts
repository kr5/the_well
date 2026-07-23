export interface LocalizedText {
  en: string;
  hi?: string;
  [locale: string]: string | undefined;
}

export interface Option {
  value: string;
  label: LocalizedText;
  next: string;
}

export interface QuestionNode {
  id: string;
  type: "question";
  prompt: LocalizedText;
  helpText?: LocalizedText;
  options: Option[];
}

export interface DeepLink {
  label: LocalizedText;
  url: string;
}

export interface Citation {
  /** id of an entry in knowledge-base/sources/*.json, resolved via @voteassist/knowledge */
  knowledgeBaseId: string;
}

export interface TerminalNode {
  id: string;
  type: "terminal";
  outcomeTitle: LocalizedText;
  outcomeDescription: LocalizedText;
  recommendedForms: string[];
  checklist: LocalizedText[];
  citations: Citation[];
  deepLinks: DeepLink[];
  caution: LocalizedText;
}

export type DecisionNode = QuestionNode | TerminalNode;

export interface DecisionTree {
  id: string;
  version: number;
  startNodeId: string;
  nodes: Record<string, DecisionNode>;
}

export interface Answer {
  questionId: string;
  value: string;
}

export interface EngineState {
  treeId: string;
  currentNodeId: string;
  history: Answer[];
}

export function isTerminal(node: DecisionNode): node is TerminalNode {
  return node.type === "terminal";
}

export function isQuestion(node: DecisionNode): node is QuestionNode {
  return node.type === "question";
}
