# Product Requirements Document: VoteAssist India

Status: Draft v0.1 (MVP planning)
Owner: VoteAssist India core team (open source, unaffiliated with ECI/GoI)
Last updated: 2026-07-23

## 1. Mission

VoteAssist India helps every eligible Indian citizen understand exactly what
they need to do to register, correct, or manage their voter status, and then
deep-links them to the **official** Election Commission of India (ECI)
portals and services to actually perform that action:

- voters.eci.gov.in (unified elector services portal, supersedes/merges NVSP)
- ECINET (ecinet.eci.gov.in)
- Voter Helpline mobile app (Android/iOS)
- State CEO (Chief Electoral Officer) portals
- National Voter Helpline 1800-11-1950 ("1950")

VoteAssist India is a **guidance layer**. It translates the Registration of
Electors Rules 1960 (as amended 2022), the Representation of the People Acts
1950/1951, and ECI circulars into plain-language decision trees, checklists,
and explanations, and then hands the user off to ECI's own systems to
actually submit anything. It complements the ERONET/voters.eci.gov.in
ecosystem; it does not replace it.

## 2. Non-Goals (hard constraints)

These are load-bearing product constraints, not aspirational values. Every
feature must be checked against them before shipping:

1. **Not an official portal.** VoteAssist India must never claim, imply, or
   visually suggest that it is operated by, endorsed by, or affiliated with
   the Election Commission of India, any State CEO, or the Government of
   India. No use of the ECI emblem, the State Emblem of India, or any
   government logo. No visual mimicry of eci.gov.in / voters.eci.gov.in.
2. **No on-behalf submission.** VoteAssist India never submits Form 6, 6A,
   7, 8, or 12D (or any other official form) on a user's behalf, and never
   stores completed official forms. It deep-links the user to the place
   where they submit the form themselves, under their own credentials.
3. **No document custody by default.** VoteAssist India does not collect or
   store official identity documents (Aadhaar, EPIC, passport, birth
   certificate, address proof, disability certificate, etc.). If a future
   feature requires temporary access to a document (e.g., client-side OCR to
   pre-fill a checklist), it must be processed client-side or
   encrypted-in-transit-and-at-rest with a strict TTL, and never persisted
   beyond that TTL without explicit, specific consent. See
   `08-database-schema.md` and `16-security-threat-model.md`.
4. **Strict political neutrality.** No content may name, favor, criticize,
   or imply a preference for any political party, candidate, coalition, or
   ideology. No content may encourage or discourage voting for any outcome.
   Guidance is limited to procedural/administrative matters: "how do I
   register," never "who should I vote for" or "why voting matters for
   cause X." See `06-legal-compliance-review.md` for Model Code of Conduct
   handling during election periods.
5. **Citations or explicit uncertainty labels, always.** Every substantive
   procedural or legal claim in the product (in the decision engine, the
   knowledge base, or the UI) must either (a) carry a citation to an
   official source (ECI, SVEEP, CEO portal, Gazette, RP Act/Rules, PIB), or
   (b) be visibly labeled "community guidance — pending verification." No
   claim ships bare. See `09-knowledge-base-schema.md` and
   `06-legal-compliance-review.md`.

## 3. Problem Statement

India's electoral roll and elector-services process is legally sound and
increasingly digitized (e-EPIC, ECINET, Voter Helpline app, unified
voters.eci.gov.in portal), but the *entry point problem* persists: a citizen
who has moved cities, turned 18, lost their EPIC, or wants to correct a
misspelled name typically does not know:

- Which of the four ECI forms (6, 6A, 7, 8) applies to their situation
  (post-2022 consolidation collapsed many older forms — 8A and 001 no
  longer exist as separate forms — and this change is not widely known).
- What documents are generally expected.
- Whether they are even eligible yet (qualifying dates), or already
  registered somewhere else.
- Where to physically or digitally complete the action.

This produces avoidable drop-off: people give up, use outdated information,
or rely on unreliable word-of-mouth. VoteAssist India closes the "which form,
what do I do first" gap with a short, plain-language decision flow, then
gets out of the way.

## 4. Target Users / Personas (summary — see `02-personas.md` for detail)

VoteAssist India must serve, at minimum, the following situations described
in the original product brief:

