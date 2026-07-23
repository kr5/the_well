export interface KnowledgeSource {
  title: string;
  url: string;
  publisher?: string;
}

export type KnowledgeTopic =
  | "registration"
  | "correction"
  | "shifting-of-residence"
  | "deletion-objection"
  | "epic"
  | "ordinary-residence"
  | "nri-voter"
  | "service-voter"
  | "pwd-voter"
  | "qualifying-dates"
  | "grievance"
  | "polling-station"
  | "roll-search"
  | "glossary";

export type KnowledgeSourceType =
  | "eci_official"
  | "state_ceo"
  | "gazette_law"
  | "sveep"
  | "pib_release"
  | "community_pending_verification";

export type KnowledgeReviewStatus = "draft" | "in_review" | "verified" | "needs_reverification";

export interface KnowledgeEntry {
  id: string;
  topic: KnowledgeTopic;
  title: string;
  summary: string;
  body: string;
  applicableStates: string[];
  sourceType: KnowledgeSourceType;
  sources: KnowledgeSource[];
  lastVerifiedDate: string;
  version: number;
  relatedForms: string[];
  relatedEntities: string[];
  language: string;
  reviewStatus: KnowledgeReviewStatus;
  caution?: string;
}
