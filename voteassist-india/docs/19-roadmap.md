# Roadmap

This roadmap sequences scope additions across MVP through v3. A recurring
theme across every phase: **election-law nuances differ by state**, and
full state-by-state legal review is treated as a hard prerequisite before
VoteAssist India claims completeness of coverage for any given state — not
just a v1 nicety. Until that review happens for a given state, in-product
content should say so plainly (e.g., "state-specific details for
[state] have not yet been legally reviewed — consult your CEO portal or
call 1950") rather than implying completeness it doesn't have.

## MVP (this build phase)

- Languages: English + Hindi, both fully human-reviewed.
- Decision engine: covers exactly the scenarios in
  `04-decision-tree-spec.md` (branches A-H) — new registration (Form 6),
  overseas registration (Form 6A), shifting of residence (Form 8),
  correction of entries (Form 8), EPIC replacement/e-EPIC (Form 8 / digital
  download), objection/deletion (Form 7), PwD marking + home voting (Form
  8 + Form 12D), and roll search / polling station locator guidance.
- Platform: web app only (`apps/web`, Next.js).
- Knowledge base: curated entries for the above scenarios only, each with
  a verified citation.
- Accessibility: WCAG 2.2 AA as the working target (full audit not
  necessarily complete by MVP ship, but the target/process is in place —
  see `12-accessibility-spec.md`).
- Analytics: aggregate-only, per `18-analytics-plan.md`.
- Explicitly NOT in MVP: any other language, WhatsApp/Telegram/IVR, PWA
  offline mode, document upload/OCR, user accounts, state-by-state legal
  completeness claims.

## v1

- Accessibility: formal WCAG 2.2 AA verification/certification pass
  (automated + manual, per `12-accessibility-spec.md` section 4), not just
  a design-time target.
- Decision-tree scope: all major scenarios enumerated in the original
  product brief and reflected in `02-personas.md` (e.g., service voters,
  transgender-specific correction flows, tribal/remote-area guidance,
  homeless-elector guidance where legally applicable) — expanding beyond
  the v1-subset branches A-H in `04-decision-tree-spec.md`.
- Languages: 5-6 languages total (English, Hindi, plus roadmap picks from
  the Eighth Schedule list based on speaker population and community
  translator availability — exact selection is a v1-planning decision, not
  fixed here).
- Platform: PWA offline support (installable, functions with no
  connection for previously-loaded KB content, syncs analytics/feedback
  when reconnected) — see `12-accessibility-spec.md` section 2.7.
- **Prerequisite for claiming any state's coverage as "complete":**
  state-by-state legal review (documents/procedures that vary by state,
  CEO portal specifics, any state-specific forms or provisions) must be
  completed and signed off per `06-legal-compliance-review.md` before
  that state's content is marked as reviewed/complete in the knowledge
  base. This applies incrementally, state by state, not as an all-or-
  nothing gate on the whole v1 release.

## v2

- Channels: WhatsApp bot and Telegram bot, both built as thin adapters
  over the same `packages/decision-engine` and `packages/api` used by the
  web app (see `13-technical-architecture.md`) — no reimplementation of
  decision logic.
- Open API for NGOs: a documented, rate-limited public API (extending
  `10-api-specification.md`) allowing partner civic-tech NGOs and campus
  groups to embed VoteAssist guidance in their own tools/sites, under the
  same non-affiliation and citation constraints.
- Languages: continued expansion beyond the v1 5-6 language set, prioritized
  by usage data (`18-analytics-plan.md` language-usage metric) and
  community translator availability (`11-multilingual-strategy.md`).
- Continued state-by-state legal review for any states not yet covered
  in v1.

## v3

- Channels: IVR / voice assistant, building on the multilingual voice-mode
  groundwork from `11-multilingual-strategy.md` and the DTMF/voice-menu-
  friendly question design noted in `13-technical-architecture.md`.
- Campus toolkit: a packaged set of materials/embeds for student
  organizations to run registration-awareness drives on campuses,
  specifically supporting the student "ordinary residence" scenario in
  `04-decision-tree-spec.md` and `02-personas.md` (Priya persona).
- Versioned knowledge-graph: a structured system tracking how ECI
  circulars/rules change over time (not just a flat "last verified date"
  per entry, but a linked history of what changed, when, and which past
  guidance it superseded) — supports both compliance auditability and
  potentially a public "how election administration has evolved" resource.
- Full Eighth Schedule language coverage (all 22 scheduled languages plus
  English), each having passed the same review gate as English/Hindi in
  MVP.
- Sign-language (ISL) video content for key flows, per the accessibility
  roadmap item in `12-accessibility-spec.md` section 3.
- By this point, the state-by-state legal review prerequisite should have
  been completed for all states/UTs; any state still outstanding is
  explicitly flagged in-product rather than silently assumed covered.

## Cross-Cutting Commitment

At every phase, expanding scope (new language, new scenario, new state, new
channel) is gated by the same non-negotiables from `01-prd.md`: no
official-portal impersonation, no on-behalf submission, no political
content, and no claim without a citation or an explicit
"pending-verification" label. Scope expands; those constraints do not
loosen.