- First-time voters (18th birthday just passed or upcoming)
- Students living in hostels/PG away from their native place
- Migrant workers who have moved for work
- Hostel/PG residents generally (non-student)
- Renters who move frequently
- People changing city or state permanently
- Married women changing residence after marriage
- Government/private employees transferred to a new posting
- NRI (overseas) voters registering for the first time
- Service voters (defence forces, government employees posted abroad, and
  their spouses, per RP Act provisions — verify exact category definitions
  against ECI's service voter rules before shipping copy)
- Persons with Disabilities (PwD), including home-voting eligibility
- Senior citizens (including the 85+ postal ballot / home voting option)
- Tribal and remote-area residents
- Urban slum residents
- Homeless citizens, where legally able to register (verify current ECI
  guidance on proof-of-ordinary-residence for the homeless before shipping
  any specific claim)
- Transgender citizens
- Citizens lacking Aadhaar, passport, driving licence, or a fixed permanent
  address
- People with a name/DOB mismatch across documents
- People whose voter ID was duplicated or wrongly deleted
- People whose polling station has changed since they last voted
- People with a missing, lost, or damaged EPIC
- New 18-year-olds and citizens who will become eligible on a future
  qualifying date

## 5. Success Metrics

Metrics must be privacy-preserving and must never measure or infer political
inclination. Acceptable metrics (aggregate only; see `18-analytics-plan.md`):

| Metric | Definition | Why it matters |
|---|---|---|
| Task completion rate | % of decision-engine sessions that reach a terminal outcome (a specific form/action recommendation) | Measures whether the flow actually resolves the user's situation |
| Time-to-clarity | Median wall-clock time from session start to terminal outcome | Measures whether guidance is fast, not just eventually correct |
| Deep-link click-through rate | % of terminal outcomes where the user clicks through to the cited official ECI/CEO resource | Measures whether we successfully hand off, rather than becoming a dead end or a substitute |
| Drop-off point (aggregate, per question node) | Which decision-tree questions have the highest abandonment | Identifies confusing questions/wording, not user identity |
| Knowledge base search success rate | % of KB searches followed by a KB entry view | Measures search relevance |
| Feedback-flagged inaccuracy rate | Count of "this info seems wrong" reports per KB entry, per period | Feeds the content review process (see `06-legal-compliance-review.md`) |

Explicitly excluded metrics: anything resembling political segmentation,
voting-intent inference, party/candidate sentiment, caste/religion/community
inference, or individual-level tracking across sessions without consent.

## 6. Legal / Ethical Guardrails (see `06-legal-compliance-review.md` for full treatment)

- Disclaimer banner on every screen: "VoteAssist India is an independent,
  non-official guidance tool. It is not affiliated with the Election
  Commission of India. For official action, use voters.eci.gov.in, ECINET,
  the Voter Helpline app, or call 1950."
- No collection of caste, religion, party affiliation, or political opinion
  data, ever, under any feature.
- Data minimization per the Digital Personal Data Protection Act 2023:
  collect only what's needed to run the decision engine session; no
  document retention by default.
- During election periods (as defined by ECI's Model Code of Conduct
  timeline for a given state/national election), the product must suspend
  any feature that could be construed as election-related campaigning or
  outreach beyond neutral procedural guidance — this is a compliance gate,
  not a feature, and needs a named compliance owner (see
  `06-legal-compliance-review.md`).

## 7. MVP Scope

MVP (this build phase) includes:

- Web app only (Next.js), English + Hindi, fully human-reviewed content.
- Decision engine covering the scenarios enumerated in
  `04-decision-tree-spec.md`: unknown registration status, new elector
  registration (Form 6), NRI registration (Form 6A), moved residence within
  or across AC (Form 8), correction of entries (Form 8), lost/damaged EPIC
  (Form 8 replacement or e-EPIC download), objection/deletion (Form 7), PwD
  marking + home voting (Form 8 + Form 12D), and constituency/polling
  station/BLO lookup guidance.
- Knowledge base of curated, cited entries for the above scenarios.
- Deep links to voters.eci.gov.in, ECINET, Voter Helpline app store
  listings, CEO portals (generic pattern; state-specific URLs are a v1
  content task), and the 1950 helpline.
- Basic accessibility (WCAG 2.2 AA target — see `12-accessibility-spec.md`).
- Aggregate, privacy-preserving analytics only.

## 8. Out of Scope for MVP

- Any language beyond English and Hindi (roadmap: see
  `11-multilingual-strategy.md`).
- WhatsApp/Telegram bot, IVR, voice assistant (roadmap: see
  `19-roadmap.md`).
- Offline-first PWA behavior (v1).
- State-by-state legal completeness — MVP decision tree is explicitly a v1
  subset pending state-level legal review (see `04-decision-tree-spec.md`
  and `06-legal-compliance-review.md`).
- Document upload/OCR of any kind.
- User accounts / authentication beyond an anonymous session.
- Sign-language video content (v3 roadmap).
- Full "every legal scenario" coverage (e.g., every service-voter category,
  every state's special provisions for tribal constituencies) — flagged
  explicitly in-product as "not yet covered, contact your ERO/BLO or call
  1950" rather than guessed at.

## 9. Open Questions

- Exact wording ECI/CEO portals require for third-party deep-linking
  disclaimers (needs direct verification, not assumption).
- Whether any ECI API/data feed exists for polling station lookup that
  VoteAssist could legitimately consume, versus always deep-linking to
  ECI's own locator tools (default assumption: deep-link only, until
  verified otherwise).
- State-specific service voter and tribal-constituency nuances requiring
  legal review before v1 (tracked in `19-roadmap.md`).
