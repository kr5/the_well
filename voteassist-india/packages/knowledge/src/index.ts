import { knowledgeEntries } from "./entries";
import type { KnowledgeEntry, KnowledgeTopic } from "./types";

export type { KnowledgeEntry, KnowledgeSource, KnowledgeTopic, KnowledgeSourceType, KnowledgeReviewStatus } from "./types";
export { knowledgeEntries };

const byId = new Map<string, KnowledgeEntry>(knowledgeEntries.map((e) => [e.id, e]));

export function getEntry(id: string): KnowledgeEntry | undefined {
  return byId.get(id);
}

export function requireEntry(id: string): KnowledgeEntry {
  const entry = byId.get(id);
  if (!entry) throw new Error(`Unknown knowledge base entry id "${id}"`);
  return entry;
}

export function listByTopic(topic: KnowledgeTopic): KnowledgeEntry[] {
  return knowledgeEntries.filter((e) => e.topic === topic);
}

/**
 * Deliberately simple substring search over title/summary/body — good enough
 * for the MVP knowledge browser. A real search index (OpenSearch, per the
 * technical architecture doc) is a v1+ concern once content volume grows.
 */
export function searchEntries(query: string): KnowledgeEntry[] {
  const q = query.trim().toLowerCase();
  if (!q) return [];
  return knowledgeEntries.filter(
    (e) =>
      e.title.toLowerCase().includes(q) ||
      e.summary.toLowerCase().includes(q) ||
      e.body.toLowerCase().includes(q) ||
      e.relatedEntities.some((entity) => entity.toLowerCase().includes(q))
  );
}
