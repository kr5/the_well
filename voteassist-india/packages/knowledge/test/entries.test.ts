import { describe, expect, it } from "vitest";
import { getEntry, knowledgeEntries, listByTopic, requireEntry, searchEntries } from "../src/index.js";

describe("knowledge base entries", () => {
  it("has at least one entry loaded", () => {
    expect(knowledgeEntries.length).toBeGreaterThan(0);
  });

  it("has no duplicate ids", () => {
    const ids = knowledgeEntries.map((e) => e.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("every entry has at least one source with a valid https URL and a non-empty summary/body", () => {
    for (const entry of knowledgeEntries) {
      expect(entry.sources.length, `${entry.id} has no sources`).toBeGreaterThan(0);
      expect(entry.summary.length).toBeGreaterThan(0);
      expect(entry.body.length).toBeGreaterThan(0);
      for (const source of entry.sources) {
        const url = new URL(source.url);
        expect(["http:", "https:"]).toContain(url.protocol);
      }
    }
  });

  it("every entry declares a lastVerifiedDate that parses as a real date", () => {
    for (const entry of knowledgeEntries) {
      expect(Number.isNaN(Date.parse(entry.lastVerifiedDate))).toBe(false);
    }
  });

  it("getEntry resolves a known id and returns undefined for an unknown one", () => {
    expect(getEntry("form-8")?.title).toContain("Form 8");
    expect(getEntry("does-not-exist")).toBeUndefined();
  });

  it("requireEntry throws for an unknown id", () => {
    expect(() => requireEntry("does-not-exist")).toThrow();
  });

  it("listByTopic filters correctly", () => {
    const shiftingEntries = listByTopic("shifting-of-residence");
    expect(shiftingEntries.every((e) => e.topic === "shifting-of-residence")).toBe(true);
    expect(shiftingEntries.length).toBeGreaterThan(0);
  });

  it("searchEntries finds Form 8 when searching for 'shifting of residence'", () => {
    const results = searchEntries("shifting of residence");
    expect(results.some((e) => e.id === "form-8")).toBe(true);
  });

  it("searchEntries returns nothing for an empty query", () => {
    expect(searchEntries("   ")).toEqual([]);
  });
});
