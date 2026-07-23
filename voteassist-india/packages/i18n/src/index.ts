export * from "./locales";

/** A record of at least an English string, keyed by BCP-47-ish locale code. */
export interface Localized {
  en: string;
  [locale: string]: string | undefined;
}

/** Resolve localized text with a guaranteed fallback to English. */
export function pick(text: Localized, locale: string): string {
  return text[locale] ?? text.en;
}
