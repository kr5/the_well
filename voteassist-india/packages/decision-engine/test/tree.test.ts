import { describe, expect, it } from "vitest";
import { validateTree } from "../src/engine";
import { voteAssistTreeV1 } from "../src/tree";
import { isTerminal, isQuestion } from "../src/types";

describe("voteAssistTreeV1 structural validity", () => {
  it("has no validation issues (no dangling options, no cycles, no unreachable nodes, every terminal cited)", () => {
    const issues = validateTree(voteAssistTreeV1);
    expect(issues).toEqual([]);
  });

  it("every question option has both an en and hi label", () => {
    for (const node of Object.values(voteAssistTreeV1.nodes)) {
      if (isQuestion(node)) {
        expect(node.prompt.en).toBeTruthy();
        expect(node.prompt.hi).toBeTruthy();
        for (const option of node.options) {
          expect(option.label.en).toBeTruthy();
          expect(option.label.hi).toBeTruthy();
        }
      }
    }
  });

  it("every terminal node has an en+hi title, description, caution, at least one citation and one deep link", () => {
    for (const node of Object.values(voteAssistTreeV1.nodes)) {
      if (isTerminal(node)) {
        expect(node.outcomeTitle.en).toBeTruthy();
        expect(node.outcomeTitle.hi).toBeTruthy();
        expect(node.outcomeDescription.en).toBeTruthy();
        expect(node.outcomeDescription.hi).toBeTruthy();
        expect(node.caution.en).toBeTruthy();
        expect(node.caution.hi).toBeTruthy();
        expect(node.citations.length).toBeGreaterThan(0);
        expect(node.deepLinks.length).toBeGreaterThan(0);
        for (const link of node.deepLinks) {
          expect(() => new URL(link.url)).not.toThrow();
        }
      }
    }
  });

  it("has at least one terminal node reachable for every major scenario in scope", () => {
    const ids = Object.keys(voteAssistTreeV1.nodes);
    const expectedTerminals = [
      "terminal_roll_search",
      "terminal_not_yet_eligible",
      "terminal_qualifying_date",
      "terminal_form6a",
      "terminal_service_voter",
      "terminal_form6_new",
      "terminal_form6_student_hostel",
      "terminal_form6_no_fixed_address",
      "terminal_form8_correction",
      "terminal_form8_shift",
      "terminal_eepic_download",
      "terminal_form8_epic_replacement",
      "terminal_pwd_marking",
      "terminal_form7",
    ];
    for (const id of expectedTerminals) {
      expect(ids).toContain(id);
    }
  });
});
