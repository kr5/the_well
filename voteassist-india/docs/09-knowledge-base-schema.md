# Knowledge Base Entry Schema

JSON Schema for a single knowledge-base entry, as consumed by
`packages/knowledge` and rendered by `apps/web` under `/learn`. This is the
canonical shape; the `knowledge_base_entries` SQL table in
`08-database-schema.md` is the storage-layer projection of the same
fields.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://voteassist.example/schemas/knowledge-base-entry.json",
  "title": "KnowledgeBaseEntry",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "id",
    "topic",
    "title",
    "summary",
    "body",
    "applicable_states",
    "source_type",
    "source_url",
    "source_title",
    "last_verified_date",
    "version",
    "language",
    "review_status"
  ],
  "properties": {
    "id": {
      "type": "string",
      "format": "uuid",
      "description": "Stable unique identifier, never reused even if an entry is retired."
    },
    "topic": {
      "type": "string",
      "description": "Controlled-vocabulary topic tag used for KB browsing and search facets.",
      "enum": [
        "new_registration",
        "overseas_registration",
        "shifting_residence",
        "correction_of_entries",
        "epic_replacement",
        "e_epic",
        "objection_deletion",
        "pwd_marking",
        "home_voting",
        "service_voters",
        "qualifying_dates",
        "student_ordinary_residence",
        "roll_search",
        "polling_station_locator",
        "glossary",
        "general_faq"
      ]
    },
    "title": {
      "type": "string",
      "maxLength": 140,
      "description": "Plain-language title shown in KB listings and page headers."
    },
    "summary": {
      "type": "string",
      "maxLength": 400,
      "description": "One- to two-sentence summary shown in search results and cards."
    },
    "body": {
      "type": "string",
      "description": "Full entry content in Markdown. Must not contain fabricated specifics beyond what source_url supports; uncertain claims must be flagged inline as 'verify against [source]'."
    },
    "applicable_states": {
      "type": "array",
      "description": "List of state codes this entry applies to, or the literal string \"all\" if nationally applicable.",
      "oneOf": [
        { "type": "array", "items": { "type": "string" }, "minItems": 1 },
        { "const": "all" }
      ]
    },
    "source_type": {
      "type": "string",
      "enum": [
        "eci_official",
        "state_ceo",
        "gazette_law",
        "sveep",
        "community_pending_verification"
      ],
      "description": "Classifies the authority behind this entry's claims. 'community_pending_verification' entries must be visually flagged in the UI (see 07-design-system.md citation badge) and are excluded from certain terminal-outcome contexts until upgraded."
    },
    "source_url": {
      "type": "string",
      "format": "uri",
      "description": "Canonical URL of the official source. Required even for community_pending_verification entries if any source exists (e.g., a forum or news report) - label accordingly."
    },
    "source_title": {
      "type": "string",
      "description": "Human-readable name of the source document/page, e.g. 'Registration of Electors (Amendment) Rules, 2022 - Gazette Notification'."
    },
    "last_verified_date": {
      "type": "string",
      "format": "date",
      "description": "Date a human reviewer last confirmed this entry's content still matches the live source. Drives the review-cadence process in 06-legal-compliance-review.md."
    },
    "version": {
      "type": "integer",
      "minimum": 1,
      "description": "Monotonically increasing version number for this entry; incremented on any substantive content change."
    },
    "related_forms": {
      "type": "array",
      "items": {
        "type": "string",
        "enum": ["FORM_6", "FORM_6A", "FORM_7", "FORM_8", "FORM_12D"]
      },
      "description": "Form codes referenced by this entry, matching forms.code in the SQL schema."
    },
    "related_entities": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Free-form tags for related concepts/entities, e.g. ['BLO', 'ERO', 'qualifying_date'], used for cross-linking and search."
    },
    "language": {
      "type": "string",
      "description": "BCP-47 language code of this entry's content, e.g. 'en', 'hi'. Each language is a separate entry sharing a common translation_group_id (see below), not a field on one multilingual object.",
      "pattern": "^[a-z]{2,3}(-[A-Za-z0-9]+)*$"
    },
    "translation_group_id": {
      "type": "string",
      "format": "uuid",
      "description": "Shared identifier linking this entry to its translations in other languages. See 11-multilingual-strategy.md for the translation workflow."
    },
    "reading_level_variant": {
      "type": "string",
      "enum": ["standard", "easy"],
      "default": "standard",
      "description": "Whether this is the standard entry or an 'Easy/simple' reading-level variant of the same content."
    },
    "review_status": {
      "type": "string",
      "enum": ["draft", "in_review", "published", "needs_reverification", "retired"],
      "description": "Workflow state. Only 'published' entries render in production. 'needs_reverification' is set automatically when last_verified_date exceeds the review cadence threshold (06-legal-compliance-review.md) or when a related citation is flagged as changed."
    }
  }
}
```

## Field Notes

- **`id` vs `translation_group_id`**: each language variant of a given
  piece of content is its own entry with its own `id`, `version`, and
  `review_status` (a Hindi translation can be `in_review` while the
  English original is `published`), but they share a
  `translation_group_id` so the app can offer "view this in English"
  fallback when a translation isn't yet published.
- **`source_type: community_pending_verification`**: entries of this type
  must never be surfaced as a terminal-outcome recommendation in the
  decision engine (see `04-decision-tree-spec.md` section 3) — they are
  for KB-browsing context only, clearly labeled, until upgraded to an
  official source type through the review process in
  `06-legal-compliance-review.md`.
- **`applicable_states`**: most MVP entries will be `"all"` since the
  covered forms (6, 6A, 7, 8, 12D) are nationally defined by the RP Act/
  Rules; state-specific entries (e.g., a CEO portal URL, a state-specific
  procedural nuance) use the array form.
- **No field in this schema stores any user-identifying or
  sensitive-category data** — this schema describes content, not user
  data; see `08-database-schema.md` for the user-data-minimization
  constraints on the storage layer.
