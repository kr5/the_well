# API Specification (Prose Overview)

A formal OpenAPI 3.x definition lives at
`/home/user/the_well/voteassist-india/openapi/voteassist-api.yaml` (not
authored in this document — this is the human-readable companion
describing intent and behavior). All endpoints are served under
`packages/api` and consumed primarily by `apps/web`, with future channels
(WhatsApp/Telegram bot, IVR) reusing the same API rather than reimplementing
decision logic — see `13-technical-architecture.md`.

## 1. Design Principles

- **Stateless where possible.** Session state for the decision engine is
  minimal (a session ID plus an ordered list of node/answer pairs); the
  API does not require authentication for core guidance features.
- **No PII required for core functionality.** Starting a session,
  answering questions, and getting a result never requires a name, email,
  phone number, or document identifier.
- **Every result payload includes citations.** Any endpoint returning a
  terminal outcome or KB content includes citation metadata inline, not
  as a separate lookup the client must remember to make.
- **Versioned decision logic.** Every session response includes the
  `decision_tree_version` it ran against, so results remain reproducible
  and auditable even as the tree evolves.

## 2. Endpoint Groups

### 2.1 Decision Engine Session

- `POST /v1/sessions` — Start a new decision-engine session. Request body
  may include `language` (defaults to `en`) and `client_platform`.
  Response includes a new `session_id`, the current
  `decision_tree_version`, and the root question node (id, question text,
  answer options).
- `POST /v1/sessions/{session_id}/answers` — Submit an answer to the
  current node. Request body: `{ node_id, answer_value }`, where
  `answer_value` must be one of the fixed options defined for that node
  (server-side validated against the decision-engine package, not
  client-trusted). Response: the next node (question/info) or, if a
  terminal has been reached, the terminal outcome payload directly.
- `GET /v1/sessions/{session_id}/result` — Retrieve the terminal outcome
  for a completed session, including `recommended_form`,
  `required_documents` (general categories), `citations` (array of
  `{source_type, source_title, source_url}`), `deep_link_target`, and the
  standard `verification_caution` string.
- `POST /v1/sessions/{session_id}/deep-link-click` — Records that the user
  clicked through to the recommended official destination (feeds the
  deep-link click-through metric in `18-analytics-plan.md`); fire-and-
  forget, no response body needed beyond a 204.

Sessions are ephemeral: the API does not require the client to have an
account, and a session with no account association is only retained per
the analytics retention window in `18-analytics-plan.md`.

### 2.2 Knowledge Base Search

- `GET /v1/kb/search?q=...&language=...&topic=...` — Full-text search
  across published KB entries (see `packages/search`), scoped by language
  and optional topic facet. Returns a ranked list of entry summaries
  (`id`, `slug`, `title`, `summary`, `topic`, `source_type`).
- `GET /v1/kb/entries/{slug}` — Fetch a full KB entry by slug, in the
  requested language (falls back to English with a flag indicating
  fallback occurred, per `11-multilingual-strategy.md`, if the requested
  language isn't published for that entry).
- `GET /v1/kb/glossary` — Returns the full glossary term list (a
  lightweight, cacheable endpoint since it changes infrequently).

### 2.3 Forms Lookup

- `GET /v1/forms` — List all forms (Form 6, 6A, 7, 8, 12D) with plain-
  language summaries and citations.
- `GET /v1/forms/{code}` — Full detail for a single form, including
  `superseded_forms` (e.g., Form 8 lists `8A` and `001` as superseded), and
  any linked KB entries.

### 2.4 Constituency / Polling Station Lookup

- `GET /v1/constituencies?state=...&type=AC|PC&q=...` — Search
  constituencies by state and name/number, for UI autocomplete purposes
  (e.g., letting a user select their AC in a question node where relevant).
- `GET /v1/locate/deep-links?state=...` — Returns the appropriate deep-link
  targets for polling station/BLO/ERO/CEO lookup for a given state (mostly
  a content-backed lookup table, not a live polling-station database in
  MVP — see `05-information-architecture.md` section 6 and the open
  question in `01-prd.md` about whether a legitimate ECI data feed exists
  for this).

### 2.5 Feedback

- `POST /v1/feedback` — Submit feedback. Request body:
  `{ category, message, kb_entry_id?, decision_session_id?, contact_email? }`.
  `contact_email` is optional and only stored if provided; no other
  identifying data is captured automatically (no IP-to-identity linkage
  beyond standard abuse-prevention rate limiting, see
  `16-security-threat-model.md`).

## 3. Error Handling Conventions

- Standard HTTP status codes; validation errors return a structured body
  identifying the offending field, never leaking internal state.
- Rate limiting applied per-IP on session creation and feedback submission
  to mitigate abuse (see `16-security-threat-model.md` denial-of-service
  section), with generous limits tuned to not penalize legitimate shared-IP
  usage patterns (e.g., public library/cyber-cafe access, common in the
  target user base).

## 4. Non-Goals for This API

- No endpoint submits any official ECI form on a user's behalf.
- No endpoint accepts document uploads (images, PDFs) in MVP.
- No endpoint requires or stores Aadhaar, EPIC, or passport numbers.
- No endpoint exposes any political/party-affiliation data, because none
  is ever collected.

See `/home/user/the_well/voteassist-india/openapi/voteassist-api.yaml` for
the formal schema-level contract (request/response bodies, status codes)
once authored.
