import { describe, expect, it } from "vitest";
import { pick, shippedLocales, supportedLocales, uiStrings } from "../src/index";

describe("i18n completeness", () => {
  it("every shipped locale has the exact same UI string keys as English (no missing/extra keys)", () => {
    const enKeys = Object.keys(uiStrings.en).sort();
    for (const locale of shippedLocales) {
      const dict = (uiStrings as Record<string, Record<string, string>>)[locale.code];
      expect(dict, `missing uiStrings dictionary for shipped locale "${locale.code}"`).toBeTruthy();
      expect(Object.keys(dict).sort()).toEqual(enKeys);
      for (const key of enKeys) {
        expect(dict[key]?.length, `empty string for "${key}" in "${locale.code}"`).toBeGreaterThan(0);
      }
    }
  });

  it("every planned (not yet shipped) locale is listed with a native name, for the language switcher roadmap UI", () => {
    for (const locale of supportedLocales) {
      expect(locale.nativeName.length).toBeGreaterThan(0);
      expect(locale.englishName.length).toBeGreaterThan(0);
    }
  });

  it("pick() falls back to English when a translation is missing", () => {
    expect(pick({ en: "Hello" }, "fr")).toBe("Hello");
    expect(pick({ en: "Hello", hi: "नमस्ते" }, "hi")).toBe("नमस्ते");
  });
});
