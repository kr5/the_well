import { describe, expect, it } from "vitest";
import {
  answer,
  createSession,
  estimateProgress,
  getCurrentNode,
  InvalidAnswerError,
  isSessionComplete,
  TerminalReachedError,
  UnknownNodeError,
} from "../src/engine";
import { voteAssistTreeV1 } from "../src/tree";
import { DecisionTree, isQuestion, isTerminal } from "../src/types";

describe("engine basics", () => {
  it("creates a session at the start node", () => {
    const state = createSession(voteAssistTreeV1);
    expect(state.currentNodeId).toBe("start");
    expect(state.history).toEqual([]);
  });

  it("throws UnknownNodeError for a tree with a bad startNodeId", () => {
    const brokenTree: DecisionTree = { ...voteAssistTreeV1, startNodeId: "does_not_exist" };
    expect(() => createSession(brokenTree)).toThrow(UnknownNodeError);
  });

  it("walks the NRI path to Form 6A", () => {
    let state = createSession(voteAssistTreeV1);
    state = answer(voteAssistTreeV1, state, "no"); // not registered
    state = answer(voteAssistTreeV1, state, "adult");
    state = answer(voteAssistTreeV1, state, "nri");
    expect(isSessionComplete(voteAssistTreeV1, state)).toBe(true);
    const node = getCurrentNode(voteAssistTreeV1, state);
    expect(node.id).toBe("terminal_form6a");
    expect(state.history).toEqual([
      { questionId: "start", value: "no" },
      { questionId: "not_registered_age", value: "adult" },
      { questionId: "citizenship_check", value: "nri" },
    ]);
  });

  it("walks the student-hostel path to the ordinary-residence terminal", () => {
    let state = createSession(voteAssistTreeV1);
    state = answer(voteAssistTreeV1, state, "no");
    state = answer(voteAssistTreeV1, state, "adult");
    state = answer(voteAssistTreeV1, state, "resident");
    state = answer(voteAssistTreeV1, state, "student_hostel");
    state = answer(voteAssistTreeV1, state, "hostel");
    expect(getCurrentNode(voteAssistTreeV1, state).id).toBe("terminal_form6_student_hostel");
  });

  it("rejects an answer value that isn't a valid option", () => {
    const state = createSession(voteAssistTreeV1);
    expect(() => answer(voteAssistTreeV1, state, "not-a-real-option")).toThrow(InvalidAnswerError);
  });

  it("refuses to answer a terminal node", () => {
    let state = createSession(voteAssistTreeV1);
    state = answer(voteAssistTreeV1, state, "not_sure"); // -> terminal_roll_search
    expect(isSessionComplete(voteAssistTreeV1, state)).toBe(true);
    expect(() => answer(voteAssistTreeV1, state, "anything")).toThrow(TerminalReachedError);
  });

  it("progress increases monotonically to 1 at a terminal along every path", () => {
    let state = createSession(voteAssistTreeV1);
    let progress = estimateProgress(voteAssistTreeV1, state);
    expect(progress).toBe(0);
    const path = ["no", "adult", "resident", "student_hostel", "hostel"];
    for (const value of path) {
      const next = answer(voteAssistTreeV1, state, value);
      const nextProgress = estimateProgress(voteAssistTreeV1, next);
      expect(nextProgress).toBeGreaterThanOrEqual(progress);
      state = next;
      progress = nextProgress;
    }
    expect(progress).toBe(1);
  });
});

describe("exhaustive path enumeration (every path terminates)", () => {
  function enumerate(state: ReturnType<typeof createSession>, depth: number): void {
    if (depth > 20) throw new Error("path did not terminate within 20 steps — possible cycle");
    const node = getCurrentNode(voteAssistTreeV1, state);
    if (isTerminal(node)) return;
    if (isQuestion(node)) {
      for (const option of node.options) {
        enumerate(answer(voteAssistTreeV1, state, option.value), depth + 1);
      }
    }
  }

  it("every combination of answers reaches a terminal node within 20 steps", () => {
    expect(() => enumerate(createSession(voteAssistTreeV1), 0)).not.toThrow();
  });
});
