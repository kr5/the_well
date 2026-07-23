import { describe, expect, it } from "vitest";
import { getEntry } from "@voteassist/knowledge";
import { voteAssistTreeV1 } from "../src/tree";
import { isTerminal } from "../src/types";

describe("every citation in the tree resolves to a real, cited knowledge base entry", () => {
  const terminals = Object.values(voteAssistTreeV1.nodes).filter(isTerminal);

  it("has at least one terminal node to check", () => {
    expect(terminals.length).toBeGreaterThan(0);
  });

  for (const node of terminals) {
    for (const citation of node.citations) {
      it(`"${node.id}" citation "${citation.knowledgeBaseId}" exists and carries at least one official source URL`, () => {
        const entry = getEntry(citation.knowledgeBaseId);
        expect(entry, `knowledge entry "${citation.knowledgeBaseId}" referenced by "${node.id}" is missing`).toBeTruthy();
        expect(entry!.sources.length).toBeGreaterThan(0);
        for (const source of entry!.sources) {
          expect(() => new URL(source.url)).not.toThrow();
        }
      });
    }
  }

  for (const node of terminals) {
    for (const formId of node.recommendedForms) {
      it(`"${node.id}" recommends "${formId}" which is a known form id`, () => {
        // Form ids referenced by terminal nodes should either have a matching
        // knowledge base entry (form-6, form-6a, form-7, form-8) or be an
        // explicitly-allowed non-KB form (form-2, form-12d) documented in
        // docs/04-decision-tree-spec.md as election-time / service-voter forms
        // outside the core Form 6/6A/7/8 knowledge base scope.
        const knownNonKbForms = new Set(["form-2", "form-12d"]);
        const hasKbEntry = getEntry(formId) !== undefined;
        expect(hasKbEntry || knownNonKbForms.has(formId)).toBe(true);
      });
    }
  }
});
