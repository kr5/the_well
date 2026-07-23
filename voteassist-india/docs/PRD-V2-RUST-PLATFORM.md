# VoteAssist India — Master PRD v2 (Rust Production Platform)

**Status:** Draft v2.0 — architecture & scope specification, pending team/legal review before implementation begins.
**Supersedes:** the technology-stack assumptions in `docs/01-prd.md` through `docs/19-roadmap.md` (v1). The v1 documents' mission, non-goals, personas, and legal/accessibility framing remain valid and are cross-referenced, not restated in full, throughout this document. What v1 got right about *what* to build stands; this document is about *how* to build it at production grade, on a Rust-based stack, with far deeper ECI research, a full admin/content-operations surface, and an exhaustive task backlog.
**Relationship to the existing codebase:** an MVP already exists in this repo (`apps/web`, `packages/*`, `knowledge-base/`) — a working, tested TypeScript/Next.js prototype covering ~21 decision-tree nodes in English and Hindi. It is retained, not deleted, during the migration described here (Section 6.17).
**Scope of this document:** research findings, technical architecture, database/knowledge-base schemas, the expanded decision-engine specification, the full admin-pages specification, the analytics specification, the security threat model, the complete feature/task backlog, testing/CI/i18n/accessibility/legal specifications, and a phased roadmap.
**Last updated:** 2026-07-23

## Table of Contents

0. Document control (above)
1. Executive Summary & Strategic Rationale for the Rust Rewrite
2. ECI Systems & Legal Landscape — Deep-Dive Research
3. Existing Open-Source / Civic-Tech Landscape — What to Reuse, What to Avoid
4. Product Vision & Non-Goals (v2 restatement)
5. Personas (cross-reference)
6. Technical Architecture — Rust Platform
7. Repos & Libraries Reference Table
8. Database Schema (Postgres 16, via sqlx)
9. Knowledge Base v2 Schema (content + i18n)
10. Decision Engine v2 — Expanded Specification
11. Admin Pages Specification
12. Analytics Specification v2
13. Security Threat Model v2
14. Full Feature & Task Backlog
15. Testing & QA Plan v2
16. CI/CD Pipeline v2
17. Multilingual Strategy v2
18. Accessibility Specification v2
19. Legal & Compliance v2
20. Analytics / Privacy Data Retention Policy (detailed)
21. Roadmap v3 (MVP-Rust-v1 → v1 → v2 → v3)
22. Open Questions & Risks
23. Appendix: Glossary & Citation Cross-Reference

---
## 1. Executive Summary & Strategic Rationale for the Rust Rewrite

### 1.1 Mission, restated

VoteAssist India is an open-source, politically-neutral guidance layer that helps an eligible Indian citizen figure out which of a small number of official Election Commission of India (ECI) actions applies to their situation — new registration, correction, shifting of residence, deletion of a stale entry, overseas/service-voter registration, PwD marking, home voting — and then deep-links them to the official system (voters.eci.gov.in, ECINET, the Voter Helpline app, a State CEO portal, or the 1950 helpline) to actually do it. It never submits a form on a user's behalf, never claims official status, never stores official ID documents by default, and never takes a political position of any kind — these are hard non-goals, not aspirational values, and every feature in this PRD is checked against them.

The product does one narrow job well: turn "I don't know which form applies to me" into "here is the exact form, here is roughly what you'll need, here is where to submit it" — in under a few minutes, in the user's own language, and on a low-end phone with a weak connection. Everything else — candidate information, election results commentary, "why voting matters," document custody, on-behalf submission — is explicitly out of scope, both because it isn't the problem VoteAssist exists to solve and because straying into it would compromise the neutrality and non-official-status guarantees the mission depends on.

This PRD (v2) does not redefine the product. The MVP already validated the concept: a working decision engine, a curated and cited knowledge base, English + Hindi content, and an accessible web app. What changes here is how it's built for production.

### 1.2 Why Rust, and why now

The MVP was built as a TypeScript/Next.js prototype, and it did its job — it proved the decision-tree model works, that citation-backed content review is tractable, and that the personas in `02-personas.md` map cleanly onto the ECI form set. But a prototype optimized for iteration speed is not the same thing as a system a nonprofit, volunteer-and-donation-funded project should run in production for the Indian public. Four factors specifically push this rebuild toward Rust rather than hardening the existing Next.js stack in place:

**Traffic-spike resilience around election announcements.** VoteAssist's usage is not steady-state; it is spiky and correlated with ECI's own announcement calendar. The moment an election schedule is announced (the same moment the Model Code of Conduct comes into force, see Section 2.6), search interest in "how do I register," "what form do I need," and "check my polling station" spikes nationally and simultaneously. Concretely: on an ordinary day the product might see a modest, geographically diffuse trickle of sessions; on the day a state or general election schedule is announced, that trickle becomes a concentrated surge — the same news cycle that puts the announcement on every citizen's phone also sends a large fraction of first-time and lapsed voters looking for exactly the guidance VoteAssist exists to give, all within the same few hours, and disproportionately over mobile data rather than broadband. A public-facing civic tool that falls over at exactly the moment it is most needed is worse than not existing — it actively erodes the trust the project depends on, at the one moment that trust matters most. Rust's combination of no garbage-collector pause spikes, low per-request memory overhead, and predictable tail latency under load (particularly relevant to the Axum/Tokio async runtime chosen for `crates/api`) gives materially more headroom per unit of hosting spend than a Node.js event loop under the same load, without needing to over-provision "just in case" infrastructure the project can't afford to keep running idle for the 350-odd non-spike days of the electoral cycle.

**Single low-cost binary deployment for a nonprofit budget.** VoteAssist has no advertising revenue and no user-data monetization path (both are structurally excluded by the mission). Its hosting cost has to stay low indefinitely, on donated or grant-funded infrastructure, and ideally be deployable by a small volunteer ops team without a dedicated SRE function. A compiled Rust service produces a single statically-ish linked binary per crate (api, web-app, admin-app, bot-telegram, jobs-worker) that runs comfortably in a distroless container on the cheapest available compute, with no `node_modules` tree, no runtime dependency resolution, and a small, auditable attack surface. This is a genuine operating-cost argument, not an aesthetic one: it is the difference between a project that can be kept alive by donated compute for a decade and one that quietly needs a bigger server every election cycle.

**Correctness guarantees for a decision engine that must never give wrong guidance.** The core of VoteAssist is a decision tree that must satisfy hard invariants: every path terminates, every terminal node carries a citation or an explicit "pending verification" label, no path ever asks for a sensitive category (caste, religion, party preference). In the TS prototype these invariants were checked with an exhaustive-path Vitest test. `crates/core-domain` supersedes that with `proptest`-based property testing, which generates and checks orders of magnitude more input combinations than a hand-enumerated exhaustive test practically can, and does so as a first-class, statically-typed part of the same language the engine is written in — not a separate test-only DSL bolted on top. For a system whose worst failure mode is "confidently tells a citizen the wrong form," that additional testing depth is the point of the rewrite, not a nice-to-have.

**One core-domain crate, reused everywhere.** The product's roadmap explicitly includes Telegram (`crates/bot-telegram`), WhatsApp (`crates/bot-whatsapp`), and eventually IVR (`crates/ivr-gateway`) as delivery channels for the same decision engine, because a large share of the target personas (migrant workers, elderly citizens, low-connectivity tribal-block residents — see `02-personas.md` personas 3, 9, 13) are more reachable by messaging app or phone call than by browsing a web page. Rust makes "one decision-engine crate, several thin channel adapters" a natural architecture: `core-domain` has zero I/O and no channel-specific assumptions baked in, so the web app, the Telegram bot, the WhatsApp integration, and eventually the IVR gateway all call the identical, identically-tested logic. Reproducing that cleanly in a Node/TypeScript monorepo is possible but requires more discipline to keep genuinely decoupled; in Rust, the crate boundary and the type system enforce it.

**Honest tradeoff: contributor onboarding.** None of this is free. Rust has a materially steeper learning curve than the TypeScript prototype for a volunteer open-source contributor base — ownership/borrowing, the trait system, and async Rust's ecosystem churn are real barriers that TypeScript, Next.js, and React do not present to the same degree. This is why the TS prototype under `apps/web` (and the sibling `@voteassist/decision-engine` and `@voteassist/knowledge` packages) is retained in the repository as a working reference implementation during migration, not deleted: it remains the executable specification for what `core-domain` and `kb-content` must reproduce, it stays useful for onboarding contributors to the domain logic before they touch Rust syntax, and it is a fallback artifact if the Rust migration stalls. The project accepts a smaller pool of Rust-capable contributors in exchange for the correctness, cost, and reuse properties above; mitigating that tradeoff (documentation, a "good first issue" pipeline into `core-domain`, pairing) is itself a workstream this PRD tracks in the roadmap section of the larger document, not something assumed away here.

### 1.3 Rust vs. the TypeScript prototype, attribute by attribute

The table below is not a rhetorical device; it is the actual set of tradeoffs weighed before committing to the rewrite, and it should be revisited if any row's assumption changes materially (e.g., if a future Rust web framework closes the tooling-maturity gap, or if the contributor pool genuinely fails to grow).

| Attribute | TypeScript/Next.js prototype (retained as reference) | Rust production stack (this PRD) | Why it matters here |
|---|---|---|---|
| Runtime memory behavior under load | V8 + Node event loop; garbage-collector pause spikes possible under sustained concurrent load | No GC; ownership-based memory management with predictable, low per-request overhead | Directly affects tail latency during the election-announcement traffic spikes described in 1.2 |
| Deployment artifact | `node_modules` tree, Next.js build output, Node runtime required on the host | Single compiled binary per crate, distroless container image | Lower hosting cost and smaller attack surface for a nonprofit-funded deployment |
| Decision-engine correctness tooling | Vitest exhaustive-path test (hand-enumerated paths) | `proptest` property-based tests (generated input space, `cargo-nextest` runner) | Property tests catch combinations a hand-enumerated exhaustive test author didn't think to write down |
| Multi-channel reuse (web, Telegram, WhatsApp, IVR) | Requires disciplined package boundaries (`@voteassist/decision-engine`) to avoid channel-specific logic leaking in | Enforced at the crate boundary: `core-domain` has zero I/O by construction, so channel-specific code cannot compile its way into the domain crate | Reduces the risk of subtly different eligibility logic per channel, which would be a correctness bug with real civic consequences |
| Contributor onboarding curve | Familiar to a very large pool of web developers; low barrier to a first PR | Steeper: ownership/borrowing, async Rust ecosystem churn, trait-based APIs | Explicit tradeoff accepted in exchange for the rows above; mitigated by keeping the TS prototype as a living reference and specification |
| Observability / self-hosting story | Typical Node APM tooling, often assumes a SaaS backend | `tracing` + OpenTelemetry OTLP to a self-hosted Grafana/Loki/Tempo stack by default | Aligns with the data-sovereignty posture appropriate for a platform handling Indian citizens' (even anonymized) usage data |
| Browser-level end-to-end testing | Playwright, native to the Node ecosystem | Playwright retained deliberately as a polyglot boundary — Rust has no equivalently mature browser-e2e tool, and reinventing one would be wasted effort | Pragmatic exception to the "everything moves to Rust" narrative, called out explicitly rather than glossed over |

### 1.4 Migration posture

The Rust rewrite proceeds alongside, not instead of, the existing TS prototype for the duration of the migration. Two concrete posture commitments follow from this: first, `core-domain` and `kb-content` are built to reproduce the TS decision-engine and knowledge-base behavior exactly (same terminal outcomes, same citations, same tree topology) before any new functionality is added on the Rust side — the TS prototype is the spec, not a discardable rough draft. Second, the TS prototype under `apps/web` is not maintained as if it were still headed for production; it is frozen in a reference-only capacity once `core-domain`'s proptest suite reaches parity with the Vitest exhaustive-path test, at which point the Rust stack becomes the system of record and any new decision-tree scenario is authored directly against `core-domain`. The detailed phase plan (which crate ships first, what "parity" is measured against, cutover criteria) belongs in the Technical Architecture and Roadmap sections of this PRD, not repeated here — this section only establishes the posture that governs those plans.

### 1.5 Relationship to the existing MVP

To be unambiguous about scope: this is a **production rebuild plan**, not a from-scratch product redefinition. The MVP already exists in this repository, has already been built, tested, and pushed — a tested decision engine, a curated and cited knowledge base, i18n scaffolding, and a working web app, all in TypeScript/Next.js. Nothing in Sections 2 or 3 of this document, nor in the sibling sections of PRD v2 covering architecture, schema, or roadmap, revisits the product decisions already made in `docs/01-prd.md` through `docs/19-roadmap.md` — personas, the decision-tree scenario list, the non-goals, the legal posture, and the accessibility/multilingual targets all carry forward unchanged. What this PRD redefines is exclusively the *implementation stack* the product is rebuilt on for production traffic: Rust instead of TypeScript/Next.js for the runtime, per explicit direction to "use rust system," with the concrete crate-level architecture given in the companion Technical Architecture section of this PRD.

Where v1 documentation assumed a Next.js/TypeScript stack (most visibly in `docs/13-technical-architecture.md`, `docs/14-repository-structure.md`, and `docs/15-cicd.md`), this PRD v2 supersedes those tech-stack assumptions specifically; the underlying product requirements those documents were built to serve (the decision tree's scenario coverage, the knowledge base's citation discipline, the accessibility and multilingual targets, the legal/compliance guardrails) are unchanged and are restated where useful for continuity, not re-litigated.

---

## 2. ECI Systems & Legal Landscape — Deep-Dive Research

This section goes deeper than the v1 documentation pass on the institutional and technical landscape a production system must design around. VoteAssist does not integrate with any ECI backend system; it deep-links to public-facing ones and designs its own behavior around the constraints those systems and the surrounding legal framework impose. Every factual claim below is either drawn directly from the verified research established for this PRD, or explicitly flagged as requiring verification — no specifics are invented.

### 2.1 voters.eci.gov.in and ECINET — public portal vs. broader digital platform

**voters.eci.gov.in** is ECI's unified elector-services portal. It supersedes and merges the earlier National Voters' Service Portal (NVSP, formerly at nvsp.in) into a single public-facing surface for elector self-service: new registration and correction forms (6/6A/7/8), electoral roll search, e-EPIC download, and related citizen-facing functions. This is VoteAssist's primary deep-link target for essentially every terminal outcome in the decision tree — the product's job ends where this portal's job begins.

**ECINET** (ecinet.eci.gov.in) is described in ECI's own materials as a broader unified digital platform, not merely a rebrand of the elector-services portal. VoteAssist treats it as a secondary/complementary deep-link target alongside voters.eci.gov.in, in particular for functions such as "Book-a-Call with BLO" (Section 2.5). The precise functional boundary between what lives on voters.eci.gov.in versus what lives on ECINET (i.e., which specific citizen actions are hosted on which surface, and whether that allocation is stable or still consolidating) is TODO: verify against eci.gov.in / ECINET's own documentation before hardcoding specific deep-link URLs per action into shipped copy — the product should treat both as valid official endpoints and route to whichever is currently authoritative for a given action, re-verified on the standard content-review cadence.

Design implication: because both portals are legitimate official destinations and their relative scope may shift over time, `kb-content` entries store the deep-link target as a structured, versioned field per terminal outcome (not hardcoded into template prose), so a portal consolidation or URL change is a content update, not a code change.

### 2.2 ERONET — why VoteAssist's public-facing design looks the way it does

ERONET (launched 2018) is ECI's internal Electoral Roll Management System. It is critical to state plainly: **ERONET is an official-use system for election officials — Booth Level Officers, Electoral Registration Officers, Assistant Electoral Registration Officers — and VoteAssist does not integrate with it, does not call it, and has no access to it.** No feature in this PRD assumes any ERONET access, present or future.

Understanding ERONET matters anyway, because it explains *why* the public-facing systems VoteAssist does deep-link to behave the way they do:

- ERONET operates in 14 languages and 11 scripts and standardizes the processing of Forms 6/6A/7/8 for officials across 29 states and 7 Union Territories on shared infrastructure. This is the reason the public form set is genuinely standardized nationally (same four forms, same rules, same amendment) rather than varying by state in the way many other citizen-facing government processes do in India's federal structure — which is exactly why a single national decision tree (rather than 28+ state-specific trees) is a defensible architecture for VoteAssist's core engine, with state-specific deep-link *targets* (CEO portal URLs) layered on top rather than state-specific *logic*.
- ERONET's backing store is UNPER — the Unified National Photo Electoral Roll, the common national elector database, on the order of ~92 crore (920 million) electors. UNPER's existence as a single national source of truth is why a roll search or duplicate-detection outcome from voters.eci.gov.in can be trusted as authoritative regardless of where in India the user is searching from, and why VoteAssist's own copy can confidently say "the electoral roll" (singular, national) rather than hedging about which state's roll a user's data might be in.
- ERONET's role-based access and audit trails for officials are the institutional reason official processing has defined expectations at all (e.g., the pattern of forms being reviewed and either accepted, queried, or rejected by a named BLO/ERO with a traceable record) — which is precisely the audit-trail discipline VoteAssist's own admin/content-review workflow (Section 6 of `06-legal-compliance-review.md`; `reviewStatus` enum in the knowledge base schema) deliberately mirrors for its own KB content, on the theory that a civic-guidance product operating adjacent to an audited official system should hold itself to a comparable standard of traceability for its own claims.

### 2.3 Forms 6/6A/7/8/2/12D — full lifecycle table

Since the Registration of Electors (Amendment) Rules 2022 (Gazette notification 17 June 2022, in force 1 August 2022), the form set is consolidated to four core electoral-roll forms plus two special-purpose forms. Forms 8A and 001 were discontinued and merged into Form 8 — this consolidation is not widely known among citizens and is precisely the kind of gap VoteAssist exists to close.

| Form | Purpose | Who files it | What triggers it | Typical official processing expectation | VoteAssist personas mapped (from `02-personas.md`) |
|---|---|---|---|---|---|
| **Form 6** | New elector registration (ordinary residence) | The individual citizen, at the ERO for their constituency of ordinary residence | Turning 18 as of any qualifying date; never previously registered; moving and choosing to register fresh rather than "shift" an existing entry | TODO: verify against voters.eci.gov.in / ECI Form 6 instructions for current stated processing-time expectations — do not assert a specific number of days without a live source | Aarav (persona 1, first-time voter); Priya (persona 2, student choosing hostel or home address); Nasreen (persona 14, no standard address proof); Colonel Verma (persona 7, moving off service-voter status onto ordinary roll) |
| **Form 6A** | Overseas/NRI elector registration | The overseas citizen, retaining Indian citizenship, registering against the India address in their passport | An NRI who has never registered wants to register as an overseas elector | TODO: verify against voters.eci.gov.in overseas-elector guidance for current processing timelines | Ananya (persona 6) |
| **Form 7** | Objection to inclusion of a name / claim for deletion | Any elector or interested party, filed with the ERO | Suspected duplicate entry, entry for a deceased person, or other name that should not be on the roll | Notice-and-inquiry process before deletion (a Form 7 objection is not a self-executing deletion — the ERO investigates); TODO: verify against ECI's Form 7 procedure for current notice/inquiry timelines before asserting specifics | Farida (persona 11, deceased mother still listed + own duplicate entry); Colonel Verma (persona 7, possibly removing a stale service-voter entry, pending verification of whether this is automatic) |
| **Form 8** | Shifting of residence (same or different AC), correction of entries, EPIC replacement/duplicate, PwD marking | The individual elector, at the ERO for the new or corrected constituency | Moving address (within or across Assembly Constituency), a name/DOB/photo/gender error on the existing entry, a lost/damaged physical EPIC, or wanting PwD status marked on the roll | This is the single "catch-all maintenance" form post-2022 consolidation — the fact that four previously separate concerns (shift, correction, EPIC replacement, PwD marking) all now route through one form is itself the single highest-value piece of guidance VoteAssist can give, since it directly overturns the common (outdated) assumption that these need separate processes | Mohammed (persona 3, shifting AC); Kavita (persona 4, cross-AC shift + name correction after marriage); Rahul (persona 5, shifting for a posting); Deepa (persona 8, PwD marking step); Kamla Devi (persona 9, EPIC replacement); Suresh (persona 10, name correction); Noor (persona 12, gender-marker correction + address shift); Birsa (persona 13, correction after a boundary change) |
| **Form 2** | Service elector registration (Armed Forces / forces to which the Army Act applies) | The service member (or, for a "Classified Service Voter," their nominated proxy arrangement), via servicevoter.nic.in | Enlistment in a qualifying service category | Requires a declaration that the applicant is not also enrolled as an ordinary elector elsewhere (prevents double-registration across the service/ordinary roll boundary) | Not directly mapped to a named persona in the current v1 set (Colonel Verma is post-service, i.e., transitioning *off* Form 2 status, not filing it) — flagged as a persona-coverage gap worth considering for a future persona addition given the size of India's service-voter population |
| **Form 12D** | Postal ballot / home voting request for PwD electors and electors aged 85+ | The eligible elector, submitted to the Returning Officer | An election is notified and the elector wants to vote by postal ballot / home voting rather than in person | Must be submitted within 5 days of that specific election's notification — a hard, short, election-specific deadline, distinct from and downstream of the one-time PwD-marking step on Form 8 | Deepa (persona 8, explicitly two separate steps with different timing); Kamla Devi (persona 9, future-eligible once she crosses 85) |

Key product-design consequence of this table: the decision engine's terminal-node copy must never conflate the *one-time, evergreen* Form 8 PwD-marking step with the *per-election, time-boxed* Form 12D request — these are sequential and have different triggers, different filing targets (ERO vs. Returning Officer), and radically different deadline structures (none vs. 5 days from notification). Persona 8 (Deepa) exists specifically to force this distinction into the tree design; it must not be collapsed into a single terminal outcome for editorial convenience.

#### 2.3.1 Document-category detail (supplementary to the lifecycle table)

The decision engine's terminal nodes must present document requirements as *categories*, not as a definitive checklist copied once and left to rot — the exact accepted-document list is exactly the kind of specific that changes without VoteAssist necessarily hearing about it, so every row below is written as a category with an explicit verification pointer, not a final answer.

| Form | Document categories typically requested (general shape only) | Verification status |
|---|---|---|
| Form 6 | Proof of age, proof of ordinary residence at the address being registered, one recent photograph | TODO: verify current accepted-document list against voters.eci.gov.in Form 6 instructions before publishing any specific document names (e.g., which ID types qualify as age/address proof) |
| Form 6A | Passport, proof of the India address being registered against | TODO: verify against voters.eci.gov.in overseas-elector guidance |
| Form 7 | Grounds/evidence supporting the objection or deletion claim (e.g., death certificate for a deceased-elector case, evidence of duplicate registration) | TODO: verify current evidentiary expectations against ECI's Form 7 procedure |
| Form 8 (shift/correction/EPIC/PwD) | Proof of new address (for a shift), proof supporting the specific correction claimed (e.g., a marriage certificate for a name change following marriage), existing EPIC (for replacement, where available), medical/disability certification (for PwD marking) | TODO: verify current accepted-document list per sub-case against voters.eci.gov.in Form 8 instructions — this form covers several distinct sub-cases and the document list is not uniform across them |
| Form 2 | Service/enlistment proof, declaration of non-enrollment as an ordinary elector | TODO: verify against servicevoter.nic.in current instructions |
| Form 12D | Proof of PwD status already marked on the roll (or age 85+ status), submitted within the 5-day window of the specific election's notification | TODO: verify against the current election notification and Form 12D instructions in force at that time — this form is inherently time-and-election-specific, so no static document list should be hardcoded far in advance |

The engine's copy for every one of these rows must ship with the phrase "confirm the current list on voters.eci.gov.in before you go" (or an equivalent, localized rendering) rather than presenting a hardcoded document checklist as final — this is a direct implementation of the non-goal against ever appearing more authoritative than the official source itself.

#### 2.3.2 Full persona-to-form completeness check

The original product brief's persona list (`02-personas.md`) is the acceptance test for decision-tree coverage: every persona must resolve to exactly one primary form (occasionally two, in sequence, where the underlying process genuinely has two steps, as with Deepa). The table below is the explicit, one-row-per-persona traceability check that the narrative mapping above satisfies completely, with no persona left unmapped or double-counted against contradictory forms.

| # | Persona | Primary form | Secondary form (if sequential) | Notes |
|---|---|---|---|---|
| 1 | Aarav — first-time voter | Form 6 | — | Gated on reaching the nearest of four qualifying dates |
| 2 | Priya — student in hostel | Form 6 | — | Either address valid; hostel address requires bonafide certificate |
| 3 | Mohammed — migrant worker | Form 8 | — | Only if he chooses to shift; postal-ballot special-category status for migrant workers is a TODO: verify, not assumed |
| 4 | Kavita — married, moved | Form 8 | — | Shift + name correction handled on the same form |
| 5 | Rahul — transferred employee | Form 8 | — | No automatic simplified path for general (non-service-voter) employees |
| 6 | Ananya — NRI first registration | Form 6A | — | Voting method for overseas electors is a TODO: verify, evolving policy area |
| 7 | Colonel Verma — ex-service, settled | Form 6 | Form 7 (possibly) | Form 7 only if the prior service-voter entry does not lapse automatically — TODO: verify |
| 8 | Deepa — PwD, home voting | Form 8 | Form 12D | Two genuinely sequential steps with different timing — must never be collapsed |
| 9 | Kamla Devi — lost EPIC | Form 8 | e-EPIC (parallel, not sequential) | e-EPIC download is a parallel alternative path, not a second required step |
| 10 | Suresh — misspelled name | Form 8 | — | No re-registration required |
| 11 | Farida — deceased relative + duplicate | Form 7 | — | Both the deceased-relative case and the duplicate-entry case route through Form 7 |
| 12 | Noor — gender marker + address | Form 8 | — | Both correction types generally requestable together — TODO: verify combined-workflow specifics |
| 13 | Birsa — remote tribal block | Roll search / locator | Form 8 (if discrepancy found) | Entry point is locator/search, not a form, until a discrepancy is confirmed |
| 14 | Nasreen — no standard address proof | Form 6 | — | Exact accepted-proof list and BLO-verification alternative flagged TODO: verify |

Two personas (13, and the implicit "just checking my status" persona embedded in the decision tree's root branch A) resolve to a non-form terminal outcome (roll search / locator) rather than a form outright — this is intentional and mirrors the actual tree structure in `04-decision-tree-spec.md`, where roll search is the entry point for anyone unsure of their status before any form-specific branch is reached.

### 2.4 Four qualifying dates, e-EPIC, PwD/postal ballot/home voting, servicevoter.nic.in

- **Four qualifying dates**: 1 January, 1 April, 1 July, and 1 October each year are the dates against which "will you have turned 18" is evaluated, replacing the older single 1-January-only cutoff. This directly changes the correct answer to Aarav's (persona 1) question — "I turned 18 in May, can I register now?" is no longer a strict "wait until next January" answer, and the decision tree's age/qualifying-date branch node must compute against the nearest *upcoming* qualifying date from four candidates, not one.
- **e-EPIC**: the digital voter ID, available since 25 January 2021, downloaded after OTP verification, and legally equivalent to the physical card. This is the fast, low-friction path for Kamla Devi (persona 9, lost physical EPIC) *if* she or a family member can complete OTP verification — the decision tree must present both the e-EPIC path and the physical-replacement-via-Form-8 path as parallel valid options, not assume digital literacy.
- **PwD / 85+ postal ballot / home voting**: available via Form 12D as above, distinct from the one-time PwD marking on Form 8 (Section 2.3).
- **servicevoter.nic.in**: the dedicated portal for Form 2 (service elector registration). VoteAssist deep-links here for the Form 2 persona-path if/when a service-voter persona is added; it is a separate domain from voters.eci.gov.in and must be presented as such, not conflated in copy.

### 2.5 National Voter Helpline (1950), Book-a-Call with BLO, NGSP, Voter Helpline App, cVIGIL, Saksham

These are the "human/analog fallback" layer that VoteAssist's product design must treat as first-class, not as an afterthought bolted onto the end of a web flow — this matters directly for personas with low digital literacy or low connectivity (Kamla Devi, persona 9; Birsa, persona 13).

- **National Voter Helpline, 1800-11-1950 ("1950")**: staffed 8am-8pm. This is the single number VoteAssist's disclaimer banner and every terminal outcome should surface as a fallback path, always, regardless of channel.
- **Book-a-Call with BLO**: a scheduling function via ECINET that lets a citizen arrange a call with their local Booth Level Officer — the single best fallback for a user like Birsa (remote, low connectivity, distant BLO office) who cannot reliably complete a multi-step online flow in one sitting.
- **National Grievance Service Portal (NGSP 2.0)**: tracks grievances, with EROs/DEOs/CEOs directed to resolve within 48 hours. VoteAssist should surface NGSP as the escalation path when a user's issue is a *complaint about process* (e.g., "my Form 7 objection was filed months ago and nothing happened") rather than a first-time procedural question — the decision tree needs a branch that distinguishes "what should I do" (routes to a form) from "something already went wrong" (routes to NGSP/complaints@eci.gov.in), which the current v1 tree does not yet explicitly separate.
- **Voter Helpline App** (Android/iOS): the mobile-native equivalent of much of voters.eci.gov.in's functionality; deep-linked as an alternative to the web portal for users who are more comfortable with an app-store install than a browser.
- **cVIGIL** ("Vigilant Citizen"): a citizen-reporting app for Model Code of Conduct violations during election periods, extended by a public MCC Violation Portal. VoteAssist does not integrate with cVIGIL and should not encourage its use as a general-purpose channel — it exists for a materially different purpose (reporting *others'* MCC violations, which sits close to politically-charged territory) than VoteAssist's procedural-guidance mission, and mentioning it in-product should be limited to a factual "if you want to report an MCC violation, that's a separate ECI tool" note rather than active promotion.
- **Saksham**: ECI's registration/services app specifically for PwD electors. VoteAssist should deep-link to Saksham alongside the Form 8 PwD-marking and Form 12D flows as a channel-specific alternative for that persona group.

### 2.6 Model Code of Conduct (MCC) — concrete product requirements, not a legal footnote

MCC comes into force the instant ECI announces an election schedule (for a state or the Union) and remains in force until results are declared. It restricts government machinery and government-adjacent messaging that could influence voters. VoteAssist is not government machinery, but it operates in exactly the space MCC is designed to police — civic/electoral communication reaching citizens at scale — so this PRD treats MCC compliance as an active product requirement, not a static legal disclaimer. Concretely, the platform SHALL, for any state currently under an active MCC window (tracked per-state, since MCC timing is state-specific for state elections and national for general elections):

1. **Throttle or suspend proactive broadcast-style messaging.** Any bot-initiated push notification, WhatsApp template broadcast, or Telegram channel announcement that could be read as "get out the vote" mobilization must be suspended or reduced to strictly factual, user-initiated-equivalent content (e.g., responding to a user's own query remains fine; unsolicited "election is coming, register now!" broadcasts do not) for the duration of the MCC window in that state. This is an explicit gate in `crates/jobs` / `crates/bot-telegram` / `crates/bot-whatsapp`: broadcast-capable send paths must check an MCC-active flag per state before firing, not rely on manual operator discipline alone.
2. **Add extra, more prominent neutral disclaimers during MCC windows.** The standing disclaimer (Section 6 of `06-legal-compliance-review.md`) is necessary but during an active MCC period the product should surface an additional, state-specific note confirming the *current* election's notified dates (since e.g. the Form 12D 5-day window is measured from that specific election's notification date, not a generic deadline) and reiterating non-affiliation more prominently than in steady-state operation.
3. **Never appear to be a government announcement channel.** During MCC, scrutiny of anything that looks like official electoral communication intensifies; VoteAssist's non-impersonation posture (no emblem, no government-style formatting, no domain/branding that could be mistaken for official — see `06-legal-compliance-review.md` Section 2) must be verified as still holding under active review at the start of every MCC window a Compliance Owner (named role, same document, Section 6) is tracking, not merely assumed to hold statically from launch.
4. **Continue pure "how do I do X" administrative guidance unthrottled.** The MCC restriction targets mobilization/announcement-style outreach, not procedural help itself — a user who arrives and asks "which form do I need" should get the same quality of answer during MCC as outside it. Throttling applies to what VoteAssist *pushes*, not to what it *answers*.

This is flagged in `06-legal-compliance-review.md` as a genuinely live legal risk area requiring direct legal review before any election-period operation; the requirements above are the product's operational response to that risk, to be confirmed, not assumed sufficient, by counsel.

### 2.7 GIGW 3.0 and DPDP Rules 2025 — as product requirements

**GIGW 3.0** (Guidelines for Indian Government Websites/apps, maintained by NIC/MeitY) sets WCAG 2.1 AA as its accessibility baseline, plus India-specific UX and multilingual requirements. VoteAssist is not a government site and GIGW is not legally mandatory for it — but this PRD adopts it as the product's own voluntary baseline, both because it is good practice for a civic-guidance product and because it eases any future adoption/endorsement conversation with ECI or a State CEO office. Concretely, as product requirements (not aspirations):

- The platform SHALL default to WCAG 2.2 AA (a stricter target than GIGW 3.0's own WCAG 2.1 AA floor), verified per `12-accessibility-spec.md` and enforced in CI via the Lighthouse accessibility budget (Section on CI/CD in the Technical Architecture; target >95).
- The platform SHALL support the multilingual roster committed to in `11-multilingual-strategy.md` (English + Hindi fully shipped, all 21 other Eighth Schedule languages on a staged roadmap), consistent with GIGW's multilingual expectations for government-adjacent public services.
- The platform SHALL degrade gracefully on low-bandwidth/2G connections and older devices, per GIGW's performance expectations — reflected architecturally in the Leptos SSR + minimal-hydration ("islands") approach in `crates/web-app` (Section on Technical Architecture).

**DPDP Rules 2025** were notified 13 November 2025 (Gazette 14 November 2025), with phased compliance through 13 May 2027, covering consent, notice, breach notification, record-keeping, extra protections for children and persons with disabilities, and heightened obligations (DPIAs, audits) for any entity classified as a Significant Data Fiduciary. As product requirements:

- The platform SHALL NOT retain any official identity document (Aadhaar, EPIC, passport, birth certificate, disability certificate, or similar) beyond the single request lifecycle in which it might transiently be processed client-side, and SHALL NOT persist such documents server-side under any default flow (consistent with the non-goal already established in `01-prd.md` Section 2.3 and `06-legal-compliance-review.md` Section 3).
- The platform SHALL collect only the minimal decision-engine inputs needed to answer the current session's question (age bracket, residence type, state), and SHALL NOT collect name, EPIC number, Aadhaar number, or other direct identifiers unless a specific opt-in feature requires it.
- The platform SHALL obtain clear, specific, opt-in consent for any optional data collection beyond the anonymous session (e.g., an email address for checklist delivery), never a bundled blanket clause.
- The platform SHALL NOT collect, infer, or store caste, religion, political affiliation, or other special-category data under any feature, under any circumstance — a stricter bar than DPDP itself requires, adopted deliberately given the political-neutrality mission constraint.
- The platform SHALL provide a documented, working right-to-erasure/access process for any user-identifiable data it does hold (e.g., an opt-in email), published at `/about/privacy`.

VoteAssist's minimal-data-by-design posture is deliberately built to make most DPDP Rules 2025 obligations moot in practice (there is very little data to protect, breach-notify on, or DPIA), but this PRD's job is to show the org has actually read the Rules and designed to them, not to assume minimalism is automatically compliant — final compliance posture still requires counsel sign-off per `06-legal-compliance-review.md` Section 9.

### 2.8 data.gov.in electoral datasets — usable vs. not usable

data.gov.in (the Open Government Data Platform) hosts **aggregate** electoral statistics: elector counts, turnout figures, PC/AC-wise results. This is usable, and the product architecture should use it, for exactly one purpose: aggregate, non-personal "your state's electoral snapshot" context content (e.g., "State X has approximately N registered electors as of the last published roll revision"). It is explicitly **not usable, and must never be used,** as an individual-level lookup source — there is no per-person data on data.gov.in, and even if there were, VoteAssist's architecture (Section 3 below) is deliberately built to never perform individual electoral-roll lookups itself under any circumstance, always deep-linking to voters.eci.gov.in's own roll-search tool instead.

### 2.9 State CEO portals

Every state and Union Territory has a Chief Electoral Officer (CEO) office with its own public-facing portal, referenced generically in `01-prd.md` Section 1 alongside voters.eci.gov.in and ECINET as an official deep-link target. These portals typically carry state-specific content (state helpline numbers, state-specific SVEEP campaigns, local notifications) layered on top of the nationally standardized form set described in Section 2.3. Two product implications follow:

- **State-specific deep-link targets, not state-specific decision logic.** As established in Section 2.2, the underlying forms and rules are nationally standardized via ERONET/UNPER, so the decision tree itself stays single and national; only the *terminal deep-link URL* varies by the user's declared state (e.g., routing a Bihar-based user to the Bihar CEO portal's specific elector-services page where one exists, alongside the national voters.eci.gov.in target). This is a `kb-content` data concern (a per-state URL table), not a `core-domain` logic concern.
- **Coverage gap, flagged explicitly.** A verified, current, per-state table of CEO portal URLs is not yet compiled and is TODO: compile and verify against each state/UT CEO office's published site before shipping any state-specific deep-link — in the interim, the product defaults to national-level deep-links (voters.eci.gov.in, ECINET, 1950) for every state, which are always correct, and adds state-specific links only once each one is individually verified and dated.

### 2.10 Glossary of acronyms used throughout this landscape section

| Term | Expansion |
|---|---|
| ECI | Election Commission of India |
| ERO | Electoral Registration Officer |
| AERO | Assistant Electoral Registration Officer |
| BLO | Booth Level Officer |
| DEO | District Election Officer |
| CEO | Chief Electoral Officer (state/UT level) |
| EPIC | Elector's Photo Identity Card (the physical/digital voter ID) |
| AC | Assembly Constituency |
| PC | Parliamentary Constituency |
| ERONET | Electoral Roll Management System (official-use, launched 2018) |
| UNPER | Unified National Photo Electoral Roll (the national elector database behind ERONET) |
| NVSP | National Voters' Service Portal (predecessor, superseded/merged into voters.eci.gov.in) |
| NGSP | National Grievance Service Portal |
| MCC | Model Code of Conduct |
| PwD | Persons with Disabilities |
| GIGW | Guidelines for Indian Government Websites (and apps), maintained by NIC/MeitY |
| DPDP | Digital Personal Data Protection (Act 2023 / Rules 2025) |
| SVEEP | Systematic Voters' Education and Electoral Participation (ECI's voter-education program) |
| RP Act | Representation of the People Act (1950 and 1951) |
| PIB | Press Information Bureau |
| OGD | Open Government Data (platform: data.gov.in) |

### 2.11 Requirements index (traceability aid)

This is a consolidated index of the concrete SHALL/must requirements this section derives, numbered for traceability from other PRD sections (architecture, test plan) back to their source reasoning above. This index does not introduce new requirements; it collects ones already stated in Sections 2.6 and 2.7.

| ID | Requirement | Source |
|---|---|---|
| R-MCC-1 | Broadcast-capable send paths (bot-telegram, bot-whatsapp, jobs) SHALL check a per-state MCC-active flag before firing proactive/broadcast messages | Section 2.6, item 1 |
| R-MCC-2 | The product SHALL surface an additional, state-specific, election-dated disclaimer during an active MCC window | Section 2.6, item 2 |
| R-MCC-3 | The Compliance Owner SHALL re-verify non-impersonation posture at the start of every tracked MCC window | Section 2.6, item 3 |
| R-MCC-4 | Procedural "how do I do X" guidance SHALL remain fully available, unthrottled, during MCC | Section 2.6, item 4 |
| R-A11Y-1 | The platform SHALL default to WCAG 2.2 AA | Section 2.7 |
| R-A11Y-2 | The platform SHALL support the committed multilingual roster per `11-multilingual-strategy.md` | Section 2.7 |
| R-A11Y-3 | The platform SHALL degrade gracefully on low-bandwidth/2G connections | Section 2.7 |
| R-DPDP-1 | The platform SHALL NOT persist official identity documents server-side under any default flow | Section 2.7 |
| R-DPDP-2 | The platform SHALL collect only minimal decision-engine inputs, no direct identifiers by default | Section 2.7 |
| R-DPDP-3 | Any optional data collection SHALL require clear, specific, opt-in consent | Section 2.7 |
| R-DPDP-4 | The platform SHALL NOT collect, infer, or store special-category data (caste, religion, political affiliation) under any feature | Section 2.7 |
| R-DPDP-5 | The platform SHALL provide a documented right-to-erasure/access process for any user-identifiable data it holds | Section 2.7 |

---

## 3. Existing Open-Source / Civic-Tech Landscape — What to Reuse, What to Avoid

VoteAssist is not the first Indian civic-tech project to touch electoral data. Understanding the existing landscape — what these projects do, and specifically what NOT to replicate from them — is necessary context for a production rebuild, because several of the most visible prior-art projects in this space made an architectural choice (scraping or mirroring the electoral roll) that VoteAssist deliberately does not make, and the reasoning for that divergence needs to be explicit and load-bearing in this PRD, not just implied.

| Project / Org | What it does | Relevance to VoteAssist | Reuse or Avoid, and why |
|---|---|---|---|
| **in-rolls / electoral_rolls** (Gaurav Sood et al.) | A static historical corpus of electoral roll PDFs collected for research purposes | Demonstrates that electoral roll data has historically been distributed as scanned/PDF documents rather than a queryable API — informs why a citizen-facing "just look it up" experience has been historically hard to build well | **Avoid mirroring.** Useful as historical/research prior art, not as a live data source. VoteAssist never ingests roll data of any kind, historical or current — it always deep-links to voters.eci.gov.in's own search tool, per Section 2.8 above. |
| **in-rolls / parse_searchable_rolls** | Parser tooling for machine-readable (searchable-text) roll PDFs | Same corpus family as above; shows the technical difficulty of even extracting *searchable* rolls reliably | **Avoid replicating the parsing approach as a product feature.** Fine as a research tool; not something VoteAssist builds a citizen-facing feature on top of, since any parser-derived roll copy is immediately stale relative to the live official roll. |
| **in-rolls / parse_unsearchable_rolls** | Parser/OCR tooling for scanned (image-only, non-searchable-text) roll PDFs | Illustrates how much of the historical roll corpus requires OCR to become machine-readable at all, and the accuracy problems that introduces | **Avoid.** Same staleness and accuracy-risk reasoning as above, compounded by OCR error rates on scanned documents — a citizen relying on an OCR'd historical PDF for their current registration status is a worse outcome than a citizen being sent to the live official search tool. |
| **RO-29 / electoral_scraper_pdf** | A scraper targeting official electoral-roll PDF endpoints | Directly on point as a cautionary example: this is the kind of tool VoteAssist explicitly chooses not to build | **Avoid, explicitly.** Scraping official roll endpoints carries three concrete risks VoteAssist's architecture is designed to avoid entirely: (1) **CAPTCHA-circumvention risk** — official portals gate roll search behind CAPTCHA/rate-limiting specifically to prevent bulk automated access, and any scraper either breaks that gate (a ToS and likely legal problem) or degrades to unreliable, easily-broken automation; (2) **staleness risk** — any scraped/cached copy of roll data is wrong the moment the live roll is updated, and electoral rolls are updated continuously (roll revisions, deletions, additions) — a citizen told stale information about their own registration status is actively harmed, not just inconvenienced; (3) **DPDP risk of processing others' personal data at scale** — a scraped roll mirror is, definitionally, a bulk store of other people's personal data (name, age, address, relative's name, EPIC number) with no consent basis and no purpose limitation, which is close to the worst possible posture for a project whose entire credibility rests on a minimal-data, DPDP-conscious design. VoteAssist always deep-links to the official, live, rate-limited, CAPTCHA-protected search tool instead — slower per-lookup for the user in the CAPTCHA sense, but correct, current, and legally clean. |
| **abhimanyu-sikarwar / electoral-roll-finder** | Another scraper/finder tool targeting official roll lookup endpoints | Same category and same cautionary value as RO-29 above | **Avoid, same reasoning as above.** Cited here as a second independent example that this pattern (build a scraper around ECI's public lookup) recurs in the ecosystem — worth naming explicitly in this PRD so a future contributor proposing "let's just scrape the roll for a faster lookup UX" can be pointed at this section rather than the reasoning being re-litigated from scratch. |
| **ADR (Association for Democratic Reforms) / MyNeta** | Candidate and elected-representative affidavit transparency (assets, criminal cases, education) sourced from nomination-paper disclosures, published with a transparent, documented methodology | Not a data source VoteAssist integrates with, and not adjacent to VoteAssist's decision-engine scope at all — but highly relevant as an **organizational/governance model** | **Reuse the governance model, avoid the data domain.** ADR/MyNeta is a well-established, broadly respected, non-partisan Indian civic-tech organization specifically *because* of its transparent, documented, consistently-applied methodology and its discipline about staying in a narrow, factual, non-partisan lane despite operating adjacent to intensely political subject matter (candidates, parties, elections). VoteAssist should emulate that organizational discipline: a public, versioned methodology for how KB entries get sourced and reviewed (`06-legal-compliance-review.md` Section 7), a named compliance owner, and a track record of citation-backed claims. What VoteAssist must explicitly avoid is scope creep into ADR/MyNeta's actual subject matter — candidate affidavits, criminal-case disclosures, asset declarations — because that data, however factually presented, is adjacent to political content (it is about specific candidates and parties) in a way that sits outside VoteAssist's "how do I register" mission and would put real pressure on the platform's neutrality guarantee the moment it started surfacing anything candidate-shaped. The lesson from ADR/MyNeta is "be rigorously non-partisan even when your subject matter is politically adjacent," not "add candidate data because it's popular civic-tech territory." |
| **cVIGIL / MCC Violation Portal** (ECI-operated, referenced here for landscape completeness alongside the citizen-facing OSS/data projects above) | Citizen reporting of MCC violations during election periods | Adjacent civic-participation tooling in the same broad space as VoteAssist, but a different mission (reporting on others' conduct, not self-service registration guidance) | **Avoid absorbing into VoteAssist's scope.** As covered in Section 2.5, VoteAssist should acknowledge cVIGIL's existence factually if asked, but should not build reporting features that overlap with it — that would pull the product toward politically-charged terrain (a citizen "reporting" activity is inherently more contestable than a citizen "registering to vote" activity) that the neutrality non-goal is designed to keep it out of. |
| **data.gov.in electoral datasets** (Open Government Data Platform) | Aggregate elector counts, turnout, PC/AC-wise results | Legitimate, usable source for non-personal aggregate context content | **Reuse, for aggregate content only.** See Section 2.8 — usable for "your state's snapshot" framing, never for anything resembling individual lookup, and never presented in a way that could be mistaken for real-time official data (it should carry its own "as of [dataset date]" labeling, distinct from the live-data caution already required for all terminal-node citations). |

### 3.1 The core architectural principle this table supports

VoteAssist's decision to **never scrape or mirror the electoral roll, and always deep-link to voters.eci.gov.in instead**, is not an incidental implementation detail — it is a direct consequence of the same non-goals that shape the rest of the product (Section 2.3 of `01-prd.md`): no on-behalf submission, no document custody, minimal data collection. A scraped roll mirror would quietly convert VoteAssist from "a stateless guidance layer that touches almost no personal data" into "an unofficial bulk store of the national electoral roll," which is a different, riskier product with a different (and much worse) DPDP posture, a different ToS-and-legal exposure profile, and a credibility problem the moment any scraped entry is discovered to be stale or wrong. Every deep-link-only design decision elsewhere in this PRD (the roll-search terminal outcome in the decision tree, the explicit avoidance of any server-side roll cache, the `crates/analytics` design that stores only anonymous funnel events rather than anything resembling roll data) traces back to this same principle, established here with its full reasoning rather than asserted as an unexplained rule.

### 3.2 Legal/ethical reasoning, expanded

The table in Section 3 compresses "avoid, because CAPTCHA/staleness/DPDP risk" into a single cell for the scraper projects; each of those three risk categories deserves its own explicit reasoning, since a future contributor may reasonably ask "why not just scrape it carefully, rate-limited, for a better UX than a redirect":

1. **CAPTCHA-circumvention risk.** Official roll-search endpoints gate bulk/automated access behind CAPTCHA and rate-limiting by design — that is the portal operator's explicit signal that automated querying is not the intended access pattern. Building a scraper against that gate means either (a) actively defeating the CAPTCHA, which is a Terms-of-Service violation and, depending on jurisdictional interpretation of India's IT Act provisions on unauthorized access, a genuine legal exposure the project should not accept on the public's behalf, or (b) building something so rate-limited and fragile in the face of CAPTCHA challenges that it provides no real UX advantage over a direct deep-link, while still carrying the ToS risk of (a). Neither branch is worth the exposure for a volunteer-run nonprofit project.
2. **Staleness risk.** Electoral rolls are living documents — continuous revision, deletions for deceased/duplicate/relocated electors, additions for new registrations. Any scraped snapshot is stale from the moment it's taken, with no mechanism to know *how* stale without re-scraping (which reintroduces risk 1). For a product whose core promise is procedural accuracy, silently serving a citizen a possibly-stale answer about their own registration status is a worse failure mode than an honest "here is the live official tool, please check there."
3. **DPDP risk of processing others' personal data at scale.** A scraped roll mirror is a bulk personal-data store — names, ages, addresses, relative's names, EPIC numbers — with no consent basis from any of the individuals whose data it contains, no defined purpose limitation, and no practical way to honor an erasure request from someone who never interacted with VoteAssist directly. This is close to the worst-case posture under the DPDP framework described in Section 2.7, and directly contradicts the minimal-data-by-design principle this entire PRD is built around. It is not a risk VoteAssist can mitigate by being careful; it is a risk that only disappears by not doing it.

### 3.3 Ongoing landscape monitoring

The civic-tech/OSS landscape referenced in Section 3's table is not static — new scraper projects, new ECI digital initiatives, and new adjacent civic-tech efforts will appear over the life of this product. This PRD does not assign a one-time "read the landscape" task; it assigns an ongoing responsibility to the Compliance Owner role (`06-legal-compliance-review.md` Section 6) to periodically re-scan for new prior-art or adjacent projects worth either learning from (governance model, per the ADR/MyNeta case) or explicitly avoiding (new scraper tooling, new candidate-data aggregators), on the same cadence as the quarterly content-review cycle already established for KB citations.

### 3.4 Open question flagged for this landscape

Whether any ECI-published API or data feed exists for polling-station lookup specifically (as distinct from full roll search) that VoteAssist could legitimately consume under its own terms, versus always deep-linking to ECI's own locator tool, remains unresolved from the v1 documentation pass (`01-prd.md` Section 9) and is not settled by this deeper landscape review — the default assumption carried forward into this PRD is deep-link only, until an official, ToS-compatible API is identified and verified.
## 4. Product Vision & Non-Goals (v2 restatement)

VoteAssist India helps every eligible Indian citizen understand exactly what
they need to do about their voter registration or status, then deep-links
them to the **official** Election Commission of India (ECI) systems —
voters.eci.gov.in, ECINET, the Voter Helpline App, servicevoter.nic.in, State
CEO portals, and the 1950 helpline — to actually take that action. It is a
guidance layer, not a government system, and this PRD's architecture choices
exist in service of that boundary, not despite it.

Non-goals carried forward unchanged from v1 (docs/01-prd.md) and now binding
on the Rust rebuild specifically:

1. **Not an official portal.** No ECI emblem, no State Emblem of India, no
   visual mimicry of eci.gov.in, no domain name implying official status.
2. **No on-behalf submission.** No crate, service, or bot channel in this
   architecture ever POSTs a completed Form 6/6A/7/8/2/12D to an ECI system
   on a user's behalf. Every terminal outcome ends in a deep link the user
   clicks themselves.
3. **No document custody by default.** No crate accepts a document upload in
   MVP-Rust-v1. If a future OCR-assisted prefill feature is ever built, it
   must run client-side (WASM) or be encrypted-in-transit/at-rest with a
   strict TTL and explicit per-use consent — never a default server-side
   store.
4. **Strict political neutrality.** No content names, favors, or criticizes
   a party, candidate, or ideology. The MCC Control Panel (admin page 8) is
   the concrete engineering mechanism that enforces this during election
   periods, not just a written policy.
5. **Citations or explicit uncertainty, always.** Every `TerminalNode` in
   `core-domain` must carry at least one citation resolving to a
   `kb-content` entry — this is enforced by `validateTree()` today in the TS
   prototype and must remain a compiled-in invariant (via `proptest`) in the
   Rust rebuild, not a manual review checklist that can be skipped.

What's NEW in v2 scope versus the v1 PRD: this document adds a production
multi-channel architecture (web, Telegram, WhatsApp, future IVR), a full
admin/content-operations surface, a privacy-preserving analytics pipeline,
and a much larger decision-tree coverage target (see Section 10). It does
NOT change the mission or the non-goals above — it exists to make them
enforceable at the infrastructure level rather than by convention alone.

## 5. Personas (cross-reference)

The full persona list from the original product brief is unchanged and is
detailed in `docs/02-personas.md` (v1): first-time voters, students, migrant
workers, hostel/PG residents, renters, people changing city/state, married
women changing residence, government/private employees transferred, NRI
voters, service voters, PwD, senior citizens, tribal/remote-area residents,
urban slum residents, homeless citizens (where legally applicable),
transgender citizens, citizens lacking Aadhaar/passport/driving
licence/permanent address, name/DOB mismatch, duplicate/deleted voter IDs,
shifted polling stations, missing/lost/damaged EPIC, new 18-year-olds, and
future eligible voters.

This PRD does not re-author those personas. What changes in v2 is that
Section 10 (Decision Engine v2) maps **every one** of them onto a concrete,
buildable tree path — the v1 MVP tree only covered a subset (see
docs/04-decision-tree-spec.md); Section 10 closes that gap and is explicit
about which personas were already served versus newly specified here.

One addition worth naming explicitly for the Rust multi-channel
architecture: a **channel persona** dimension layered on top of the existing
situational personas — the same "migrant worker" persona might reach
VoteAssist via the web app, via WhatsApp on a basic smartphone with limited
data, or (in v3) via an Exotel voice call from a feature phone with no data
connection at all. Section 6.7 (multi-channel architecture) is written so
that `core-domain`'s decision logic is identical across all three; only the
presentation layer changes. No persona-specific business logic should ever
live inside a channel adapter (`bot-telegram`, `bot-whatsapp`,
`ivr-gateway`) — if it does, that's a design smell to catch in review.
## 6. Technical Architecture — Rust Platform

### 6.1 High-level architecture

```mermaid
flowchart TB
    subgraph Channels
        WEB[web-app<br/>Leptos SSR + islands]
        TG[bot-telegram<br/>teloxide]
        WA[bot-whatsapp<br/>Meta Cloud API]
        IVR[ivr-gateway<br/>Exotel — v2/v3]
    end

    subgraph Core["Core services (all consume core-domain)"]
        API[api<br/>Axum + utoipa]
        DOMAIN[core-domain<br/>pure decision engine]
        KB[kb-content<br/>KB loader + validation]
    end

    subgraph Ops
        ADMIN[admin-app<br/>Leptos SSR, RBAC]
        JOBS[jobs<br/>apalis + apalis-cron]
        ANALYTICS[analytics<br/>event ingestion + rollups]
    end

    subgraph Data
        PG[(Postgres 16)]
        MEILI[(Meilisearch)]
    end

    subgraph Observability
        OTEL[tracing + OpenTelemetry OTLP]
        GRAFANA[Grafana / Loki / Tempo<br/>self-hosted]
    end

    WEB --> API
    TG --> API
    WA --> API
    IVR --> API
    API --> DOMAIN
    API --> KB
    ADMIN --> API
    API --> PG
    KB --> MEILI
    JOBS --> PG
    JOBS --> KB
    ANALYTICS --> PG
    API --> ANALYTICS
    API --> OTEL
    JOBS --> OTEL
    OTEL --> GRAFANA
```

Every public-facing channel is a thin adapter over the same `api` service,
which in turn is a thin HTTP layer over the same `core-domain` decision
engine. This is the single most important structural property of the
architecture: **decision logic exists in exactly one place**, and every
channel — including ones that don't exist yet (IVR) — gets identical,
independently-tested behavior for free. This directly serves the "operating
system for Indian voters" framing from the original project brief: the
value is the decision engine and knowledge base, and channels are
interchangeable, replaceable presentation layers on top of it.

### 6.2 Cargo workspace layout

```
voteassist-india/
  rust/                          <- NEW: Rust workspace root
    Cargo.toml                   <- workspace manifest
    crates/
      core-domain/               <- pure decision-engine + KB types, zero I/O
      kb-content/                <- KB loader/validator, FTL-backed
      api/                       <- Axum HTTP API, utoipa OpenAPI, sessions
      web-app/                   <- public site, Leptos SSR + islands
      admin-app/                 <- admin dashboard, Leptos SSR, separate deploy
      bot-telegram/               <- teloxide adapter
      bot-whatsapp/               <- reqwest wrapper over Meta Cloud API
      ivr-gateway/                <- reqwest wrapper over Exotel (v2/v3 scaffold)
      jobs/                      <- apalis + apalis-cron workers
      analytics/                 <- event ingestion + rollup jobs
    migrations/                  <- sqlx migrations (Postgres 16)
    i18n/
      en/*.ftl
      hi/*.ftl
      <21 other locale dirs, staged>
    xtask/                       <- cargo-xtask: validate KB, gen OpenAPI, seed DB
  apps/web/                       <- EXISTING TypeScript/Next.js prototype, retained
  packages/*                      <- EXISTING TS packages, retained during migration
  knowledge-base/                 <- SHARED source-of-truth content (JSON), used by
                                     BOTH the TS prototype and kb-content during
                                     migration, per Section 6.17
  docs/                           <- this PRD and the v1 doc suite
```

Rationale for a single Cargo workspace rather than one repo-per-service:
shared `Cargo.lock` (single source of truth for `cargo audit`/`cargo deny`),
`cargo nextest run --workspace` runs everything in one CI job, and
`core-domain`/`kb-content` changes are type-checked against every consumer
in the same build — a channel adapter cannot silently drift from the
decision engine's actual types.

### 6.3 Backend framework: Axum (not Loco, not Actix-web)

Axum is the foundation for `api`, and — via Leptos's Axum integration — for
`web-app` and `admin-app` too. Loco.rs ("Axum with batteries included," per
research) was seriously considered for the admin/CRUD-heavy surfaces, since
Loco's code-generation and opinionated project structure genuinely
accelerate building admin panels. It was not chosen as the primary framework
because:

- This system is NOT primarily a CRUD app — its core value is a
  hand-validated, property-tested decision graph, which fits better under a
  framework that gets out of the way (Axum) than one with strong opinions
  about how models/controllers are wired (Loco).
- Reusing the exact same Axum-based `api` service across web, Telegram,
  WhatsApp, and future IVR channels is simpler when there is one
  un-opinionated HTTP layer rather than a Rails-like framework whose
  conventions assume a single primary web frontend.
- Axum is proven in production at meaningful scale (research found it
  powers core Lichess background services), and its tower/tower-http
  middleware ecosystem (rate limiting, tracing, compression, CORS) is
  exactly what `api` needs.

Actix-web was not chosen because Axum's tower-based middleware model
composes more naturally with the rest of this stack's choices (tower-
sessions, tower-governor for rate limiting) and its ecosystem momentum in
2026 is stronger for new projects, per the research comparisons.

### 6.4 Database access: sqlx (primary), SeaORM (noted alternative)

`sqlx` is the default for all crates that touch Postgres. Compile-time
query checking (`sqlx::query!`) catches schema drift at build time, and
`cargo sqlx prepare` supports fully offline CI builds (no live DB needed to
compile, only to run the integration tests that actually execute queries).
This is a deliberate "start simple, no ORM magic" choice appropriate for a
transparency-focused open-source project where every query should be
readable by a contributor unfamiliar with an ORM's abstractions.

SeaORM (built on sqlx, ActiveRecord-style, reached 1.0/2.0 maturity per
2026 research, 250k+ weekly downloads) is explicitly noted as the fallback
choice if the `admin-app`'s CRUD surfaces (Section 11) turn out to need more
velocity than hand-written sqlx queries comfortably provide — this is
listed as an open question in Section 22, not silently decided. Diesel was
not chosen because its synchronous-by-default design needs `diesel-async`
bolted on, an awkward fit for an all-async Axum/Tokio stack.

### 6.5 Search: Meilisearch (primary), tantivy (noted alternative)

Meilisearch is the recommended search backend for the knowledge base,
glossary, and forms search, accessed from Rust via its official Rust SDK
crate. It was chosen over the pure-Rust `tantivy` library specifically
because it is **ready-to-deploy** (a self-hosted container, not a library
requiring a hand-built HTTP wrapper) and because its typo-tolerance matters
disproportionately for this product: users searching in a second language,
with phonetic/transliterated spelling, or with low digital literacy need
forgiving fuzzy matching more than they need raw query throughput.

`tantivy` remains noted as the "zero extra infrastructure" alternative for
smaller self-hosted deployments (e.g., an NGO running their own instance for
a single state, per the v3 "open API for NGOs" roadmap item) where running
an additional Meilisearch container is a real operational cost. This
tradeoff is deliberately left open per Section 22 rather than foreclosed.

### 6.6 Frontend: Leptos (public site + admin), SSR + islands

Leptos was chosen over Dioxus for the web-facing surfaces specifically
because of its SSR-first design and fine-grained (no virtual-DOM)
reactivity, which together produce smaller client-side JS payloads than a
VDOM-diffing approach — directly serving the original brief's "must work on
2G" and accessibility requirements. Dioxus's strength (one codebase
targeting web/desktop/mobile/TUI) is not a priority here: VoteAssist has no
desktop or mobile-native app in its MVP-Rust-v1 through v2 roadmap, so
Leptos's narrower, deeper focus on the web is the better fit. (Dioxus is
noted as worth revisiting if a native mobile app ever becomes a roadmap
item — its production use at Airbus/ESA per 2026 research shows it is a
credible choice for that specific future need, just not this one.)

Concretely: `web-app` renders every page server-side by default (plain
semantic HTML, works with JS disabled, fast on 2G). Only two islands
hydrate client-side: the decision-tree question/answer widget (needs
interactivity without a full page reload between questions) and the
language switcher. The admin app (`admin-app`) hydrates more of its surface
client-side since it's used by trusted, better-connected operators, not
the public — the accessibility/bandwidth constraint that drives the public
site's islands architecture doesn't apply the same way to internal tooling.

### 6.7 Multi-channel architecture

A `ChannelAdapter` trait in `core-domain` (or a thin shared crate) defines
the minimal surface every channel needs: submit an answer, receive the next
node (question or terminal) as channel-agnostic data, and — separately —
render that data into the channel's native format. `web-app` renders it as
HTML; `bot-telegram` renders it as an inline-keyboard message via teloxide;
`bot-whatsapp` renders it as WhatsApp interactive list/button messages via
the Meta Cloud API; `ivr-gateway` (v2/v3) would render it as TTS prompts and
DTMF/speech-recognized input via Exotel. None of these adapters contain
decision logic — they only translate.

- **Telegram**: `teloxide`, chosen because it is a mature, actively
  maintained, fully-async, pure-Rust framework purpose-built for this exact
  job — no gap to bridge via HTTP wrapping.
- **WhatsApp**: no mature Rust-native SDK exists for the Meta Cloud API, so
  `bot-whatsapp` is a thin `reqwest`-based wrapper calling Meta's REST/JSON
  API directly. This is the right level of abstraction for a simple
  request/response API; a full third-party crate would add a dependency for
  little benefit. Direct Meta Cloud API access (rather than a Business
  Solution Provider like Gupshup) is the MVP-Rust-v1/v1 choice to avoid BSP
  cost and lock-in. Gupshup — a Meta-certified Indian BSP that, per 2026
  research, now also owns Knowlarity (a CPaaS/IVR provider) — is the noted
  v2 upgrade path if template-messaging-at-scale or unified WhatsApp+voice
  billing becomes valuable; it would let one vendor relationship cover both
  `bot-whatsapp` and `ivr-gateway`.
- **IVR**: `ivr-gateway` wraps Exotel's telephony/voice-streaming APIs via
  `reqwest`, chosen over Twilio/Ozonetel/Knowlarity specifically because
  Exotel's infrastructure is India-native (lower latency, local number
  economics) and its voice-streaming platform is vernacular-first with
  Indic STT/TTS support from day one, per 2026 research — directly relevant
  to a product whose whole premise is serving citizens in their own
  language. This crate is scaffolded (interface defined, not fully wired)
  in MVP-Rust-v1; full implementation is v2/v3.

### 6.8 Background jobs: apalis + apalis-cron

`apalis` (Postgres-backed, no extra Redis dependency required) runs the
`jobs` crate's workers: a nightly dead-citation-link checker (HEAD-requests
every URL in `knowledge_entry_sources`, writes to `link_check_results`), a
KB re-verification-due digest (flags entries whose `lastVerifiedDate` has
aged past a configurable threshold), hourly analytics rollups, and a
translation-completeness report triggered on KB content changes.
`apalis-cron` (built on `apalis`, tower-middleware-compatible per research)
provides the time-based scheduling for these without needing a separate
`tokio-cron-scheduler` dependency — one job-processing crate covers both
"run when triggered" and "run on a schedule."

### 6.9 API contract: utoipa (compile-time OpenAPI generation)

`utoipa` + `utoipa-swagger-ui` annotate Axum handlers directly and generate
the OpenAPI 3.x spec at build/test time — this **replaces** the hand-written
`openapi/voteassist-api.yaml` committed during the TS-prototype phase, which
becomes a pre-migration historical reference rather than the contract of
record. The generated spec is emitted by an `xtask` command and committed
to the repo (so it's diff-reviewable in PRs) rather than only generated at
runtime, keeping the contract visible without requiring a running server to
inspect it.

### 6.10 Auth: session-based, admin/reviewer only

Public decision-engine endpoints in `api` remain fully anonymous — no
account, no login — consistent with the non-goals in Section 4. Admin/
reviewer accounts (the six RBAC roles from Section 11) authenticate via
`tower-sessions` for session management and `argon2` for password hashing.
Passkey support (a passwordless WebAuthn flow, following the pattern of
community crates like `oauth2-passkey-axum` found in 2026 research) is
noted as a near-term hardening upgrade once the admin app has real
non-founder operators, since it removes password-reuse/phishing risk for
what is otherwise the single highest-privilege attack surface in the whole
system (Section 13 covers this as a threat-model item, not just a feature).

### 6.11 Internationalization: Fluent (FTL), not flat key-value dictionaries

The TS prototype's `packages/i18n` used flat `{en: string, hi: string}`
dictionaries, which is fine for simple UI chrome but cannot correctly
express plural forms, grammatical gender agreement, or the kind of
sentence-structure variation that some Indic languages require for natural-
sounding text. `kb-content` (and `web-app`/`admin-app` for UI chrome) use
`fluent` / `fluent-templates` (Project Fluent / FTL syntax) instead. The
same MVP language roster carries over unchanged: English + Hindi shipped
and fully reviewed; the remaining 21 Eighth Schedule languages staged with
a contribution + review-gate workflow (Section 17 covers this in full).

### 6.12 Observability: self-hosted by default

`tracing` + `tracing-subscriber` instrument every crate; an OpenTelemetry
OTLP exporter ships traces/logs/metrics to a **self-hosted** Grafana/Loki/
Tempo stack by default. This is a deliberate data-sovereignty choice: a
platform whose entire premise is minimizing what it collects about Indian
citizens' voter-registration situations should not, by default, route even
operational telemetry through a foreign-hosted SaaS. `sentry-rust`
integration exists and is supported but is **off by default** — an operator
who wants it must explicitly enable it, and if they do, the privacy policy
must disclose it (self-hosted Sentry is preferred if error tracking beyond
Grafana/Loki/Tempo is needed).

### 6.13 PDF generation: headless_chrome (not printpdf)

Where the product generates documents (checklists, cover-letter drafts —
never fake government forms, per Section 4's non-goals), `headless_chrome`
drives a real Chromium instance to render an HTML template to PDF. This was
chosen over the pure-Rust `printpdf` crate specifically because a real
browser engine's text-shaping (via HarfBuzz) correctly handles complex
Indic-script rendering — conjunct consonants, vowel reordering, ligatures —
that a lower-level PDF library typically does not get right out of the box.
For a genuinely multilingual civic product, correct script rendering in
generated documents is not a nice-to-have.

### 6.14 Testing

`cargo-nextest` is the standard test runner (faster, better output than
`cargo test` at workspace scale). `proptest` provides property-based tests
for `core-domain` — the Rust-side equivalent, and eventual replacement, of
the TS prototype's exhaustive-path-enumeration test: every path through the
tree terminates, every terminal node has ≥1 citation and ≥1 deep link, no
node is unreachable. `criterion` benchmarks perf-sensitive paths (KB search
ranking, tree traversal at scale) so regressions are caught before they
reach production. Playwright (Node.js) is retained as a deliberate polyglot
boundary for browser end-to-end tests against the Leptos SSR app — Rust has
no equivalently mature browser-automation tool, and reusing Playwright here
also reuses the exact same e2e-smoke-testing pattern already proven useful
against the TS prototype in this repo.

### 6.15 CI/CD

GitHub Actions run: `cargo fmt --check`, `cargo clippy -- -D warnings`,
`cargo nextest run --workspace`, `cargo audit` (known-vulnerability
scanning), `cargo deny check` (license policy + duplicate-dependency
detection), `cargo sqlx prepare --check` (schema-drift detection against
committed query metadata), Playwright e2e, and a Lighthouse CI budget
assertion (>95, carried over unchanged from the v1 performance target).
Section 16 gives the concrete workflow YAML.

### 6.16 Deployment

Multi-stage Docker builds produce a distroless (or `scratch`) final image
per service (`api`, `web-app`, `admin-app`, `bot-telegram`, `jobs-worker`).
Rust's ahead-of-time-compiled, largely-static binaries are cheap to run on
modest, low-cost hosting — a real consideration for a nonprofit/open-
source-funded civic project that cannot assume a large cloud budget, and a
direct rebuttal to any assumption that "production-grade" implies expensive
infrastructure. `docker-compose` provides local dev (Postgres, Meilisearch,
the Grafana/Loki/Tempo stack). Kubernetes manifests are a v2 stretch goal,
not an MVP-Rust-v1 requirement — running single binaries behind a reverse
proxy is entirely sufficient at the traffic scale this project will see
before it has proven demand for horizontal orchestration.

### 6.17 Migration path from the existing TypeScript prototype

The existing TS/Next.js MVP in `apps/web` + `packages/*` is **retained**,
not deleted, for the duration of the migration, for two reasons: it is the
only currently-working, deployable version of the product, and its
decision-tree data model (`DecisionTree`/`QuestionNode`/`TerminalNode` in
`packages/decision-engine/src/types.ts`) and knowledge-base schema
(`knowledge-base/schema/entry.schema.json`) are the proven source of truth
that `core-domain`/`kb-content` must faithfully port, not redesign from
scratch. Concretely:

1. `knowledge-base/sources/*.json` stays exactly where it is and in exactly
   its current JSON-Schema-validated shape — both the TS prototype and the
   new `kb-content` crate read from the same directory during the
   migration window, so content edits are never lost or forked.
2. `core-domain`'s `DecisionTree`/`Node` Rust types are a direct, tested
   port of the existing TS types — the existing `voteAssistTreeV1` tree
   (packages/decision-engine/src/tree.ts) is the literal MVP-Rust-v1
   content target, expanded per Section 10 once ported.
3. Cutover is per-surface, not big-bang: `crates/api` + `crates/web-app`
   reaching feature parity with the existing `apps/web` (same ~21-node
   tree, en+hi, same knowledge browser) is the MVP-Rust-v1 exit criterion
   (Section 21). Only after that parity is confirmed (via the same
   Playwright smoke-test pattern already used against the TS prototype)
   does the TS prototype get formally deprecated in favor of the Rust
   stack; it is not deleted preemptively.

## 7. Repos & Libraries Reference Table

Every crate/tool below was selected against a real 2026-era comparison, not
assumed. Version numbers are indicative (the ones current at time of
writing) — `cargo update`/Renovate keep them current; do not treat these as
pinned forever.

### 7.1 Core Rust crates

| Concern | Crate(s) | Why chosen | Alternative considered |
|---|---|---|---|
| Async runtime | `tokio` 1.x | De facto standard, everything else in this table depends on it | — |
| Web framework | `axum` 0.8+ | Tower-based, unopinionated, proven in production (Lichess) | Loco.rs (too CRUD-opinionated), Actix-web (less tower-native) |
| Fullstack web UI | `leptos` 0.7+ | SSR-first, fine-grained reactivity, small JS payload | Dioxus (multi-platform but not needed here), Yew (older, less active) |
| DB access | `sqlx` 0.8+ | Async-first, compile-time query checking, offline CI mode | `sea-orm` 2.0 (noted alternative for admin CRUD velocity), `diesel` (sync-first, async bolted on) |
| Migrations | `sqlx-cli` / `sqlx::migrate!` | Ships with sqlx, sufficient for this project's schema complexity | `refinery` |
| Search client | `meilisearch-sdk` | Official Rust SDK for the chosen search backend | direct `reqwest` calls, `tantivy` (embedded alternative) |
| OpenAPI generation | `utoipa` + `utoipa-swagger-ui` | Code-first, compile-time generated, flexible | `aide` (simpler but less flexible per research) |
| Sessions | `tower-sessions` | Tower-native, works cleanly with Axum + Leptos | rolling custom cookie logic |
| Password hashing | `argon2` | Current best-practice KDF for password storage | `bcrypt` (older, weaker parameters by default) |
| JWT (if needed for any service-to-service auth) | `jsonwebtoken` | Standard, supports RS256 | — |
| Telegram bot | `teloxide` | Mature, pure-Rust, async, declarative (`dptree`) | `rutebot`, raw Bot API calls via `reqwest` |
| WhatsApp | `reqwest` (thin custom wrapper over Meta Cloud API) | No mature Rust SDK exists; a thin wrapper is the right size | Gupshup BSP HTTP API (v2 option) |
| IVR | `reqwest` (thin custom wrapper over Exotel APIs) | Same reasoning as WhatsApp | Twilio (weaker India-specific infra/Indic STT-TTS) |
| Background jobs | `apalis` + `apalis-cron` | Postgres-backed, no extra Redis dependency, tower-middleware compatible | `tokio-cron-scheduler` alone (no job-queue semantics, retries, etc.) |
| i18n | `fluent` + `fluent-templates` | Correct plural/gender handling (Project Fluent/FTL) | flat key-value dictionaries (what the TS prototype uses today) |
| Tracing | `tracing` + `tracing-subscriber` | De facto standard Rust instrumentation | `log` crate alone (less structured) |
| Telemetry export | `opentelemetry` + OTLP exporter | Standardized, routes to self-hosted Grafana/Loki/Tempo | vendor-specific SDKs |
| Error tracking (optional, off by default) | `sentry` + `sentry-opentelemetry` | Supports OTLP bridge if operator opts in | none (Grafana/Loki alone is the default) |
| PDF generation | `headless_chrome` | Correct Indic script shaping via real browser engine | `printpdf` (weak complex-script shaping) |
| Schema validation | `schemars` + `jsonschema` | Derive JSON Schema from Rust structs, validate KB content in CI | hand-maintained schema only (today's TS approach) |
| Test runner | `cargo-nextest` | Faster, better parallelism/output than `cargo test` at workspace scale | `cargo test` |
| Property testing | `proptest` | Exhaustive-ish invariant testing for the decision tree | `quickcheck` (less actively maintained) |
| Benchmarking | `criterion` | Standard statistical benchmarking harness | `divan` (newer, smaller ecosystem) |
| Supply-chain security | `cargo-audit`, `cargo-deny` | Vulnerability + license/duplicate-dependency scanning in CI | manual dependency review only |
| Rate limiting | `tower-governor` (or equivalent tower middleware) | Tower-native, composes with the rest of the middleware stack | reverse-proxy-only rate limiting (also recommended in addition, not instead) |
| HTTP client (bot/IVR adapters) | `reqwest` | Standard, async, well-documented | `hyper` directly (lower-level than needed) |

### 7.2 External services / infrastructure

| Service | Role | Why chosen |
|---|---|---|
| PostgreSQL 16 | Primary datastore | Mature, JSONB support for semi-structured content, one datastore for the whole system reduces operational surface |
| Meilisearch | KB/glossary/forms search | Ready-to-deploy, typo-tolerant, good multilingual behavior |
| Grafana + Loki + Tempo | Self-hosted observability stack | Data-sovereignty-conscious default (Section 6.12) |
| Docker + docker-compose | Local dev + deployment packaging | Standard, works with the distroless multi-stage build strategy |
| GitHub Actions | CI/CD | Already used by this repo; no new tooling to adopt |
| Meta WhatsApp Cloud API | WhatsApp channel (direct) | No BSP fee at MVP-Rust-v1 scale |
| Gupshup (noted v2 option) | WhatsApp BSP / potential unified voice+WhatsApp vendor | Meta-certified, now also owns Knowlarity (CPaaS/IVR) |
| Exotel | IVR / voice channel (v2/v3) | India-native telephony infra, vernacular-first Indic STT/TTS |

### 7.3 Prior art / reference projects (not dependencies — see Section 3 for full analysis)

| Project | Use in this PRD |
|---|---|
| `in-rolls/electoral_rolls`, `in-rolls/parse_searchable_rolls`, `in-rolls/parse_unsearchable_rolls` | Reference for understanding historical electoral-roll data structure; NOT a live data source VoteAssist integrates with |
| `RO-29/electoral_scraper_pdf`, `abhimanyu-sikarwar/electoral-roll-finder` | Cautionary examples of scraping friction (CAPTCHA, rate limits, staleness) that justify VoteAssist's deep-link-only design |
| ADR / MyNeta | Governance/credibility model (transparent, non-partisan methodology) — explicitly NOT a data integration, to avoid scope creep into candidate/affidavit content |
| data.gov.in electoral datasets | Source for aggregate-only "your state's electoral snapshot" context content |
## 8. Database Schema (Postgres 16, via sqlx)

### 8.0 Design posture

This schema backs `crates/core-domain`, `crates/kb-content`, `crates/api`,
and `crates/jobs`. It carries forward the MVP's minimal-data posture
(`docs/08-database-schema.md`), tightened for a stack with a real admin
surface and analytics pipeline — both more capable of accidentally
accumulating PII than the TS prototype ever was. Four rules hold
throughout: (1) **no document custody** — no column stores an EPIC number,
Aadhaar number, address, phone number, or document scan, an unchanged hard
non-goal from `docs/01-prd.md`; (2) **public users are not rows** — no
`users` table exists for the public product, the only accounts are
`admin_users` (contributors/reviewers/translators), and public interaction
state lives client-side or in an ephemeral cache, never in Postgres (8.7);
(3) **content is versioned/audited, usage is aggregated/rotated** — admin
edits get an append-only revision trail (`changed_by`/`changed_at`) for
DPDP record-keeping and editorial accountability, while citizen usage gets
bucketed with an explicit retention window, since DPDP minimization and
storage-limitation apply even to non-identifying data; (4) **reference
data is aggregate-only** — `states`/`assembly_constituencies`/
`parliamentary_constituencies` mirror data.gov.in/ECI notifications at
constituency level, with no elector- or roll-level table anywhere (see
the OSS/data-landscape notes on roll-scraper projects above as
cautionary prior art, not integration targets).

Enums use native Postgres `ENUM` types (mapped via `sqlx::Type`) where the
value set is genuinely closed and compile-time-checkable via
`sqlx::query!`. Controlled vocabularies expected to grow without a
migration (e.g. `topic`) stay `TEXT`, validated at the `kb-content` layer
against the JSON Schema in section 9 instead.

### 8.1 Reference / geography data

```sql
CREATE TABLE states (
    id                  SMALLSERIAL PRIMARY KEY,
    name                TEXT NOT NULL UNIQUE,
    code                TEXT NOT NULL UNIQUE,      -- TODO: verify canonical code convention against data.gov.in/ECI
    is_union_territory  BOOLEAN NOT NULL DEFAULT false,
    ceo_portal_url      TEXT,
    source_note         TEXT NOT NULL DEFAULT
        'Aggregate reference data from data.gov.in / ECI public notifications. No elector-level data.',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE assembly_constituencies (
    id              SERIAL PRIMARY KEY,
    state_id        SMALLINT NOT NULL REFERENCES states(id),
    ac_number       INTEGER NOT NULL,
    name            TEXT NOT NULL,
    reserved_status TEXT NOT NULL DEFAULT 'none' CHECK (reserved_status IN ('none', 'sc', 'st')),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (state_id, ac_number)
);

CREATE TABLE parliamentary_constituencies (
    id              SERIAL PRIMARY KEY,
    state_id        SMALLINT NOT NULL REFERENCES states(id),
    pc_number       INTEGER NOT NULL,
    name            TEXT NOT NULL,
    reserved_status TEXT NOT NULL DEFAULT 'none' CHECK (reserved_status IN ('none', 'sc', 'st')),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (state_id, pc_number)
);

CREATE INDEX idx_ac_state ON assembly_constituencies(state_id);
CREATE INDEX idx_pc_state ON parliamentary_constituencies(state_id);
```

These power "your state" context content (CEO portal links, aggregate
data.gov.in snapshots) and let `applicable_states` resolve to structured
data when the UI needs to filter. Refreshed by a periodic content-ops
import job, never populated with per-elector rows.

### 8.2 Knowledge base content

```sql
CREATE TYPE kb_source_type AS ENUM (
    'eci_official', 'state_ceo', 'gazette_law', 'sveep', 'pib_release', 'community_pending_verification'
);

CREATE TYPE kb_review_status AS ENUM (
    'draft', 'in_review', 'verified', 'needs_reverification'
);

CREATE TABLE knowledge_entries (
    id                  TEXT PRIMARY KEY CHECK (id ~ '^[a-z0-9-]+$'),  -- matches sources/*.json "id" exactly, e.g. 'form-8'
    topic               TEXT NOT NULL,       -- controlled vocab, enforced by kb-content vs entry.schema.json (9.2), not a DB CHECK
    title               TEXT NOT NULL,
    summary             TEXT NOT NULL,
    body                TEXT,                -- markdown; nullable at draft, required before status leaves 'draft' (app-layer)
    applicable_states   TEXT[] NOT NULL DEFAULT ARRAY['all'],
    source_type         kb_source_type NOT NULL,
    last_verified_date  DATE NOT NULL,
    version             INTEGER NOT NULL DEFAULT 1,
    review_status       kb_review_status NOT NULL DEFAULT 'draft',
    language            TEXT NOT NULL DEFAULT 'en',   -- BCP-47 locale of inline strings; see 9.1 for Fluent-based other locales
    related_forms       TEXT[] NOT NULL DEFAULT '{}',
    related_entities    TEXT[] NOT NULL DEFAULT '{}',
    caution             TEXT,
    created_by          UUID REFERENCES admin_users(id),  -- admin_users defined in 8.6; migration order puts it first
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_kb_entries_topic          ON knowledge_entries(topic);
CREATE INDEX idx_kb_entries_review_status  ON knowledge_entries(review_status);
CREATE INDEX idx_kb_entries_language       ON knowledge_entries(language);
CREATE INDEX idx_kb_entries_last_verified  ON knowledge_entries(last_verified_date);  -- nightly reverification sweep
CREATE INDEX idx_kb_entries_states_gin     ON knowledge_entries USING GIN (applicable_states);
CREATE INDEX idx_kb_entries_forms_gin      ON knowledge_entries USING GIN (related_forms);
CREATE INDEX idx_kb_entries_entities_gin   ON knowledge_entries USING GIN (related_entities);
```

`applicable_states`/`related_forms`/`related_entities` are arrays, not join
tables — small, bounded, queried by containment (`@>`/`&&`), so a GIN
index per column makes "entries touching Maharashtra" or "entries
mentioning Form 8" cheap without a many-to-many table. These indexes exist
for admin filtering and internal joins only; typo-tolerant end-user search
is Meilisearch's job, not Postgres's.

```sql
CREATE TABLE knowledge_entry_sources (
    id             BIGSERIAL PRIMARY KEY,
    entry_id       TEXT NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    title          TEXT NOT NULL,
    url            TEXT NOT NULL,
    publisher      TEXT,
    display_order  SMALLINT NOT NULL DEFAULT 0
);

CREATE INDEX idx_kb_sources_entry ON knowledge_entry_sources(entry_id, display_order);
CREATE INDEX idx_kb_sources_url   ON knowledge_entry_sources(url);  -- joined by the nightly link-check job
```

One row per citation, one-to-many against `knowledge_entries` — the SQL
projection of the `sources` array in section 9's JSON Schema, populated by
`kb-content` on ingest; the JSON file stays the authored source of truth.

```sql
CREATE TABLE knowledge_entry_revisions (
    id                      BIGSERIAL PRIMARY KEY,
    entry_id                TEXT NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    version                 INTEGER NOT NULL,
    snapshot                JSONB NOT NULL,   -- full serialized KnowledgeEntry (9.2) at this version -- the authoritative record
    diff_from_previous      JSONB,            -- machine diff vs previous version, UI convenience only, NOT authoritative; NULL for v1
    changed_by              UUID NOT NULL REFERENCES admin_users(id),
    changed_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    review_status_at_time   kb_review_status NOT NULL,
    change_note             TEXT,
    UNIQUE (entry_id, version)
);

CREATE INDEX idx_kb_revisions_entry ON knowledge_entry_revisions(entry_id, version DESC);

-- Append-only: app DB role granted INSERT + SELECT only. Belt-and-suspenders trigger:
CREATE FUNCTION forbid_revision_mutation() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'knowledge_entry_revisions is append-only';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_kb_revisions_append_only
    BEFORE UPDATE OR DELETE ON knowledge_entry_revisions
    FOR EACH ROW EXECUTE FUNCTION forbid_revision_mutation();
```

This is the audit-trail backbone the admin Content Editor needs (9.3):
every save — content edit, status transition, or automated
`needs_reverification` flip — inserts exactly one row, never mutates one.
Storing a full `snapshot` (not just a diff) makes "view as of version N"
and rollback plain reads/re-inserts; `diff_from_previous` is cached for
fast rendering but explicitly non-authoritative, so a diff-algorithm bug
can never corrupt the historical record.

### 8.3 Decision tree storage: artifact vs. node-by-node rows

Two shapes were considered for storing the question/terminal graph, its
localized prompts, options, citations, and deep links:

| | Node-by-node rows (`decision_tree_nodes`: `tree_id`, `node_id`, `node_type`, `prompt`/`options`/`citations`/`deep_links` jsonb) | Single versioned artifact (one validated JSON blob per version) |
|---|---|---|
| Edit granularity | Natural, one row per node | Needs a draft layer for node-level UX (below) |
| Referential integrity | Weak — nothing stops a mid-edit transaction leaving a cycle, a dangling option, or an uncited terminal | Strong — the whole graph is validated as one unit before it's ever written |
| Fits `core-domain`'s model | No — it wants one complete, already-valid tree per version, same as the TS prototype | Yes — directly loadable and `proptest`-able (every path terminates, every terminal cited) |
| Publish semantics | Ambiguous — when is a multi-row edit "published"? | Clean — publish = insert one immutable row, flip `is_active` |

**Recommendation: single versioned JSON artifact, not relational node
rows.** A decision tree is only meaningful as a whole validated graph;
relational rows would make that invariant something the table has to
uphold at every instant, which a multi-statement edit transaction can't
guarantee. A JSON artifact makes it a property checked once, at publish,
then never re-checked, because the artifact is immutable thereafter —
mirroring how the TS prototype already loads one tree module wholesale.

```sql
CREATE TABLE decision_trees (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version             INTEGER NOT NULL UNIQUE,
    artifact            JSONB NOT NULL,       -- full validated graph: nodes, FTL-localized prompts (9.1), options, citations, deep_links
    artifact_checksum   TEXT NOT NULL,        -- sha256 of the canonicalized artifact, integrity + audit
    validation_report   JSONB NOT NULL,       -- acyclic/fully-cited/resolvable-options validator output at publish time
    is_active           BOOLEAN NOT NULL DEFAULT false,
    published_at        TIMESTAMPTZ,
    created_by          UUID REFERENCES admin_users(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_decision_trees_one_active ON decision_trees (is_active) WHERE is_active = true;
CREATE INDEX idx_decision_trees_artifact_gin ON decision_trees USING GIN (artifact);

CREATE TABLE decision_tree_revisions (
    id                      BIGSERIAL PRIMARY KEY,
    tree_version            INTEGER NOT NULL REFERENCES decision_trees(version),
    diff_summary            JSONB,
    changed_by              UUID REFERENCES admin_users(id),
    changed_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    review_status_at_time   TEXT NOT NULL
);
```

Node objects inside `artifact` (illustrative, not rows) — a question node
and, downstream of it, a terminal node with `deep_links`:

```json
{
  "node_id": "q_moved_house",
  "node_type": "question",
  "prompt": { "message_id": "tree-q-moved-house-prompt" },
  "options": [
    { "value": "yes_diff_ac", "label": { "message_id": "tree-opt-yes-diff-ac" }, "next": "t_form_8_shift" },
    { "value": "no", "label": { "message_id": "tree-opt-no" }, "next": "q_correction_needed" }
  ],
  "citations": ["form-8"]
},
{
  "node_id": "t_form_8_shift",
  "node_type": "terminal",
  "prompt": { "message_id": "tree-terminal-form-8-shift" },
  "citations": ["form-8"],
  "deep_links": [
    { "target": "voters_eci_portal", "url": "https://voters.eci.gov.in/", "label": { "message_id": "deep-link-voters-portal" } }
  ]
}
```

**Reconciling node-level editing UX with whole-artifact publish:** the
visual editor's working state is node-level — it has to be, for usable
UX — but that state is explicitly a *draft*, never queried by
`core-domain`:

```sql
CREATE TABLE decision_tree_drafts (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    base_version   INTEGER REFERENCES decision_trees(version),
    draft_nodes    JSONB NOT NULL,   -- node_id -> node map, freely mutable; NOT validated as a whole graph
                                     -- while in this state (may be transiently cyclic or under-cited)
    updated_by     UUID REFERENCES admin_users(id),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

Publish assembles `draft_nodes` into a candidate artifact, runs the same
validator `core-domain`'s `proptest` suite exercises (acyclic, every node
reachable, every terminal cited, every option resolvable) in one
transaction, and only on success inserts a new `decision_trees` row
(`version + 1`), flips `is_active` atomically, writes a
`decision_tree_revisions` row, and clears the draft. A failed validation
returns node-level errors to the editor without touching `decision_trees`
— the invalid intermediate state is never visible to `core-domain` or any
live session, since sessions only ever read the currently `is_active`
artifact wholesale.

### 8.4 Forms reference

```sql
CREATE TABLE forms (
    code               TEXT PRIMARY KEY,   -- 'form-6', 'form-6a', 'form-7', 'form-8', 'form-2', 'form-12d'
    title              TEXT NOT NULL,
    summary            TEXT NOT NULL,
    superseded_forms   TEXT[] NOT NULL DEFAULT '{}',  -- e.g. ARRAY['form-8a','form-001'] for 'form-8'
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

Six rows at MVP. `superseded_forms` lets content/UI say "Form 8A is now
Form 8" per the 2022 Amendment Rules — a real support-burden reducer, not
decoration.

### 8.5 Feedback

```sql
CREATE TYPE feedback_category AS ENUM ('inaccurate_info', 'confusing', 'broken_link', 'general', 'other');

CREATE TABLE feedback (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category               feedback_category NOT NULL,
    message                TEXT NOT NULL,
    kb_entry_id            TEXT REFERENCES knowledge_entries(id) ON DELETE SET NULL,
    decision_session_id    TEXT,   -- the client-held ephemeral session token (8.7), not a FK -- there is
                                    -- no sessions table for public users to join against
    contact_email          TEXT,   -- optional, only if the user wants a reply
    status                 TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'triaged', 'resolved', 'wontfix')),
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at            TIMESTAMPTZ
);

CREATE INDEX idx_feedback_kb_entry   ON feedback(kb_entry_id);
CREATE INDEX idx_feedback_status     ON feedback(status);
CREATE INDEX idx_feedback_created_at ON feedback(created_at);  -- scanned by the retention-purge job
```

**Retention:** `contact_email` is nulled 90 days after `resolved_at` (or
after `created_at` if never resolved) — long enough to close the reply
loop, not indefinitely. `message`/`category` are retained longer as
editorial signal but remain subject to right-to-erasure if a
`contact_email` was attached and the submitter requests deletion.

### 8.6 Admin users (RBAC)

```sql
CREATE TYPE admin_role AS ENUM (
    'contributor', 'reviewer', 'legal_reviewer', 'translator', 'analytics_viewer', 'superadmin'
);

CREATE TABLE admin_users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email           TEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,   -- argon2id via the `argon2` crate, current-OWASP-recommended params
    role            admin_role NOT NULL,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login_at   TIMESTAMPTZ
);

CREATE INDEX idx_admin_users_role ON admin_users(role);
```

This table, and every table below it, exists only for the internal
admin/reviewer surface — no public signup flow, no citizen account concept
anywhere in this schema. Role semantics (enforced server-side in
`crates/api` handlers, never only hidden in the `crates/admin-app` UI):

| Role | Can do |
|---|---|
| `contributor` | Create/edit KB entries and translations up to `in_review`; cannot self-approve |
| `reviewer` | Above, plus move `draft`/`in_review`; cannot set `verified` |
| `legal_reviewer` | Only role that can transition an entry or tree publish into `verified`/active |
| `translator` | Edit FTL message bundles (9.1) and per-locale completeness only; no access to `sourceType`, `sources`, or base-language body |
| `analytics_viewer` | Read-only on `analytics_rollups_*`; no KB/tree write access |
| `superadmin` | All of the above, plus admin-user management |

### 8.7 Admin sessions vs. public decision-engine "sessions"

`tower-sessions` needs a Postgres store for the admin app (standard shape
from `tower-sessions-sqlx-store`):

```sql
CREATE TABLE sessions (
    id           TEXT PRIMARY KEY,
    data         BYTEA NOT NULL,
    expiry_date  TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_sessions_expiry ON sessions(expiry_date);  -- supports tower-sessions' expired-row sweep
```

Scoped to `admin_users` logins only (cookie-based, CSRF-protected, short
expiry) — **not** the same concept as a public decision-engine "session"
(where an anonymous citizen currently is in the tree).

**Recommendation: public decision-engine sessions are not a database table
at all in MVP-Rust-v1.** Persisting "which node is this anonymous person
on, what did they answer" as Postgres rows would create exactly the kind
of timestamped, resumable record of an anonymous citizen's in-progress
eligibility reasoning the minimal-data-by-design posture exists to
prevent, and such tables tend to quietly grow PII-adjacent fields over
time ("let's log their state too"). Preferred implementations, in order:

1. **Client-held state** (default, web app): the Leptos islands widget
   keeps current node/answers in browser memory or a signed, encrypted,
   short-lived cookie; the server stays stateless per request, evaluating
   "given this state and this answer, what's next" via `core-domain`'s
   pure function.
2. **Redis-ephemeral** (bot/IVR channels where client-side state doesn't
   fit naturally): a short-TTL (30–60 min inactivity) store keyed by a
   rotating token holds in-progress state. Never written to Postgres.

The only place a "session" identifier appears in Postgres at all is an
opaque, rotating string in `analytics_events.session_id` and optionally
`feedback.decision_session_id` — a write-once reference to a client-side
token, never a row the server can look up to reconstruct an answer history.

### 8.8 Analytics

```sql
CREATE TABLE analytics_events (
    id                  BIGSERIAL PRIMARY KEY,
    session_id          TEXT NOT NULL,   -- ephemeral, client-rotated; NOT linkable to admin_users or
                                          -- feedback.contact_email by any join key in this schema
    node_id             TEXT NOT NULL,
    answer_option       TEXT,            -- one of a fixed enum of option values for that node; app layer
                                          -- rejects anything not in the tree's own options -- never free text
    event_type          TEXT NOT NULL CHECK (event_type IN
                            ('session_start', 'node_view', 'answer_selected', 'terminal_reached', 'deep_link_click')),
    deep_link_clicked   BOOLEAN NOT NULL DEFAULT false,
    deep_link_target    TEXT,            -- e.g. 'voters_eci_portal', 'voter_helpline_app', '1950_helpline', 'ceo_portal'
    locale              TEXT NOT NULL,
    client_platform     TEXT NOT NULL CHECK (client_platform IN ('web', 'telegram', 'whatsapp', 'ivr')),
    occurred_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    occurred_hour       TIMESTAMPTZ GENERATED ALWAYS AS (date_trunc('hour', occurred_at)) STORED
);

CREATE INDEX idx_analytics_events_hour    ON analytics_events(occurred_hour);
CREATE INDEX idx_analytics_events_node    ON analytics_events(node_id);
CREATE INDEX idx_analytics_events_session ON analytics_events(session_id);  -- de-duplicating one session's
                                                                             -- funnel before rollup only

CREATE TABLE analytics_rollups_hourly (
    bucket_hour           TIMESTAMPTZ NOT NULL,
    node_id               TEXT NOT NULL,
    answer_option         TEXT,
    locale                TEXT NOT NULL,
    client_platform       TEXT NOT NULL,
    event_count           BIGINT NOT NULL DEFAULT 0,
    deep_link_click_count BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (bucket_hour, node_id, answer_option, locale, client_platform)
);

CREATE TABLE analytics_rollups_daily (
    bucket_day            DATE NOT NULL,
    node_id               TEXT NOT NULL,
    answer_option         TEXT,
    locale                TEXT NOT NULL,
    client_platform       TEXT NOT NULL,
    event_count           BIGINT NOT NULL DEFAULT 0,
    deep_link_click_count BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (bucket_day, node_id, answer_option, locale, client_platform)
);

CREATE INDEX idx_rollups_hourly_node ON analytics_rollups_hourly(node_id, bucket_hour);
CREATE INDEX idx_rollups_daily_node  ON analytics_rollups_daily(node_id, bucket_day);
```

`crates/analytics` writes only `analytics_events`; an `apalis-cron` job in
`crates/jobs` folds each hour into `analytics_rollups_hourly`, and a daily
job folds hours into `analytics_rollups_daily`. **No IP address is stored
in this table, or anywhere in Postgres** — the only place a client IP
exists at all is a short-TTL (minutes) in-memory/Redis rate-limiting cache
for abuse prevention, never persisted to disk, out of scope for this
schema.

**Retention:** raw `analytics_events` purged 90 days after `occurred_at`
once folded into both rollups. `analytics_rollups_hourly` rolls off after
~13 months (year-over-year comparison window); `analytics_rollups_daily`
is retained indefinitely, since it carries no session-level signal.
`analytics_viewer` admins query rollup tables only; raw-event queries are
`superadmin`-only, for pipeline debugging, not routine reporting.

### 8.9 Link checker

```sql
CREATE TABLE link_check_results (
    id             BIGSERIAL PRIMARY KEY,
    entry_id       TEXT NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    source_url     TEXT NOT NULL,
    http_status    INTEGER,
    check_error    TEXT,      -- populated on network/timeout failure when there's no http_status
    checked_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_link_check_entry_checked ON link_check_results(entry_id, checked_at DESC);
CREATE INDEX idx_link_check_bad_status
    ON link_check_results(checked_at) WHERE http_status IS NULL OR http_status >= 400;
```

The nightly `crates/jobs` checker walks distinct URLs from
`knowledge_entry_sources`, requests each, and appends one row per check
per entry (never an upsert — history of a citation's uptime helps decide
whether a source has gone stale). The partial index on non-2xx/timeout
results feeds the admin "broken citations" queue and contributes to the
`needs_reverification` auto-flag logic in 9.3.

### 8.10 Retention summary

Indefinite: `knowledge_entry_revisions` (DPDP record-keeping/audit trail,
append-only, small relative to traffic) and `analytics_rollups_daily`
(aggregate-only, no re-identification surface). Time-boxed: raw
`analytics_events` (90 days post-event, once rolled up),
`analytics_rollups_hourly` (~13 months, for seasonality comparison),
`feedback.contact_email` (90 days post-resolution; `message`/`category`
persist), `link_check_results` (12 months, then summarized), admin
`sessions` (rolling, per-session expiry via `tower-sessions`). Never
persisted at all: public decision-engine session state (8.7).

## 9. Knowledge Base v2 Schema (content + i18n)

### 9.1 Content-authoring model

Source-of-truth entries stay JSON files under `knowledge-base/sources/`,
one per entry, validated against `knowledge-base/schema/entry.schema.json`
— unchanged as the schema-of-record, still draft-07, because the
structured metadata it governs (topic taxonomy, source typing, form
references, versioning) hasn't changed in substance and there's no
workflow benefit to churning it during a backend rewrite.

**JSON vs. TOML for source files.** Keeping JSON (recommended): every
existing entry continues to validate unmodified, and `serde_json` +
`schemars` round-tripping (9.2) is the best-traveled path in Rust. TOML
would read marginally better for long markdown `body` blocks in a diff
(`"""..."""` vs. escaped `\n`), but JSON Schema doesn't apply to TOML
directly — validation would detour through `toml::Value` →
`serde_json::Value` anyway, removing most of the ergonomic win while
adding a translation step that can itself drift. Net: JSON stays.

**What changes: UI-facing/body strings move to Fluent.** The v1 TS model
made each language variant a fully separate JSON file sharing a
`translation_group_id`, duplicating every structured field to carry one
translated string and giving translators write access to fields they
shouldn't touch (`sourceType`, citations). v2 keeps one canonical JSON
entry per piece of content — English, by convention, per `language` — and
moves rendered strings for every other locale into Fluent bundles under
`i18n/<locale>/*.ftl`, keyed by message IDs. Because `entry.schema.json`
has no `additionalProperties: false`, this is additive: entries may carry
an optional `messageIds` object mapping `title`/`summary`/`body`/`caution`
to FTL message IDs. When present, the render layer resolves each field via
`fluent-templates` for the requested locale, falling back to the entry's
own inline strings when no translation exists yet — also how an entry
with no `messageIds` keeps working unmodified. Translators get a narrow
write surface (FTL files only, per the `translator` scoping in 8.6), and
the product gets real CLDR plural/gender rules in place of the TS
prototype's flat string-interpolation dictionaries.

Concrete example — `knowledge-base/sources/form-8.json` (trimmed):

```json
{
  "id": "form-8",
  "topic": "shifting-of-residence",
  "title": "Form 8 — Shifting of Residence, Correction of Entries, EPIC Replacement, PwD Marking",
  "summary": "Since the Registration of Electors (Amendment) Rules 2022 (in force 1 August 2022), Form 8 covers shifting of residence, correction of entries, EPIC replacement, and PwD marking in one form.",
  "body": "Use Form 8 for any of the following about your own existing entry:\n\n- You moved house...\n",
  "applicableStates": ["all"],
  "sourceType": "eci_official",
  "sources": [
    { "title": "Voter Portal — Forms Dashboard", "url": "https://voters.eci.gov.in/home/forms", "publisher": "Election Commission of India" }
  ],
  "lastVerifiedDate": "2026-07-23",
  "version": 1,
  "relatedForms": ["form-8", "form-12d"],
  "relatedEntities": ["ERO", "BLO", "EPIC", "electoral-roll"],
  "language": "en",
  "reviewStatus": "in_review",
  "caution": "Confirm on voters.eci.gov.in that the online flow still labels this as a single 'Form 8' with sub-options.",
  "messageIds": {
    "title": "form-8-title", "summary": "form-8-summary", "body": "form-8-body", "caution": "form-8-caution"
  }
}
```

`i18n/en/kb.ftl`:

```ftl
form-8-title = Form 8 — Shifting of Residence, Correction of Entries, EPIC Replacement, PwD Marking
form-8-summary = Since the Registration of Electors (Amendment) Rules 2022 (in force 1 August 2022), Form 8 covers shifting of residence, correction of entries, EPIC replacement, and PwD marking in one form.
form-8-body =
    Use Form 8 if you moved house and need your registration updated, if an
    entry needs correcting, if your EPIC is lost/damaged, or if you want to
    be marked as PwD in the electoral roll.
form-8-caution = Confirm on voters.eci.gov.in that the online flow still labels this as a single "Form 8" with sub-options.
form-8-processing-note = { $days ->
        [one] Submit within { $days } day of the election notification.
       *[other] Submit within { $days } days of the election notification.
    }
```

`i18n/hi/kb.ftl`:

```ftl
form-8-title = फॉर्म 8 — निवास स्थान में परिवर्तन, प्रविष्टियों में सुधार, EPIC प्रतिस्थापन, दिव्यांग अंकन
form-8-summary = निर्वाचक रजिस्ट्रीकरण (संशोधन) नियम 2022 (1 अगस्त 2022 से प्रभावी) के अनुसार, फॉर्म 8 अब निवास परिवर्तन, प्रविष्टि सुधार, EPIC प्रतिस्थापन और दिव्यांग अंकन को एक ही फॉर्म में शामिल करता है।
form-8-body =
    यदि आपने निवास बदला है, आपकी प्रविष्टि में सुधार चाहिए, आपका EPIC खो
    गया/क्षतिग्रस्त है, या आप दिव्यांग (PwD) के रूप में अंकित होना चाहते
    हैं — तो फॉर्म 8 का उपयोग करें।
form-8-caution = कृपया voters.eci.gov.in पर पुष्टि करें कि ऑनलाइन प्रक्रिया अभी भी इसे एकल "फॉर्म 8" के रूप में दिखाती है।
form-8-processing-note = { $days ->
        [one] चुनाव अधिसूचना के { $days } दिन के भीतर जमा करें।
       *[other] चुनाव अधिसूचना के { $days } दिनों के भीतर जमा करें।
    }
```

`form-8-processing-note` is why this migration matters beyond tidiness:
Fluent's `{ $days -> [one]/*[other] }` selector gives correct CLDR plural
agreement per locale, which a flat `%s`-style table (the TS prototype's
approach) cannot express — the "real pluralization/gender support" needed
ahead of the 21-language Eighth Schedule rollout.

### 9.2 Rust struct and schema validation

`crates/kb-content` defines the canonical Rust type, with `schemars`
deriving a JSON Schema from it and `serde` handling I/O — both aimed at
staying in lockstep with `entry.schema.json` as schema-of-record, not the
other way around:

```rust
use chrono::NaiveDate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEntry {
    pub id: String,                      // stable slug, e.g. "form-8"; never reused
    pub topic: Topic,
    pub title: String,
    pub summary: String,
    #[serde(default)]
    pub body: Option<String>,            // markdown; nullable at draft, required to leave `draft` (app-layer rule)
    pub applicable_states: Vec<String>,  // state/UT names, or the literal "all"
    pub source_type: SourceType,
    #[serde(default)]
    pub sources: Vec<EntrySource>,
    pub last_verified_date: NaiveDate,
    pub version: u32,
    #[serde(default)]
    pub related_forms: Vec<FormCode>,
    #[serde(default)]
    pub related_entities: Vec<String>,
    pub language: String,                // BCP-47 locale of the inline strings above; see 9.1 for message_ids
    pub review_status: ReviewStatus,
    #[serde(default)]
    pub caution: Option<String>,
    // v2 addition, additive-only: entry.schema.json has no `additionalProperties: false`,
    // so this does not invalidate any existing v1-shaped entry that omits it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_ids: Option<MessageIds>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageIds {
    #[serde(default)] pub title: Option<String>,
    #[serde(default)] pub summary: Option<String>,
    #[serde(default)] pub body: Option<String>,
    #[serde(default)] pub caution: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntrySource {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub publisher: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Topic {
    Registration, Correction, ShiftingOfResidence, DeletionObjection, Epic,
    OrdinaryResidence, NriVoter, ServiceVoter, PwdVoter, QualifyingDates,
    Grievance, PollingStation, RollSearch, Glossary,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    EciOfficial, StateCeo, GazetteLaw, Sveep, PibRelease, CommunityPendingVerification,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub enum FormCode {
    #[serde(rename = "form-6")]   Form6,
    #[serde(rename = "form-6a")]  Form6a,
    #[serde(rename = "form-7")]   Form7,
    #[serde(rename = "form-8")]   Form8,
    #[serde(rename = "form-12d")] Form12d,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Draft, InReview, Verified, NeedsReverification,
}
```

**Validation strategy — two layers, neither replaces the other:**

1. **JSON Schema validation against the file of record.** An `xtask`
   subcommand (`cargo xtask check-kb`, also wired as a `build.rs` step for
   fast local feedback) walks every file in `knowledge-base/sources/*.json`,
   parses it as `serde_json::Value`, and validates it against
   `entry.schema.json` using the `jsonschema` crate. This is what replaces
   the TS prototype's hand-rolled Ajv wrapper — same schema file, now
   validated by a maintained Rust validator with no duplicated rule set.
2. **Struct round-trip test.** A `cargo nextest` integration test in
   `crates/kb-content/tests/` deserializes every file in
   `knowledge-base/sources/*.json` into `KnowledgeEntry`, re-serializes it,
   and asserts value-equality with the original (modulo key order) —
   catching struct/schema drift that step 1 alone would miss (e.g. a
   schema field the struct forgot, or a `serde` rename that happens to
   still produce schema-valid JSON by accident). Both checks run in CI
   before `cargo nextest run` proper; both must pass to merge a KB PR.

`schemars`'s own generated schema is used for API documentation
(`utoipa`/OpenAPI, since `crates/api` exposes KB entries over HTTP) but is
treated as documentation, not a second schema-of-record — it targets a
newer JSON Schema draft than the hand-authored draft-07 file, so
byte-level parity isn't the goal. A lightweight CI check instead asserts
the two schemas agree on the *set of field names and enum values*, which
is what actually guards against silent drift.

### 9.3 Versioning, diff, rollback, and review-status enforcement

Every Content Editor save — body edit, metadata change, or pure status
transition — follows the same transactional path: (1) validate the
proposed state through both 9.2 layers plus `kb-content`'s rule that
`body` and every `sources[].url` must be non-empty before `review_status`
leaves `draft`; (2) write the new state to `knowledge_entries`,
incrementing `version`; (3) insert one `knowledge_entry_revisions` row
(full `snapshot`, `diff_from_previous`, `changed_by`,
`review_status_at_time`) in the same transaction as (2) — `version` never
advances without a matching audit row.

**Rollback** is forward-moving, never a history rewrite: selecting an old
`snapshot` re-runs it through (1)–(3) as a newly proposed state, so rolling
back to v3 from v7 produces v8 with v3's content and the full v1–v8
history stays intact — keeping the DPDP-relevant record ("what did this
say, who approved it, when") permanently reconstructable.

**`reviewStatus` state machine and role gates** (enforced server-side in
`crates/api`, never only in the `crates/admin-app` UI):

```
draft ──────────────► in_review ──────────────► verified
  ▲                        │                        │
  │                        ▼                        ▼
  └──────────── needs_reverification ◄──────────────┘
```

- `draft → in_review`: any `contributor` or above.
- `in_review → verified`: **only `legal_reviewer` or `superadmin`.** The
  sign-off gate defined above for anything touching forms/
  eligibility claims — a `reviewer` can shepherd content through drafting
  but is not the final legal sign-off; a `reviewer` token's request to set
  `verified` is rejected with 403 at the handler, not merely hidden in UI.
- `verified → needs_reverification`: **automatic, never manual.** The
  nightly sweep scans `knowledge_entries WHERE review_status = 'verified'
  AND last_verified_date < now() - interval '{threshold} days'` (default
  180 days, a config value so legal/editorial can tighten it per topic
  without a code change), flips matches, and writes a revision row with
  `change_note` naming the automated trigger and `changed_by` set to a
  reserved system `admin_users` row, so the audit trail is never ambiguous
  about whether a human acted.
- `needs_reverification → in_review` (or straight back to `verified` if a
  `legal_reviewer` re-confirms in one action): any `contributor`/`reviewer`
  can re-open; only `legal_reviewer`/`superadmin` can close it, and doing
  so requires bumping `last_verified_date` to the confirmation date — the
  editor UI blocks `verified` unless `last_verified_date` is "today," so a
  reviewer can't reconfirm without re-checking the source.
- `translator` is scoped to FTL bundles referenced by `messageIds` and a
  per-locale completeness flag only — no access to `sourceType`,
  `sources`, or base-language body, and no ability to move
  `review_status`; a translation-completeness report (`crates/jobs`,
  triggered on KB change) surfaces lagging locales, it is not a status
  translators can self-approve out of.

This state machine, the role table in 8.6, and the append-only
`knowledge_entry_revisions` table together let the admin Content Editor
answer, for any live entry: who wrote it, who approved it into `verified`,
when it was last reconfirmed against its source, and — if currently
`needs_reverification` — exactly how it got there.
## 10. Decision Engine v2 — Expanded Specification

### 10.0 Purpose and Relationship to the MVP Tree

This section specifies the full v1-post-MVP ("v2") decision tree that the
Rust `crates/core-domain` crate must implement, superseding the TypeScript
`@voteassist/decision-engine` package's `voteAssistTreeV1` (see
`packages/decision-engine/src/tree.ts`) as the source of truth going
forward. The TS tree is retained as a reference implementation during
migration, per the architecture decisions in Section 6, and is not deleted.

This is explicitly an **extension**, not a rewrite: every node ID, prompt,
option, and terminal outcome already shipped in `tree.ts` is preserved
unless a specific edge change is called out in section 10.2. The MVP tree
covered the branches documented in `04-decision-tree-spec.md` (A-H) — new
registration (Form 6), overseas registration (Form 6A), service voters
(Form 2), shifting of residence (Form 8), correction of entries (Form 8),
EPIC replacement/e-EPIC, objection/deletion (Form 7), PwD marking + home
voting (Form 8 + Form 12D), and roll/polling-station search. This section
closes the gap between that MVP subset and the full persona list in the
original product brief, reflected in `02-personas.md` and flagged as the
v1-scope item in `19-roadmap.md` ("Decision-tree scope: all major scenarios
enumerated in the original product brief").

**Engine invariants carry over unchanged** (see `04-decision-tree-spec.md`
section 1 and section 3): every path from `start` must reach a `terminal`
node in a bounded number of steps; every `terminal` node must carry at
least one citation or be explicitly labeled "pending verification"; no
`question` node may collect sensitive categories (caste, religion, party
preference); the tree is versioned, and this expansion ships as tree
version 2. In the Rust implementation, these invariants are enforced by
`proptest` property tests in `core-domain` (reachability, termination,
citation-completeness) replacing the TS exhaustive-path `vitest` suite —
per the architecture decision in Section 6, every terminal added below must
have a corresponding property-test case asserting it is reachable and
cited before this tree version can ship.

**Design principle carried forward and made explicit for v2:** where a
persona from the original brief already resolves cleanly to an existing
MVP terminal, this spec says so and adds no new node. New nodes are added
only where a persona's correct outcome genuinely differs from every
existing terminal, or where existing terminals need their `helpText`/
citation set extended so users self-identify into the right existing path.
This keeps the tree's combinatorial size proportional to genuinely distinct
outcomes, not proportional to the number of personas in the brief (several
personas share one outcome; see the coverage matrix in section 10.4).

All new prompt/option/terminal copy below is specified in English only, in
keeping with a spec document; every string requires the same Hindi
human-review pass as MVP content (per `19-roadmap.md`) before this tree
version ships, using the existing `i18n` review workflow (migrating to
`fluent`/FTL per the Rust architecture decision) — this is tracked as an
implementation task, not repeated per-string in this document.

### 10.1 Summary of New/Modified Nodes at a Glance

| # | Cluster | New question/branch nodes | New terminal nodes | Existing nodes/terminals reused or extended |
|---|---|---|---|---|
| 1 | Migrant workers / changing cities & states | `moved_service_voter_check`, `moved_followup_correction_check` | — | `moved_same_or_diff_ac`, `terminal_form8_shift` |
| 2 | Married women changing residence | (shares `moved_followup_correction_check` above) | `terminal_form8_shift_and_correction` | `terminal_form8_shift`, `terminal_form8_correction` |
| 3 | Govt vs. private employees transferred | (shares `moved_service_voter_check` above) | — | `terminal_service_voter`, `terminal_form8_shift` |
| 4 | Senior citizens / postal ballot | `postal_ballot_eligibility_check` | `terminal_senior_postal_ballot` | `terminal_pwd_marking` (extended) |
| 5 | Tribal areas / remote villages / urban slums | (new options on `residence_type`) | `terminal_address_proof_local_verification` | `terminal_form6_no_fixed_address` (pattern reused) |
| 6 | Homeless citizens | — (confirmed, no change) | — | `terminal_form6_no_fixed_address` |
| 7 | Transgender citizens | — | (shares `terminal_form8_shift_and_correction` above) | `terminal_form8_correction` (extended); **NEW KB ENTRY NEEDED** |
| 8 | Missing Aadhaar/passport/DL/permanent address proof | `documents_checklist` (new option on `residence_type`) | `terminal_documents_alternatives` | — |
| 9 | Name mismatch / DOB mismatch | — | — | `terminal_form8_correction` (helpText extended) |
| 10 | Duplicate / deleted voter IDs | `duplicate_or_deleted_check`, `duplicate_entry_type_check`, `deleted_entry_check` | `terminal_deleted_entry_contest` | `terminal_form7`, `terminal_form8_correction`, `terminal_form6_new` |
| 11 | Shifted polling stations (no address change) | (new option on `registered_action`) | `terminal_polling_station_changed` | `terminal_roll_search` (pattern reused) |
| 12 | Missing/lost/damaged EPIC, future eligible voters | — (confirmed, no change) | — | `terminal_eepic_download`, `terminal_form8_epic_replacement`, `terminal_qualifying_date`, `terminal_not_yet_eligible` |

**Totals: 7 new question/branch nodes, 6 new terminal nodes.** See section
10.5 for the full node-count reconciliation.

### 10.2 Edge Changes to the Existing MVP Tree

These are the only modifications to edges that already exist in
`tree.ts`. Every other node, option, and edge in the MVP tree is unchanged.

| Source node.option | Old `next` | New `next` | Reason |
|---|---|---|---|
| `registered_action.moved` | `moved_same_or_diff_ac` | `moved_service_voter_check` | Insert employee/service-voter disambiguation before the existing move flow (cluster 1, 3) |
| `moved_same_or_diff_ac.same` | `terminal_form8_shift` | `moved_followup_correction_check` | Insert combined-correction follow-up before the existing terminal (cluster 1, 2) |
| `moved_same_or_diff_ac.different` | `terminal_form8_shift` | `moved_followup_correction_check` | Same as above |
| `registered_action.pwd` | `terminal_pwd_marking` | `postal_ballot_eligibility_check` | Route through the new shared PwD/senior postal-ballot concept (cluster 4) |
| `registered_action.object_delete` | `terminal_form7` | `duplicate_or_deleted_check` | Disambiguate duplicate vs. wrongful-third-party-entry vs. own-deletion cases (cluster 10) |

New options added to existing nodes (additions only, no existing option is
removed or retargeted):

| Node | New option value | Label (en) | `next` |
|---|---|---|---|
| `registered_action` | `polling_station_changed` | "My polling station changed but I didn't move" | `terminal_polling_station_changed` (cluster 11) |
| `residence_type` | `tribal_remote` | "A tribal area or remote village" | `terminal_address_proof_local_verification` (cluster 5) |
| `residence_type` | `urban_slum` | "An urban slum / informal settlement" | `terminal_address_proof_local_verification` (cluster 5) |
| `residence_type` | `unsure_documents` | "I have an address, but I'm not sure my documents are the 'standard' ones asked for" | `documents_checklist` (cluster 8) |

### 10.3 Scenario Cluster 1: Migrant Workers / People Changing Cities or States

**Personas covered:** migrant workers, people changing cities, people
changing states (also: Mohammed persona, `02-personas.md` #3).

**Key product-copy decision:** interstate vs. intrastate relocation has
**no legal difference** for Form 8 purposes — the only legally relevant
distinction is same-Assembly-Constituency vs. different-Assembly-
Constituency (which can occur even within one city, or conversely a move
across a state border can occasionally leave someone in the same AC in
rare boundary cases, though this is unusual). This is a common user
misconception worth addressing head-on in UI copy rather than silently
routing around it.

**Node: `moved_service_voter_check`** (question) — new entry point for the
"moved" flow.
- Prompt (en): "Is this move because you were posted/transferred by the
  Armed Forces or a notified government service — as opposed to a general
  government job, PSU, or private-sector transfer?"
- helpText (en): "This only matters for a small, specific category of
  government service. If you're not sure, choose 'No' — that covers the
  large majority of transfers, government or private."
- Options:
  | value | label (en) | next |
  |---|---|---|
  | `yes` | "Yes — Armed Forces / notified service posting" | `terminal_service_voter` (existing, reused) |
  | `no` | "No — general government, PSU, or private-sector transfer, or I moved for a non-work reason" | `moved_same_or_diff_ac` (existing, unchanged) |

**Node: `moved_same_or_diff_ac`** (existing, prompt unchanged). Its
`helpText` should be extended with one additional sentence: "Moving to a
different state always means a different Assembly Constituency, but
moving within the same state or even the same city can also mean a
different AC — and the legal process (Form 8) is identical either way.
There is no separate 'interstate' form or procedure."

**Node: `moved_followup_correction_check`** (question) — new node
interposed before the existing shift terminal.
- Prompt (en): "Besides your address, do you also need to correct any
  other detail on your entry — such as your name, date of birth, photo, or
  gender marker?"
- Options:
  | value | label (en) | next |
  |---|---|---|
  | `no` | "No, just the address" | `terminal_form8_shift` (existing, unchanged) |
  | `yes` | "Yes, I also need a correction" | `terminal_form8_shift_and_correction` (new terminal) |

**Terminal outcome (existing, reused):** `terminal_form8_shift` — no
content change beyond the `helpText` note above on
`moved_same_or_diff_ac`.

**Citations:** `form-8` (existing, unchanged).

**Explicit note:** no new terminal is needed for "migrant worker" or
"changing states" as distinct personas — this cluster is entirely a
generalization/copy-clarification of the existing `moved_same_or_diff_ac`
→ `terminal_form8_shift` path, plus the two new interposed nodes shared
with clusters 2 and 3 below.

### 10.4 Scenario Cluster 2: Married Women Changing Residence

**Persona covered:** married women changing residence after marriage (also:
Kavita persona, `02-personas.md` #4).

This is **not a new form or a new terminal category** — it is the existing
shifting-of-residence flow (`terminal_form8_shift`) combined with the
existing correction-of-entries flow (`terminal_form8_correction`) when a
name change is also involved, surfaced through the generic
`moved_followup_correction_check` node introduced in cluster 1. Deliberately,
this node is framed around "did you also change any other detail," not
"are you a married woman" — marriage-driven relocation-plus-name-change is
the single most common real-world instance of this combination, but the
same combined path applies equally to anyone else moving and correcting a
detail at the same time (including the transgender-citizen scenario in
cluster 7). A dedicated "married women" question or form would be
unnecessary special-casing that a careful reviewer should reject, since
Form 8 already natively supports submitting a residence-shift and a
correction together.

**Terminal: `terminal_form8_shift_and_correction`** (new)
- `outcomeTitle` (en): "You need Form 8, covering both your address change
  and your correction, in one submission"
- `outcomeDescription` (en): "Form 8 covers shifting of residence and
  correction of entries (name, date of birth, photo, gender marker,
  relative's name) in the same form — you do not need to submit two
  separate applications or wait for one to finish before starting the
  other."
- `recommendedForms`: `["form-8"]`
- Checklist:
  - "Gather proof of ordinary residence at your new address."
  - "Gather a supporting document for whichever detail you're correcting
    (e.g., marriage certificate or gazette notification for a name
    change)."
  - "Submit a single Form 8 selecting both the 'shifting of residence' and
    the relevant correction option."
- Citations: `form-8`.
- `caution` (en): "Confirm on the official portal that both changes can
  currently be submitted in a single Form 8 session in your state — the
  online workflow for combined requests can vary; if the portal only
  accepts one change type per submission, submit the address shift first
  and the correction as a prompt follow-up, or ask your BLO to help file
  both together."

### 10.5 Scenario Cluster 3: Government Employees Transferred vs. Private Employees Transferred

**Personas covered:** government employees transferred, private employees
transferred (also: Rahul persona, `02-personas.md` #5).

Functionally, both are **identical to the general "moved" flow** — Form 8,
shifting of residence — with one clarifying branch: a government or
private employee's ordinary job transfer never by itself grants
service-voter eligibility under Form 2. Service-voter status is reserved
for specified categories (Armed Forces personnel and members of a force to
which the Army Act applies, or an equivalent notified service), not "any
government employee who relocates for work." This is exactly the branch
already introduced as `moved_service_voter_check` in cluster 1 — no
additional node is needed for this cluster; this subsection exists to make
the private-vs-government distinction explicit in the spec (per the task
brief) rather than to introduce new tree structure.

**Explicit UI-copy requirement on the `no` option of
`moved_service_voter_check`:** the option's `helpText`, when the "no" path
is taken, must state plainly: "Private-sector employment transfers — no
matter how senior the role or how far the transfer — do not qualify for
service-voter registration under Form 2. Only the specific government/
armed-forces categories above do. If that's not you, this is a standard
residence-shift case." This directly targets the misconception the brief
calls out.

**Terminal outcomes:** both government (non-service-voter category) and
private-sector transfers resolve to `terminal_form8_shift` (or
`terminal_form8_shift_and_correction` if combined with another correction,
per cluster 2) — reused, unmodified. Government/armed-forces personnel who
answer "yes" resolve to the existing `terminal_service_voter`.

**Extension to `terminal_service_voter`:** its `helpText`/checklist should
gain one additional bullet for the "already an ordinary elector elsewhere,
now transferred" case: "If you're already registered as an ordinary
elector at a previous address, ask your service unit's records office and
your ERO whether that old entry needs to be formally addressed (e.g., via
Form 7) or is superseded automatically when your service-voter entry is
created — this transition process is not fully documented in public ECI
guidance and should be confirmed case-by-case (see also the retired-
service-voter case in `02-personas.md` persona #7, Colonel Verma, which
flags the same open question)." No new citation is invented for this;
existing `service-voter` KB entry citation stands, with the caution
carried over from `06-legal-compliance-review.md`'s existing treatment of
this gap.

### 10.6 Scenario Cluster 4: Senior Citizens and the Shared "Postal Ballot Eligibility" Concept

**Persona covered:** senior citizens (also: Kamla Devi persona,
`02-personas.md` #9, for the EPIC-loss half of her situation, and Deepa
persona, `02-personas.md` #8, for the PwD half).

Per verified ECI guidance, both PwD electors and electors aged 85+ may use
postal ballot / home voting via Form 12D, submitted to the Returning
Officer within 5 days of an election's notification — this is a single
shared eligibility concept with two qualifying routes, one of which (PwD)
requires a one-time roll marking via Form 8 first, and one of which
(age 85+) requires no separate marking step at all, since date of birth is
already on the roll from registration.

**Node: `postal_ballot_eligibility_check`** (question) — new node, now the
target of `registered_action.pwd` (edge change, section 10.2).
- Prompt (en): "Which applies to you?"
- helpText (en): "Postal ballot / home voting at election time (Form 12D)
  is available to both persons with disabilities and electors aged 85 or
  older — the paperwork differs slightly between the two."
- Options:
  | value | label (en) | next |
  |---|---|---|
  | `pwd` | "I have a disability and want to be marked as PwD on the roll" | `terminal_pwd_marking` (existing, reused) |
  | `senior_85` | "I am, or will soon be, 85 or older" | `terminal_senior_postal_ballot` (new terminal) |
  | `both` | "Both — I have a disability and I'm 85 or older" | `terminal_pwd_marking` (existing, reused, extended helpText) |

**Extension to `terminal_pwd_marking`:** add one checklist bullet: "If
you're also 85 or older, you do not need any separate roll-marking step
for the age-based eligibility — it applies automatically based on the date
of birth already on your roll entry. The PwD marking above is only needed
for the disability-based route." Citations unchanged (`form-8`,
`pwd-home-voting`).

**Terminal: `terminal_senior_postal_ballot`** (new)
- `outcomeTitle` (en): "At 85 or older, you may request postal ballot /
  home voting directly — no roll-marking step needed"
- `outcomeDescription` (en): "Unlike the PwD route, there is no separate
  one-time Form 8 marking for age-based postal-ballot eligibility — your
  date of birth is already on the electoral roll. When an election is
  announced in your constituency, watch for the Form 12D window."
- `recommendedForms`: `["form-12d"]` — **note:** Form 12D is not in the
  existing enumerated form set in `04-decision-tree-spec.md` section 3
  (`none, Form 6, Form 6A, Form 7, Form 8, Form 12D, or consult BLO/1950`);
  it is already listed there, so this is consistent, not a new invented
  form — this is simply the first v2 terminal whose primary recommendation
  is Form 12D itself, rather than Form 8 with Form 12D mentioned
  secondarily as in `terminal_pwd_marking`.
- Checklist:
  - "Confirm your date of birth is correctly recorded on your roll entry
    (if not, that's a Form 8 correction first)."
  - "When an election is notified in your constituency, submit Form 12D to
    the Returning Officer within 5 days of that notification."
  - "Form 12D requests either a postal ballot or a home-visit voting
    facility, depending on current election-specific arrangements."
- Citations: `pwd-home-voting`.
- `caution` (en): "Exact home-voting eligibility age thresholds and
  procedures are set per election by ECI notification — confirm the
  current criteria close to the relevant election, the same caution
  already applied to the PwD route."

### 10.7 Scenario Cluster 5: Tribal Areas, Remote Villages, Urban Slums

**Personas covered:** tribal areas, remote villages, urban slums (also:
Birsa persona, `02-personas.md` #13).

These populations typically **do have** a fixed address — the issue is
that acceptable proof-of-ordinary-residence documentation for that address
varies by state and by local ERO/BLO practice in ways this PRD will not
invent a specific accepted-document list for. This is distinct from
cluster 6 (no fixed address at all) and from cluster 8 (has an address,
lacks a specific ID document like Aadhaar) — this cluster's issue is proof
of an address that exists but that formal paperwork may not evidence in
the way a standard rental agreement or utility bill would (e.g., communally
held land, informal tribal-block housing, unaddressed slum dwellings).

**New options on `residence_type`** (edge additions, section 10.2):
`tribal_remote` and `urban_slum`, both routing to a single new terminal.

**Terminal: `terminal_address_proof_local_verification`** (new)
- `outcomeTitle` (en): "Address-proof flexibility here depends on your
  local ERO — flagging this for direct BLO/ERO contact"
- `outcomeDescription` (en): "In tribal areas, remote villages, and urban
  slums or informal settlements, the electoral registration process
  (Form 6) is the same, but which documents count as proof of ordinary
  residence can be handled with more flexibility by the local Electoral
  Registration Officer than the standard document list assumes. VoteAssist
  India does not yet have verified, state-specific guidance on what your
  local ERO currently accepts here — the safest next step is direct
  contact."
- `recommendedForms`: `["form-6"]`
- Checklist:
  - "Call the National Voter Helpline (1950) or use the Voter Helpline
    App's 'Book-a-Call with BLO' feature."
  - "Ask specifically what your local BLO/ERO currently accepts as proof
    of ordinary residence in your area."
  - "If you have any of the standard documents (Aadhaar, ration card,
    utility bill, even informal ones), bring them — a partial document set
    is still useful context for the ERO even if none of it is the
    'complete' standard bundle."
- Citations: `helpline-grievance`, `form-6`.
- `caution` (en): "This is the same honest 'flag for local verification'
  pattern already used for the no-fixed-address terminal
  (`terminal_form6_no_fixed_address`) — it is a pointer to the right
  official contact, not a complete procedure. Exhaustive per-state accepted-
  document research for tribal areas, remote villages, and urban slums is
  explicitly out of scope for this PRD and is tracked as a v2/v3 roadmap
  item requiring dedicated field research, consistent with the existing
  `19-roadmap.md` state-by-state legal-review prerequisite."

**NEW KB ENTRY NEEDED (flagged, not fabricated):** "Per-state accepted
address-proof documents for tribal/scheduled areas, remote/hill villages,
and urban informal settlements — requires state-by-state field research
with CEO offices and/or district ERO circulars; cannot be sourced from a
single central ECI document." This is listed here for completeness but
deliberately not assigned a KB id or specifics, per this PRD's style
rule against inventing facts.

### 10.8 Scenario Cluster 6: Homeless Citizens (Where Legally Applicable)

**Persona covered:** homeless citizens where legally applicable.

**No new node or terminal is needed.** This scenario is **already solved**
by the existing MVP `residence_type.no_fixed_address` option, which routes
to `terminal_form6_no_fixed_address` — that terminal's existing
`outcomeDescription` in `tree.ts` already explicitly reads "Registration
rules for citizens without a fixed or permanent address (including
homeless citizens, where legally applicable)..." and follows the identical
"flag for local verification, contact BLO/1950" pattern specified for
cluster 5 above. This cluster is called out here only to confirm, for the
PRD reader's benefit, that it requires no v2 work — it is cited as the
template that clusters 5, 10 (deleted-entry contest), and 11 pattern-match
against.

### 10.9 Scenario Cluster 7: Transgender Citizens

**Persona covered:** transgender citizens (also: Noor persona,
`02-personas.md` #12).

The mechanical path already exists: `terminal_form8_correction`'s
`outcomeDescription` already lists "gender" among the correctable fields,
and if the citizen has also moved, the new `terminal_form8_shift_and_correction`
outcome from cluster 2 covers the combined update in one Form 8 submission
— exactly Noor's stated goal of not having to "explain myself over and
over" through separate processes.

**Extension to `terminal_form8_correction`'s `helpText`:** add a sentence
making the gender-marker path explicit and self-identifying, so
transgender users recognize this as their path without having to infer it
from a generic field list: "This includes updating your recorded gender
marker (including a third-gender/'Other' option where offered) to match
your self-identified gender — this is treated the same as any other
correction of entries, with the same Form 8 process."

**NEW KB ENTRY NEEDED:** "Self-identified gender-marker update process on
Form 8/Form 6 — verify (a) the current gender field options exposed on the
official portal and paper form (Male/Female/Third Gender/'Other', and
whether the label and legal recognition of a third-gender option is
uniform nationally or state-variable); (b) whether ECI has issued any
dedicated circular or guidance specifically on transgender elector
registration/correction (tone, required documentation, whether a
self-declaration suffices or whether external certification, e.g., under
the Transgender Persons (Protection of Rights) Act 2019 gender-recognition
certificate, is required or merely optional); (c) whether combining a
gender correction with an address-shift in one Form 8 submission is
supported end-to-end in the current online workflow, not just on paper."
This entry requires legal/content research before any specific document
list or procedure is published in-product — until it is verified, the
`terminal_form8_correction` outcome must not assert specific required
documents for the gender-marker sub-case beyond "a supporting document
showing the correct detail," which is already the existing generic
language and is left unchanged.

**Explicit note:** no separate "transgender" terminal, form, or dedicated
question node is introduced. Per the task brief's own caution against
inventing specifics without confidence, this cluster is intentionally a
"confirm/extend existing node + flag research need" cluster, not a new
branch.

### 10.10 Scenario Cluster 8: Citizens Without Aadhaar / Passport / Driving Licence / Permanent Address (Documents)

**Personas covered:** citizens without Aadhaar, citizens without passport,
citizens without driving licence, citizens without permanent address (the
"has an address, lacks standard proof" variant — see cluster 5/6 for the
"no address at all" variant) (also: Nasreen persona, `02-personas.md`
#14).

**Design choice, stated explicitly:** rather than branching the tree once
per possible missing-document combination (Aadhaar-only-missing,
passport-only-missing, Aadhaar+DL missing, all-four missing, etc. — a
combinatorial explosion of 2⁴−1 = 15+ distinct branches for just four
document types, worse once ration card, utility bill, bank passbook, etc.
are added), this cluster uses a single **multi-select checklist question**
that asks what the user *does* have, feeding one shared terminal. The
terminal's guidance is the same regardless of which specific subset is
missing: no single document is uniquely mandatory, a combination of
whatever is available is generally usable, and the authoritative accepted-
document list must be confirmed on the current official portal rather than
assumed from this spec. This "checklist of what you DO have" pattern keeps
the tree's size proportional to the number of *distinct outcomes* (one)
rather than the number of *input combinations* (many), consistent with the
overall v2 design principle in section 10.0.

**Node: `documents_checklist`** (question, multi-select) — new node, target
of the new `residence_type.unsure_documents` option (section 10.2).
- Prompt (en): "Which of these do you currently have? Select all that
  apply."
- Options (multi-select, all lead to the same `next` once submitted):
  - Aadhaar card
  - Passport
  - Driving licence
  - Ration card
  - Bank or post office passbook/statement showing your address
  - A utility bill (electricity/water/gas/telephone) in your name or a
    close relative's
  - A rent agreement or similar tenancy document
  - None of the above
- `next` (all selections): `terminal_documents_alternatives`
- helpText (en): "You do not need all of these — this just helps us give
  you the most relevant guidance for what you already have."

**Terminal: `terminal_documents_alternatives`** (new)
- `outcomeTitle` (en): "You can likely still register — the accepted
  document list is broader than 'Aadhaar or passport only'"
- `outcomeDescription` (en): "Registration (Form 6) does not require any
  one specific document like Aadhaar or a passport. A range of documents
  can serve as proof of age and proof of ordinary residence, and the
  currently accepted list is set and updated by ECI on the official
  portal — it is broader than the small set of documents people often
  assume are mandatory."
- `recommendedForms`: `["form-6"]`
- Checklist:
  - "Check the current Form 6 instructions on the official portal for the
    full accepted-document list before assuming you're excluded."
  - "Bring whatever you selected above, even if it doesn't feel like
    'enough' — partial documentation plus a BLO conversation is often
    sufficient."
  - "If you selected 'None of the above,' contact your local BLO/ERO or
    call 1950 directly — do not assume registration is impossible without
    confirming with them first."
- Citations: `form-6`.
- `caution` (en): "This spec deliberately does not enumerate a specific
  accepted-document list here, because that list is set by ECI, updated
  periodically, and must be confirmed against the current official Form 6
  instructions rather than hardcoded in this product — hardcoding it would
  risk the product asserting a stale or wrong requirement."

**Cross-reference, not a new node:** the Form 8 terminals (`terminal_form8_shift`,
`terminal_form8_correction`, `terminal_form8_epic_replacement`,
`terminal_pwd_marking`) should each gain a one-line cross-reference in
their `helpText` along the lines of "Not sure your documents qualify? See
the documents guidance above" rather than each spawning their own copy of
the `documents_checklist` node — this avoids duplicating the same
node/terminal pair four more times for no added value, per the same
anti-combinatorial-explosion principle.

**Distinguishing "no permanent address" clusters:** "citizens without
permanent address" maps to *two different* existing/new terminals
depending on which is true, and the tree must route based on that
distinction rather than merging them:
- Truly no fixed/permanent address at all (nomadic, homeless, itinerant) →
  existing `residence_type.no_fixed_address` → `terminal_form6_no_fixed_address`
  (cluster 6, unchanged).
- Has a place they live, just lacks the standard paperwork proving it →
  new `residence_type.unsure_documents` → `documents_checklist` →
  `terminal_documents_alternatives` (this cluster).

### 10.11 Scenario Cluster 9: Name Mismatch / DOB Mismatch

**Personas covered:** people with name mismatch, people with DOB mismatch,
electors requiring corrections generally (also: Suresh persona,
`02-personas.md` #10).

**No new node or terminal.** Both are already fully covered by the existing
`terminal_form8_correction` outcome. The only v2 change is to its
`helpText`, extended so users confidently self-identify into this path
without wondering whether a name or DOB discrepancy needs some different,
more drastic process (a common fear, per the Suresh persona's barrier:
"Assumes correcting his name will require re-registering from scratch").

**Extension to `terminal_form8_correction`'s `helpText`:** "This covers
both a misspelled or outdated name and an incorrect date of birth on your
existing entry — in both cases you are correcting one field on your
current record, not re-registering from scratch, and you keep the same
EPIC number and roll continuity." (This sits alongside the gender-marker
sentence added in cluster 7 — the node ends up with three explicit
self-identification sentences covering name, DOB, and gender, plus its
existing coverage of photo and relative's-name corrections.)

**Citations:** unchanged (`form-8`).

### 10.12 Scenario Cluster 10: Duplicate Voter IDs / Deleted Voter IDs

**Personas covered:** duplicate voter IDs, deleted voter IDs (also: Farida
persona, `02-personas.md` #11).

This cluster replaces the MVP's single `registered_action.object_delete` →
`terminal_form7` edge with a short disambiguation chain, because "duplicate"
and "deleted" cover four legally distinct situations that the MVP
collapsed into one option:

1. Two separate roll entries exist for the same person (a true duplicate)
   → the erroneous extra entry is objected to/deleted via **Form 7**.
2. What looks like a duplicate is actually one entry with inconsistent or
   wrong details (e.g., a records-matching quirk, not really two people/
   two entries) → this is a **Form 8 correction**, not a Form 7 objection.
3. A third party's entry shouldn't be on the roll (deceased, moved away,
   otherwise ineligible) → **Form 7**, unchanged from MVP behavior.
4. The user's own entry was deleted and they want to (a) simply register
   again, or (b) contest that the deletion itself was wrongful.

**Node: `duplicate_or_deleted_check`** (question) — new node, target of the
`registered_action.object_delete` edge change (section 10.2).
- Prompt (en): "What's the situation?"
- Options:
  | value | label (en) | next |
  |---|---|---|
  | `duplicate` | "There seem to be two entries for the same person (me or a family member)" | `duplicate_entry_type_check` (new node) |
  | `someone_else_ineligible` | "Someone else's entry shouldn't be on the roll (deceased, moved away, or otherwise ineligible)" | `terminal_form7` (existing, unchanged) |
  | `i_was_deleted` | "My own name was removed from the roll and I need to act on that" | `deleted_entry_check` (new node) |

**Node: `duplicate_entry_type_check`** (question) — new node.
- Prompt (en): "Are there genuinely two separate roll entries for the same
  person, or is it one entry whose details look inconsistent or wrong?"
- helpText (en): "If you're not sure, search the roll by name/EPIC number
  for both entries first — if you can find two distinct entries, it's the
  first case; if you only find one but something about it looks
  contradictory, it's the second."
- Options:
  | value | label (en) | next |
  |---|---|---|
  | `two_entries` | "Two separate entries — one needs to be removed" | `terminal_form7` (existing, reused) |
  | `one_entry_wrong_details` | "One entry, but the details are wrong or inconsistent" | `terminal_form8_correction` (existing, reused) |

**Extension to `terminal_form7`'s checklist:** add a bullet distinguishing
the two Form-7 paths that now feed into it: "If this is about a duplicate
entry for yourself or a family member, the evidence needed is proof that
the extra entry is the same person as an existing, correct entry (e.g.,
matching EPIC/roll details) — different from the deceased/moved-away case,
which needs a death certificate or equivalent."

**Node: `deleted_entry_check`** (question) — new node.
- Prompt (en): "Do you want to register again since you're no longer on
  the roll, or do you believe the deletion itself was wrong and want to
  contest it?"
- Options:
  | value | label (en) | next |
  |---|---|---|
  | `reregister` | "Register again, as if new" | `terminal_form6_new` (existing, reused) |
  | `contest_wrongful` | "I believe the deletion was wrong and I want to contest it" | `terminal_deleted_entry_contest` (new terminal) |

**Extension to `terminal_form6_new` for the re-registration sub-case:** add
a checklist bullet: "A prior deletion does not disqualify you from
registering again — treat this exactly like a first-time Form 6
registration at your current address."

**Terminal: `terminal_deleted_entry_contest`** (new)
- `outcomeTitle` (en): "Contesting a wrongful deletion needs direct
  BLO/ERO or grievance-portal contact"
- `outcomeDescription` (en): "Form 7 is documented by ECI as covering
  objection to inclusion of a name, or a claim for deletion of a name —
  it is not clearly documented as also covering the reverse case, i.e.,
  restoring or contesting the removal of your own already-deleted entry.
  VoteAssist India does not have verified guidance on the correct
  instrument for this specific situation, so the safest next step is
  direct contact with your ERO or the national grievance channel, rather
  than us guessing at a procedure."
- `recommendedForms`: `[]` (no form asserted, per the same pattern as
  `terminal_form6_no_fixed_address`)
- Checklist:
  - "Call the National Voter Helpline (1950) or file a grievance via the
    National Grievance Service Portal (NGSP 2.0), which EROs/DEOs/CEOs are
    directed to resolve within 48 hours."
  - "Ask specifically how to contest a roll deletion you believe was made
    in error, and whether that requires a fresh Form 6 registration, a
    representation to the ERO, or another mechanism."
  - "If the deletion happened as part of a special intensive revision or
    similar roll-cleanup exercise, mention that explicitly — the
    applicable appeal window and process may be specific to that exercise."
- Citations: `helpline-grievance`.
- `caution` (en): "**TODO: verify** against official ECI guidance whether
  Form 7, a distinct claims/appeal mechanism during roll revision, or
  simple re-registration (Form 6) is the correct instrument for contesting
  a wrongful deletion, before this outcome asserts a specific form. Until
  verified, this terminal intentionally stops at 'contact your BLO/ERO
  directly,' the same honest pattern already used for
  `terminal_form6_no_fixed_address`, rather than fabricating a procedure
  ECI hasn't documented."

### 10.13 Scenario Cluster 11: Shifted Polling Stations (No Address Change)

**Persona covered:** shifted polling stations.

This is the case where delimitation, redistricting, or a local
administrative change moves which polling station serves an address that
the elector never changed — **not** a Form 8 scenario, since the elector
did not move. This is a good example of the same principle stated
elsewhere in this spec: don't invent a form where none is needed. This is
purely an informational "go find your new polling station" outcome,
reusing the exact pattern already established by `terminal_roll_search`.

**New option on `registered_action`** (edge addition, section 10.2):
`polling_station_changed`, label "My polling station changed but I didn't
move," routing to the new terminal below.

**Terminal: `terminal_polling_station_changed`** (new)
- `outcomeTitle` (en): "Find your current polling station — no application
  needed"
- `outcomeDescription` (en): "Polling stations are sometimes reassigned by
  the Election Commission (for example after delimitation or local
  administrative changes) even when you haven't moved. If your usual
  polling station has changed, you don't need to file anything — you just
  need to look up where your (unchanged) registration now points."
- `recommendedForms`: `[]`
- Checklist:
  - "Search the electoral roll using your EPIC number to see your current
    polling station."
  - "If your address, constituency, and other details are unchanged and
    only the polling station differs from what you remember, no form is
    required."
  - "If, on checking, you find your address or constituency actually did
    change, treat that as the 'I moved' case instead (Form 8)."
- Citations: `roll-search-polling-station`.
- `caution` (en): "Confirm this by checking the roll directly — this
  outcome assumes the underlying registration is otherwise correct and
  only the assigned polling station changed; if anything else looks wrong,
  route back into the correction or shift flows instead."

**Explicit note:** this terminal deliberately mirrors `terminal_roll_search`
in structure (no form, pure lookup-and-confirm guidance) rather than being
folded into it, because the entry question differs meaningfully (a
registered user reporting a specific observed change vs. a user who
doesn't know their status at all) — but the underlying action for the user
is the same roll/polling-station search.

### 10.14 Scenario Cluster 12: Missing/Lost/Damaged EPIC, Future Eligible Voters — Confirmed Already Solved

**Personas covered:** missing EPIC, lost voter card, damaged EPIC, new
18-year-olds, future eligible voters.

**No new nodes or terminals.** These already map cleanly to existing MVP
terminals, and this cluster exists in the spec purely to confirm that
explicitly for the PRD reader, distinguishing what's already solved from
what's new in this document:

| Scenario | Existing path | Existing terminal |
|---|---|---|
| Missing / never received EPIC | `registered_action.lost_epic` → `lost_or_damaged_epic` | `terminal_eepic_download` or `terminal_form8_epic_replacement` |
| Lost voter card | Same as above | Same as above |
| Damaged EPIC | Same as above | Same as above |
| New 18-year-olds (just became eligible) | `not_registered_age.turning_18_soon` or `.adult` → `citizenship_check` → `residence_type` | `terminal_qualifying_date` or `terminal_form6_new` |
| Future eligible voters (will turn 18 later) | `not_registered_age.turning_18_soon` | `terminal_qualifying_date` |
| Not yet eligible at all | `not_registered_age.under_18` | `terminal_not_yet_eligible` |

No content changes are proposed for any of these six terminals in this
v2 pass.

### 10.15 Full Persona Coverage Matrix

Every persona named in the original product brief, mapped to its resolving
path. "MVP" = already covered, unchanged, in `tree.ts`. "v2" = newly
specified or newly extended in this document.

| # | Persona (from brief) | Status | Resolving path (entry → ... → terminal) |
|---|---|---|---|
| 1 | First-time voters | MVP | `not_registered_age` → `citizenship_check` → `residence_type` → `terminal_form6_new` |
| 2 | Students | MVP | `residence_type.student_hostel` → `student_ordinary_residence_choice` → `terminal_form6_new` / `terminal_form6_student_hostel` |
| 3 | Migrant workers | v2 (cluster 1) | `registered_action.moved` → `moved_service_voter_check` → `moved_same_or_diff_ac` → `moved_followup_correction_check` → `terminal_form8_shift` |
| 4 | Hostel residents | MVP | Same as #2 |
| 5 | PG residents | MVP | `residence_type.rented` → `terminal_form6_new` |
| 6 | Renters | MVP | Same as #5 |
| 7 | People changing cities | v2 (cluster 1) | Same as #3 |
| 8 | People changing states | v2 (cluster 1) | Same as #3, with explicit "no legal difference" copy |
| 9 | Married women changing residence | v2 (cluster 2) | Same as #3, ending at `terminal_form8_shift_and_correction` if a name change is also needed |
| 10 | Government employees transferred | v2 (cluster 3) | `moved_service_voter_check` (`no`) → `moved_same_or_diff_ac` → `terminal_form8_shift` |
| 11 | Private employees transferred | v2 (cluster 3) | Same as #10, with explicit never-service-voter copy |
| 12 | NRI voters | MVP | `citizenship_check.nri` → `terminal_form6a` |
| 13 | Service voters | MVP + v2 extension | `citizenship_check.service` → `terminal_service_voter`; also `moved_service_voter_check` (`yes`) for already-registered transferees (cluster 3) |
| 14 | Persons with disabilities | MVP + v2 extension | `registered_action.pwd` → `postal_ballot_eligibility_check` → `terminal_pwd_marking` |
| 15 | Senior citizens | v2 (cluster 4) | `postal_ballot_eligibility_check.senior_85` → `terminal_senior_postal_ballot`; EPIC-loss half via MVP `lost_or_damaged_epic` |
| 16 | Tribal areas | v2 (cluster 5) | `residence_type.tribal_remote` → `terminal_address_proof_local_verification` |
| 17 | Remote villages | v2 (cluster 5) | Same as #16 |
| 18 | Urban slums | v2 (cluster 5) | `residence_type.urban_slum` → `terminal_address_proof_local_verification` |
| 19 | Homeless citizens (where legally applicable) | MVP (confirmed, cluster 6) | `residence_type.no_fixed_address` → `terminal_form6_no_fixed_address` |
| 20 | Transgender citizens | v2 extension + KB flag (cluster 7) | `terminal_form8_correction` (extended); `terminal_form8_shift_and_correction` if also moving |
| 21 | Citizens without Aadhaar | v2 (cluster 8) | `residence_type.unsure_documents` → `documents_checklist` → `terminal_documents_alternatives` |
| 22 | Citizens without passport | v2 (cluster 8) | Same as #21 |
| 23 | Citizens without driving licence | v2 (cluster 8) | Same as #21 |
| 24 | Citizens without permanent address | MVP or v2, depending | `terminal_form6_no_fixed_address` (no address at all) or `terminal_documents_alternatives` (has address, lacks proof) |
| 25 | People with name mismatch | MVP (helpText extended, cluster 9) | `registered_action.correction` → `terminal_form8_correction` |
| 26 | People with DOB mismatch | MVP (helpText extended, cluster 9) | Same as #25 |
| 27 | Duplicate voter IDs | v2 (cluster 10) | `registered_action.object_delete` → `duplicate_or_deleted_check` → `duplicate_entry_type_check` → `terminal_form7` or `terminal_form8_correction` |
| 28 | Deleted voter IDs | v2 (cluster 10) | `duplicate_or_deleted_check` → `deleted_entry_check` → `terminal_form6_new` or `terminal_deleted_entry_contest` |
| 29 | Shifted polling stations | v2 (cluster 11) | `registered_action.polling_station_changed` → `terminal_polling_station_changed` |
| 30 | Missing EPIC | MVP (confirmed, cluster 12) | `lost_or_damaged_epic` → `terminal_eepic_download` / `terminal_form8_epic_replacement` |
| 31 | Lost voter card | MVP (confirmed, cluster 12) | Same as #30 |
| 32 | Damaged EPIC | MVP (confirmed, cluster 12) | Same as #30 |
| 33 | Electors requiring corrections | MVP | `terminal_form8_correction` |
| 34 | New 18-year-olds | MVP (confirmed, cluster 12) | `not_registered_age` → `terminal_qualifying_date` |
| 35 | Future eligible voters | MVP (confirmed, cluster 12) | `not_registered_age.under_18` → `terminal_not_yet_eligible`, or `.turning_18_soon` → `terminal_qualifying_date` |

Every persona in the brief resolves to a documented path above; none
requires a fabricated procedure, and the four genuinely under-documented
cases (tribal/remote/slum address proof, transgender gender-marker
specifics, wrongful-deletion contest, and no-fixed-address generally) all
terminate honestly in a "consult BLO/ERO/1950" or "NEW KB ENTRY NEEDED"
outcome rather than an invented one.

### 10.16 v2 Tree Size Estimate

**MVP baseline:** the current `tree.ts` contains 8 question/branch nodes
(`start`, `registered_action`, `moved_same_or_diff_ac`,
`lost_or_damaged_epic`, `not_registered_age`, `citizenship_check`,
`residence_type`, `student_ordinary_residence_choice`) and 14 terminal
nodes, for 22 nodes total (the task brief's "~21 nodes" characterization is
consistent with this, depending on whether `start` is counted separately).

**v2 additions specified in this document:**

| Type | Count | Node IDs |
|---|---|---|
| New question/branch nodes | 7 | `moved_service_voter_check`, `moved_followup_correction_check`, `postal_ballot_eligibility_check`, `documents_checklist`, `duplicate_or_deleted_check`, `duplicate_entry_type_check`, `deleted_entry_check` |
| New terminal nodes | 6 | see table below |
| Existing nodes with extended `helpText`/checklist only (no structural change) | 8 | `terminal_form8_correction`, `terminal_pwd_marking`, `terminal_form7`, `terminal_form6_new`, `terminal_service_voter`, `moved_same_or_diff_ac`, `registered_action` (new options, no retarget), `residence_type` (new options, no retarget) |
| Existing edges retargeted through a new node | 5 | see section 10.2 |

**v2 total node count: 22 (MVP) + 7 (new question/branch) + 6 (new
terminal) = 35 nodes.**

This is toward the lower end of the "45-70 node" range floated as a rough
estimate in the task brief, and that gap is deliberate, not a shortfall:
several of the largest personas clusters in the original brief (students/
hostel/PG/renters, name-mismatch/DOB-mismatch, homeless, missing/lost/
damaged EPIC, new-18/future-eligible, married women, government/private
employee transfers, senior-citizen EPIC-loss) are explicitly documented
above as resolving to **existing** MVP terminals with at most a helpText
extension — not new nodes. A brute-force approach that gave every named
persona its own branch would produce a much larger, harder-to-maintain
tree with many nodes that are functionally identical to each other (e.g.,
a separate "government employee moved" terminal and "private employee
moved" terminal that say exactly the same thing). Sections 10.3-10.14
argue explicitly, cluster by cluster, for collapsing those cases rather
than inflating the node count — consistent with the anti-special-casing
principle stated in the task brief itself (the married-women cluster) and
the anti-combinatorial-explosion principle (the documents-checklist
cluster). If a future reviewer wants the tree to instead enumerate more
state-specific or category-specific terminals (e.g., splitting
`terminal_address_proof_local_verification` per state once the state-by-
state legal review in `19-roadmap.md` completes), that growth should come
from verified, cited content replacing today's "flag for local
verification" placeholders — not from speculative branching now.

**New terminal nodes proposed in this document:**

| Terminal node ID | One-line description |
|---|---|
| `terminal_form8_shift_and_correction` | Combined residence-shift + correction-of-entries guidance for one Form 8 submission (married-name-change, transgender gender-marker-plus-move, or any other combined case) |
| `terminal_senior_postal_ballot` | Age-85+ postal ballot/home voting eligibility via Form 12D, with no separate roll-marking step required (contrast with PwD marking) |
| `terminal_address_proof_local_verification` | Tribal-area/remote-village/urban-slum address-proof flexibility flagged for direct local BLO/ERO verification, pending state-by-state research |
| `terminal_documents_alternatives` | General "you likely still qualify" guidance for citizens missing one or more of Aadhaar/passport/driving licence/standard address proof, based on a documents-you-have checklist rather than per-combination branching |
| `terminal_deleted_entry_contest` | Contesting a believed-wrongful roll deletion, routed to direct BLO/ERO/grievance-portal contact pending verification of the correct ECI instrument |
| `terminal_polling_station_changed` | Informational-only outcome for a changed polling station with no underlying address change — no form required |

### 10.17 Open Questions Carried Into Implementation

1. Whether Form 7 (or another instrument) is the correct mechanism for
   contesting a wrongful roll deletion (`terminal_deleted_entry_contest`,
   cluster 10) — flagged `TODO: verify` rather than asserted.
2. Whether the online Form 8 workflow currently supports submitting a
   residence-shift and a correction of entries in a single session, or
   requires two sequential submissions (`terminal_form8_shift_and_correction`,
   cluster 2) — flagged as a portal-behavior confirmation, not a legal
   question.
3. The retired-service-voter transition process (old service-voter entry
   vs. new ordinary registration) remains an open item carried over
   unchanged from `06-legal-compliance-review.md` and `02-personas.md`
   persona #7 — this document does not attempt to resolve it, only to
   surface it again at the point (`terminal_service_voter` extension,
   cluster 3) where v2 now also reaches it via the transferred-employee
   path, not only the fresh-registration path.
4. The gender-marker-specific NEW KB ENTRY (cluster 7) and the per-state
   address-proof NEW KB ENTRY (cluster 5) are both explicit legal/content
   research prerequisites before their respective terminals can carry
   more specific guidance than "consult your ERO/BLO directly."

These four items should be tracked as concrete follow-up tasks against
`19-roadmap.md`'s v1 state-by-state legal review prerequisite and
`06-legal-compliance-review.md`'s sign-off gate, not left implicit in this
specification.
## 11. Admin Pages Specification

### 11.0 Scope and conventions

This section specifies every page in `crates/admin-app`, the Leptos SSR admin dashboard. The admin app is a separate deploy target from `crates/web-app` (public site) for blast-radius containment: an XSS, auth bug, or outage in admin tooling cannot touch the anonymous public decision-engine experience, and vice versa. Admin-app is server-rendered per request against Postgres via `crates/api` (or direct `sqlx` calls from admin-app handlers where it makes sense to avoid a network hop — an open question, see 11.14), with islands of interactivity hydrated only where needed (decision-tree node-graph editor, diff viewers, live-check buttons). Unlike the public site, admin-app does **not** need to optimize for 2G/low-bandwidth or screen-reader-first design as a primary constraint — Section 18 explicitly scopes that requirement to the public app. Admin users are assumed to be on broadband/office/home connections using a desktop or laptop browser. Each page below notes its mobile/low-bandwidth posture explicitly where it differs from the default ("desktop-first, mobile not optimized, functional-but-cramped on a phone browser is acceptable").

**RBAC roles** (from `admin_users.role`, Section 11.0's admin roles):

| Role | Can do |
|---|---|
| `contributor` | Draft/edit KB entries and translations; cannot publish/verify. |
| `reviewer` | Everything `contributor` can, plus approve KB entries to `verified`* and flag `needs_reverification`. |
| `legal_reviewer` | Sign-off gate for anything touching forms/eligibility claims: promote KB entries to `verified`, publish decision-tree versions, approve KB-body-text translations. |
| `translator` | Draft/edit translations (UI strings and KB body text), cannot approve/publish. |
| `analytics_viewer` | Read-only access to Analytics Dashboard and Dashboard summary tiles. No content/config write access anywhere. |
| `superadmin` | All of the above, plus User & Role Management, MCC panel, Bot Channel Management, Data Export & Retention. |

\* Note a deliberate refinement of "reviewer approves content into verified": per the RBAC line for `legal_reviewer` ("sign-off gate for anything touching forms/eligibility claims"), and because essentially every KB entry in this product touches a form or eligibility claim, **only `legal_reviewer` can promote an entry to `verified`**; plain `reviewer` can advance `draft` → `in_review` and can flag any entry `needs_reverification`, but the final verified sign-off is reserved for `legal_reviewer`. This is called out again in §11.3 because it is a common point of confusion and is tested explicitly in `core-domain`/`api` integration tests as a permission matrix, not left to UI-layer enforcement alone (server-side authorization check on every state transition, UI merely reflects/disables what the authenticated role cannot do).

**Assumed tables.** Beyond the seven named in the task (`knowledge_entries`, `knowledge_entry_revisions`, `decision_trees`, `admin_users`, `feedback`, `analytics_events`, `link_check_results`), several admin pages require additional tables not enumerated there but necessary to build the specified functionality. These are flagged inline as **[NEW TABLE]** and must be added to the `migrations/` schema section of this PRD: `audit_log` (unified audit trail — see §11.11), `mcc_windows` (§11.8), `bot_channel_config` (§11.9), `translation_status` (or a computed view, §11.5). Where a page reads "the JSON Schema fields," this refers to the existing knowledge-base entry JSON Schema carried over from the TS MVP's `knowledge-base/sources/*.json` validation (see `crates/kb-content`).

**Common guardrail pattern.** Any action in this section described as "destructive" or "high-stakes" (publish, verify, purge, role change, MCC toggle, forced logout) follows the same UI pattern unless noted otherwise: (1) action button disabled until preconditions are met, with a tooltip explaining why; (2) a modal confirmation summarizing exactly what will change (before/after diff where applicable); (3) for the highest-stakes actions (decision-tree publish, data purge, role grant to `superadmin`), the modal requires typing the entity's name/slug to confirm; (4) every such action writes one `audit_log` row before returning success to the client — if the audit write fails, the action itself is rolled back in the same DB transaction (no silent unaudited state changes are possible by construction).

---

### 11.1 Login / Session

**Purpose.** Authenticate admin users and establish a session; the single front door to every other page in this section.

**Primary roles.** All roles (every admin user must authenticate here first).

**Key UI elements.**
- Email + password fields, "Sign in" button.
- "Use a passkey instead" secondary action (only shown if the account has a registered passkey; passkeys are the v2 upgrade path (Section 6.10), gated behind an `oauth2-passkey-axum`-style crate integration — until that ships, this control is hidden entirely rather than shown-and-disabled).
- Post-login landing shows the authenticated user's display name and role badge (e.g., "Priya Nair — legal_reviewer") in the persistent top nav on every subsequent admin page, so the acting role is always visible.
- "Forgot password" flow: emails a time-limited (15 min) reset link; does not reveal whether the email exists (generic "if that address has an account, an email was sent" message) to avoid account enumeration.
- Session timeout banner: a non-blocking toast at 5 minutes before expiry ("Your session will expire in 5 minutes — click to extend") to avoid losing in-progress edits (particularly relevant for the KB editor and decision-tree editor, where an unsaved multi-field form could otherwise be lost).

**Actions.**
- Sign in (email/password → argon2 hash verification server-side via `crates/api`, session established via `tower-sessions`, session cookie set `HttpOnly`, `Secure`, `SameSite=Strict`).
- Sign in via passkey (v2).
- Request password reset.
- Sign out (also reachable from top nav on every page, not just here).

**Reads/writes.**
- Reads `admin_users` (email, `password_hash`, `role`, `is_active`, `passkey_credential_id` nullable).
- Writes a session record (server-side session store backing `tower-sessions`, not one of the seven named tables — either Postgres-backed session table or in-memory/Redis-free per the "no extra Redis dependency" architecture preference noted for jobs; recommend Postgres-backed sessions for consistency with the rest of the stack).
- Writes `audit_log` row on login and on logout (actor, timestamp, IP, user-agent) — login/logout activity is itself an auditable event on a platform handling civic-sensitive content moderation.

**Guardrails.**
- Account lockout after 10 consecutive failed attempts within 15 minutes (per-account, not per-IP, to avoid trivially locking out a legitimate user from a shared office IP); lockout duration 15 minutes, logged to `audit_log`.
- `is_active = false` accounts (see §11.10, forced deactivation) are rejected at login with a generic "account disabled, contact your administrator" message — no distinction shown between "disabled" and "wrong password" to avoid leaking account state to an attacker who has a stale credential.
- Session timeout policy: 8-hour absolute session lifetime, 2-hour idle timeout (sliding — any authenticated request resets the idle clock). Rationale: admin sessions touch legally-reviewed civic content and a safety-critical MCC control surface; short-lived sessions reduce the window of exposure if a laptop is left unlocked. Idle timeout is longer than a typical "banking app" 15-minute pattern because content review and tree editing sessions involve long stretches of reading/thinking between clicks — 2 hours balances security against interrupting legitimate review work.
- Passkey registration/management UI is out of scope for MVP-Rust-v1 (ties to Section 6.10's optional-passkey framing — "optional" here means password auth is the only supported flow at initial launch; passkey support ships when the crate dependency is ready).

**Mobile/low-bandwidth.** Fully functional on mobile (a superadmin locking down the MCC panel from a phone in an emergency is a realistic scenario) but not specifically optimized; standard responsive form layout is sufficient.

---

### 11.2 Dashboard (home)

**Purpose.** Single at-a-glance landing page after login: surfaces everything that needs admin attention right now, with every tile linking into the relevant detail page. Read-only — no mutating actions live on this page itself.

**Primary roles.** All roles see this page, but tile visibility is role-filtered (e.g., `analytics_viewer` sees only the analytics snapshot tile; `translator` does not see the MCC or bot-channel tiles).

**Key UI elements (tiles, each a card linking to its detail page).**
- **KB entries needing re-verification** — count of `knowledge_entries` where `review_status = 'needs_reverification'` OR `last_verified_date` is older than the configured re-verification threshold (default 180 days, configurable per topic-sensitivity in a future iteration — flagged as an open question, §11.14). Click-through to KB editor pre-filtered to this set. Visible to `contributor`, `reviewer`, `legal_reviewer`, `superadmin`.
- **Pending review queue count** — count of `knowledge_entries` where `review_status = 'in_review'`, broken out by "awaiting reviewer" vs. "awaiting legal_reviewer sign-off" (i.e., already reviewer-approved, pending the `verified` promotion). Click-through to KB editor filtered view. Visible to `reviewer`, `legal_reviewer`, `superadmin`.
- **Recent feedback count** — count of `feedback` rows with `status = 'new'` in the last 7 days, with a small breakdown by category. Click-through to Feedback & Grievance Triage. Visible to `reviewer`, `legal_reviewer`, `superadmin` (feedback triage is not a `contributor`/`translator` responsibility by default).
- **Dead-citation-link alerts** — count of `link_check_results` rows from the most recent nightly job run (per `crates/jobs` nightly dead-citation-link checker) where `http_status` is not in the 200-299 range or the request timed out/errored. Shows the timestamp of that last run. Click-through to Citation & Link Health. Visible to `contributor`, `reviewer`, `legal_reviewer`, `superadmin`.
- **Decision-tree draft-vs-published status** — for each decision tree (keyed by topic/scenario), shows current published version number, whether an unpublished draft with newer edits exists, and validation status of that draft (passes / fails / not yet validated). Click-through to Decision Tree Visual Editor. Visible to `reviewer`, `legal_reviewer`, `superadmin`.
- **Analytics snapshot** — sessions today (count of distinct rotated `session_id` values in `analytics_events` for the current day, hour-bucket aggregated per the analytics architecture — never raw per-user), top drop-off node (the decision-tree node with the highest "session ended here without reaching a terminal" rate today), top terminal outcomes (most-reached terminal nodes today). Click-through to Analytics Dashboard. Visible to `analytics_viewer` and above.
- **MCC status strip** (superadmin only, shown at the very top when any state currently has an active MCC window) — a persistent banner-style tile listing which states are currently under MCC suppression, not just a clickable card, because this is safety-critical and should not require a click to discover. Visible to `superadmin` only (matches §11.8 role scoping).

**Actions.** None mutating; every element is a link to a detail page.

**Reads/writes.** Read-only aggregate queries across `knowledge_entries`, `feedback`, `link_check_results`, `decision_trees`, `analytics_events`, `mcc_windows`. All counts should be served from the hourly rollup tables produced by `crates/jobs` where available (analytics snapshot) rather than computed live over raw `analytics_events` on every dashboard load, to keep page load fast; KB/feedback/link-check counts are cheap enough to query live given expected data volumes (low thousands of KB entries, not millions).

**Guardrails.** None (read-only page). The one behavioral guardrail: this page must load in under ~1.5s server-rendered even as data volume grows, because it's the page every admin user sees on every login — enforce with a Lighthouse/perf budget check in CI consistent with the project's existing CI perf gates.

**Mobile/low-bandwidth.** Tiles reflow to a single column on narrow viewports; no functional loss. No specific low-bandwidth optimization beyond standard responsive CSS — desktop is the primary expected surface.

---

### 11.3 Knowledge Base Content Editor

**Purpose.** The primary content-authoring surface for the curated, per-entry knowledge base that backs both the decision engine's cited answers and any standalone glossary/FAQ-style content.

**Primary roles.** `contributor` (create/edit drafts), `reviewer` (advance to `in_review`, flag `needs_reverification`), `legal_reviewer` (promote to `verified`), `superadmin` (all of the above).

**Key UI elements.**
- **List/search/filter view**: paginated table of `knowledge_entries` with filters for topic, `review_status`, language, and `applicable_states` (multi-select, since many entries are national but some are state-specific procedural variants); full-text search over title/body via Meilisearch (typo-tolerant, matches the Meilisearch choice for KB/glossary search from Section 6.5); columns: title, topic, status badge (color-coded by `review_status`), language, last-verified date (highlighted red if past threshold), last-edited-by, last-edited-at.
- **Entry create/edit form**: a form matching the JSON Schema fields one-to-one (title, body/answer text, topic, `applicable_states`, `review_status` — read-only display, changed only via the state-machine actions below, not a free-dropdown — language, `last_verified_date`, cited-sources sub-editor described below, and any structured fields the schema defines for deep-links/related-forms). Inline validation runs against the same JSON Schema used by `crates/kb-content` at load/parse time — the admin-app form validation calls a shared validation function (or the same server-side validation endpoint used by the API) rather than re-implementing schema rules in the frontend, so a KB entry that passes the editor's validation is guaranteed to load cleanly in production.
- **Diff view**: before saving any edit to an existing entry, show a side-by-side or inline diff against the current `knowledge_entries` row (which becomes the previous `knowledge_entry_revisions` row once saved). Diff is field-level (title changed, body changed with word-level highlight, sources added/removed) not just a raw-JSON diff, so a reviewer can actually read what changed.
- **Cited sources sub-editor**: a repeatable list within the entry form — each source has title, URL, publisher fields, an inline "Check now" button per source (HEAD request to the URL; shows a small status pill: OK / 404 / timeout / redirect, and populates a "last checked" timestamp), and add/remove controls. This is entry-authoring-time convenience distinct from the nightly bulk check in §11.6 — same underlying check mechanism, invoked ad hoc here as the contributor types instead of on a schedule.
- **State-machine action bar**: buttons for the valid next transitions from the entry's current `review_status`, each shown/enabled only for the role(s) permitted to perform it (see below); disabled buttons show a tooltip naming the required role.
- **Revision history panel**: list of prior `knowledge_entry_revisions` for this entry (who, when, diff summary), each clickable to view that historical version read-only, with a "restore this version as new draft" action (creates a new draft revision copying that historical content — never overwrites history in place).

**Actions and the `review_status` state machine.**
- `draft` → `in_review`: any `contributor`, `reviewer`, `legal_reviewer`, or `superadmin` who authored/owns the entry (or any `reviewer`+ regardless of authorship) can submit for review. Requires all required JSON Schema fields present and at least one cited source with a passing (or not-yet-checked, but not known-failing) link check.
- `in_review` → `verified`: **`legal_reviewer` or `superadmin` only** (see §11.0 note — this is the sign-off gate, not plain `reviewer`, because every entry touches eligibility/procedural claims). Requires: all cited sources currently passing their last link check (no known 404s at time of promotion — a hard block, not just a warning, since a broken citation on a "verified" entry undermines the entire cited-sources trust model); a non-empty `last_verified_date` set to today's date at the moment of promotion (the act of verifying stamps the date, not a manually-typed field); confirmation modal per the common guardrail pattern summarizing the entry title and stating "This will mark the entry VERIFIED and visible as an authoritative answer in the live decision engine and public site."
- `in_review` → `draft` (send back for revision): any `reviewer`+ can bounce an entry back with a required "reason for returning" comment, stored with the revision.
- Any status → `needs_reverification`: any `reviewer`, `legal_reviewer`, or `superadmin` can flag an entry at any time (e.g., they notice a form number changed or a source went stale) — this does not require going through `draft`/`in_review` first; it's an out-of-band flag that surfaces the entry on the Dashboard's re-verification tile and on the KB list filter, prompting a fresh authoring pass.
- `needs_reverification` → `in_review`/`verified`: re-enters the normal pipeline; a `legal_reviewer` re-verifying can promote straight back to `verified` if content is unchanged and sources still check out (updates `last_verified_date` without requiring a content edit).
- Every transition writes a new `knowledge_entry_revisions` row (append-only; `knowledge_entries` holds current state, `knowledge_entry_revisions` holds full history) and an `audit_log` row (actor, entry id, from-status, to-status, timestamp).

**Reads/writes.**
- Reads/writes `knowledge_entries` (current row per entry) and `knowledge_entry_revisions` (append-only history, one row per saved edit/transition).
- Reads `link_check_results` to populate cited-source status pills and to gate the `verified` promotion.
- Writes `link_check_results` rows when the ad hoc "check now" button is used (same table the nightly job writes to, distinguished by a `triggered_by` column — `nightly_job` vs. an admin user id — so the Citation & Link Health page can show provenance).
- Writes `audit_log` on every status transition.

**Guardrails.**
- Server-side role check on every transition endpoint (not just UI button disabling) — a `contributor` cannot POST a `verified` transition even by crafting the request directly.
- Promotion to `verified` hard-blocked if any cited source's last known check is a failure (404/5xx/timeout) — must fix or remove the source first.
- Saving any edit to a `verified` entry automatically demotes it to `in_review` (content changed = needs re-review before being trusted again as verified) — this is not optional or bypassable, and the UI states this plainly before the save completes ("Saving this edit will move the entry from Verified back to In Review pending re-approval").
- Deleting an entry entirely (as opposed to editing) is not exposed as a one-click action; entries are archived (a `status = 'archived'` / soft-delete flag) rather than hard-deleted, preserving citation history and audit continuity — hard delete, if ever needed, is a `superadmin` + Data Export & Retention Tools operation (§11.13), not a KB-editor action.

**Mobile/low-bandwidth.** Desktop-oriented; the multi-pane editor (form + diff + sources sub-editor + revision history) does not need a dedicated mobile layout for MVP-Rust-v1. A read-only mobile view of the list/search (e.g., a reviewer quickly checking status from a phone) works via standard responsive reflow but editing is expected to happen at a desktop.

---

### 11.4 Decision Tree Visual Editor

**Purpose.** Author and QA the branching decision trees that drive the public decision-engine experience. The most complex admin page in the product; treated as such in this spec.

**Primary roles.** `contributor`/`reviewer` can edit draft nodes; `legal_reviewer`/`superadmin` can publish a new version. `analytics_viewer` has no access (this page is content-authoring, not analytics).

**Key UI elements.**
- **Node-graph canvas**: pan/zoom graph view of the current draft tree. Question nodes render as rectangular boxes containing the node id and a truncated prompt preview; branching options render as labeled arrows from a question node to its next node(s) (one arrow per answer option); terminal nodes render as a visually distinct shape/color (e.g., rounded pill, green if it deep-links to an ECI system, gray if purely informational) so a reviewer can instantly see where paths end. Nodes missing a citation or missing a deep-link (where the node type requires one) show a warning badge directly on the canvas, not only in a separate validation report.
- **Node inspector panel**: clicking a node opens a side panel to edit that node's prompt text, its answer options (label + target node id per option, reorderable), citations (linked to `knowledge_entries` — a node's citation is a reference to a verified KB entry, not free-typed text, so the tree stays consistent with the KB's own citation-verification pipeline), `deepLinks` (target ECI URL + label, e.g., "Go to voters.eci.gov.in to file Form 8"), and `checklist` items (structured list of "bring/prepare this" items for that node, if the node type supports it).
- **Add/remove node controls**: add a new question or terminal node, connect it via a new option arrow from an existing node, delete a node (blocked if it would leave another node's option arrow pointing at a nonexistent target — the UI prevents creating an orphaned reference at edit time rather than only catching it at validation time).
- **Validate action**: a single button that calls a server-side validation endpoint. Critically, **this endpoint is backed by the actual `core-domain` `validateTree` logic** — the same Rust function exercised by `proptest` property tests and used to gate any tree the production API will serve. The admin-app does not re-implement acyclic-graph-checking, full-citation-checking, or reachability-checking in the frontend/admin layer; it is a thin client over one shared validation function, so there is exactly one implementation of "what makes a tree valid" in the entire system. Validation results render as a report: list of cycles detected (if any, with the offending node chain), list of nodes missing a citation, list of unreachable nodes (no incoming option arrow from the root), and a pass/fail summary banner.
- **Publish new version action**: disabled (grayed out with tooltip) until the validate action has been run against the current draft state and returned a pass. Enabled only for `legal_reviewer`/`superadmin`.
- **Version history / rollback view**: a list of all published versions of this tree (version number, published-by, published-at, validation report snapshot at time of publish), each viewable read-only in the same canvas. "Rollback" does not delete or mutate a later version; it creates a **new** version whose content is a copy of the selected historical version, through the same publish-new-version flow (including re-validation) — consistent with "never mutate a published version in place."
- **Preview-as-a-user mode**: a toggle that switches the canvas into an interactive walkthrough — the admin answers each question as a test user would, and the page actually invokes the live `core-domain` decision engine (against the in-progress draft, not the published version) to compute the next node exactly as production would, rendering the same prompt/option/citation/deep-link/checklist content a public user would see at each step. This lets a reviewer walk every path for QA before publishing without needing a separate staging deploy.

**Actions.**
- Edit node content (prompt, options, citations, deep-links, checklist) — writes to the draft state of the tree.
- Add/remove/reconnect nodes.
- Run validation (read-only against `core-domain`, does not mutate).
- Publish new version (see guardrails).
- View version history; initiate rollback-as-new-version.
- Enter/exit preview mode.

**Reads/writes.**
- Reads/writes `decision_trees` — the draft is a mutable working row (or a `status = 'draft'` version row) per tree; publishing writes a **new** immutable version row (incrementing a version number column) rather than updating the previously published row's content in place. The previously published row remains queryable/servable until the new version is published, and remains in history afterward.
- Reads `knowledge_entries` for the citation picker (only `verified` entries are selectable as citations for a node being prepared for publish — a node can reference a `draft`/`in_review` KB entry while the tree itself is still in draft, but validation/publish blocks on any citation not yet `verified`, keeping the same trust chain as §11.3's citation-gates-verification rule).
- Writes `audit_log` on every publish and rollback action (actor, tree id, from-version, to-version, validation report reference, timestamp). Node-level edits within a draft are also logged, at minimum as a coarser "draft saved" audit entry with a diff summary, to preserve accountability even before a version is published.

**Guardrails.**
- Publish requires `legal_reviewer` or `superadmin` — enforced server-side.
- Publish requires validation to have been run and passed against the **exact current draft state being published** — if the draft is edited after a passing validation run, the pass result is invalidated and validation must be re-run (the UI tracks a content hash/version stamp for this, not just a boolean "was validated at some point").
- Publish confirmation modal (per common guardrail pattern) requires typing the tree's name/slug to confirm, and displays: version number being created, number of nodes, validation report summary, and an explicit statement "This creates a new published version. The previous version remains available in history but users will immediately see this new version's content." Sessions already mid-flow on the previous published version are not retroactively affected (in-flight anonymous decision sessions complete against whichever version they started on, per the session/versioning design — flagged for confirmation against the core-domain session model in the relevant architecture section).
- A published version is immutable at the data layer — there is no "edit published tree" code path at all; editing always operates on a draft/working copy, by construction rather than by convention.
- Deleting a tree entirely is out of scope for this page; trees are never hard-deleted (a decommissioned tree is simply not linked from the active topic list, but its version history remains for audit purposes).

**Mobile/low-bandwidth.** Explicitly desktop-only for authoring. The node-graph canvas requires meaningful screen real estate and precise pointer interaction (dragging connections, arranging nodes); no mobile layout is planned for MVP-Rust-v1. Preview mode (the linear Q&A walkthrough) is the one sub-feature that would work fine on a phone screen if ever needed, since it's just sequential question/answer, but is not a build priority separate from the desktop canvas.

---

### 11.5 Translation Management

**Purpose.** Track and drive translation completeness across the language roster (English + Hindi fully shipped/reviewed; 21 other Eighth Schedule languages staged as roadmap) for three content categories: KB entry body text, decision-tree node text, and UI chrome strings (FTL files under `i18n/<locale>/*.ftl`).

**Primary roles.** `translator` (draft translations), `reviewer` (approve non-legal UI-string translations), `legal_reviewer` (approve KB-body-text and decision-tree-text translations — see guardrail below), `superadmin`.

**Key UI elements.**
- **Per-locale completeness dashboard**: a table, one row per locale in the roster, columns for % of KB entries translated+reviewed, % of decision-tree node text translated+reviewed, % of UI strings translated+reviewed, and an overall weighted completeness bar. Locales still at 0% (any of the 21 roadmap languages before contribution work starts) show plainly as "not started" rather than a misleading 0%-styled-as-broken bar. **[NEW TABLE]** `translation_status` — either a materialized/derived view computed from FTL-file coverage + `knowledge_entries` per-language rows, or an explicit tracking table populated by the translation-completeness report job (`crates/jobs`, "translation-completeness report on KB change" from Section 6.8) — recommend the job-populated table for query speed on this dashboard rather than computing coverage live on every page load.
- **Translation workbench**: source-language text (English, or Hindi where English is not the canonical source for a given field) shown side-by-side with an editable target-language field. FTL-aware: where a string has plural or gender variants (FTL's `{$count ->[one] ... *[other] ...}` syntax, relevant across Indic languages with different plural/gender grammar than English), the workbench shows each variant as its own editable sub-field with a label (e.g., "one," "other," "masculine," "feminine" as applicable to the target locale's FTL variant set) rather than one flat text box — a translator should never have to hand-edit raw FTL variant syntax.
- **Machine-translation-assist button**: per string/entry, an "MT draft" button that calls a machine-translation service and populates the target field with a clearly-labeled draft (visually distinct background/border, a persistent "MACHINE-TRANSLATED DRAFT — REQUIRES HUMAN REVIEW" label that does not disappear until a human edits or explicitly accepts the text). MT output can never be submitted for approval in its unedited, unacknowledged state — the approve action is disabled while the draft-label flag is still set; a translator must either edit the text (which clears the flag) or click a separate explicit "I have reviewed this MT draft and it is accurate" acknowledgment before submitting it onward for review.
- **Review/approve queue**: list of translated strings/entries pending approval, filtered by locale and content category, with the same side-by-side source/target view for the reviewer to check against.

**Actions.**
- Draft/edit a translation (any `translator`, `reviewer`, `legal_reviewer`, `superadmin`).
- Request MT-assist draft (any of the above).
- Approve a translation for publish:
  - **UI chrome strings (non-legal)**: gated to `translator`+`reviewer` per Section 11.0's RBAC design — meaning a `translator` who did not author the string, or a `reviewer`, can approve. In practice this means the approving role must be `reviewer` or above (a `translator` should not self-approve their own UI-string translation; four-eyes review applies even to low-stakes chrome text).
  - **KB body-text translation and decision-tree node text translation**: requires `legal_reviewer` (or `superadmin`), explicitly, since that's a legal/procedural claim, not just UI chrome. A mistranslation of "Form 8" or a deadline in a regional language is a legal/procedural accuracy issue, not a cosmetic one, and gets the same sign-off gate as the English/Hindi source content does in §11.3.
  - This means the approval-gate UI differs by content type on this same page: an approve button for a UI string is enabled for `reviewer`+, while the approve button for a KB-entry translation is enabled only for `legal_reviewer`+, with a tooltip explaining the distinction when disabled.

**Reads/writes.**
- Reads/writes `knowledge_entries` (per-language content — assume the schema stores translations either as sibling rows keyed by `entry_id + language` or as a JSON map field per entry; either way, translation edits go through the same revisioning as §11.3, i.e., they also produce `knowledge_entry_revisions` rows, since a KB entry's translation is part of its content history).
- Reads/writes decision-tree node text per locale (part of `decision_trees` content, scoped to the tree's draft state the same way §11.4 describes — translating a published tree's text still goes through the draft/publish cycle, it does not silently mutate a live version).
- Reads/writes the FTL files under `i18n/<locale>/*.ftl` for UI chrome — the admin-app either edits these files directly (with the change committed via an internal git-backed workflow) or, more practically for a small non-technical translator audience, stores UI-string translations in a database table that a build step exports to `.ftl` at deploy time. This tradeoff is flagged as an open question (§11.14) since it affects whether translators need any git familiarity at all.
- Writes `audit_log` on every approval.

**Guardrails.**
- MT-drafted text cannot be approved/published while still flagged as an unreviewed MT draft (see above) — enforced server-side, not just a disabled button, since this is a factual-accuracy safeguard for a civic-guidance product.
- KB/decision-tree-text translation approval hard-gated to `legal_reviewer`/`superadmin`; UI-chrome approval gated to `reviewer`+. Both enforced server-side per transition endpoint.
- A locale cannot be exposed on the public site as "available" until its core content (at minimum, all `verified` KB entries' translations and all published decision-tree text) reaches a completeness threshold — this page is where that threshold is monitored, and the public-site language switcher (per `crates/web-app`) only lists locales that have crossed it, preventing a half-translated language from going live prematurely. Exact threshold percentage is a product decision to be set in the roadmap section, not fixed here.

**Mobile/low-bandwidth.** Desktop-oriented; side-by-side source/target text editing is cramped on a phone screen and translators are expected to work at a desk. Not a build priority for mobile.

---

### 11.6 Citation & Link Health

**Purpose.** Central table of every cited source URL used anywhere in the knowledge base, its health-check history, and tools to force a recheck or find what's overdue for human re-verification.

**Primary roles.** `contributor`, `reviewer`, `legal_reviewer`, `superadmin` (anyone who can touch KB content benefits from this page; `translator` and `analytics_viewer` do not need access).

**Key UI elements.**
- Table with one row per distinct cited source URL (deduplicated across KB entries that happen to cite the same source), columns: URL, publisher, which KB entries cite it (linked), last-checked timestamp, last HTTP status, current status pill (OK / broken / redirect-only / never checked), "days since lastVerifiedDate" computed per the *owning KB entry's* `last_verified_date` (a link can be technically alive at HTTP level while the KB entry's underlying factual claim is stale — the two are tracked distinctly, and this page surfaces both signals).
- Sort/filter controls: sortable by "days since last verified" (to surface the most-overdue entries first, exactly as specified), filterable to "broken only," filterable by publisher domain (useful to spot, e.g., an entire eci.gov.in subdomain restructuring that broke many links at once).
- "Recheck now" action, both per-row and a bulk "recheck all broken" action.

**Actions.**
- Manual recheck (single URL or bulk) — issues a HEAD request (falling back to GET if the target server doesn't support HEAD, common for some government sites) and records the result.
- Navigate to the owning KB entry(ies) to fix a broken citation.

**Reads/writes.**
- Reads/writes `link_check_results` (this page is effectively the primary UI over that table). Writes go through the same check mechanism as the nightly job and the ad hoc per-entry check in §11.3, with a `triggered_by` column distinguishing `nightly_job` vs. a specific admin user id, so provenance is always visible.
- Reads `knowledge_entries` to join citation-to-entry and to compute "days since lastVerifiedDate."

**Guardrails.**
- A "recheck all broken" bulk action is rate-limited (e.g., a small delay between outbound requests, or a cap on concurrent requests) to avoid the admin tool itself looking like an abusive crawler against government domains — courteous-crawling behavior is a project value given this PRD's explicit caution (Section 3) against scraper-style access patterns toward ECI systems (this tool checks a small, curated, human-picked set of citation URLs, not a crawl — but should still behave politely).
- No destructive action lives on this page (it does not delete citations or entries); it is a health-monitoring and recheck surface, with edits happening in §11.3.

**Mobile/low-bandwidth.** Standard responsive table; acceptable to view on mobile for a quick check, no special optimization needed.

---

### 11.7 Feedback & Grievance Triage

**Purpose.** Inbox and workflow for user-submitted feedback (bug reports, content corrections, general comments) captured by the public site/bots. Explicitly **not** a channel for resolving official ECI application status inquiries — this is called out in the UI itself, not just in this spec, to prevent triage staff from drifting into acting as an unofficial ECI support desk.

**Primary roles.** `reviewer`, `legal_reviewer`, `superadmin`. (`contributor` and `translator` do not triage feedback by default; `analytics_viewer` has no access — feedback message text is qualitative/potentially-identifying content, not an analytics aggregate.)

**Key UI elements.**
- Inbox list: category, message excerpt, optional `contact_email` (shown only if the submitter provided one — many won't, by design, per the platform's minimal-data posture), linked KB entry or decision-session id if the feedback was submitted in-context (e.g., "was this answer helpful?" attached to a specific node), submitted-at timestamp, status badge.
- **Persistent non-goal banner at the top of this page** (not dismissible, or dismissible only per-session): "VoteAssist cannot resolve ECI application status. Do not attempt to look up or explain an individual's application/form status here — redirect the user to the National Voter Helpline (1950, 8am-8pm) or the Voter Helpline App. This tool is for triaging feedback about VoteAssist itself (content errors, bugs, UX issues), not for acting as ECI support." This is placed directly in the triager's working view, not buried in a help doc, precisely because the failure mode being guarded against is well-intentioned staff trying to be helpful and overstepping the product's explicit non-goal.
- Status workflow buttons: `new` → `triaged` → `resolved`/`wontfix`.
- Internal notes field per feedback item (free text, staff-only, never shown to the submitter — there is no submitter-facing reply channel in this product, consistent with the anonymous/no-account public site design; if a `contact_email` was given, any follow-up happens over email outside this tool, not as an in-app reply feature, unless a future iteration adds one).
- Filter by category, status, date range, "has contact email" (to prioritize items where a follow-up is actually possible).

**Actions.**
- Change status (`new`/`triaged`/`resolved`/`wontfix`).
- Add/edit internal notes.
- Link/unlink to a KB entry (e.g., triager realizes the feedback should prompt a KB edit and cross-references it) — this is a soft link for traceability, it does not itself trigger a KB state change; the triager still goes to §11.3 to actually edit the entry.

**Reads/writes.**
- Reads/writes `feedback` (category, message, `contact_email` nullable, linked KB entry id nullable, linked decision-session id nullable, status, internal notes, timestamps).
- Writes `audit_log` on status changes (lighter-weight than content/tree audit entries, but still tracked for compliance — e.g., to demonstrate grievance-adjacent items were handled in reasonable time, relevant given Section 2's note about NGSP's official 48-hour resolution expectation for actual ECI grievances — VoteAssist is not that system, but should still be able to show its own feedback loop isn't neglected).

**Guardrails.**
- No feature in this tool enables looking up an individual's electoral roll status, application status, or any ECI-system data — by construction, this page only ever touches the `feedback` table and links to VoteAssist's own KB/session data, never anything resembling a query against ECI systems. This is as much an architecture guardrail (don't build the integration) as a UI one.
- `contact_email`, where present, should be treated as sensitive personal data under DPDP — visible only to `reviewer`+, and subject to the retention purge described in §11.13 (purge resolved feedback `contact_email`s after N days).

**Mobile/low-bandwidth.** Standard responsive inbox/list-detail layout works fine on mobile if a triager wants to clear items on the go; no special optimization needed.

---

### 11.8 MCC / Election-Period Control Panel

**Purpose.** Safety-critical control surface for Model Code of Conduct compliance. When an MCC window is active for a state, the platform must suppress or relabel anything that could be read as government-adjacent announcement/endorsement messaging in that state, per Section 2's discussion of MCC handling ("pure how-do-I-do-X administrative guidance continues, but proactive push notifications/bot broadcasts should throttle or add extra neutral disclaimers"). This page is the operator interface for that behavior, treated with commensurate rigor: restricted access, explicit confirmation, full audit trail.

**Primary roles.** `superadmin` only. This is intentionally not delegated to `reviewer`/`legal_reviewer` — it's an operational/compliance control, not a content-review action, and the blast radius of getting it wrong (failing to suppress broadcasts during an active MCC window, or an unauthorized toggle) is high enough to keep the role list minimal. (Open question for a future iteration: whether a narrower `mcc_operator` role should exist so this doesn't require full `superadmin` — flagged in §11.14.)

**Key UI elements.**
- **[NEW TABLE]** `mcc_windows` — one row per state per election-schedule announcement: state, `window_start` (date MCC came into force per ECI's schedule announcement), `window_end` (nullable until results are declared — per Section 2, MCC "stays in force until results are declared," so an admin may need to enter the start immediately upon ECI's announcement and only later fill in the end date), `is_active` (derived from whether today falls in [`window_start`, `window_end`) when `window_end` is set, or simply "any window with `window_end` null and `window_start` in the past" when not yet closed), entered-by, entered-at.
- Per-state toggle/list: a table of all Indian states/UTs, each showing current MCC status (inactive / active with window dates), with an "Add MCC window" action per state (opens a small form: start date, optional end date) and an "End this window" action (sets `window_end` to today, used once ECI declares results).
- The window entry is admin-entered (per the task's phrasing "auto-fed from an admin-entered list"), not automatically scraped from ECI — there is no reliable public real-time feed of MCC status suitable for automated ingestion, so a human enters it upon the ECI's public schedule announcement; this is stated plainly on the page so operators understand it is not self-updating.
- **Effect preview**: for any state currently toggled active, a summary of what changes: "Proactive bot broadcasts to users in/selecting [state] are suppressed. An additional neutral-disclaimer banner is shown to users detected or self-selecting this state." (ties to Bot Channel Management's kill-switch and to `crates/web-app`/bot content rendering logic elsewhere in this PRD.)
- **Audit log strip** embedded directly on this page (in addition to being part of the unified Audit Log Viewer, §11.11) showing who toggled what state, when, for quick on-page accountability without navigating away — justified by this being called out as a safety-critical surface that should "treat it that way in the spec."

**Actions.**
- Add an MCC window for a state (mark active, with a start date).
- Close an MCC window for a state (mark end date, once results are declared).
- Edit a window's dates (e.g., correction of a typo'd date) — still logged as a full audit entry, not treated as a no-op edit.

**Reads/writes.**
- Reads/writes `mcc_windows`.
- Downstream consumers (read-only from this table): `crates/bot-telegram`, `crates/bot-whatsapp`, and `crates/web-app` all check `mcc_windows` (via `crates/api` or a shared query) before sending a proactive broadcast/notification or rendering state-specific content, to decide whether to suppress/relabel. This page is the single source of truth those systems read from — there is no separate "MCC flag" duplicated elsewhere.
- Writes `audit_log` on every add/close/edit.

**Guardrails.**
- Every toggle requires the common confirmation-modal pattern, upgraded for this page: the modal explicitly states the real-world consequence ("This will suppress proactive outbound messaging to users associated with [state] starting [date]. Confirm you have verified this against ECI's official election schedule announcement.") and requires the acting `superadmin` to type the state name to confirm — this is one of the highest-consequence actions in the entire admin app (getting it wrong in either direction is a real regulatory-adjacent risk for the project), so it gets the strictest confirmation tier defined in §11.0.
- No auto-expiry silently reactivating suppressed messaging without a human action — closing a window is always an explicit admin action (matching Section 2's note that MCC "stays in force until results are declared," which is not a fixed calendar duration ECI publishes upfront for the end date, only the start).
- This page's audit entries are the primary artifact were the project ever to need to demonstrate MCC compliance behavior after the fact (e.g., in response to an ECI or Election Commission-adjacent inquiry) — retained indefinitely, exempt from any general audit-log retention/purge policy (§11.13 must explicitly exclude MCC-related audit rows from purge).

**Mobile/low-bandwidth.** Should work acceptably on mobile despite being a desktop-first admin tool overall — an MCC schedule announcement can happen at any time and a superadmin may need to act from a phone; keep this specific page's layout simple enough (a table + a small form) that it degrades gracefully rather than explicitly building a separate mobile design.

---

### 11.9 Bot Channel Management

**Purpose.** Operational status and configuration for each outbound channel (Telegram, WhatsApp, IVR), plus a platform-wide kill-switch tightly integrated with the MCC panel.

**Primary roles.** `superadmin` (channel config/tokens are credentials-adjacent and platform-wide operational risk; not delegated to content roles).

**Key UI elements.**
- **Telegram**: bot token status (configured / missing / last-validated timestamp — the actual token value is never displayed in the UI once set, only a masked indicator and a "replace token" action), webhook health (last successful update received), basic message-volume stat for the day.
- **WhatsApp Cloud API**: Meta app/config status (masked credential indicators as above); a **template message review** section listing all message templates submitted for Meta pre-approval, their approval status (pending / approved / rejected by Meta), and content preview. Explicit note rendered on this page: **"Meta requires template pre-approval for any outbound message sent outside a user-initiated 24-hour window. Only approved templates may be used for proactive outbound messages (e.g., re-verification reminders, MCC-period neutral disclaimers). Messages sent as replies within 24 hours of a user's last message are not subject to template restrictions."** This constraint directly affects what kinds of proactive nudges (e.g., "your saved checklist item is due") are even possible on WhatsApp and must be designed around, not discovered late.
- **IVR (Exotel)**: a **placeholder/status-only panel** for v2/v3 — per Section 6.7, `crates/ivr-gateway` is "scaffolded interface in MVP-Rust-v1, full implementation is a v2/v3 roadmap item." This section of the page shows "Not yet active — planned for v2/v3" rather than a fully built config UI, avoiding building UI for a backend capability that doesn't exist yet.
- **Per-channel kill-switch**: one toggle per channel (Telegram / WhatsApp / IVR-when-live) to pause all proactive outbound messaging on that channel immediately, independent of MCC status (useful for, e.g., an incident unrelated to elections — a bug causing message spam, or a Meta API outage).
- **Platform-wide kill-switch**: a single prominent button — "Pause all proactive outbound messaging, all channels" — that flips all per-channel switches at once. This is the one-click control called out in the task: during a sensitive period (an active MCC window, a breaking news event the team wants to pause around out of caution, an incident), a single action stops every outbound channel without needing to visit each channel's toggle individually. This control reads from and writes to the same suppression mechanism the MCC panel's per-state suppression uses — the platform-wide switch is the "all states, all channels" superset of what MCC per-state suppression does per-state per-channel, not a separate parallel mechanism (a design goal to prevent the two controls from ever disagreeing about whether a message should go out).

**Actions.**
- View channel status.
- Replace/rotate a channel credential (token/API key) — masked input, write-only from the UI's perspective (never re-displayed after saving).
- Review a WhatsApp template's Meta-approval status (read-only sync from Meta's API — approval itself happens on Meta's side, this page reflects it).
- Toggle a per-channel kill-switch.
- Toggle the platform-wide kill-switch.

**Reads/writes.**
- **[NEW TABLE]** `bot_channel_config` — one row per channel: channel name, masked-credential-reference (the actual secret lives in an environment variable / secrets manager, never in this table in plaintext — this table stores only status metadata and a reference/last-rotated timestamp), `proactive_messaging_paused` boolean, `paused_by`, `paused_at`.
- Reads `mcc_windows` to show, per active-MCC state, which channels are currently additionally constrained by MCC suppression logic layered on top of (not replacing) any manual pause.
- Writes `audit_log` on every credential rotation and every kill-switch toggle (per-channel and platform-wide) — kill-switch toggles get the same strict confirmation tier as §11.8 given the equivalent safety-criticality.

**Guardrails.**
- Credential values are never rendered back to the browser after being set — standard "write-only secret" UX (masked field, "replace" rather than "edit").
- Platform-wide kill-switch requires the strict confirmation tier (type-to-confirm) given its blast radius.
- WhatsApp template content cannot be edited from this page in a way that silently invalidates Meta's pre-approval — the page is a status/review surface for templates, not a template-content editor; actual template submission/editing happens through Meta's Business Manager or the `bot-whatsapp` crate's own template-management flow, referenced from here but not duplicated here, to avoid the admin UI drifting out of sync with what Meta has actually approved.

**Mobile/low-bandwidth.** The platform-wide and per-channel kill-switches specifically should be reachable and usable from mobile (same "3am incident, superadmin is on their phone" scenario as §11.8); the more detailed config/token-rotation UI is desktop-oriented and does not need mobile optimization.

---

### 11.10 User & Role Management

**Purpose.** CRUD on admin accounts and role assignment — the control surface for who has access to everything else in this section.

**Primary roles.** `superadmin` only.

**Key UI elements.**
- Table of all `admin_users`: name, email, role, `is_active` status, last login timestamp, passkey-registered indicator (once passkeys ship).
- Create-user form: name, email, initial role, temporary-password-or-invite-link flow (recommend invite-link — the new user sets their own password on first login rather than an admin choosing/transmitting one, avoiding the password ever existing in a chat/email in plaintext).
- Edit-user: change role, toggle `is_active`.
- "Force logout" action per user: invalidates all active sessions for that account immediately (relevant for offboarding, suspected compromise, or a role downgrade that should take effect immediately rather than at next natural re-login).
- Filter by role, by active/inactive status.

**Actions.**
- Create admin user (invite flow).
- Edit role.
- Deactivate/reactivate account.
- Force logout (invalidate sessions).
- (Hard delete of an admin user account is deliberately not offered — use deactivate; preserves `audit_log` actor-attribution integrity, since a hard-deleted user id would orphan historical audit rows that reference it.)

**Reads/writes.**
- Reads/writes `admin_users` (role, `is_active`, and related fields — not `password_hash`, which is only ever written via the user's own password-set/reset flow in §11.1, never directly edited by another admin, to avoid a superadmin ever being able to silently set/know another user's password).
- Invalidating sessions writes to the session store (§11.1) — deletes/marks-invalid all session rows for the target user id.
- Writes `audit_log` on every create/role-change/deactivate/force-logout action, including the before-and-after role value for role changes specifically (not just "role changed" — the diff itself, since role-change history is exactly the kind of thing an external security review would want to see).

**Guardrails.**
- Role changes use the common confirmation pattern; a change **to** `superadmin` specifically uses the strict type-to-confirm tier (granting the highest-privilege role is the highest-consequence action on this page).
- A `superadmin` cannot deactivate or demote their own account through this UI (prevents accidental self-lockout, and prevents a compromised session from being used to silently strip other superadmins' ability to intervene while leaving itself as sole admin — at least one other active `superadmin` must exist, or the action is blocked with an explanatory message, for any deactivate/demote-self or demote-last-remaining-superadmin action).
- Every role-change audit entry is also visible in the unified Audit Log Viewer (§11.11) — this page is not the only place this history can be reviewed, by design.

**Mobile/low-bandwidth.** Desktop-oriented; user/role administration is low-frequency enough that no mobile optimization is planned.

---

### 11.11 Audit Log Viewer

**Purpose.** Unified, searchable, filterable view over every audit-logged action across the admin app: content edits and state transitions (§11.3), decision-tree publishes/rollbacks (§11.4), translation approvals (§11.5), feedback status changes (§11.7), MCC toggles (§11.8), bot-channel kill-switches/credential rotations (§11.9), role changes/forced-logouts (§11.10), and logins/logouts (§11.1). This is the compliance backbone of the admin app.

**Primary roles.** `superadmin` (full access); `legal_reviewer` gets read-only access scoped to content/tree/translation-related entries (relevant to their sign-off responsibilities) but not to role-management or MCC/bot-channel entries. Other roles have no access to this page (they can see their own action's confirmation at the time they take it, but not the aggregate log).

**Key UI elements.**
- **[NEW TABLE]** `audit_log` — the schema every other page above writes to: `id`, `actor_admin_user_id`, `action_type` (enumerated: e.g., `kb_entry_status_change`, `kb_entry_edit`, `tree_publish`, `tree_rollback`, `translation_approved`, `feedback_status_change`, `mcc_window_added`, `mcc_window_closed`, `bot_channel_paused`, `bot_credential_rotated`, `admin_role_changed`, `admin_deactivated`, `admin_force_logout`, `login`, `logout`, `data_export`, `retention_purge_executed`), `entity_type`, `entity_id`, `before_value` (JSON, nullable), `after_value` (JSON, nullable), `timestamp`, `ip_address`, `user_agent` (nullable, mainly populated for login/logout).
- Search/filter bar: by actor, by action type, by entity, by date range, free-text search over the JSON before/after payloads (useful for "find every edit that touched this specific KB entry" style queries, which also overlaps with the revision-history views embedded in §11.3/11.4 — this page is the cross-cutting view, those pages show entity-scoped history inline for convenience).
- Row detail expansion: shows before/after diff rendered readably (not raw JSON dump) where both values are present.
- **Export** action: CSV/JSON export of the current filtered result set, for external compliance review (e.g., providing an auditor or funder a slice of the log without giving them direct admin-app access).

**Actions.**
- Search/filter/browse (read-only).
- Export current filtered view.

**Reads/writes.**
- Reads `audit_log` exclusively. This page performs no writes of its own (browsing the audit log is not itself an audited mutation, though the export action arguably should be — see guardrail below).

**Guardrails.**
- The `audit_log` table itself is insert-only from the application's perspective — no UPDATE/DELETE code path exists anywhere in the admin app for audit rows (retention-driven purges, §11.13, are the one sanctioned exception, and even those are themselves logged as a `retention_purge_executed` audit entry describing what was purged, so the purge doesn't create a gap with no explanation).
- The **export** action itself writes a `data_export` audit entry (who exported, what filter criteria, when) — exporting a compliance artifact is itself worth being able to account for later.
- MCC-related and role-management-related audit rows are explicitly excluded from any general retention/purge policy (cross-reference §11.8, §11.13) — these are the categories most likely to matter for external accountability years later, so they are kept indefinitely regardless of whatever general audit-log retention window is eventually configured for lower-stakes categories (e.g., routine login/logout rows might reasonably age out; MCC and role-change rows should not).

**Mobile/low-bandwidth.** Desktop-oriented for serious review work; functional on mobile for a quick spot-check but not optimized.

---

### 11.12 Analytics Dashboard

**Purpose.** Admin-facing visualization over the aggregate analytics data produced by `crates/analytics`' hourly/daily rollup jobs. (The analytics event/data model itself — schema of `analytics_events`, rollup job design, privacy architecture — is specified in its own PRD section; this subsection covers only the admin UI built on top of it.)

**Primary roles.** `analytics_viewer` (primary), `reviewer`, `legal_reviewer`, `superadmin` (all can view; only `analytics_viewer` has this as their *sole* admin-app access).

**Key UI elements.**
- **Time-range picker** (today / 7d / 30d / custom range), applied to all charts on the page.
- **Funnel visualization per decision-tree version**: a Sankey-style or step-funnel chart showing session volume at each node of a selected decision tree version, letting a viewer see where volume concentrates and where it thins out, versioned so a funnel for v3 of a tree isn't conflated with v4's after a publish (ties directly to §11.4's immutable-version design — funnels are always scoped to a specific version id).
- **Top drop-off nodes**: ranked list of nodes with the highest rate of "session ended here without reaching any terminal," across the selected time range.
- **Top terminal outcomes**: ranked list of which terminal nodes are reached most often (e.g., "X% of completed sessions ended at the 'file Form 8 at voters.eci.gov.in' terminal").
- **Deep-link click-through rate**: for terminal/checklist nodes with a deep-link, the rate at which the deep-link was actually clicked (an anonymous boolean event per the analytics architecture, not a tracked outbound-click-to-identity linkage).
- **Locale breakdown**: session volume and completion rate split by locale, useful for spotting a language whose funnel behaves very differently (a leading indicator of a translation-quality problem, cross-referencing §11.5).
- **CSV export** of the currently-displayed aggregate data for the selected time range/filters.

**Actions.**
- Change time range / filters (tree, version, locale).
- Export CSV of current view.

**Reads/writes.** Reads only — from the hourly/daily rollup tables produced by `crates/jobs` against `analytics_events` (this page should not query raw `analytics_events` directly for anything beyond the smallest "today so far" slice not yet rolled up, to keep the page fast and to keep the aggregation boundary consistent with the privacy architecture). No writes.

**Guardrails.**
- **No user-level drill-down anywhere on this page or its export.** Every view is an aggregate over a cohort (a node, a time bucket, a locale) — there is no click-path from an aggregate number down to an individual session's raw event stream, and no per-session identifier is ever displayed, because none should be linkable to an identity in the first place per the analytics architecture (rotated ephemeral session ids, no free text, no IP-to-identity linkage beyond abuse-prevention rate limiting). This is restated here deliberately, even though it's primarily an architectural property of the data model, because it is the single most important guardrail on this specific page and must not be undermined by a future "just add one more filter" feature request (e.g., a "search by session id" box must never be added to this page).
- CSV export is likewise aggregate-only; the export code path draws from the same rollup tables as the on-screen charts, so there is no export-specific bypass that could leak more granular data than the UI itself shows.

**Mobile/low-bandwidth.** Charts reflow responsively but this is a desktop-oriented analysis tool; no specific mobile optimization planned.

---

### 11.13 Data Export & Retention Tools

**Purpose.** Two related but distinct capabilities: (a) proactive, transparency-motivated full-KB export for third-party mirroring, consistent with the project's open-source ethos; (b) admin-triggered enforcement of data-retention policy, consistent with DPDP's minimal-data posture.

**Primary roles.** `superadmin` only (both capabilities touch platform-wide data at once and are individually high-consequence).

**Key UI elements.**
- **Full KB export**: a single "Export full knowledge base" action producing a downloadable archive (JSON, matching the existing KB schema, one file per entry or a single bundled file — consistent with how `knowledge-base/sources/*.json` is structured today) of all **verified** entries (draft/in_review content is excluded from the public transparency export by default, since it hasn't cleared the legal-review gate — a configurable toggle could include non-verified content for internal mirroring/backup purposes specifically, clearly labeled as "includes unverified draft content, not for public redistribution" if used). This directly serves the project's stated openness goal: any third party (a researcher, another civic-tech project, an archival effort) can mirror VoteAssist's curated, cited content without needing database access, matching the spirit of citing ADR/MyNeta as a governance model in Section 3.
- **Retention policy panel**: a small config table of the retention rules currently in force, each with a configured window and a "run now" action in addition to whatever scheduled job (`crates/jobs`) runs it automatically:
  - Purge `analytics_events` rows older than the configured window (default per the analytics architecture section — likely a modest number of months, since rollups already preserve the aggregate value and raw events are the more privacy-sensitive granular layer).
  - Purge `contact_email` from `feedback` rows where `status = 'resolved'` and resolved-at is older than N days (default e.g. 90 days) — the message/category/notes remain for institutional memory, only the personal contact field is nulled out, since it's no longer needed once the loop is closed and retaining it indefinitely has no purpose but adds DPDP exposure.
  - (MCC-related and role-management audit_log rows are explicitly excluded from any purge rule offered here, per §11.8/§11.11 — the UI for configuring purge rules should not even present those categories as selectable targets, to make the exclusion structural rather than a matter of an admin remembering not to select them.)
- Each retention rule shows: last-run timestamp, rows affected on last run, next scheduled run.

**Actions.**
- Trigger full KB export (download).
- Trigger an ad hoc "run now" for a specific retention rule.
- Edit a retention rule's window (e.g., change analytics retention from 12 months to 6) — a policy change of its own significance, logged accordingly.

**Reads/writes.**
- Reads `knowledge_entries` (and joins to sources) for the export.
- Deletes/nulls rows in `analytics_events` and `feedback` per the retention rules described.
- Writes `audit_log` for every export (`data_export`, distinguishing this full-KB export from the Audit Log Viewer's own export in §11.11 by `entity_type`) and every purge execution (`retention_purge_executed`, recording rule name, row count affected, time range purged, triggered-by — manual "run now" vs. scheduled job — matching the requirement that purges get a confirmation step and an audit entry).

**Guardrails.**
- Purge actions use the strict confirmation tier: the modal states exactly what will be deleted (e.g., "This will permanently delete `analytics_events` rows older than [date] — approximately [N] rows based on a dry-run count. This cannot be undone.") and requires typing a confirmation phrase. A **dry-run count** is always shown before the irreversible action is confirmed — the admin never purges "blind."
- Full KB export defaults to verified-only content, with any inclusion of non-verified content requiring an explicit extra toggle and a warning label on the resulting archive's manifest, to avoid accidentally distributing not-yet-legally-reviewed claims as if they were the vetted product.
- Retention-rule-window edits (not just executions) are themselves audited and require `superadmin`, since loosening a retention window (e.g., extending analytics retention from 6 months to 5 years) has real DPDP posture implications and should not be a casual change.

**Mobile/low-bandwidth.** Desktop-oriented; a full KB export download and a destructive purge confirmation are exactly the kind of actions better done deliberately at a desk, not from a phone — no mobile optimization planned, and arguably should not be encouraged on mobile given the consequence level of what lives here.

---

### 11.14 Open questions (cross-cutting, admin app)

- Whether admin-app handlers call `crates/api` over HTTP/loopback for all data access (cleaner separation, one authorization surface) or query Postgres directly via `sqlx` from within `crates/admin-app` (fewer moving parts, but duplicates authorization logic across two crates unless carefully shared) — affects several pages above and should be settled in the architecture section this PRD part complements, not decided ad hoc per page.
- Whether UI-chrome FTL-file translations are edited as literal `.ftl` files through a git-backed workflow, or stored in a database table with a build-time export step (§11.5) — changes what tooling/familiarity is required of non-technical `translator` role users.
- Whether a narrower `mcc_operator` role should be introduced so the MCC panel (§11.8) doesn't require full `superadmin` — currently scoped to `superadmin` only for MVP-Rust-v1 given this PRD's defined role list (Section 11.0) has no better fit, but this is worth revisiting once real operational load on the MCC panel is observed.
- The exact numeric re-verification-aging threshold (Dashboard tile, §11.2) and retention windows (§11.13) are placeholders pending a policy decision from the content/legal team; this spec fixes the *mechanism* (a configurable threshold surfaced on relevant pages) but not the final number.
- Whether KB entries store per-language content as sibling rows or a JSON map column (§11.5) is a schema decision belonging to the data-model section, referenced here because it affects the Translation Management page's read/write shape.
## 12. Analytics Specification v2

### 12.1 Non-negotiables (restated as design decisions, not oversights)

These are not defaults we haven't gotten around to changing. Each one was considered against the obvious alternative and rejected deliberately.

1. **No cookies, no cross-site tracking.** The event pipeline has no concept of a tracking cookie, a `_ga`-style client ID, or any identifier that survives a page reload by design (see session rotation, §12.3). There is no `Set-Cookie` header from `crates/analytics` and no client-side storage of an analytics identifier beyond the in-memory session token already used for decision-tree state.
2. **No third-party analytics SaaS — explicitly, not by omission.** Google Analytics, Mixpanel, Amplitude, Segment, PostHog Cloud, Hotjar, and equivalents are out of scope permanently, not "not yet integrated." The reasons: (a) every one of these vendors is a foreign-hosted data processor for a platform whose entire premise is helping Indian citizens with a government process — routing behavioral data about that through a third party contradicts the product's own credibility argument; (b) their SDKs load third-party JS that most of them use for cross-site identity resolution somewhere in their business model, which is precisely the category of tracking this product tells users to be suspicious of on other sites; (c) DPDP Rules 2025 data-fiduciary and cross-border-transfer obligations become materially harder to reason about the moment a foreign SaaS vendor is a data processor of record. This is recorded here so that a well-intentioned future contributor proposing "let's just drop in PostHog, it's free" has an explicit decision to overturn, not a gap to fill.
3. **No IP-to-identity linkage beyond short-TTL abuse prevention.** Client IP addresses are used only inside the rate-limiting middleware (`tower-governor`, §13.4/§13.5) with a TTL measured in minutes, never persisted to `analytics_events` or any other durable table, and never joined against session or event data even transiently in application code. If IP-derived geolocation is ever wanted for a "which state is this traffic from" metric, it must be resolved to a coarse bucket (state-level, from a local MaxMind-style GeoIP database run in-process) and only the bucket is retained — the raw IP itself never reaches a table.
4. **No political-inclination data ever collected — because there is none to collect, by design.** The decision tree has no question, node, or terminal that asks about or infers party preference, caste, religion, or community. This isn't a redaction step applied to a broader dataset; the schema in §12.2 has no field capable of holding such a value. "We don't have this data" is true because the schema forecloses it, not because of a policy layered on top of a schema that could hold it.

### 12.2 Event schema

`crates/analytics` owns a single raw ingestion table, `analytics_events`, extending the shape sketched in the Rust architecture decisions. All writes go through one Axum handler (`POST /internal/events`, called server-side from `crates/api` and the bot adapters — never directly from the public client, so a malicious client cannot forge arbitrary event rows).

```sql
-- migrations/xxxx_analytics_events.sql (sqlx)
CREATE TYPE analytics_event_type AS ENUM (
    'session_started',
    'question_answered',
    'terminal_reached',
    'deep_link_clicked',
    'kb_search_performed',
    'kb_entry_viewed'
);

CREATE TYPE client_platform AS ENUM ('web', 'telegram', 'whatsapp', 'ivr');

CREATE TABLE analytics_events (
    id                  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    event_type          analytics_event_type NOT NULL,
    session_id          UUID NOT NULL,             -- ephemeral, see 12.3
    decision_tree_version TEXT NOT NULL,            -- e.g. "2026.07.1"
    node_id             TEXT,                       -- NULL for kb_* events
    answer_option       TEXT,                       -- enum VALUE, never free text
    terminal_id         TEXT,                       -- for terminal_reached / deep_link_clicked
    kb_query_category   TEXT,                       -- hashed/bucketed, see 12.2.1
    kb_entry_id         TEXT,                       -- for kb_entry_viewed
    locale              TEXT NOT NULL,              -- BCP-47, e.g. "hi-IN"
    client_platform     client_platform NOT NULL,
    occurred_at_hour    TIMESTAMPTZ NOT NULL,       -- floor(occurred_at, '1 hour'), see 12.2.2
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX ON analytics_events (event_type, occurred_at_hour);
CREATE INDEX ON analytics_events (decision_tree_version, node_id);
```

| Field | Type | Notes |
|---|---|---|
| `event_type` | enum | One of the six values above. No open-ended "custom event" mechanism — adding a new event type requires a migration + code review, which is intentional friction against schema creep toward richer per-user tracking. |
| `session_id` | UUID | Ephemeral, rotated per §12.3. Never joined to any table outside `analytics_events` and `analytics_rollups_*`. |
| `decision_tree_version` | text | Enables per-version funnel comparison (§12.4) when a tree edit changes drop-off behavior. |
| `node_id` | text, nullable | The decision-tree node identifier (e.g. `"student_status"`), not the question text. |
| `answer_option` | text, nullable | The **option value** the user selected (e.g. `"student_hostel"`), matching the enum defined in `core-domain`'s node schema — never the free-text label shown in the UI, and never any user-typed content. |
| `terminal_id` | text, nullable | Which terminal outcome was reached / which terminal's deep link was clicked. |
| `kb_query_category` | text, nullable | See §12.2.1 — never raw search text. |
| `kb_entry_id` | text, nullable | Which curated KB entry was viewed. |
| `locale` | text | For language-distribution metrics (§12.5) and to prioritize translation work. |
| `client_platform` | enum | `web` / `telegram` / `whatsapp` / `ivr`. |
| `occurred_at_hour` | timestamptz | **Not** a full-precision timestamp — see §12.2.2. |

#### 12.2.1 `kb_search_performed`: hashed/bucketed query category, never raw text

Raw search-box text is the single highest-risk field in this entire schema if handled naively: a user searching the KB will often type something close to their actual situation ("I moved from Bihar to Delhi after marriage do I need new voter id" or worse, something naming a relative, a specific address fragment, or a disability). That is exactly the kind of incidentally-collected personal narrative DPDP treats as personal data even though it wasn't the product's intent to collect it, and it is exactly the kind of data this product has otherwise engineered itself to never hold.

Instead of storing the query string, the search handler in `crates/api`:

1. Runs the query through the same intent-classification step already used to route Meilisearch/tantivy results to KB categories (e.g. `"shifting"`, `"student"`, `"nri"`, `"pwd"`, `"correction"`, `"new_registration"`, `"deadline"`, `"other"`).
2. Writes only the resulting **category label** (a small fixed vocabulary, versioned alongside the KB taxonomy) to `kb_query_category`.
3. Discards the raw string immediately after classification — it is never logged, never written to `analytics_events`, and application-level tracing spans must not include it either (see §13.4 on log hygiene).

If the classifier is uncertain, the category is `"unclassified"` rather than falling back to storing the raw text. This means the `kb search success rate` metric is slightly coarser than a naive implementation could produce, which is an accepted cost.

#### 12.2.2 Hour-bucketed timestamps: not a full-precision timestamp

`occurred_at_hour` is `date_trunc('hour', now())`, computed server-side at ingestion time — the exact minute/second is never captured anywhere in the pipeline (not dropped after the fact; never computed). The reason is timing-correlation re-identification: a sequence of full-precision timestamps for `session_started` → `question_answered` × N → `terminal_reached` for one session, cross-referenced against, say, a household's shared-router access logs or a cyber-café's own sign-in log, can single out an individual even without any name field, because the timing fingerprint of "answered 5 questions taking exactly this long, from this location, at this exact minute" is unique enough to be identifying. Hour-bucketing destroys that fingerprint while preserving everything the funnel/rollup metrics in §12.4 actually need (which never require sub-hour granularity).

### 12.3 Session ID rotation policy

`session_id` is a v4 UUID generated client-side (or by the channel adapter for Telegram/WhatsApp/IVR) at the start of a decision-tree walkthrough. Concrete policy:

- **Generation:** A fresh random UUID is generated at `session_started`, held only in the client's in-memory/tab-scoped state (or the bot adapter's short-lived conversation context), never derived from any stable input (no hash of phone number, Telegram user ID, or device fingerprint feeds into it).
- **Lifetime:** The session ID lives for the duration of one decision-tree walkthrough only. It is discarded by the client after `terminal_reached` (or after an idle timeout — proposed 30 minutes of inactivity) and a brand-new, unrelated UUID is generated for the next walkthrough, including a same-user, same-device, immediately-following session.
- **No server-side linkage across rotations.** `crates/analytics` never stores or infers a mapping from old session ID to new session ID. There is no "returning user" concept in the analytics schema at all — repeat usage by the same person is invisible to the system, which is intentional: funnel metrics measure walkthrough behavior, not user retention.
- **No persistent device fingerprint anywhere in the pipeline** — no canvas fingerprinting, no audio-context fingerprinting, no combination-of-headers fingerprint, none of the usual techniques for re-deriving identity across a rotated ID. This is a permanent constraint on any future analytics feature request, not just the MVP-Rust-v1 scope.
- Bot adapters (`crates/bot-telegram`, `crates/bot-whatsapp`) must generate a fresh `session_id` per conversation-walkthrough the same way, and must not reuse the platform's own persistent chat/user ID as the analytics `session_id` — that would silently reintroduce a stable cross-session identifier via the back door.

### 12.4 Rollup and aggregation jobs

Scheduled via `apalis-cron` (Postgres-backed, per the Rust architecture decisions — no separate Redis needed):

| Job | Schedule | Reads | Writes | Purpose |
|---|---|---|---|---|
| `hourly_rollup` | `@hourly` (offset a few minutes past the hour to avoid racing in-flight writes) | `analytics_events` for the just-completed hour | `analytics_rollups_hourly` | Per-hour, per-`(event_type, node_id, answer_option, terminal_id, locale, client_platform, decision_tree_version)` counts |
| `daily_rollup` | `@daily`, 03:00 IST | `analytics_rollups_hourly` for the prior 24 hours | `analytics_rollups_daily` | Coarser daily aggregates for dashboard trend views and for anything feeding the public transparency dashboard (§12.6) |
| `raw_event_purge` | `@daily`, after `hourly_rollup` and `daily_rollup` for the day have both succeeded | `analytics_events` older than the retention window | (deletes) | Enforces the retention hard requirement below |

```sql
CREATE TABLE analytics_rollups_hourly (
    hour_bucket         TIMESTAMPTZ NOT NULL,
    event_type          analytics_event_type NOT NULL,
    decision_tree_version TEXT NOT NULL,
    node_id             TEXT,
    answer_option       TEXT,
    terminal_id         TEXT,
    kb_query_category   TEXT,
    locale              TEXT NOT NULL,
    client_platform     client_platform NOT NULL,
    session_count       BIGINT NOT NULL,   -- distinct session_ids, not raw event count
    event_count         BIGINT NOT NULL,
    PRIMARY KEY (hour_bucket, event_type, decision_tree_version, node_id, answer_option, terminal_id, kb_query_category, locale, client_platform)
);
-- analytics_rollups_daily mirrors this with day_bucket in place of hour_bucket
```

**Retention: raw `analytics_events` rows are purged 30 days after ingestion, once rolled up.** This is stated as a hard requirement, enforced by the `raw_event_purge` job with a `NOT EXISTS`-style guard that refuses to delete a hour-window's raw rows until both rollup jobs have confirmed success for that window (a failed rollup blocks purge and pages the on-call, rather than silently losing data or silently retaining it past the window). 30 days is chosen as enough runway to catch and re-run a broken rollup job without requiring the raw data for normal dashboard use, and short enough that the ephemeral-session-ID discipline in §12.3 doesn't quietly become moot by leaving months of raw sequences sitting around. Any change to the 30-day figure is a Compliance Owner decision (per the DPDP-facing sections of this PRD) to be published in the public privacy policy, not an engineering-only knob.

### 12.5 Funnel metrics for the admin dashboard (section 11)

The admin analytics dashboard (`crates/admin-app`, `analytics_viewer` role) reads exclusively from `analytics_rollups_hourly` / `analytics_rollups_daily`, never from raw `analytics_events` — the same structural safeguard carried over from the v1 plan, now enforced by the fact that raw rows won't even exist past 30 days.

| Metric | Computation | Purpose |
|---|---|---|
| Per-node drop-off rate | 1 − (sessions reaching node X+1 / sessions reaching node X), per `decision_tree_version` | Flags confusing or ambiguous questions for content review |
| Path completion rate per tree version | sessions with a `terminal_reached` event / sessions with a `session_started` event, grouped by `decision_tree_version` | Lets the team A/B-compare a tree edit against its predecessor over the same calendar period without needing individual-level data |
| Deep-link click-through rate per terminal | sessions with `deep_link_clicked` for terminal T / sessions with `terminal_reached` for terminal T | Measures hand-off effectiveness per scenario, surfaces terminals whose deep link may be broken or whose guidance isn't compelling enough to act on |
| Locale distribution | session count grouped by `locale`, per period | Feeds the translation-priority roadmap (mirrors v1 intent) |
| Channel distribution | session count grouped by `client_platform` | Capacity planning and roadmap prioritization across web/Telegram/WhatsApp/IVR |
| KB search success rate | (`kb_search_performed` sessions followed by a `kb_entry_viewed` in the same session) / total `kb_search_performed` sessions | Search relevance, computed at the category level per §12.2.1 |

All of the above are computed as SQL aggregate queries over the rollup tables; there is no bespoke query language or BI tool — this keeps the admin dashboard's data layer inside `crates/api`/`crates/admin-app` with ordinary `sqlx` queries.

### 12.6 Forward-looking: public transparency dashboard (v2/v3 roadmap, not MVP-Rust-v1)

The org's stated values include eventually publishing a public, aggregate-only transparency dashboard — both to build trust in an open-source civic tool and to model the transparency VoteAssist expects of the systems it helps users navigate. This is explicitly **not** an MVP-Rust-v1 commitment; it is scoped here only so that the analytics schema above is designed to make it possible later without a redesign.

**What would be safe to publish publicly**, subject to a k-anonymity suppression threshold (proposed: suppress any bucket representing fewer than 20 distinct sessions in the aggregation period):

- Overall task-completion rate (site-wide, per week/month).
- Terminal-outcome distribution (which scenarios are most common), at national or state level only where the state-level bucket clears the 20-session threshold for that period — smaller states/UTs or short time windows would show "insufficient data" rather than a small, potentially identifying count.
- Locale/channel distribution, same threshold rule.
- Deep-link click-through rate, aggregated across all terminals (not broken out by a terminal so specific + so low-volume that it could imply something about an identifiable small group, e.g. a rare PwD sub-scenario in a sparsely-populated district).

**What stays admin-only, indefinitely:**

- Anything at the `hour_bucket` granularity — public publication should only ever read from day/week/month aggregates, specifically to avoid enabling the timing-correlation re-identification risk described in §12.2.2 from being reconstructed by an external party who has other side-channel timing information VoteAssist doesn't know about.
- Any per-node drop-off breakdown finer than the terminal level, since node-path combinations for rare scenarios can shrink to very small counts that would fail k-anonymity even where the top-level metric passes.
- Raw or rollup-table access of any kind — the public dashboard would be a *separate*, further-aggregated read-only view/materialized export, never a public credential onto `analytics_rollups_hourly` itself.
- `kb_query_category` breakdowns below a coarse category grouping, since a rare category name combined with a small state bucket could be more identifying in combination than either alone.

Any implementation of this feature must re-run the same privacy design review as the internal metrics before it ships, including an explicit sign-off from whoever holds the Compliance Owner role at that time — this section records intent and constraints, not a shipped feature.

### 12.7 Why custom Rust `crates/analytics`, not Plausible/Umami/PostHog (self-hosted)

Self-hosting a mature open-source analytics tool was seriously considered and is worth naming honestly rather than waving away. Plausible (Elixir/Phoenix), Umami (Node/Next.js), and PostHog (primarily Python/Django + ClickHouse, with a large surrounding product) are all real, credible, privacy-respecting, self-hostable options, and any one of them would arrive with a working ingestion pipeline, a working dashboard UI, and a maintained upgrade path that this project would otherwise have to build and maintain itself. Choosing to build `crates/analytics` from scratch is a real cost, not a free win: it means the team owns rollup-job correctness, dashboard query performance, and every future "can we add metric X" request that a mature tool would have handled out of the box, forever, with no upstream community to share that maintenance burden.

The recommendation is still custom Rust, for two concrete reasons rather than a generic "control our own stack" preference: (a) the event schema this product needs is decision-tree-specific — `node_id`, `answer_option` against a versioned tree schema, `terminal_id`, `kb_query_category` — not generic pageview/click analytics, so adopting Plausible/Umami/PostHog would still require building a custom event layer on top of (or instead of) their built-in pageview model, meaning the "we get a working tool for free" argument is weaker than it first looks; and (b) it keeps the entire production stack in one language (Rust) for what is expected to be a small maintainer team, avoiding the operational cost of running and keeping current an Elixir or Node runtime and its own dependency-security posture solely for analytics, alongside Rust for everything else. This is recorded as a deliberate tradeoff: the team is accepting real engineering cost (building and maintaining rollup jobs and a dashboard query layer) in exchange for schema fit and operational homogeneity, and should revisit this decision if the analytics feature set ever grows closer to what a general-purpose tool provides out of the box.

### 12.8 Event ingestion contract

`crates/analytics` exposes exactly one internal ingestion endpoint. It is
internal in the sense that only server-side components (`crates/api`,
`crates/bot-telegram`, `crates/bot-whatsapp`, `crates/ivr-gateway`) call it;
it is not reachable from the public internet under its own route, and it
sits behind the same service-mesh/internal-network boundary as other
inter-crate calls, so a public client cannot POST arbitrary events directly
even if it could guess the shape of the payload.

```json
POST /internal/events
Content-Type: application/json

{
  "event_type": "question_answered",
  "session_id": "b2b1a9d0-2f66-4d0a-9d34-2b6f9b4d6a11",
  "decision_tree_version": "2026.07.1",
  "node_id": "student_status",
  "answer_option": "student_hostel",
  "locale": "hi-IN",
  "client_platform": "web"
}
```

Validation rules enforced by the handler before any row is written:

1. `session_id` must be a syntactically valid v4 UUID; the handler does not
   check it against any prior table (there is nothing to check against —
   sessions are not pre-registered), it only rejects malformed values.
2. `event_type` must be one of the six enum values; an unrecognized value
   is a `400`, not silently coerced or stored as free text.
3. `node_id` / `answer_option` are validated against `core-domain`'s
   in-memory decision-tree definition for the given
   `decision_tree_version` — an `answer_option` that isn't a valid option
   for that `node_id` in that tree version is rejected, which doubles as a
   tamper check (§13.2) for anything that reaches this endpoint through an
   unexpected path.
4. `kb_query_category` (when present) must be one of the fixed taxonomy
   values described in §12.2.1; there is no "other free text" escape
   hatch.
5. The handler computes `occurred_at_hour` itself from the request's
   arrival time — callers cannot supply their own timestamp, which
   forecloses a caller (buggy or malicious) from writing a
   full-precision or backdated timestamp into the table.

Ingestion is fire-and-forget from the calling request's perspective: the
public-facing request (e.g. answering a question) returns to the user
immediately, and the analytics write happens on a background task /
short internal queue, so a slow or degraded `crates/analytics` instance
cannot add latency to — or become an outage vector for — the core
decision-engine experience described in §13.5.

### 12.9 Monitoring the pipeline itself

The rollup and purge jobs in §12.4 are exactly the kind of scheduled job
that fails silently if nobody is watching, and a silent rollup failure has
two bad outcomes at once: the dashboard quietly goes stale, and (because
purge is gated on rollup success) raw events start piling up past the
intended 30-day window. Per the observability decision (`tracing` +
OpenTelemetry OTLP to self-hosted Grafana/Loki/Tempo), each job run emits
a span (`analytics.hourly_rollup`, `analytics.daily_rollup`,
`analytics.raw_event_purge`) with row-count and duration attributes, and:

- A Grafana alert fires if `hourly_rollup` has not completed successfully
  within a set window (proposed: 2 hours without a success), paging the
  on-call the same way an API outage would.
- A separate alert tracks the age of the oldest un-purged
  `analytics_events` row; if it exceeds the 30-day retention window plus a
  small grace period, that is treated as a policy-compliance incident, not
  merely an ops nuisance, because it means the retention hard requirement
  in §12.4 is being violated in production.
- Rollup/purge job logs must obey the same log-hygiene rule as request
  logs (§13.4): no `answer_option`, `kb_query_category`, or any other
  event-level field is ever logged at a per-row level, only aggregate
  counts. A rollup job that logged its own input rows for debugging would
  quietly recreate the raw-event exposure the purge job exists to bound.

### 12.10 Open questions

- Whether `analytics_rollups_daily` retention should be a fixed 24 months
  or tied to an explicit product decision to sunset older comparisons —
  flagged here as a Compliance Owner call rather than resolved by this
  document.
- Whether channel-adapter-specific identifiers (a Telegram `chat_id`, a
  WhatsApp phone-number-derived identifier) need an additional explicit
  hashing step before even transiently touching adapter code, beyond
  "must not be reused as `session_id`" — worth a dedicated review once
  `crates/bot-telegram`/`crates/bot-whatsapp` are implemented rather than
  scaffolded, since bot platforms often hand the adapter a stable user
  identifier by default and the discipline of never letting it leak into
  `analytics_events` needs to be enforced in code, not just in this PRD.

## 13. Security Threat Model v2

STRIDE categories below, each with concrete VoteAssist-specific threats and mitigations tied to the Rust architecture decisions and to sections 8 (database), 9 (compliance), 11 (admin/RBAC), and 12 (analytics) of this PRD.

### 13.1 Spoofing

| Threat | Detail | Mitigation |
|---|---|---|
| Phishing sites impersonating VoteAssist-as-if-official-ECI | A third party clones VoteAssist's UI (or builds a lookalike) to harvest Aadhaar/EPIC numbers or credentials, trading on either VoteAssist's or ECI's name recognition | Persistent, non-dismissible-past-minimized "we are not the ECI, we are an independent civic-tech project" banner is a product-design requirement (not just a legal footnote), carried into the Leptos SSR templates so it renders even for non-JS/low-bandwidth clients; domain/branding guidance for operators — never register or promote a domain that could be mistaken for `eci.gov.in`/`voters.eci.gov.in`/`ecinet.eci.gov.in`, and defensively register close look-alike variants of VoteAssist's own domain where feasible |
| Spoofed API clients hitting public endpoints | Since core decision-engine endpoints are intentionally anonymous/no-auth (consistent with the non-goal of requiring an account), a scripted client can impersonate a normal browser session at volume | Rate limiting + bot-detection heuristics are the primary defense here since there is no auth layer to strengthen — see §13.5; server-side validation (§13.2) prevents a spoofed client from injecting invalid state even if it gets requests through |
| Admin auth spoofing | Credential-stuffing or brute-force against `crates/admin-app` login | `argon2` password hashing (memory-hard, per the Rust architecture decisions) with a tuned cost parameter reviewed periodically as hardware improves; rate-limited login attempts per account and per IP (`tower-governor`); optional passkeys (WebAuthn) as a stronger-than-password path for `superadmin`/`legal_reviewer` roles specifically, recommended as required (not merely optional) for those two highest-impact roles once the passkey integration lands |

### 13.2 Tampering

- **KB content and decision-tree tampering.** Because terminal outcomes drive real user action on a real government process, unauthorized or unreviewed modification of `crates/kb-content` source files or `crates/core-domain` tree definitions is a high-impact risk, not merely an editorial one. Mitigations, layered:
  - The revision/audit-trail table (§11) records every content and tree change with who/when/before-after diff, independent of Git history, so the review record survives even if repository history were somehow rewritten.
  - RBAC gates (`contributor` drafts, `reviewer` approves into `verified`, `legal_reviewer` sign-off required for anything touching forms/eligibility claims) mean no single compromised account can push tampered content straight to production.
  - The decision-tree validate-before-publish gate (§11) runs the `proptest`-verified invariants (every path terminates, every terminal cites a source) against any candidate tree before it can be promoted, so a tampered or malformed tree fails structurally before a legal reviewer even sees it, catching a class of tampering that a human reviewer skimming a diff might miss.
  - `decision_tree_version` is recorded per session (§12.2), so if a bad version does ship, every affected session is traceable to the exact reviewed version for incident response.
- **Client-side tampering with answer submission.** A malicious client could submit an `answer_option` value outside the valid enum for a node, or submit answers out of the tree's actual reachable order. Mitigation: `crates/api` validates every submitted answer against `core-domain`'s authoritative node schema server-side, rejecting anything not in the node's defined option set or not reachable from the session's current tree position — the server, never the client, is the source of truth for valid tree state.
- **Supply-chain tampering (Rust-specific).** A compromised or maliciously-updated Cargo dependency is a realistic vector for a Rust-heavy codebase with many small crates in its dependency graph.
  - `cargo audit` runs in CI against the RustSec advisory database on every PR and on a scheduled basis (to catch newly-disclosed CVEs against already-merged dependencies, not just at merge time).
  - `cargo deny` enforces license policy and flags duplicate/multiple-version dependencies in CI, both a supply-chain hygiene signal and a build-hygiene one.
  - `Cargo.lock` is committed and treated as authoritative; CI builds never float to a newer transitive version silently.
  - **New-dependency policy:** any PR introducing a new *direct* dependency requires a second reviewer's explicit sign-off on that dependency specifically (not just the surrounding code change) — checking maintainer reputation, recent commit activity, and whether the functionality could reasonably be hand-rolled or covered by an existing dependency already in the tree. This is deliberately more friction than a typical PR review, proportionate to `core-domain` and `crates/api` being the trust boundary for a civic-guidance tool.

### 13.3 Repudiation

- Every content-review action (draft → in_review → verified → needs_reverification transitions), every decision-tree publish, and every RBAC role change is written to the audit log (§11) with actor, timestamp, and before/after state. This log is append-only from the application's perspective (no UPDATE/DELETE path exposed through `crates/admin-app`; only `superadmin`-gated, logged-separately direct DB access could alter it, which is itself an anomaly worth alerting on).
- Admin actions are tied to an authenticated account (argon2-hashed password or passkey, §13.1), so "who approved this KB change" is always answerable and non-repudiable in a way the anonymous public product deliberately is not.
- Feedback-form `contact_email` remains explicitly **not** a non-repudiation mechanism — it's unauthenticated and trivially spoofable/omittable, so no security or compliance decision may rely on it as proof of who submitted feedback (unchanged from the v1 model, restated here because it's easy to forget once an email field exists in a schema).
- Anonymous public-product usage (decision-engine sessions, analytics events) has no repudiation concept at all, by design — there is no "user" to hold accountable for their own answers, since the product doesn't authenticate them and structurally can't (§12.3's session-rotation policy forecloses building one later without a deliberate, reviewed change).

### 13.4 Information Disclosure

- **Analytics data minimization** is the primary information-disclosure control for the whole product — see §12 in full. The schema, hour-bucketing, session rotation, and 30-day raw retention window together mean that even a full compromise of `analytics_events` yields no persistent per-person timeline, no raw search text, and no full-precision timing fingerprint.
- **Rate limiting on the KB search endpoint** (`tower-governor` or equivalent Axum middleware) is a disclosure control, not just a DoS control: an unthrottled search endpoint would let an attacker scrape query-response pairs at a rate/pattern that could itself become a fingerprinting or profiling vector against whichever anonymous client is issuing the requests, separate from the deliberate non-storage of raw query text in §12.2.1. Rate limits here are tuned per-session and per-IP, generous enough not to disrupt a real user's normal search-then-browse behavior.
- **No verbose error messages leaking internal state.** `crates/api` error responses to public/admin clients are mapped to a small, deliberately uninformative set of client-facing error variants (validation error, not found, rate limited, internal error) — stack traces, SQL error text, and internal type names never cross the API boundary. Full detail goes to `tracing`/OpenTelemetry spans server-side only (see observability decision), and those spans themselves must never include the raw search text, raw answer free-text (there isn't any, by schema), or any DPDP-sensitive field — log-scrubbing is a required code-review check on any new `tracing::info!`/`error!` call that touches request data.
- **Feedback `contact_email` handling.** Encrypted at rest at minimum via column-level encryption or, at minimum, access restricted to the `analytics_viewer`/`superadmin` roles that actually need it for follow-up, never readable by the general `contributor`/`reviewer`/`translator` roles; short retention (proposed: purged or anonymized after the feedback item is resolved plus a bounded grace period, exact number a Compliance Owner call per §9), and never joined against `analytics_events`/session data under any circumstance.
- **Deep-link referrer leakage.** Outbound links to `voters.eci.gov.in`/ECINET must not leak session or answer-derived path information to the destination via the referrer header — `Referrer-Policy: strict-origin-when-cross-origin` (or stricter, `no-referrer` for the most sensitive terminals, e.g. PwD-related ones) is required on all deep-link anchors, so the destination site cannot infer a user's specific scenario (e.g., "arrived via the PwD home-voting terminal") from the referring URL alone.

### 13.5 Denial of Service

- **Public decision-engine endpoints are cheap by design.** Per the Rust architecture's preference for client-held session/tree-position state, answering a question in the common case does not require a database write on the hot path — the client holds its own position in the tree and only emits an analytics event asynchronously (fire-and-forget from the request's perspective, batched/queued rather than blocking the user-facing response). This shrinks the DoS blast radius structurally: an attacker flooding the question-answer endpoint is mostly hitting stateless compute, not contending for database connections or write throughput, which is the resource that would actually take the whole product down under load.
- **Per-IP rate limiting tuned for shared-IP contexts.** Given the target audience genuinely includes public libraries and cyber-cafes (a real, named usage pattern, not a hypothetical edge case), rate limits must be generous enough that a dozen concurrent legitimate users behind one NAT/proxy IP are not throttled as if they were a single abusive client. `tower-governor` (or equivalent) should key primarily on session ID for endpoints where one exists, falling back to IP-based limits only for pre-session endpoints (e.g. the very first `session_started` call), and IP-based limits should use a noticeably higher threshold than a typical single-user-per-IP assumption would suggest.
- **CDN/WAF layer recommended in front of the public site** (e.g. Cloudflare or an equivalent, self-hostable-origin-preserving provider) specifically for the real traffic-spike scenario this product will face: an election-schedule announcement day, when usage can spike by an order of magnitude with almost no warning. This is a recommendation for the actual deployment topology, not a requirement baked into the origin services themselves — the origin (Axum services in distroless Docker images) must remain fully self-hostable and functional without the CDN/WAF layer present, so that operators without access to (or budget/comfort for) a commercial CDN can still run the product; the CDN/WAF is an additive resilience layer for the specific spike scenario, not a load-bearing dependency of the architecture.
- **Static/cacheable public content** (KB entries, form explainers) should be served with cache headers aggressive enough that a CDN or even browser cache absorbs most read traffic during a spike, leaving origin capacity for the genuinely dynamic decision-engine and search endpoints.

### 13.6 Elevation of Privilege

- **RBAC enforcement matches section 11's role gates exactly, point-by-point:**

  | Role | Can do | Explicitly cannot |
  |---|---|---|
  | `contributor` | Draft KB entries/translations | Self-promote to `reviewer`/`legal_reviewer`/any elevated role; approve their own drafts into `verified` |
  | `reviewer` | Approve content into `verified` status | Sign off on anything touching forms/eligibility claims (that's `legal_reviewer`-only); grant roles to other accounts |
  | `legal_reviewer` | Sign-off gate for forms/eligibility-claim content | Bypass the `reviewer` approval step that must precede legal sign-off; edit content directly without going through the draft/review pipeline |
  | `translator` | Submit/edit translations | Alter the underlying English-source content or decision-tree logic |
  | `analytics_viewer` | Read `analytics_rollups_hourly`/`daily` and the admin dashboard | Read raw `analytics_events`; read feedback `contact_email` values |
  | `superadmin` | Grant/revoke roles, direct DB access for break-glass scenarios | Exempt from the audit log — every `superadmin` action is still logged, specifically because this is the role a compromised-account scenario would target |

  Every one of the "cannot" cells is enforced server-side in `crates/api`/`crates/admin-app` authorization middleware, not left as a UI-only restriction — the admin app's Leptos client never trusts its own hidden-button logic as a security boundary.
- **Session fixation / CSRF protections for the Leptos admin app.** `tower-sessions` configuration for `crates/admin-app` specifically: session ID regenerated on login (never reusing a pre-auth session ID post-authentication, closing the standard fixation vector), `SameSite=Strict` cookies (the admin app is a same-site-only surface, unlike the public product which has no login at all), `Secure` + `HttpOnly` flags always set, short absolute session lifetime with re-authentication required for `legal_reviewer`/`superadmin` actions specifically (step-up auth for the highest-impact role actions, not just a flat session timeout), and CSRF tokens on all state-changing admin forms (content publish, role grant) validated server-side per request, not relying on `SameSite` alone given the sensitivity of the actions involved.
- **Least-privilege database roles.** The `crates/api` runtime Postgres role has DML rights only (SELECT/INSERT/UPDATE/DELETE on the specific tables it needs) and explicitly **no DDL rights** in production — it cannot `CREATE`/`ALTER`/`DROP` a table even if a bug or injection somehow reached raw SQL execution. Schema migrations run under a separate, more-privileged role used only by the CI/CD deploy pipeline at migration time, never by the long-running application process, and that elevated role's credentials are not present in the running service's environment/secrets at all — a compromised running API process therefore cannot alter its own schema, only misuse the data access it already legitimately has.

### 13.7 Data Classification Table

Synthesizing the DPDP-driven decisions from sections 8 (database schema), 9 (compliance), and 12 (analytics):

| Data category | Sensitivity | Storage location | Retention policy |
|---|---|---|---|
| KB content (entries, form explainers, citations) | Public | Postgres (`kb_entries` + versioned revisions), served via `crates/kb-content` / Meilisearch index | Indefinite; superseded versions retained in the audit-trail table for accountability, not deleted |
| Decision-tree definitions | Public (the logic itself is open-source), but change-controlled | `core-domain` source + versioned tree table, audit-trail revisions | Indefinite; every published version retained and addressable by `decision_tree_version` for traceability |
| Analytics events (raw) | Internal, aggregate-sensitive, no PII by schema design | `analytics_events` (Postgres) | 30 days, then hard-deleted after rollup confirmation (§12.4) — not an aspiration |
| Analytics rollups (hourly/daily) | Internal, low individual-re-identification risk given aggregation + hour/day bucketing | `analytics_rollups_hourly` / `analytics_rollups_daily` | Retained longer-term (proposed: 24 months) to support year-over-year comparisons; exact figure a Compliance Owner call, published in the privacy policy |
| Feedback + optional `contact_email` | Personal data (low-sensitivity but still DPDP-in-scope) | Postgres, access-restricted to `analytics_viewer`/`superadmin`, encrypted at rest where the hosting environment's disk/column encryption supports it | Short retention: purged or anonymized after resolution + bounded grace period; deletable on request per DPDP data-principal rights |
| Admin credentials (password hashes, passkey public keys) | Confidential | Postgres (`argon2` hashes only, never plaintext; passkey public key material only, no private key ever leaves the user's authenticator) | Retained while the account is active; rotated/revoked immediately on role change-off or suspected compromise |
| Infrastructure secrets (DB credentials, API keys, signing keys, OTel/Grafana credentials) | Confidential | Secrets manager / CI secrets store, never committed to the repository | Rotated on suspected exposure and on a routine schedule; never logged, never present in `tracing` spans |
| Never collected, by schema (Aadhaar, EPIC, passport numbers, caste, religion, party affiliation, political opinion, uploaded ID documents, raw KB search text, full-precision event timestamps, cross-session device fingerprints) | Prohibited | N/A — no field in any schema in this PRD is capable of holding these | Any accidental capture is treated as a security incident requiring immediate deletion, root-cause fix, and a schema-level change to make recurrence structurally impossible, not just a data-scrub |

### 13.8 Observability and its own security surface

The observability decision (`tracing` + `tracing-subscriber` + OpenTelemetry
OTLP export to a self-hosted Grafana/Loki/Tempo stack, with `sentry-rust`
supported but off by default) is itself a security-relevant design choice,
not a neutral ops convenience, for two reasons:

1. **Self-hosted-by-default avoids creating a second, less-scrutinized data
   processor.** If observability defaulted to a foreign-hosted SaaS (a
   commercial APM vendor, or Sentry's own cloud), every span and log line —
   which, despite the log-hygiene rules in §13.4, still contains request
   metadata, error context, and infrastructure detail — would be leaving
   Indian-controlled infrastructure by default for a platform whose core
   promise is data-minimal, India-conscious handling of citizens'
   interactions with an electoral process. Keeping Grafana/Loki/Tempo
   self-hosted means the operator retains the same control over
   observability data that they have over the primary Postgres data.
2. **Sentry-off-by-default is a deliberate reduction of information-
   disclosure surface, not just a cost decision.** Error-tracking tools are
   specifically designed to capture rich context around a failure —
   request bodies, stack frames, local variable state — which is exactly
   the kind of capture that could defeat the log-hygiene discipline in
   §13.4 if enabled carelessly. If an operator does opt in to `sentry-rust`,
   it must either point at a self-hosted Sentry instance, or, if a
   commercial Sentry SaaS tier is used, that fact must be disclosed in the
   public privacy policy as a data processor per DPDP, and the Sentry
   integration must apply the same scrubbing rules (no raw search text, no
   full-precision cross-session timing, no `contact_email`) before
   attaching request context to an error event — the "off by default"
   posture exists precisely because getting that scrubbing right is easy to
   get wrong under the pressure of debugging a live incident.

Grafana/Loki/Tempo access itself follows the same least-privilege posture as
§13.6: only operators with an operational need see traces/logs, and the
dashboards must not become a shadow analytics tool that lets someone
reconstruct individual-session behavior by grepping trace attributes —
the same event-schema minimization in §12 applies equally to what
`tracing` spans are allowed to carry as attributes.

### 13.9 Incident response posture (summary)

This PRD does not replace a standalone incident-response runbook, but the
threat model above implies a minimum posture:

| Trigger | First responder | Immediate action |
|---|---|---|
| KB/decision-tree content found to be materially wrong or tampered | `legal_reviewer` + `superadmin` | Roll back to the last audit-trail-verified version; suspend publish rights for the implicated account pending review |
| `analytics_events` retention alert (§12.9) fires | On-call engineer | Treat as a compliance incident, not just an ops ticket; fix the blocked rollup job and confirm purge catches up before closing |
| Suspected credential compromise (`admin-app`) | `superadmin` | Force-rotate the affected account's credentials/passkey, review the audit log for actions taken under that account since the suspected compromise window, rotate any secrets that account could reach |
| Accidental capture of a prohibited data category (§13.7 "Never collected" row) | Whoever discovers it, immediately | Delete the data, treat as a security incident (not merely a bug), root-cause the schema/validation gap, ship a structural fix (schema constraint or validation rule) that makes the specific recurrence impossible, not just a one-off deletion |
| Dependency CVE flagged by `cargo audit` | On-call engineer / dependency owner | Assess exploitability against actual usage of the affected crate; patch or pin away from the vulnerable version; if unpatchable immediately, document the accepted-risk window explicitly rather than silently ignoring the finding |

### 13.10 Open questions

- Exact passkey rollout timeline for `superadmin`/`legal_reviewer` (the
  Rust architecture decisions note passkeys as an "optional later" addition
  via an `oauth2-passkey-axum`-style crate) — recommended to become
  mandatory, not optional, for those two roles once available, but the
  concrete migration date is a product/ops call outside this document's
  scope.
- Whether column-level encryption for `contact_email` is handled at the
  application layer (encrypt before INSERT, decrypt only in the
  `analytics_viewer`/`superadmin` read path) or via Postgres-native
  mechanisms (e.g. `pgcrypto`) — both are viable; the choice affects key-
  management design and should be settled alongside the migration that
  introduces the feedback table, not assumed here.
- Whether the CDN/WAF layer recommended in §13.5 should be named as a
  specific vendor in deployment documentation or left generic — naming one
  risks reading as an endorsement inconsistent with the project's open-
  source, self-hostable-first posture; leaving it generic risks operators
  under-provisioning for the election-day spike scenario with no concrete
  guidance. Flagged for the deployment/ops section of this PRD to resolve,
  not decided here.
## 14. Full Feature & Task Backlog

This backlog is organized as Epics, each with numbered Features, each with
numbered Tasks (`E<epic>.F<feature>.T<task>`). It is written to be directly
importable into an issue tracker. Every task is scoped to be independently
completable and testable. Tasks marked **[MVP-Rust-v1]** are required for
the migration-parity milestone (Section 21); tasks marked **[v1]**, **[v2]**,
or **[v3]** belong to later roadmap phases. Unmarked tasks are assumed
**[MVP-Rust-v1]** unless the epic header says otherwise.

### EPIC 1 — Rust Workspace Foundation

**Feature 1.1 — Workspace scaffolding**
- E1.F1.T1 Create `rust/` Cargo workspace with the crate layout in Section 6.2.
- E1.F1.T2 Set up workspace-level `Cargo.toml` with shared dependency version pins.
- E1.F1.T3 Configure `rustfmt.toml` and `clippy.toml` with project-wide lint levels (`-D warnings` in CI).
- E1.F1.T4 Add `xtask` crate with subcommands: `validate-kb`, `gen-openapi`, `seed-dev-db`.
- E1.F1.T5 Add `deny.toml` (cargo-deny config: license allowlist, banned crates, duplicate-version policy).
- E1.F1.T6 Add `.cargo/config.toml` with any workspace-wide build settings (e.g., faster local linker config).

**Feature 1.2 — Local dev environment**
- E1.F1.T1 Write `docker-compose.yml`: Postgres 16, Meilisearch, Grafana+Loki+Tempo.
- E1.F2.T2 Write a `README-rust.md` (or extend the main README) with local setup steps.
- E1.F2.T3 Add a `.env.example` documenting every environment variable each service reads.
- E1.F2.T4 Verify `cargo nextest run --workspace` succeeds against an empty/scaffolded workspace as the very first CI-green baseline.

### EPIC 2 — `core-domain` (decision engine port + expansion)

**Feature 2.1 — Type port from TypeScript**
- E2.F1.T1 Port `LocalizedText`/`Option`/`QuestionNode`/`TerminalNode`/`DecisionTree` types from `packages/decision-engine/src/types.ts` into Rust structs/enums with `serde`.
- E2.F1.T2 Port `createSession`/`getCurrentNode`/`answer`/`isSessionComplete`/`estimateProgress` engine functions.
- E2.F1.T3 Port `validateTree` (acyclic check, citation/deep-link presence check, reachability check) as a Rust function returning a `Vec<TreeValidationIssue>`.
- E2.F1.T4 Port the existing ~21-node `voteAssistTreeV1` tree content as a Rust data literal (or a loaded JSON/TOML artifact — resolve per Section 9's authoring-format decision).
- E2.F1.T5 Port the existing Vitest test suite's assertions into `cargo nextest` unit tests (structural validity, walk-specific-paths, invalid-answer rejection, terminal-refusal-to-answer).

**Feature 2.2 — Property-based testing**
- E2.F2.T1 Write a `proptest` strategy that generates arbitrary answer sequences and asserts every session reaches a terminal within a bounded step count.
- E2.F2.T2 Write a `proptest`/exhaustive test asserting every terminal node has ≥1 citation resolving to a real `kb-content` entry (integration test spanning both crates).
- E2.F2.T3 Write a test asserting no `DecisionTree` node is unreachable from `startNodeId`.
- E2.F2.T4 Wire `criterion` benchmarks for tree traversal at a synthetic large-tree scale (target: define and assert a p99 latency budget).

**Feature 2.3 — Decision tree v2 content (see Section 10 for full scenario list)**
- E2.F3.T1 Add migrant worker / interstate-vs-intrastate move clarification node + UI copy addressing the "interstate move needs a different form" misconception.
- E2.F3.T2 Add married-woman-changing-residence path (verify it correctly composes existing shifting + correction terminals, no new form invented).
- E2.F3.T3 Add government/private employee transfer clarification (branch to service-voter terminal only for qualifying government/armed-forces categories).
- E2.F3.T4 Extend PwD terminal into a shared "postal ballot eligibility" concept covering both PwD and 85+ senior citizens.
- E2.F3.T5 Add "flag for local verification" terminal pattern for tribal/remote/urban-slum/homeless scenarios (explicitly not fabricating state-specific procedures).
- E2.F3.T6 Add transgender citizen self-identified gender-marker path — **[MVP-Rust-v1 blocked pending KB research, see E5.F2.T5]**.
- E2.F3.T7 Add "which documents do you have" multi-select node for citizens lacking Aadhaar/passport/DL/permanent address, feeding a documents-alternatives terminal.
- E2.F3.T8 Extend `terminal_form8_correction` helpText to explicitly cover both name-mismatch and DOB-mismatch framing.
- E2.F3.T9 Add duplicate-voter-ID clarification (Form 7 objection vs. Form 8 correction depending on whether it's the same person's data duplicated or a genuinely separate erroneous entry).
- E2.F3.T10 Add deleted-voter-ID path (re-registration via Form 6 vs. Form 7 to contest a wrongful deletion).
- E2.F3.T11 Add shifted-polling-station-without-address-change informational terminal (reuses roll-search pattern, explicitly not a Form 8 scenario).
- E2.F3.T12 Run `validateTree` against the expanded v2 tree and fix any issues before merge.
- E2.F3.T13 **[v1]** Expand tribal/remote/slum/homeless flagging into actual state-specific guidance once per-state legal review (Section 21 v2 item) produces verified content.

**Feature 2.4 — Versioning**
- E2.F4.T1 Add a `version` field + semver-ish bump policy to `DecisionTree`.
- E2.F4.T2 Define the JSON/TOML artifact format that `kb-content`/the admin tree editor read and write (resolve open question in Section 22).
- E2.F4.T3 **[v1]** Support loading multiple tree versions simultaneously (for admin preview/rollback and analytics A/B comparison across versions, Section 12).

### EPIC 3 — `kb-content` (knowledge base loader/validator)

- E3.F1.T1 Derive `schemars`-based `KnowledgeEntry` struct matching `knowledge-base/schema/entry.schema.json` exactly (Section 9).
- E3.F1.T2 Write a round-trip test: every existing file in `knowledge-base/sources/*.json` deserializes into the struct and reserializes without data loss.
- E3.F1.T3 Add a CI check (`xtask validate-kb`) that fails the build if any KB entry fails schema validation.
- E3.F1.T4 Implement `getEntry`/`listByTopic`/`searchEntries` equivalents (port from `packages/knowledge/src/index.ts`), backed initially by in-memory lookup, later by Meilisearch (E9.F1).
- E3.F1.T5 Set up `.ftl` file structure under `i18n/<locale>/` and wire `fluent-templates` loading.
- E3.F1.T6 Migrate existing inline `en`/`hi` strings (KB body text + tree prompts/options) into FTL message format with correct message IDs (Section 9's worked example).
- E3.F1.T7 Add an i18n completeness test: every shipped locale's FTL files contain the same message IDs as English (Rust-side equivalent of the existing TS `packages/i18n` test).
- E3.F1.T8 Add a "days since lastVerifiedDate" computed field/query used by the re-verification digest job (E11.F2).

### EPIC 4 — `api` (Axum HTTP layer)

- E4.F1.T1 Scaffold Axum app with the route groups from `docs/10-api-specification.md` / `openapi/voteassist-api.yaml`: sessions, kb, forms, constituencies/locate, feedback.
- E4.F1.T2 Implement `POST /v1/sessions`, `POST /v1/sessions/{id}/answers`, `GET /v1/sessions/{id}/result`, `POST /v1/sessions/{id}/deep-link-click` against `core-domain`.
- E4.F1.T3 Decide and implement public session storage: client-held signed token (recommended, Section 8) vs. short-TTL Redis/Postgres row — resolve per Section 8's DB design.
- E4.F1.T4 Implement `GET /v1/kb/search`, `GET /v1/kb/entries/{slug}`, `GET /v1/kb/glossary`.
- E4.F1.T5 Implement `GET /v1/forms`, `GET /v1/forms/{code}`.
- E4.F1.T6 Implement `GET /v1/constituencies`, `GET /v1/locate/deep-links` against the `states`/`assembly_constituencies`/`parliamentary_constituencies` reference tables (Section 8).
- E4.F1.T7 Implement `POST /v1/feedback`.
- E4.F1.T8 Annotate every handler with `utoipa` macros; wire `utoipa-swagger-ui` at a docs endpoint.
- E4.F1.T9 Add `xtask gen-openapi` to emit and commit the generated spec, replacing the hand-written YAML as the contract of record.
- E4.F1.T10 Add `tower-governor` rate limiting tuned for shared-IP usage (public library/cyber-cafe scenario, Section 13).
- E4.F1.T11 Add `tracing` spans across every handler; wire OpenTelemetry OTLP export.
- E4.F1.T12 Add structural error handling that never leaks internal state (Section 13 requirement).

**Feature 4.2 — Admin auth**
- E4.F2.T1 Implement `tower-sessions` session middleware for `admin-app`/`api` admin routes.
- E4.F2.T2 Implement `argon2` password hashing + login/logout endpoints.
- E4.F2.T3 Implement RBAC middleware enforcing the six roles (Section 11) per route.
- E4.F2.T4 **[v1]** Add passkey/WebAuthn login option.

### EPIC 5 — Content Operations (Knowledge Base)

**Feature 5.1 — Content migration**
- E5.F1.T1 Confirm all 11 existing KB entries (form-6, form-6a, form-7, form-8, service-voter, qualifying-dates, ordinary-residence-student, pwd-home-voting, e-epic, helpline-grievance, roll-search-polling-station) pass the new `kb-content` schema round-trip test unchanged.
- E5.F1.T2 Migrate each entry's inline body text into FTL message references (Section 9).

**Feature 5.2 — New content required by decision tree v2**
- E5.F2.T1 Research and write a KB entry for interstate-vs-intrastate move clarification (confirm no legal distinction exists for Form 8, cite RP Act/Rules).
- E5.F2.T2 Research and write a KB entry for duplicate/deleted entry resolution mechanics (Form 7 vs. re-registration nuance).
- E5.F2.T3 Research and write a KB entry for shifted-polling-station-without-move (delimitation/redistricting) scenario.
- E5.F2.T4 Research and write a KB entry generalizing "alternate documents accepted" guidance for citizens lacking Aadhaar/passport/DL/permanent address.
- E5.F2.T5 Research (with legal_reviewer sign-off) and write a KB entry for transgender elector self-identified gender-marker updates on Form 6/Form 8 — flagged in Section 10 as needing dedicated research, do not guess.
- E5.F2.T6 **[v1]** Begin per-state legal review program for tribal-area/urban-slum/homeless address-proof variations (Section 21).

**Feature 5.3 — Content quality processes**
- E5.F3.T1 Define the `reviewStatus` state machine transitions and role gates precisely (draft → in_review → verified → needs_reverification) as enforced code, not just documentation.
- E5.F3.T2 Implement the nightly dead-citation-link checker job (`jobs` crate, `link_check_results` table).
- E5.F3.T3 Implement the re-verification-due digest job (KB entries aged past threshold).
- E5.F3.T4 Write the `knowledge_entry_revisions` audit-trail write path (every save creates a revision row).

### EPIC 6 — `web-app` (public site, Leptos SSR)

- E6.F1.T1 Scaffold Leptos SSR app with the routes from `docs/05-information-architecture.md`: home/decision-flow, knowledge browser, locate, about/legal.
- E6.F1.T2 Port the question/answer island component (equivalent of the TS `VoteAssistApp`/`QuestionCard`/`TerminalResult` components) to Leptos.
- E6.F1.T3 Port the language switcher island.
- E6.F1.T4 Implement the persistent "not an official ECI site" banner (non-negotiable, Section 4).
- E6.F1.T5 Implement the knowledge-base browser page (list + search, backed by `GET /v1/kb/search`).
- E6.F1.T6 Implement the locate page (polling station/BLO/ERO/CEO deep links, backed by `GET /v1/locate/deep-links`).
- E6.F1.T7 Implement offline/low-bandwidth caching strategy (PWA-equivalent for the SSR app, Section 18).
- E6.F1.T8 Verify Lighthouse score >95 on the ported app.
- E6.F1.T9 Run the existing Playwright smoke-test pattern (question flow → terminal → citation → deep link, English + Hindi) against the Leptos app, adapted from the TS-prototype smoke test already proven in this repo.
- E6.F1.T10 **[v1]** Add remaining accessibility features from Section 18: dyslexia-friendly font toggle, text resizing control, reduced-motion respect, high-contrast mode.

### EPIC 7 — `admin-app` (Leptos SSR admin dashboard)

**Feature 7.1 — [v1] Core admin shell**
- E7.F1.T1 Scaffold `admin-app` as a separate Leptos SSR binary/deploy target.
- E7.F1.T2 Implement login page + session handling.
- E7.F1.T3 Implement the Dashboard (home) page per admin spec (Section 11, page 2).

**Feature 7.2 — [v1] Content & tree management**
- E7.F2.T1 Implement the Knowledge Base Content Editor (Section 11, page 3): list/filter, create/edit form, diff view, reviewStatus state-machine UI, source-citation sub-editor with live link-check button.
- E7.F2.T2 Implement the Decision Tree Visual Editor (Section 11, page 4): node-graph view, inline node editing, server-side validation call reusing `core-domain::validate_tree`, publish-new-version flow gated to `legal_reviewer`, version history/rollback, "preview as a user" mode.
- E7.F2.T3 Implement the Translation Management page (Section 11, page 5): per-locale completeness dashboard, translation workbench, MT-assist-as-draft-only button, role-gated review/approve.
- E7.F2.T4 Implement the Citation & Link Health page (Section 11, page 6).

**Feature 7.3 — [v1] Operations**
- E7.F3.T1 Implement Feedback & Grievance Triage (Section 11, page 7), including the explicit in-UI reminder against acting as an unofficial ECI support desk.
- E7.F3.T2 Implement the MCC / Election-Period Control Panel (Section 11, page 8), including the per-channel broadcast kill-switch.
- E7.F3.T3 Implement Bot Channel Management (Section 11, page 9) — Telegram/WhatsApp config status, WhatsApp template-message review workflow (Meta's 24h/template-approval constraint).
- E7.F3.T4 Implement User & Role Management (Section 11, page 10), superadmin-only.
- E7.F3.T5 Implement the unified Audit Log Viewer (Section 11, page 11).
- E7.F3.T6 Implement the Analytics Dashboard UI (Section 11, page 12) — consumes the aggregates from EPIC 12, no user-level drill-down.
- E7.F3.T7 Implement Data Export & Retention Tools (Section 11, page 13), including confirmation steps and audit logging for retention-purge actions.

### EPIC 8 — Multi-channel bots

**Feature 8.1 — [v2] Telegram**
- E8.F1.T1 Scaffold `bot-telegram` with `teloxide`, wire webhook or long-polling per deployment environment.
- E8.F1.T2 Implement the `ChannelAdapter` rendering of question nodes as inline keyboards.
- E8.F1.T3 Implement terminal-outcome rendering (checklist, citations, deep links) as Telegram messages.
- E8.F1.T4 Implement language selection command.
- E8.F1.T5 Wire the MCC kill-switch (E7.F3.T2) to pause/throttle proactive Telegram broadcasts.
- E8.F1.T6 End-to-end test the full flow against a test Telegram bot instance.

**Feature 8.2 — [v2] WhatsApp**
- E8.F2.T1 Scaffold `bot-whatsapp` with a Meta Cloud API client (thin `reqwest` wrapper).
- E8.F2.T2 Implement interactive list/button message rendering for question nodes.
- E8.F2.T3 Implement the 24-hour-session / template-message constraint handling (Meta policy) — coordinate with E7.F3.T3's admin review workflow.
- E8.F2.T4 Implement terminal-outcome rendering.
- E8.F2.T5 **[v2, evaluate at implementation time]** Evaluate migrating to Gupshup BSP if template/scale needs exceed direct Meta API practicality.

**Feature 8.3 — [v3] IVR**
- E8.F3.T1 Define the `ivr-gateway` interface (already scaffolded per Section 6.7) against Exotel's actual webhook/streaming API.
- E8.F3.T2 Implement TTS rendering of question prompts in at least Hindi + English via Exotel's Indic STT/TTS support.
- E8.F3.T3 Implement DTMF and/or speech-recognized answer capture.
- E8.F3.T4 Implement terminal-outcome rendering as a spoken summary + SMS/WhatsApp follow-up with the deep link (a phone call can't "click" a link — design the handoff explicitly).

### EPIC 9 — Search

**Feature 9.1 — [MVP-Rust-v1 candidate, confirm scope at implementation time]**
- E9.F1.T1 Stand up Meilisearch in `docker-compose` and production deployment.
- E9.F1.T2 Index KB entries + glossary + forms into Meilisearch on content publish (triggered from E5.F3.T4's revision write path).
- E9.F1.T3 Wire `GET /v1/kb/search` to query Meilisearch instead of the naive in-memory substring search (E3.F1.T4's initial implementation).
- E9.F1.T4 **[v2]** Evaluate multilingual search quality across shipped locales; tune typo-tolerance settings per language.

### EPIC 10 — Forms & Constituency Reference Data

- E10.F1.T1 Seed the `forms` table with Form 6/6A/7/8/2/12D metadata (title, summary, superseded_forms).
- E10.F1.T2 Source and seed `states`/`assembly_constituencies`/`parliamentary_constituencies` reference data from public ECI notifications / data.gov.in aggregate datasets (never per-elector data).
- E10.F1.T3 Implement `GET /v1/constituencies` autocomplete search.
- E10.F1.T4 Implement `GET /v1/locate/deep-links` per-state deep-link lookup table (polling-station locator, BLO/ERO/CEO contact deep links).
- E10.F1.T5 **[v1]** Add a data-freshness job re-checking constituency reference data after each delimitation/notification cycle.

### EPIC 11 — Background Jobs

- E11.F1.T1 Scaffold the `jobs` crate with `apalis` + `apalis-cron`, Postgres-backed queue.
- E11.F2.T1 Implement the nightly dead-citation-link checker (ties to E5.F3.T2).
- E11.F2.T2 Implement the KB re-verification-due digest (ties to E5.F3.T3).
- E11.F2.T3 Implement hourly/daily analytics rollup jobs (EPIC 12).
- E11.F2.T4 Implement the translation-completeness report job, triggered on KB content change.
- E11.F2.T5 Add job-failure alerting into the observability stack (Grafana alert rule on repeated job failures).

### EPIC 12 — Analytics

- E12.F1.T1 Scaffold the `analytics` crate and `analytics_events`/`analytics_rollups_hourly`/`analytics_rollups_daily` tables (Section 8/12).
- E12.F1.T2 Wire `api` to emit `session_started`/`question_answered`/`terminal_reached`/`deep_link_clicked`/`kb_search_performed`/`kb_entry_viewed` events with the privacy-preserving schema (hour-bucketed timestamps, option values not free text, ephemeral rotating session ids).
- E12.F1.T3 Implement the hourly/daily rollup jobs (funnel drop-off per node, terminal-outcome frequency, deep-link CTR, locale/channel distribution).
- E12.F1.T4 Implement raw-event purge after the 30-day retention window once rolled up.
- E12.F1.T5 Wire the Analytics Dashboard admin page (E7.F3.T6) to the rollup tables.
- E12.F1.T6 **[v2]** Design and implement the k-anonymity-thresholded public transparency dashboard (Section 12's forward-looking subsection).

### EPIC 13 — Security Hardening

- E13.F1.T1 Configure `cargo-audit` and `cargo-deny` as required CI checks (blocking merge on failure).
- E13.F1.T2 Implement least-privilege Postgres roles (API service role without DDL rights; migrations run under a separate elevated role only during deploy).
- E13.F1.T3 Implement CSRF protection for `admin-app` form submissions.
- E13.F1.T4 Implement session-fixation protections (`tower-sessions` configuration review).
- E13.F1.T5 Add a documented process for accepting new direct dependencies (second-reviewer requirement, Section 13).
- E13.F1.T6 **[v1]** Commission or conduct an external security review before any v1 public launch beyond the MVP-Rust-v1 parity milestone.
- E13.F1.T7 **[v1]** Add a CDN/WAF layer (e.g., Cloudflare) in front of the public site for election-day traffic-spike resilience, while keeping the origin self-hostable.

### EPIC 14 — Accessibility & Multilingual

- E14.F1.T1 Run an automated axe-core accessibility audit (via Playwright) against every `web-app` page as a CI gate.
- E14.F1.T2 Conduct a manual WCAG 2.2 AA audit (Section 18) before any v1 public launch.
- E14.F1.T3 Implement the FTL-based translation contribution workflow (Section 17): community PR → `translator` review → `legal_reviewer` sign-off for KB body text → merge triggers the i18n completeness CI check.
- E14.F1.T4 **[v1]** Ship 3-4 additional languages beyond en+hi (candidates: Tamil, Telugu, Bengali, Marathi, by population reach) fully reviewed.
- E14.F1.T5 **[v2/v3]** Complete the remaining Eighth Schedule language roster.

### EPIC 15 — Legal & Compliance Operations

- E15.F1.T1 Write and publish a DPDP-compliant privacy notice reflecting the actual minimal-data posture (Section 19).
- E15.F1.T2 Establish the compliance-owner governance role (even for an all-volunteer OSS project) and a review cadence (quarterly or triggered by ECI rule changes).
- E15.F1.T3 Implement the MCC-window data-entry mechanism feeding the MCC Control Panel (E7.F3.T2) — who updates it, from what source, how often.
- E15.F1.T4 **[v1]** Commission a one-time external legal review of the full decision-tree v2 content before any public launch beyond MVP-Rust-v1 parity.
- E15.F1.T5 **[v2]** Begin the per-state legal review program for tribal/remote/slum/homeless address-proof variations (ties to E5.F2.T6).

### EPIC 16 — Documentation & Governance

- E16.F1.T1 Write CONTRIBUTING.md covering the Rust workspace, content-contribution workflow (for non-engineer KB/translation contributors), and the RBAC review gates.
- E16.F1.T2 Write a GOVERNANCE.md naming maintainer roles, the legal-reviewer responsibility, and decision-making process for architecture changes.
- E16.F1.T3 Keep this PRD (and the v1 doc suite) versioned and updated as decisions in Section 22's open questions get resolved.
- E16.F1.T4 **[v2]** Publish the "open API for NGOs" developer documentation once that roadmap item is scoped.

### Cross-cutting acceptance criteria for "MVP-Rust-v1 done"

A task in any epic above tagged (implicitly or explicitly) MVP-Rust-v1 is
only considered complete when:
1. It has `cargo nextest` coverage (unit and, where applicable, `proptest`).
2. It passes `cargo clippy -- -D warnings` and `cargo fmt --check`.
3. Any new dependency has passed `cargo audit`/`cargo deny` and the
   second-reviewer process (E13.F1.T5).
4. Any new terminal decision-tree node has ≥1 citation and ≥1 official
   deep link, verified by the property tests in EPIC 2.
5. Any new public-facing UI text exists in both English and Hindi FTL
   files before merge (i18n completeness gate).
## 15. Testing & QA Plan v2

### 15.1 Overview

The Rust rebuild keeps the v1 test plan's intent (`docs/17-test-plan.md`:
every legal/procedural claim is exercised, every accessibility affordance
is checked automatically, every language switcher entry is complete) but
re-platforms every layer onto Rust-native tooling except the one
deliberate polyglot boundary: Playwright stays Node-based because Rust has
no mature equivalent browser-automation harness. Test tooling by layer:

| Layer | v1 (TS) | v2 (Rust) | Status |
|---|---|---|---|
| Unit tests | `vitest` | `cargo nextest run` | Replaced |
| Property tests | `fast-check` (vitest) | `proptest` | Replaced — see 15.3 |
| Perf tests | ad hoc / Lighthouse only | `criterion` | New |
| E2E browser tests | Playwright | Playwright (unchanged, against Leptos SSR) | Kept — polyglot boundary |
| Visual regression | Playwright snapshots | Playwright snapshots | Kept |
| Accessibility (automated) | `@axe-core/playwright` | `@axe-core/playwright` | Kept |
| i18n completeness | custom vitest check | Rust integration test over `.ftl` files | Replaced — see 15.8 |
| Load/soak | k6 (ad hoc) | k6 or Artillery against Axum API | Reworked target, see 15.9 |

`cargo nextest run` replaces `cargo test` as the default runner across all
CI and local workflows: faster parallel execution (process-per-test
isolation), better failure reporting, and JUnit XML output for CI
annotations. `.config/nextest.toml` pins a shared profile (retries=2 for
flaky-network-dependent citation-link checks, disabled elsewhere).

### 15.2 Unit Tests

Every crate carries conventional `#[test]` / `#[tokio::test]` unit tests
run under `cargo nextest run --workspace --all-features`. Minimum bar per
crate:

- `crates/core-domain`: deterministic transition tests (given node ID +
  answer, assert exact next node or terminal outcome) — same intent as
  v1's deterministic decision-engine tests, now type-checked at compile
  time via Rust's exhaustive `match` on the answer-option enum (a
  structural improvement over TS: an unhandled answer variant is a
  compile error, not a runtime gap caught only by a test).
- `crates/kb-content`: schema validation tests (every KB source file,
  whichever format section 9 lands on, deserializes and satisfies its
  schema; every entry has a non-empty citation or an explicit
  `source_type: community_pending_verification` marker).
- `crates/api`: handler-level tests via `axum::body` + `tower::ServiceExt`
  test-request helpers (no live HTTP server needed), covering auth
  middleware, session cookie handling, and RBAC role gating (section 11).
- `crates/jobs`: unit tests per job function with a fake/injected clock
  and a test Postgres schema (via `sqlx::test` macro, which spins an
  isolated transactional test database per test).

### 15.3 Property-Based Tests (`proptest`) for `crates/core-domain`

This is the highest-scrutiny test suite in the codebase, superseding the
TS exhaustive-path `vitest`/`fast-check` test in
`packages/decision-engine` (**migration item**: the TS test is retained
read-only as a reference oracle during migration — the Rust proptest
suite must be run against the same tree fixture data and produce
equivalent pass/fail verdicts before the TS test is retired from CI).

Invariants asserted via generated arbitrary answer sequences (using
`proptest`'s `prop_oneof!`/`Strategy` combinators over the answer-option
enum, seeded so failures are reproducible via the printed seed):

```rust
// crates/core-domain/tests/tree_invariants.rs
use proptest::prelude::*;
use core_domain::{DecisionTree, Answer, NodeId, MAX_TREE_DEPTH};

proptest! {
    #![proptest_config(ProptestConfig { cases: 4096, .. ProptestConfig::default() })]

    /// Every reachable sequence of valid answers reaches a terminal node
    /// within MAX_TREE_DEPTH steps. No cycles, no infinite branches.
    #[test]
    fn every_path_terminates(answers in arb_valid_answer_sequence()) {
        let tree = DecisionTree::load_current_version();
        let mut node = tree.root();
        let mut steps = 0;
        for answer in answers {
            if node.is_terminal() { break; }
            node = tree.transition(node.id(), &answer)
                .expect("valid answer must map to a defined edge");
            steps += 1;
            prop_assert!(steps <= MAX_TREE_DEPTH,
                "path exceeded {} steps without reaching a terminal", MAX_TREE_DEPTH);
        }
    }

    /// Every terminal node's outcome payload carries >=1 citation and
    /// >=1 deep link, or is explicitly labeled pending-verification.
    #[test]
    fn every_terminal_has_citation_and_deep_link(node_id in arb_terminal_node_id()) {
        let tree = DecisionTree::load_current_version();
        let terminal = tree.terminal(node_id).unwrap();
        prop_assert!(
            !terminal.citations.is_empty()
                || terminal.source_type == SourceType::CommunityPendingVerification
        );
        prop_assert!(!terminal.deep_links.is_empty(),
            "terminal {:?} has no deep link to an official resource", node_id);
    }
}

/// No orphan nodes: every defined node is reachable from root via >=1 path.
/// Run once per tree version, not per-case (whole-tree structural check,
/// not a per-input property) — implemented as a plain #[test], not inside
/// the proptest! block above.
#[test]
fn no_unreachable_nodes() {
    let tree = DecisionTree::load_current_version();
    let reachable = tree.reachable_node_ids_from_root();
    let defined: std::collections::HashSet<NodeId> = tree.all_node_ids().collect();
    let unreachable: Vec<_> = defined.difference(&reachable).collect();
    assert!(unreachable.is_empty(), "unreachable nodes: {unreachable:?}");
}
```

Additional invariants carried over unchanged in spirit from v1: answer-set
validity (no dangling edges — enforced partly at compile time by the
answer enum, partly by a proptest generator that only ever produces
answers the node's option set declares), and a static/no-sensitive-field
check (a `const`-time or build-script assertion that no node/terminal
struct definition contains a field resembling caste, religion, or party
affiliation — defense in depth alongside `08-database-schema.md`
constraints, now backed by Rust's stronger type system rather than a
runtime schema check alone).

Version-pinned golden-fixture tests (frozen input→output fixtures per
shipped `decision_tree_version`) are kept as plain `#[test]` functions
reading fixture JSON from `crates/core-domain/tests/fixtures/`, unchanged
in purpose from v1: they protect already-reviewed guidance from silent
drift.

### 15.4 Performance Tests (`criterion`)

`criterion` benchmarks live in `crates/core-domain/benches/` and
`crates/kb-content/benches/` (or a dedicated `crates/kb-content/benches/
search_ranking.rs` once Meilisearch/tantivy ranking logic exists
server-side), run via `cargo bench` — not part of the blocking PR gate
(too slow/noisy for that), but run on a scheduled nightly job and on any
PR explicitly labeled `perf-sensitive`, with results tracked over time via
`criterion`'s HTML report artifacts uploaded to CI.

Two benchmark targets and rough targets (to be refined once real
production traffic/hardware is known — treat as a design target, not a
committed SLA):

- **Tree traversal to terminal, server-side**: p99 wall-clock time for a
  full session replay (root to terminal, worst-case depth) against the
  in-memory `DecisionTree` structure should stay **under 5ms** on
  reference hardware (a single modest cloud vCPU, e.g. 2 vCPU / 4GB
  class). This is deliberately generous headroom versus what an
  in-memory tree-walk actually costs (sub-microsecond in practice) —
  the 5ms budget exists to catch regressions (e.g., an accidental clone
  of the whole tree per request, or an accidental disk read introduced
  by a future caching layer), not because traversal is expected to be
  slow.
- **KB search ranking latency**: p99 time to rank and return the top-20
  results for a representative query set (including intentionally
  misspelled/phonetic queries, per the low-literacy-user persona) against
  the full MVP-scope KB corpus, via whichever backend section 9/13
  finalizes (Meilisearch over its Rust SDK, or embedded `tantivy`) —
  target **under 50ms p99** end-to-end from the `crates/api` handler's
  perspective (network round-trip to Meilisearch included, if
  Meilisearch is the chosen backend).

Criterion's statistical rigor (multiple sampling, outlier detection,
regression-vs-baseline comparison) is specifically useful here because it
catches gradual creep (e.g., tree growing from expanded v1-scope
scenarios) that a simple wall-clock timestamp wouldn't reliably flag.

### 15.5 Playwright End-to-End Tests

Playwright remains the E2E tool, run against the actual Leptos SSR
`crates/web-app` binary (built and started in CI, not mocked) — the one
deliberate polyglot boundary in an otherwise all-Rust stack, since Rust
has no mature browser-e2e equivalent. It exercises the real HTML Leptos
emits plus the two hydrated islands (question widget, language switcher),
per the SSR + islands architecture in section 18.

Journeys covered mirror `docs/03-user-journeys.md` conceptually, adapted
to the MVP-Rust-v1 decision-tree scope:

1. **First-time voter (Aarav)** — confirms date of birth, learns the
   applicable qualifying date (1 Jan/1 Apr/1 Jul/1 Oct), reaches Form 6
   with a citation badge and a deep link to `voters.eci.gov.in`.
2. **Student hostel choice (Priya)** — answers "student living away from
   home," sees the hostel-vs-home-address choice with course-recognition/
   1-year/bonafide-certificate conditions, reaches Form 6 with the
   bonafide-certificate checklist item.
3. **Moved house (Mohammed/Kavita composite)** — reports a cross-AC move,
   optionally bundled with a name correction, reaches Form 8 (shifting of
   residence + correction of entries).
4. **Lost EPIC (Kamla Devi)** — chooses between e-EPIC digital download
   and physical replacement, both branches independently verified,
   including the "e-EPIC is legally equivalent" citation.
5. **NRI first-time registration (Ananya)** — confirms overseas elector
   status, is routed to Form 6A (not Form 6), sees the passport-address
   basis, deep link points at the overseas-elector section of
   `voters.eci.gov.in`.

Each test also asserts: the non-affiliation banner is present and
screen-reader-reachable (18.4), a citation badge renders its source-type
distinction, and the deep-link CTA's `href` resolves to an allow-listed
official domain (`voters.eci.gov.in`, `ecinet.eci.gov.in`, a CEO portal,
or a helpline reference) — a plain string/regex assertion, so any
accidental non-official link fails CI immediately.

`baseURL` points at a CI-local instance of the built `web-app` binary
(16.3) rather than a hosted preview: the Rust binary starts in well under
a second, needing no separate deploy step — a real CI-speed win over the
v1 preview-deployment-then-test flow.

### 15.6 Visual Regression Testing

Unchanged in approach from v1: Playwright's built-in screenshot comparison
against the design-system component inventory (question card, progress
indicator, result checklist, citation badge both variants, language
switcher, accessibility toolbar), run at standard and 200% zoom, light and
dark color-scheme preference, with the same explicit palette-compliance
check (no saffron/green-as-primary-brand-color, no party-symbol-adjacent
iconography). The Leptos islands architecture means most of the page is
static server-rendered HTML, so visual regression here is largely testing
CSS/layout stability rather than client-side JS render timing — snapshots
should be taken after the hydration-complete signal for the two
interactive islands (question widget, language switcher), not before, to
avoid false positives from a hydration-in-progress frame.

### 15.7 Accessibility Automated Checks

`axe-core` via `@axe-core/playwright`, run against every major page/flow
state in the same CI job as the E2E suite (home, decision-engine question
screen, result/checklist screen, KB browser, locate page, accessibility
settings page): zero tolerance for "critical"/"serious" severity
violations blocking merge; "moderate"/"minor" tracked as issues. Because
`crates/web-app` server-renders semantic HTML by default (section 18.2),
axe's static-DOM checks (landmark regions, heading order, label
association) are largely validating markup that Leptos emits directly
from Rust view macros rather than markup assembled by client-side JS after
the fact — this makes a large class of accessibility regressions visible
even in a pure HTML-diff review, before axe ever runs.

Manual screen-reader walkthroughs (NVDA, VoiceOver) remain a required,
non-automatable supplement, scheduled at least once per minor release
(unchanged from v1 — automated tooling catches a meaningful subset of
issues, not all of them).

### 15.8 i18n Completeness Tests

Rust-side equivalent of the existing TS `packages/i18n` completeness test,
superseding it (**migration item**: the TS check stays as reference during
migration, retired once the Rust suite is green against the same locale
set). Implemented as an integration test in `crates/kb-content` (or a
standalone `xtask` binary invoked from CI) that:

1. Parses every `i18n/<locale>/*.ftl` file with the `fluent-syntax` crate.
2. Computes the full set of message IDs (including term references,
   `-term-name` entries) defined in `i18n/en/*.ftl` (the source-of-truth
   locale) for every FTL resource file.
3. For every **exposed** locale (currently `en`, `hi` — see section 17.2),
   asserts the same message-ID set is present with a non-empty pattern —
   any missing key fails the build. A locale not yet exposed in the
   language switcher is allowed to be incomplete (staged/roadmap
   languages, section 17.1).
4. For messages containing FTL placeables (`{ $count }`, `{ $name }`),
   asserts every exposed locale's version references the same variable
   set — the FTL equivalent of v1's interpolation-consistency check,
   preventing a translated string from silently dropping a variable a
   plural/select expression depends on.
5. For KB body-text templated via FTL (section 17), cross-checks against
   `knowledge_entry` records: every `published` entry in an exposed
   locale must have a corresponding `published` (not merely `draft`)
   translation in every other exposed locale, or an explicit intentional
   fallback marker — same rule as v1, now checked against
   `crates/kb-content`'s loaded entry set rather than a TS JSON tree walk.

This test runs on every PR touching `i18n/**` or `knowledge-base/**`,
mirroring the v1 CI trigger condition (`docs/15-cicd.md` section 2).

### 15.9 Load / Soak Testing — Election-Day Spike Scenario

Baseline load testing tool: k6 (or Artillery), scripted against
`crates/api`'s session-creation and answer-submission endpoints plus
`crates/web-app`'s SSR page-render path, sized to a concrete scenario:

- **Target scenario**: a state-election-day traffic spike — ramp from
  baseline to peak over ~15 minutes, sustained at peak for ~2 hours (the
  shape of real polling-day-morning traffic), then a slower decay.
- **Rough target** (a design target, to recalibrate against real usage
  data before treating as a committed SLA): sustain **2,000 requests/
  second** of mixed session-creation + answer-submission + SSR traffic
  with **p99 under 300ms** for answer-submission and **p99 under 500ms**
  for full SSR page render, on a modest reference deployment (a small
  number of low-cost instances, not a large fleet). This is achievable
  specifically *because* of the Rust/Axum rewrite: no GC pauses, no
  event-loop contention, cheap async task spawning via Tokio mean the
  same modest hardware that would struggle to sustain a few hundred req/s
  on a comparable Node/Express or Python stack can plausibly sustain
  low-thousands req/s here — directly relevant to an open-source,
  modest-hosting-budget civic project (section 22's funding risk).
- **Soak component**: a lower-throughput (~100 req/s) sustained 24-hour
  soak ahead of any high-traffic period, checking for memory growth,
  `sqlx` Postgres pool exhaustion, and slow leaks in the `apalis`
  job-queue tables a short spike test wouldn't surface.
- **Graceful degradation test**: if `crates/api` becomes slow/
  unavailable, `crates/web-app`'s SSR pages still render static KB
  content and the disclaimer messaging — no full failure mode for
  read-only content, now easier to guarantee structurally since only the
  question-widget island depends on a live API round-trip.

Cadence: before each minor/major release and ahead of known high-traffic
periods, not on every PR — same rationale as v1, too slow/costly per-PR.

### 15.10 Test Ownership and Cadence Summary

| Suite | Trigger | Blocking? |
|---|---|---|
| `cargo nextest run` (unit + integration) | every PR | Yes |
| `proptest` property suite | every PR touching `core-domain`/`kb-content` | Yes |
| `criterion` benchmarks | nightly + PRs labeled `perf-sensitive` | No (regression report only) |
| Playwright E2E | every PR touching `web-app`/`core-domain`/`kb-content` | Yes |
| Visual regression | every PR touching `web-app` | Yes (new snapshots require explicit approval) |
| axe-core accessibility | every PR touching `web-app` | Yes (critical/serious only) |
| i18n completeness | every PR touching `i18n/**` or `knowledge-base/**` | Yes |
| Load/soak | pre-release + pre-high-traffic-period | No (release gate via manual sign-off, not automatic CI block) |
| Manual screen-reader walkthrough | at least once per minor release | Release-gate, not CI |

---

## 16. CI/CD Pipeline v2

### 16.1 Goals

Same goals as v1 (`docs/15-cicd.md` section 1: every PR checked for
correctness/accessibility/performance before merge; decision-engine
changes get extra scrutiny; dependency updates are automated-but-reviewed,
never blind-auto-merged given compliance sensitivity), re-expressed for a
Rust workspace with two additions specific to this stack: a compiled-query
safety net (`cargo sqlx prepare --check`, since `sqlx::query!` macros
validate against a real schema at compile time and must not silently drift
from the committed migrations) and a supply-chain/license gate
(`cargo audit` + `cargo deny`) that has no direct v1 analogue since the TS
stack relied on `npm audit`/Renovate alone.

### 16.2 Pipeline Stages

| Stage | Trigger | What it does |
|---|---|---|
| `fmt` | every PR | `cargo fmt --all --check` |
| `clippy` | every PR | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| `nextest` | every PR | `cargo nextest run --workspace --all-features` (unit, integration, proptest) |
| `cargo audit` | every PR + daily schedule | Scans `Cargo.lock` against the RustSec advisory database |
| `cargo deny` | every PR | License policy, duplicate-dependency policy, banned-crate policy (`deny.toml`) |
| `sqlx prepare --check` | every PR touching `crates/api`, `crates/jobs`, or `migrations/**` | Verifies committed `.sqlx` query metadata matches the current migrations (requires a live Postgres service container in CI, see 16.3) |
| i18n completeness | every PR touching `i18n/**` or `knowledge-base/**` | Section 15.8 |
| Playwright E2E | every PR touching `crates/web-app`, `crates/core-domain`, `crates/kb-content` | Builds and runs the `web-app` binary + a seeded test DB, runs journeys from 15.5 |
| Lighthouse CI | every PR touching `crates/web-app` | Performance/accessibility/best-practices/SEO budget, score >95 each, build fails below threshold |
| Docker image build matrix | push to `main` only | Multi-stage builds for `api`, `web-app`, `admin-app`, `bot-telegram`, `jobs-worker` |
| Dependency updates | scheduled (weekly) | Renovate opens PRs for `Cargo.toml`/`Cargo.lock` bumps, routed through every check above including `cargo deny` |

### 16.3 Example Workflow YAML

```yaml
# .github/workflows/ci.yml
name: CI

on:
  pull_request:
  push:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  SQLX_OFFLINE: "true"

jobs:
  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all --check

  clippy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --workspace --all-targets --all-features -- -D warnings

  nextest:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_USER: voteassist
          POSTGRES_PASSWORD: voteassist
          POSTGRES_DB: voteassist_test
        ports: ["5432:5432"]
        options: >-
          --health-cmd pg_isready
          --health-interval 5s
          --health-timeout 5s
          --health-retries 10
    env:
      DATABASE_URL: postgres://voteassist:voteassist@localhost:5432/voteassist_test
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@nextest
      - uses: Swatinem/rust-cache@v2
      - name: Run migrations
        run: |
          cargo install sqlx-cli --no-default-features --features postgres,rustls
          sqlx migrate run --source migrations
      - run: cargo nextest run --workspace --all-features --profile ci

  cargo-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: taiki-e/install-action@cargo-audit
      - run: cargo audit --deny warnings

  cargo-deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: EmbarkStudios/cargo-deny-action@v2
        with:
          command: check all

  sqlx-prepare-check:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_USER: voteassist
          POSTGRES_PASSWORD: voteassist
          POSTGRES_DB: voteassist_dev
        ports: ["5432:5432"]
        options: >-
          --health-cmd pg_isready
          --health-interval 5s
          --health-timeout 5s
          --health-retries 10
    env:
      DATABASE_URL: postgres://voteassist:voteassist@localhost:5432/voteassist_dev
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: |
          cargo install sqlx-cli --no-default-features --features postgres,rustls
          sqlx migrate run --source migrations
      - name: Verify .sqlx query metadata is committed and current
        run: cargo sqlx prepare --check --workspace -- --all-features

  e2e:
    runs-on: ubuntu-latest
    needs: [nextest]
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_USER: voteassist
          POSTGRES_PASSWORD: voteassist
          POSTGRES_DB: voteassist_e2e
        ports: ["5432:5432"]
        options: >-
          --health-cmd pg_isready
          --health-interval 5s
          --health-timeout 5s
          --health-retries 10
    env:
      DATABASE_URL: postgres://voteassist:voteassist@localhost:5432/voteassist_e2e
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - name: Migrate + seed test DB
        run: |
          cargo install sqlx-cli --no-default-features --features postgres,rustls
          sqlx migrate run --source migrations
          cargo run --bin seed-e2e-fixtures -- --scenario mvp-rust-v1
      - name: Build web-app + api release binaries
        run: cargo build --release --bin web-app --bin api
      - name: Start api + web-app in background
        run: |
          ./target/release/api &
          echo $! > api.pid
          ./target/release/web-app &
          echo $! > web-app.pid
          npx wait-on http://localhost:8080/healthz http://localhost:3000/
      - name: Install Playwright browsers
        working-directory: e2e
        run: |
          npm ci
          npx playwright install --with-deps chromium
      - name: Run Playwright suite (E2E + axe-core + visual regression)
        working-directory: e2e
        run: npx playwright test
        env:
          E2E_BASE_URL: http://localhost:3000
      - name: Stop background servers
        if: always()
        run: kill $(cat api.pid) $(cat web-app.pid) || true

  lighthouse:
    runs-on: ubuntu-latest
    needs: [e2e]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release --bin web-app
      - run: |
          ./target/release/web-app &
          npx wait-on http://localhost:3000/
      - uses: treosh/lighthouse-ci-action@v11
        with:
          urls: |
            http://localhost:3000/
            http://localhost:3000/start
            http://localhost:3000/learn
          budgetPath: ./lighthouse-budget.json
          uploadArtifacts: true
          temporaryPublicStorage: true
      # Fails the job if performance/accessibility/best-practices/SEO < 0.95
      # (assertions configured in lighthouserc.json alongside the budget file)

  docker-images:
    if: github.ref == 'refs/heads/main' && github.event_name == 'push'
    needs: [nextest, cargo-audit, cargo-deny, sqlx-prepare-check]
    runs-on: ubuntu-latest
    strategy:
      matrix:
        service: [api, web-app, admin-app, bot-telegram, jobs-worker]
    steps:
      - uses: actions/checkout@v4
      - uses: docker/setup-buildx-action@v3
      - uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - uses: docker/build-push-action@v5
        with:
          context: .
          file: docker/${{ matrix.service }}.Dockerfile
          push: true
          tags: ghcr.io/${{ github.repository }}/${{ matrix.service }}:${{ github.sha }},ghcr.io/${{ github.repository }}/${{ matrix.service }}:latest
          cache-from: type=gha,scope=${{ matrix.service }}
          cache-to: type=gha,mode=max,scope=${{ matrix.service }}
```

Each `docker/<service>.Dockerfile` is a multi-stage build: a `rust:1-slim`
(or `rust:1-bookworm`) builder stage compiling the workspace with
dependency-layer caching (`cargo chef` recommended to avoid rebuilding
dependencies on every source change), copying only the resulting binary
plus static assets into a `gcr.io/distroless/cc-debian12` (or `scratch`,
for services with zero dynamic-linked runtime needs) final stage — the
single-static-ish-binary property called out in Section 6.16
architecture is what makes this cheap: each final image is tens of
megabytes, not the few-hundred-MB-plus a Node.js equivalent image
typically carries, which directly supports low-cost hosting for an
open-source/nonprofit-funded project.

### 16.4 Renovate Configuration

Renovate (kept over Dependabot for grouping/scheduling flexibility, same
rationale as v1) configured for a Cargo workspace:

```json5
// renovate.json5
{
  extends: ["config:recommended"],
  cargo: {
    enabled: true,
  },
  packageRules: [
    {
      matchManagers: ["cargo"],
      matchUpdateTypes: ["minor", "patch"],
      groupName: "cargo minor/patch updates",
      schedule: ["before 6am on monday"],
    },
    {
      matchManagers: ["cargo"],
      matchUpdateTypes: ["major"],
      groupName: null, // individually reviewed, never grouped
      dependencyDashboardApproval: true,
    },
    {
      // Security advisories bypass the weekly schedule and open immediately
      matchManagers: ["cargo"],
      vulnerabilityAlerts: {
        labels: ["security"],
        schedule: ["at any time"],
      },
    },
  ],
  prConcurrentLimit: 5,
  rebaseWhen: "conflicted",
}
```

The key safety property: **`cargo deny` (section 16.2) runs on every
Renovate PR exactly like a human-authored PR** — Renovate proposing a
dependency bump does not bypass the license/duplicate-dependency/banned-
crate policy. This matters specifically because Renovate can (and does)
occasionally propose an update to a crate whose license changed, or that
pulls in a new transitive dependency with a disallowed license
(`deny.toml`'s `[licenses]` section denies copyleft-incompatible or
unreviewed licenses by default) — `cargo deny` is the safety net that
catches this automatically rather than relying on a human reviewer to
notice a license change buried in a changelog. No dependency PR — Renovate
or human-authored — auto-merges without a human approval, given the
compliance sensitivity of this product (unchanged policy from v1).

### 16.5 Branch Protection

`main` requires: all CI jobs in 16.3 passing, at least one human review
approval, and — for any PR touching `crates/kb-content`, `knowledge-base/
**`, or `crates/core-domain`'s terminal-node definitions — sign-off
consistent with the content review process (legal/compliance section 19,
carrying forward `docs/06-legal-compliance-review.md` section 7
unchanged). `cargo sqlx prepare --check` failing is treated as a hard
merge blocker, not a warning: a stale `.sqlx` cache means the compiled
query metadata committed to the repo no longer matches the migrations,
which is exactly the class of bug compile-time query checking exists to
prevent.

---

## 17. Multilingual Strategy v2

### 17.1 Full Target Language Roster

English plus the complete Eighth Schedule roster (23 languages total):

1. Assamese
2. Bengali
3. Bodo
4. Dogri
5. Gujarati
6. Hindi
7. Kannada
8. Kashmiri
9. Konkani
10. Maithili
11. Malayalam
12. Manipuri
13. Marathi
14. Nepali
15. Odia
16. Punjabi
17. Sanskrit
18. Santali
19. Sindhi
20. Tamil
21. Telugu
22. Urdu

Plus English (not Eighth-Schedule, but the effective source/interlingua
for authoring and review). Full roster coverage is a v3-horizon goal
(section 21); this v2 restates the same target list as
`docs/11-multilingual-strategy.md` unchanged — the language list itself
doesn't change with the Rust rebuild, the *tooling* underneath it does.

### 17.2 MVP Statement (unchanged)

MVP-Rust-v1 ships **English and Hindi only, both fully human-reviewed**
for every decision-tree scenario and KB entry in scope. No other language
is exposed in the production language switcher until it passes the review
gate in 17.5. This restates `docs/11-multilingual-strategy.md` section 7
without change — the language-exposure policy is a product/legal decision
independent of the Rust migration.

### 17.3 FTL / Fluent-Based Workflow

The Rust stack replaces the TS flat-dictionary i18n approach
(`packages/i18n`, plain key→string JSON/TS objects with manual plural
handling) with **Project Fluent (FTL)** via the `fluent` and
`fluent-templates` crates, loaded and served from `crates/kb-content`.
This is a materially better fit for Indic-language grammar than a flat
dictionary: FTL natively expresses plural categories (which differ by
language — Hindi has two plural categories, some languages have more
complex CLDR plural rule sets), gender/case agreement via `SELECT`
expressions, and term references for consistent reuse (e.g., a `-eci-
helpline` term referenced across many messages so the helpline number
formatting stays consistent without copy-paste).

**Message ID conventions** (namespaced, dot-delimited, mirroring the v1
key convention so translators moving from the TS prototype recognize the
pattern):

```
start-question-residence-type-label = ...
result-checklist-item-bonafide-certificate = ...
kb-entry-form6-eligibility-body = ...
```

- UI chrome strings: `<page>-<component>-<element>` (e.g.
  `start-question-residence-type-label`).
- KB body-text templated fragments: `kb-entry-<slug>-<field>` (e.g.
  `kb-entry-form6-eligibility-body`) — these are still full
  `knowledge_entry` records with their own citation/versioning metadata
  per `09-knowledge-base-schema.md`, *not* plain string-table entries;
  FTL is used here specifically for grammatical templating (plurals,
  variable interpolation with correct agreement) within an already-
  reviewed, already-cited entry body, not as a substitute for the KB
  entry's own review workflow (section 17.5 covers why this distinction
  matters for the review gate).
- Terms (shared, reusable fragments referenced with a leading `-`):
  `-eci-helpline`, `-form-6a`, `-epic-term`.

**Plural/gender example** — a checklist item counting required documents,
in Hindi (`i18n/hi/checklist.ftl`):

```ftl
# Number of documents still required before the applicant can submit Form 8.
# Hindi has two CLDR plural categories: one, other.
documents-remaining =
    { $count ->
        [one] आपको { $count } दस्तावेज़ जमा करना बाकी है।
       *[other] आपको { $count } दस्तावेज़ जमा करने बाकी हैं।
    }

# Gendered verb agreement example (SELECT on an explicit $gender argument,
# since Hindi verb forms agree with subject gender — never inferred from a
# name; always an explicit, user-provided or ungendered-neutral variant).
applicant-submitted-form =
    { $gender ->
        [feminine] { $name } ने फ़ॉर्म जमा कर दिया है।
        [masculine] { $name } ने फ़ॉर्म जमा कर दिया है।
       *[other] { $name } का फ़ॉर्म जमा हो चुका है।
    }
```

The English source equivalent uses FTL's simpler `one`/`other` plural
categories (English has only two, unlike some Eighth Schedule languages
which may need `one`, `two`, `few`, `many`, `other` per CLDR — FTL/`fluent`
handles this via the ICU plural-rules data the crate ships with, so a
translator working in a language with richer plural categories isn't
constrained by English's simpler two-category shape):

```ftl
# i18n/en/checklist.ftl
documents-remaining =
    { $count ->
        [one] You still need to submit { $count } document.
       *[other] You still need to submit { $count } documents.
    }
```

The i18n completeness test (section 15.8) parses both files, confirms
`documents-remaining` exists in both locales with the same `$count`
placeable referenced, and does not require the *set of plural categories*
to match 1:1 across languages (that would be linguistically wrong — a
language with more CLDR categories than English is expected to use more
branches, not be forced into English's two-category shape).

### 17.4 Contribution and Review Workflow

Unchanged two-track distinction from v1, re-anchored to RBAC roles defined
in the RBAC architecture defined in Section 11 (`contributor`, `reviewer`,
`legal_reviewer`, `translator`, `analytics_viewer`, `superadmin`):

1. **Community PR**: a contributor (any GitHub contributor, or an
   authenticated `contributor`/`translator` role user via the admin app
   once it exists — section 21's v1 phase) proposes new/updated `.ftl`
   entries or a KB-entry translation.
2. **`translator` role review — fluency gate**: at least one person with
   the `translator` role for that language reviews for fluency, clarity,
   and correct FTL plural/select syntax. This gate applies to *all*
   content, UI strings and KB body text alike.
3. **`legal_reviewer` role review — required specifically for KB body-text
   translation**: because a translated KB entry (decision-tree node text,
   terminal outcome, citation, disclaimer) is a re-expression of a
   procedural/legal claim, not decorative UI copy, it requires a second,
   distinct sign-off from someone holding the `legal_reviewer` role, who
   confirms the translation doesn't drift in meaning from the cited
   source (same standard as v1's Compliance-Owner-delegate review, now
   modeled as an explicit RBAC role rather than an ad hoc "compliance
   owner or delegate" designation). Plain UI-chrome strings (button
   labels, navigation, empty states) do **not** require `legal_reviewer`
   sign-off — only `translator` fluency review, matching v1's
   distinction between the two content categories.
4. **Merge triggers the i18n completeness CI check** (section 15.8):
   merging a translation PR re-runs the FTL-parity check across all
   exposed locales; if the merge is for a not-yet-exposed language, the
   check runs but does not block (incomplete coverage for an unexposed
   language is expected and fine).
5. A translation whose source entry is later updated (new KB `version`)
   is automatically flagged `needs_reverification` until a
   `legal_reviewer` confirms the translation still matches — unchanged
   from v1.

### 17.5 Machine Translation: Draft-Only, Never Auto-Published

Explicit, non-negotiable rule carried forward and made concrete for the
admin tooling: **machine-translation output is never auto-published under
any circumstance.** It may only be used as a labeled draft starting point
that a human translator then edits and takes ownership of. Concretely,
this is enforced in the admin app's translation tooling (admin page 5,
per the admin application specification — see section 11): triggering a
machine-translation draft for a KB entry or UI string creates a record
explicitly tagged `draft_source: machine_translation`, visibly labeled as
such in the reviewer UI (not visually indistinguishable from a
human-authored draft), and that tag can only be cleared by a human
`translator` who has edited and submitted the entry for `translator`
(and, for KB body text, `legal_reviewer`) review per 17.4 — there is no
code path that flips a `machine_translation`-tagged draft directly to
`published` or `verified` status without passing through both human
review gates. This is a stricter rule than most production i18n
pipelines adopt, justified by the same "translation is a re-expression of
a legal/procedural claim, not decoration" reasoning as 17.4.

### 17.6 i18n Architecture Notes

- `crates/kb-content` owns FTL resource loading (`fluent-templates`'
  `static_loader!` macro or an equivalent runtime loader if hot-reloading
  draft translations in the admin app is required) and exposes a typed
  lookup API to `crates/web-app` and `crates/admin-app` — no raw FTL
  message IDs are string-literal-scattered through view code; a thin
  wrapper type keeps message-ID typos from becoming silent runtime
  fallbacks-to-English.
- KB body-text legal/procedural content is **not** stored purely as FTL
  message-value strings — it remains full `knowledge_entry` records (per
  `09-knowledge-base-schema.md`, ported into `crates/kb-content`'s schema)
  carrying citation, version, and `review_status` metadata; FTL is the
  *rendering/templating layer* over that content (handling plural/gender
  agreement in the already-reviewed text), not a replacement for the
  entry's own review workflow. This mirrors and clarifies v1's existing
  distinction between `packages/i18n` (UI strings) and
  `knowledge_base_entries` (legal content) — the storage split is
  preserved, only the UI-string half's file format changes (flat
  JSON/TS → FTL).
- A language only appears in the production switcher once it clears the
  same minimum-coverage threshold as v1 (defined published-not-draft
  legal/procedural entry coverage for the MVP scenario set) — partial
  coverage stays behind a preview/beta-languages flag, unchanged policy.

---

## 18. Accessibility Specification v2

### 18.1 Baseline: GIGW 3.0 + WCAG 2.2 Level AA

VoteAssist India's accessibility baseline is **WCAG 2.2 Level AA**,
adopted alongside **GIGW 3.0** (Guidelines for Indian Government Websites/
Apps, maintained by NIC/MeitY) as the product's own voluntary standard.
GIGW 3.0 itself sets WCAG 2.1 AA as its floor plus India-specific
requirements this PRD adopts explicitly rather than treating as
implicit: responsive design across the low-end Android device range
common among target personas, multilingual support expressed down to the
API level (not just UI strings — see section 17.6's FTL architecture,
which is what makes API-level multilingual support tractable), and a
user-centric information architecture (matching `05-information-
architecture.md`'s task-oriented, not org-chart-oriented, navigation).
This PRD upgrades the WCAG baseline from GIGW 3.0's nominal WCAG 2.1 floor
to **WCAG 2.2 AA**, the more current standard, and treats GIGW 3.0's
India-specific UX requirements as additive on top of WCAG.

**GIGW is not legally mandatory** for VoteAssist India — it is not a
government site (section 19 covers this explicitly as a passing-off
concern to actively avoid implying otherwise) — so adoption here is
voluntary, for two reasons: genuine accessibility/UX value for the same
target population GIGW was designed around (Indian citizens across a wide
range of device, bandwidth, and literacy conditions), and because it
lowers friction in any future conversation about ECI/state CEO office
recognition or endorsement, should VoteAssist ever pursue that (a
possibility to keep open, not a plan to build toward at the expense of
the product's independence).

A note on AAA, restated from v1 and still true under WCAG 2.2: this
document deliberately does **not** claim blanket AAA compliance. AAA
contains success criteria that are frequently infeasible to satisfy
universally across an entire product — some AAA contrast requirements and
sign-language criteria are appropriately treated as per-criterion,
context-dependent goals rather than an all-or-nothing target this PRD can
honestly claim. Where a specific AAA-level criterion is both feasible and
valuable for this product's mission (a stronger contrast ratio in
practice; the sign-language video roadmap item in section 18.10), it is
adopted explicitly and named as such — rather than asserting "AAA" as a
blanket label that would not hold up under audit. This honesty matters
because the product's credibility depends on not overclaiming, consistent
with the citation discipline that runs through the whole PRD.

### 18.2 Why Leptos SSR + Islands Architecture Is an Accessibility Choice

The choice of Leptos with server-side rendering and an islands
architecture (only the interactive question/answer widget and the
language switcher hydrate client-side; everything else — headings,
landmark regions, KB body text, the non-affiliation banner, citation
badges, navigation — is static server-rendered semantic HTML) is not
merely a performance optimization. It is deliberately an accessibility
decision, for reasons specific to this product's user base:

- **Minimal JS payload matters for screen-reader users on assistive
  technology stacks that can be sensitive to heavy client-side
  hydration/re-render churn**, and matters enormously for **2G/low-
  bandwidth users** in the remote/tribal-area personas this product
  targets (`02-personas.md` persona 13) — a full client-side-rendered SPA
  ships megabytes of JS before anything is interactive; an SSR-first
  islands app ships a complete, readable, navigable HTML document on the
  first response, with only two small islands' worth of JS to hydrate
  afterward.
- **Server-rendered semantic HTML by default means assistive technology
  encounters complete, correctly-structured landmark regions and text
  content the moment the HTTP response arrives** — there is no "wait for
  JS to build the DOM" gap where a screen reader encounters an empty or
  partially-built page. Since the vast majority
  of VoteAssist's content (KB entries, informational pages, the result/
  checklist screen's static portions) is read-only, server-rendering it
  as real HTML by default means axe-core and screen readers alike are
  evaluating the actual markup a user gets, not a client-side
  reconstruction of it.
- **Only the genuinely interactive surfaces pay the hydration cost.** The
  question/answer widget needs client-side interactivity (immediate
  feedback, no full-page reload per answer); the language switcher needs
  it for instant locale switching without losing in-progress session
  state. Everything else — including the entire KB browser, the locate
  page, and the accessibility settings page — can be plain server-
  rendered forms/links with no client JS dependency at all, which is a
  meaningfully stronger accessibility posture than "we tested it works
  with JS disabled" after the fact.

### 18.3 Keyboard Navigation

Unchanged requirements from v1, restated as the WCAG 2.2 AA baseline:
every interactive element (question answer options, buttons, language
switcher, accessibility toolbar, deep-link CTAs) reachable and operable
via keyboard alone in a logical tab order matching visual order; visible
focus indicators on all interactive elements (WCAG 2.2's 2.4.11 Focus Not
Obscured is an explicit new AA criterion in 2.2 versus 2.1, relevant here
— sticky headers/toolbars must not obscure the focused element); no
keyboard traps anywhere in the decision-engine flow including any
expander ("why we ask this") modal.

### 18.4 Screen-Reader Labeling for the Decision-Tree Question/Option Pattern

Unchanged pattern from v1, now implemented via Leptos view macros emitting
the semantic markup directly server-side rather than assembled by client
JS: each question node is a properly labeled form region (`fieldset`/
`legend` or `aria-labelledby` binding the question text as the accessible
name of the answer-option group); each answer option has a clear,
unambiguous accessible name; progress-indicator changes are announced via
an `aria-live="polite"` region; citation badges expose their official-
vs-community-pending distinction in the accessible name/description, not
color/icon alone; the non-affiliation banner and verification-caution
boilerplate are real, reachable, readable DOM content — never
visually-present-only decoration — which the SSR-by-default architecture
in 18.2 makes structurally easier to guarantee (there is no client-side
"inject the banner after mount" step that could fail silently).

### 18.5 Color Contrast

WCAG 2.2 AA 1.4.3 (4.5:1 normal text) / 1.4.11 (3:1 large text and UI
component boundaries), unchanged from v1. The civic blue/teal palette
(`07-design-system.md`) is contrast-checked against both light and dark
backgrounds before finalizing tokens; contrast requirements take
precedence over exact hue preference if a conflict arises. Citation-badge
and caution-banner color coding is never the sole differentiator — an
icon/text label always accompanies color, for color-blind users.

### 18.6 Dyslexia-Friendly Font Toggle

Accessibility toolbar toggle to switch body text to a dyslexia-friendly
font option, persisted via local preference storage (a `localStorage`
write from the language-switcher/accessibility-toolbar island — one of
the two client-hydrated surfaces, per 18.2), no account required.

### 18.7 Text Resizing

Legible, non-breaking layout up to 200% browser zoom (WCAG 2.2 AA 1.4.4),
using relative units (`rem`/`em`) throughout server-rendered CSS. An
in-app text-size stepper in the accessibility toolbar supplements browser
zoom for users who prefer not to use it.

### 18.8 Reduced Motion

Respects `prefers-reduced-motion` at the OS/browser level by default; the
accessibility toolbar island additionally offers an explicit in-app
override for cases where an OS-level setting doesn't reach the browser
context. Progress transitions, fade-ins, and any decorative animation have
a reduced/near-instant equivalent when this preference is active — since
almost none of the page outside the two islands has any animation to
begin with (static SSR content), the surface area needing this treatment
is small and concentrated in the question-widget island.

### 18.9 Offline / Low-Bandwidth Mode

MVP-Rust-v1 commitment (matching v1's MVP commitment, not the fuller PWA
goal): the web app degrades gracefully on slow connections — aggressive
asset optimization, text-first rendering, a lightweight "data saver" mode
deferring non-essential images/fonts. The Leptos SSR + islands
architecture is a direct enabler here: because the default page weight is
mostly HTML/CSS with two small JS islands rather than a full client
framework bundle, "degrades well on 2G" is close to the *default*
behavior rather than a separate optimization project layered on top of an
SPA.

Full offline-first PWA-equivalent behavior (installable, functions with
no connection for previously-loaded KB content, syncs feedback/analytics
when reconnected) is a **v1 roadmap item**, not MVP-Rust-v1 (section 21).
For a server-rendered Leptos app, the PWA-equivalent caching strategy
differs somewhat from a typical client-SPA service-worker pattern: a
service worker still owns the offline cache (Leptos SSR doesn't remove
the need for a service worker to intercept fetches when offline), but the
cache-population strategy is "cache the server-rendered HTML response for
visited KB pages/completed decision-tree results" (a stale-while-
revalidate strategy over already-rendered HTML documents) rather than
"cache a JS app shell and hydrate against cached API JSON" — simpler to
reason about, and it means a cached page is exactly what a user already
saw, not a reconstruction that could drift from server behavior.

### 18.10 Roadmap Items (Not MVP-Rust-v1)

- **Voice navigation**: voice input for answering decision-engine
  questions, voice output (text-to-speech) for reading KB content and
  questions aloud — tied to the multilingual voice-mode roadmap
  (section 17) and the future `crates/ivr-gateway` channel (section 21,
  v3).
- **Sign-language video**: Indian Sign Language (ISL) video explainers for
  key flows and KB entries — a v3-horizon goal, explicitly named as an
  aspirational AAA-adjacent commitment (section 18.1) rather than an
  MVP-Rust-v1 requirement, given the production resources it requires.

---

## 19. Legal & Compliance v2

This section is about platform-level legal/compliance requirements
specific to the Rust-based architecture and current (2025-2026) DPDP
rulemaking; it deliberately does not duplicate the broader security
threat model (a different PRD section) and carries forward, rather than
re-derives, `docs/06-legal-compliance-review.md`'s foundational framing
(informational-not-legal-advice nature of the content, section 7's
content review process) where unchanged.

### 19.1 DPDP Rules 2025 — Concrete Platform Requirements

The Digital Personal Data Protection Rules 2025 were notified 13 November
2025 (Gazette 14 November 2025), with phased compliance through 13 May
2027. Translated into concrete requirements for what VoteAssist actually
collects (which, by design, is minimal — section 19.2):

- **Consent and notice**: the only data points collected from an
  ordinary anonymous user are ephemeral session-correlation identifiers
  (rotated, non-persistent) and, optionally, a **feedback contact email**
  if a user opts in to be followed up with. For the feedback email: a
  clear, specific, itemized consent notice at the point of collection
  (not a bundled "by using this site you agree..." clause) stating what
  it will be used for (following up on the specific feedback submitted)
  and the retention period (section 20). **Admin accounts** (contributor/
  reviewer/legal_reviewer/translator/analytics_viewer/superadmin) are the
  other category of personal data the platform holds — these are
  operator/volunteer accounts, not end-user accounts, but DPDP notice/
  consent obligations apply to them too: an admin account creation flow
  must present what's collected (name/email, authentication credential
  hash) and why.
- **Breach notification process**: a documented incident-response runbook
  (owned by the Compliance Owner, section 19.7) specifying detection
  (Grafana/Loki alerting on anomalous admin-auth patterns or database
  access, per the observability stack), internal escalation timeline, and
  external notification obligations under DPDP Rules 2025 to the Data
  Protection Board and affected data principals — the runbook must exist
  and be exercised (e.g., a tabletop walkthrough) before public launch,
  not written and shelved.
- **Record-keeping**: audit trails for admin actions (already an
  architectural requirement per the RBAC design — every KB/decision-tree
  change versioned with who/when/diff) double as the record-keeping
  substrate DPDP expects for demonstrating data-handling accountability.
- **Extra protections for children and persons with disabilities**: DPDP
  Rules 2025 impose heightened obligations here; VoteAssist's posture is
  that it does not knowingly collect additional data from minors beyond
  the same minimal anonymous-session data any user generates (the
  decision engine itself is relevant to users nearing/at voting age, 18+,
  so this is a lower-risk surface than a general-audience consumer app,
  but the rule still applies to the platform's general design and must
  be stated as such rather than assumed inapplicable).

### 19.2 Significant Data Fiduciary Avoidance — An Explicit Design Goal

VoteAssist India's minimal-data-by-design posture (no direct identifiers
collected by default, no document uploads, no persistent user profiles,
aggregate-only analytics per section 20) is **deliberately engineered to
avoid the platform ever being classified as a Significant Data Fiduciary**
(SDF) under DPDP — a classification that would trigger materially heavier
obligations (mandatory Data Protection Impact Assessments, periodic data
audits, a data protection officer requirement). This is stated here as an
explicit design goal, not an incidental side effect: any future feature
proposal that would meaningfully increase the volume or sensitivity of
personal data processed (e.g., persistent user accounts for end users,
document upload/OCR, location tracking beyond coarse self-reported state)
must be evaluated against this goal before being greenlit, with the
Compliance Owner (19.7) as the required sign-off.

### 19.3 Model Code of Conduct (MCC) Handling

MCC engineering requirements are specified concretely in the admin
application specification's MCC control panel (admin page 8, section 11)
— this section references that control rather than re-deriving it. The
platform-level legal requirement it exists to satisfy: MCC comes into
force the moment ECI announces an election schedule for a given state/the
Union and remains in force until results are declared; during that
window, VoteAssist must suppress anything readable as government-
machinery-adjacent announcement, endorsement, or voter-mobilization
messaging (no "get out the vote" push notifications, no turnout-
encouragement framing beyond stating factual deadlines) while continuing
to serve pure "how do I do X" administrative guidance unaffected. The
admin page 8 control panel is the operational lever the Compliance Owner
uses to toggle this posture per active state/election, on the timeline
ECI's own schedule announcements set — this section's job is only to
confirm the legal *why*, since the *how* is the admin spec's job.

### 19.4 GIGW 3.0 — Voluntary-but-Adopted Baseline

Restated from section 18.1 for legal/compliance completeness: GIGW 3.0
is **not legally required** — VoteAssist India is not a government
website or app — but is adopted voluntarily as the product's own
accessibility/UX baseline (alongside WCAG 2.2 AA) for genuine
accessibility value and because it eases any future ECI/state CEO
recognition conversation, should one arise. Adopting it voluntarily is
also a defensive-legal posture in its own right: it demonstrates good-
faith accessibility diligence that would matter if RPwD Act obligations
(19.5) were ever tested against this product.

### 19.5 RPwD Act 2016 Alignment

The Rights of Persons with Disabilities Act 2016 establishes accessibility
obligations relevant to a public-facing civic information service.
Precise applicability to a non-government open-source project is a
question for counsel (carried forward from `docs/06-legal-compliance-
review.md` section 5, unresolved there and still unresolved here — this
PRD does not manufacture a legal conclusion it can't support), but WCAG
2.2 AA is adopted as the technical baseline regardless of the precise
legal-applicability answer, because it serves the product's accessibility
mission independent of whether RPwD strictly compels it. RPwD Act 2016 is
the domestic legal framework; WCAG 2.2 is the international technical
standard — treated as complementary, not identical; any RPwD-specific
documentation/certification expectations remain a named follow-up for
counsel (section 19.7).

### 19.6 Trademark / Passing-Off Risk — Concrete "SHALL NOT" Requirements

Restated from Section 4's explicit non-goals as a binding requirements
list, not a narrative risk discussion:

- **SHALL NOT** display the ECI emblem, in whole or stylized/partial form,
  anywhere in the product (branding, favicon, UI chrome, marketing
  material, app store listing assets).
- **SHALL NOT** display the State Emblem of India (the Ashoka Chakra /
  Lion Capital iconography) or any Government of India tricolor-as-
  official-seal treatment.
- **SHALL NOT** display any CEO office or ECI-affiliated body's logo.
- **SHALL NOT** produce any visual design (layout, color palette,
  typography, iconography) that mimics `eci.gov.in` or
  `voters.eci.gov.in` closely enough to risk a user believing they are on
  an official ECI property — `07-design-system.md`'s deliberately
  distinct visual language exists specifically to satisfy this
  requirement, not as an independent aesthetic choice.
- **SHALL NOT** register or operate under a domain name, app-store
  listing name, or social-media handle containing terms like "official,"
  "govt," "gov," "eci," or "election commission" in a way that implies
  endorsement or official status.
- **SHALL NOT** use government letterhead styling, ministry seals, or any
  simulated "official notice" formatting for VoteAssist-authored content.
- **SHALL** present "VoteAssist India" with an explicit "independent" /
  "unofficial" qualifier on first mention on every page or app screen
  (the non-affiliation disclaimer requirement from `06-legal-compliance-
  review.md` section 1, restated here as a naming/branding rule
  specifically, not just a disclaimer-copy rule).

This is a passing-off / trademark / possibly Emblems and Names
(Prevention of Improper Use) Act 1950 concern; the specific statutes and
any compliance steps beyond the design/naming precautions above remain
flagged for counsel confirmation (carried forward, unresolved, from
`06-legal-compliance-review.md` section 2).

### 19.7 Legal Review Cadence and Compliance Owner Governance

- **Cadence**: a full legal/compliance review recommended **quarterly**,
  and additionally **triggered immediately by any ECI forms/rules
  amendment** affecting covered content (the Form 6/6A/7/8 consolidation
  via the 2022 Amendment Rules is the concrete precedent for why this
  trigger condition exists — see section 22's open-risk entry on ECI
  changing these forms again) — this is the same cadence already named in
  `06-legal-compliance-review.md` section 7, restated here as the
  standing legal-review policy for the whole v2 PRD, not just KB content
  review.
- **Compliance Owner**: a named individual, even for this open-source
  project, distinct from "Maintainer," accountable for: signing off on
  any change to legal/procedural claims before it ships; serving as point
  of contact for takedown/correction requests (including from ECI or CEO
  offices); operating the MCC control panel (19.3) during active
  election periods; maintaining `/about/legal`, `/about/privacy`, and the
  legal-compliance-review document as living artifacts; and owning the
  DPDP breach-notification runbook (19.1). This role must be filled
  before any public, non-preview deployment — carried forward from
  `docs/06-legal-compliance-review.md` section 6 as a hard prerequisite,
  not a nice-to-have, for the Rust-based production launch specifically
  (a preview/staging deployment of the Rust stack without a named
  Compliance Owner is acceptable for internal testing; a public launch
  is not).

---

## 20. Analytics / Privacy Data Retention Policy

Itemized retention table synthesizing decisions from the accessibility
(section 18), analytics-plan, and legal-compliance (section 19) sections.
"Deletion mechanism" specifies how enforcement actually happens (a
scheduled `crates/jobs` task, not a manual/aspirational process).

| Data category | Retention period | Deletion mechanism | Who can access | Audit trail |
|---|---|---|---|---|
| `analytics_events` (raw: node_id, answer option, hour-bucketed timestamp, locale, client_platform, deep-link-click booleans, ephemeral rotated session_id — no free text, no direct identifiers) | 30 days | Nightly `apalis-cron` job in `crates/jobs` hard-deletes rows older than 30 days after they've been folded into `analytics_rollups` | `analytics_viewer` and `superadmin` roles, read-only, via the admin analytics dashboard | Job run itself is logged (row count deleted, run timestamp) in the jobs-execution audit table; no per-row access log needed since access is aggregate-dashboard-only |
| `analytics_rollups` (pre-aggregated, non-reversible summary tables/materialized views: e.g., daily task-completion-rate, per-node drop-off rate, terminal-outcome distribution) | Indefinite | N/A — retained indefinitely because these are fully anonymized aggregates with no path back to an individual session; DPDP retention limits apply to personal data, and these rollups are, by construction, not personal data | `analytics_viewer`, `superadmin`; a curated subset also feeds the public transparency dashboard (section 21, v2 phase) | Rollup-generation job logged; no further audit needed post-generation since the data is aggregate |
| `feedback.contact_email` (optional, opt-in, only collected if the user wants a follow-up on their specific feedback) | 180 days, unless the user is in active correspondence (an open feedback thread with unresolved follow-up), in which case retention extends until the thread is closed, then the 180-day clock starts | Scheduled `crates/jobs` task nulls out `contact_email` (retaining the anonymized feedback text/category itself, which is not personal data on its own) 180 days after thread closure | `superadmin` and the specific reviewer/Compliance-Owner handling that feedback thread; not visible to general `contributor`/`translator`/`reviewer` roles | Every access to a feedback row with a non-null `contact_email` is logged (who, when) given its personal-data status |
| `admin_users` audit logs (who/when/diff for every KB/decision-tree change, admin login events, RBAC role grants) | Indefinite | No deletion job — retained indefinitely for compliance/accountability purposes (this is the record-keeping DPDP itself expects, section 19.1) | `superadmin` only for the raw log; individual admins can see their own action history | The audit log **is** the audit trail — access to it is itself logged (who viewed whose history), preventing quiet unaccountable surveillance of contributors by other admins |
| `knowledge_entry_revisions` (full version history of every KB entry: content diffs, citation changes, `review_status` transitions, translator/reviewer/legal_reviewer sign-offs) | Indefinite | No deletion job | Public (this is versioned content history, visible the same way a public wiki's edit history is) for published-entry history; draft-stage revision history restricted to `contributor`/`reviewer`/`legal_reviewer`/`superadmin` roles until publication | Each revision is inherently self-documenting (who authored/reviewed, when); no separate audit layer needed |

**Why `knowledge_entry_revisions` retention is not a DPDP question**: this
category is content history — the evolution of a piece of published
civic-procedural guidance and its citation trail — not personal data about
an identifiable individual. DPDP's retention-limitation principle governs
personal data (data that identifies or could identify a natural person);
a KB entry's edit history identifies *content changes and their authors
in their capacity as contributors/reviewers* (itself a narrower, RBAC-
scoped personal-data surface handled like the `admin_users` audit log
row above), not end-user personal data at all. Treating unlimited content-
history retention as a privacy problem would conflate the platform's
legitimate, indefinite need for a legal/procedural audit trail (arguably
its most important compliance asset, since it is what lets anyone verify
a given claim was reviewed and cited correctly at the time) with the
DPDP concern about holding end-user personal data longer than necessary,
which is a genuinely different thing. The two should not be reasoned
about with the same retention clock.

---

## 21. Roadmap v3 (MVP-Rust-v1 → v1 → v2 → v3)

Sequencing rationale, not calendar dates — each phase is described by what
it depends on and what it unblocks, deliberately avoiding fabricated ship
dates.

### MVP-Rust-v1

- Port the existing TS decision-engine logic and knowledge base into
  `crates/core-domain` and `crates/kb-content`, achieving feature parity
  with the existing TS MVP (same decision-tree scenarios, branches A-H
  per `04-decision-tree-spec.md`) — not new scope, a re-platforming.
- Ship `crates/api` (Axum) and `crates/web-app` (Leptos SSR + islands).
- Languages: English + Hindi only, both fully reviewed — unchanged from
  the existing TS MVP's language scope, since the FTL migration
  (section 17) is a tooling change, not a language-coverage expansion.
- No admin app yet. Content editing remains direct JSON (or TOML, per
  section 9's open tradeoff) file PRs through the same GitHub review
  workflow the TS MVP already uses — deliberately deferring the
  admin-app/RBAC build until the core engine/rendering rebuild is proven,
  rather than building both at once.
- **Sequencing rationale**: this phase must land first because every
  later phase (admin app, bot channels, analytics dashboard) is a thin
  layer over `core-domain`/`kb-content`/`api`; building those before the
  core is stable in Rust would mean building them twice.

### v1

- `crates/admin-app` ships, implementing full RBAC (`contributor`,
  `reviewer`, `legal_reviewer`, `translator`, `analytics_viewer`,
  `superadmin`) and the complete admin pages specification (section 11) —
  this is what finally retires the direct-JSON-PR content workflow from
  MVP-Rust-v1.
- Decision tree expands beyond branches A-H to the fuller scenario set
  from `02-personas.md` (service voters, transgender-specific correction
  flows, tribal/remote-area guidance, homeless-elector guidance where
  legally applicable) — deferred to v1 rather than MVP-Rust-v1 because
  expanding tree scope is exactly the kind of change the admin app's
  review workflow (draft → reviewer → legal_reviewer → verified) is built
  to safely absorb; doing it via raw JSON PRs at MVP scale would be
  unsafe at this larger scope.
- Formal WCAG 2.2 AA audit/verification pass (automated + manual),
  building on the CI-level axe checks already running since MVP-Rust-v1.
- Languages: expand to 5-6 total, prioritized by speaker population and
  community translator availability, gated through the section 17.4
  workflow — this is sequenced after the admin app specifically because
  the `translator`/`legal_reviewer` role-based review gate is an
  admin-app feature; before the admin app exists, adding languages beyond
  en/hi via raw file PRs would lack the structured two-gate review this
  PRD requires for legal/procedural translations.
- **Sequencing rationale**: admin-app + RBAC unblocks both the expanded
  decision tree and the language expansion, since both require a
  structured multi-role review workflow that MVP-Rust-v1's direct-PR
  model doesn't provide at scale.

### v2

- `crates/bot-telegram` goes live (thin adapter over `core-domain`/`api`,
  per the architecture — no decision-logic reimplementation).
- `crates/bot-whatsapp` goes live, direct Meta Cloud API integration per
  the architecture decision in Section 6.7 (avoiding BSP vendor lock-in/
  cost at this stage); Gupshup-BSP remains the noted v2-or-later upgrade
  path if template-messaging-at-scale or unified WhatsApp+voice billing
  becomes a real need.
- Analytics dashboard (internal, `analytics_viewer` role) plus the public
  transparency dashboard (aggregate-only, per section 20's rollup-table
  design) ship together, since the public dashboard is a curated view
  over the same rollup tables the internal dashboard already needs.
- Per-state legal review program formally kicks off (state-by-state
  content completeness sign-off, per `06-legal-compliance-review.md`
  section 7's cadence) — sequenced here rather than earlier because it is
  resourcing-heavy (section 22 flags this as an organizational risk) and
  benefits from the admin-app review workflow (v1) already being in
  place to track per-state sign-off status structurally rather than
  informally.
- Meilisearch-backed KB search ships (or `tantivy`, per the section 9/13
  open tradeoff), replacing whatever simpler search MVP-Rust-v1/v1
  shipped with — sequenced at v2 because search relevance work benefits
  from having a larger, v1-expanded KB corpus to actually rank over;
  building a sophisticated ranking layer over the small MVP-Rust-v1
  corpus would be premature optimization.
- **Sequencing rationale**: bot channels and search both depend on
  `core-domain`/`kb-content` being stable and the KB corpus being
  large enough (post-v1 expansion) to be worth the integration effort;
  the transparency dashboard depends on the internal analytics dashboard
  existing first (same underlying rollup tables).

### v3

- `crates/ivr-gateway` (Exotel) goes live for real, building on the
  scaffolded interface from MVP-Rust-v1 and the DTMF/voice-menu-friendly
  question design that the FTL-structured content pipeline (section 17)
  was built to support from early on.
- Full Eighth Schedule language coverage (all 22 scheduled languages plus
  English), each having passed the same `translator` + `legal_reviewer`
  review gate as English/Hindi at MVP-Rust-v1.
- Campus toolkit for student-organization registration-awareness drives,
  building specifically on the student "ordinary residence" scenario
  already covered since MVP-Rust-v1/v1.
- Open API for NGOs (documented, rate-limited, extending `crates/api`'s
  existing OpenAPI spec via `utoipa`) — sequenced late because it commits
  to a stable public contract, which is safer to offer once the
  underlying decision-engine/KB schema has had several phases to
  stabilize.
- Versioned public knowledge-graph: a structured system tracking how ECI
  circulars/rules change over time (not just `last_verified_date` per
  entry, but linked history of what changed, when, and what past guidance
  it superseded) — this is a natural extension of
  `knowledge_entry_revisions` (section 20) once that history is deep
  enough (multiple phases of content review behind it) to be worth
  surfacing as its own product surface rather than an internal audit
  trail.
- By this point, state-by-state legal review (kicked off in v2) should be
  complete for all states/UTs; any state still outstanding is flagged
  in-product rather than silently assumed covered — unchanged
  cross-cutting commitment from `docs/19-roadmap.md`.
- **Sequencing rationale**: IVR, full-language coverage, and the open
  NGO API are all "maximize reach" investments that make sense only once
  the core product (content correctness, review workflow, accessibility)
  has been proven and hardened across v1/v2 — investing in reach before
  correctness/trust is the wrong order for a civic-information product
  whose entire value proposition rests on being trustworthy.

### Cross-Cutting Commitment (unchanged)

At every phase, expanding scope (new language, new scenario, new state,
new channel) is gated by the same non-negotiables from the core PRD: no
official-portal impersonation, no on-behalf submission, no political
content, no claim without a citation or an explicit pending-verification
label. Scope expands; these constraints do not loosen, regardless of
which phase or which crate is shipping.

---

## 22. Open Questions & Risks

Deliberately unresolved decisions and named risks this PRD does not paper
over with a guess:

1. **JSON vs. TOML for KB source files** (`crates/kb-content`, also
   flagged in the KB schema section): both are viable; JSON keeps
   continuity with the existing TS MVP's `knowledge-base/sources/*.json`
   and existing schema tooling, TOML is arguably more human-editable for
   non-technical contributor PRs. Needs a team decision before
   MVP-Rust-v1's content-porting work locks in a format.
2. **`SeaORM` vs. raw `sqlx` for admin-CRUD velocity**: `sqlx` with
   compile-time-checked queries is the primary choice for correctness
   guarantees, but `SeaORM`'s ActiveRecord ergonomics could meaningfully
   speed up the v1 admin-app's CRUD-heavy screens. Both are viable; not
   silently decided here — flagged for the v1-admin-app implementation
   team to choose, potentially per-module (e.g., `sqlx` for
   `core-domain`-adjacent reads, `SeaORM` for admin CRUD forms) rather
   than an all-or-nothing pick.
3. **Meilisearch vs. `tantivy`** for KB/glossary/forms search: Meilisearch
   recommended for MVP-Rust-v1 (ready-to-deploy typo tolerance, valuable
   for low-literacy/phonetic-spelling queries), `tantivy` flagged as the
   in-process, zero-extra-infra alternative for smaller/self-hosted
   deployments that don't want to run an extra service. Final call
   deferred to whoever operates the v2 search rollout (section 21),
   informed by real deployment-cost constraints at that point.
4. **WhatsApp: direct Meta Cloud API vs. Gupshup-BSP, and at what phase**:
   Section 6.7 recommends direct Meta API at MVP-Rust-v1/early phases to
   avoid BSP vendor lock-in and cost, with Gupshup as the v2 upgrade path
   once template-messaging-at-scale or unified WhatsApp+voice billing
   matters. This tradeoff should be revisited concretely before v2's bot
   rollout, not assumed settled by this recommendation alone — Meta API
   rate limits and template-approval friction at real user volume could
   force the Gupshup decision earlier than planned.
5. **Resourcing for state-by-state legal review**: this is an open-source
   project, not a funded legal team. The v2 roadmap phase names "per-
   state legal review program kicks off" as if it will simply happen;
   it will not happen without either volunteer legal contributors with
   relevant state-level expertise, a funded legal-review budget, or a
   partnership with an existing civic-legal organization. This is an
   **organizational risk**, not a technical one, and is currently
   unresourced.
6. **ECI changing Form 6/6A/7/8 again**: ECI already restructured these
   forms once via the Registration of Electors (Amendment) Rules 2022,
   discontinuing Form 8A/Form 001 and merging them into Form 8. Nothing
   prevents another such restructuring. The nightly re-verification job
   (`crates/jobs`) and quarterly/trigger-based legal review cadence
   (section 19.7) *mitigate* the consequence (stale content gets flagged),
   but the underlying risk — that the platform's entire form-mapping
   logic in `core-domain` could need a non-trivial restructuring on short
   notice, with no advance warning from ECI — is real and should be named
   as a standing risk, not treated as fully solved by the mitigation.
7. **Risk of being mistaken for an official ECI tool despite disclaimers**:
   this is a genuine UX/trust risk, not merely a legal checkbox satisfied
   by disclaimer copy (section 19.6). Users under time pressure or with
   low digital literacy may not register a disclaimer banner even when
   it's present and accessible; the actual mitigating factor is
   deliberate visual/naming distinctiveness (section 19.6's SHALL-NOT
   list) plus user testing that specifically probes for this confusion
   (not just accessibility/usability testing in the abstract) — that
   targeted user-confusion testing is not yet defined anywhere in this
   PRD and should be.
8. **Funding/sustainability model**: an open-source civic-tech project
   serving national-scale Indian traffic (especially around election
   periods, per section 15.9's load-testing scenario) has real hosting,
   Meilisearch-infrastructure, Exotel/WhatsApp-API, and maintainer-time
   costs. No funding model is defined in this PRD — grant funding,
   sponsorship, a civic-tech foundation partnership, and pure volunteer
   hosting-donation are all plausible but none is committed to, and this
   materially affects which roadmap phases (section 21) are realistic on
   any timeline at all.
9. **Admin-app authentication roadmap**: `argon2` password hashing is the
   MVP-Rust-v1/v1 baseline; passkeys are noted as a later option "via an
   `oauth2-passkey-axum`-style crate" without a committed crate or
   timeline. Given admin accounts are the highest-privilege attack
   surface in the system, this should be revisited earlier than "later,
   unspecified" implies once the security threat model (separate PRD
   section) is finalized.
10. **Self-hosted observability stack operational ownership**: `tracing`/
    OpenTelemetry/Grafana/Loki/Tempo self-hosting is the stated default
    for data-sovereignty reasons, but self-hosting an observability stack
    is itself an ongoing operational burden for a volunteer-heavy
    project — who operates and pays for this infrastructure is
    unresolved, and ties directly into risk 8.
11. **Machine-translation quality baseline across 21 non-MVP languages**:
    section 17.5's draft-only rule assumes a human `translator` exists
    per target language to correct machine-translation drafts. For
    lower-resource Eighth Schedule languages (e.g., Bodo, Santali,
    Dogri, Kashmiri), qualified volunteer translators with both language
    fluency and enough comfort with FTL/procedural-content review may be
    genuinely scarce — this is a real bottleneck on the v3 full-coverage
    goal that no tooling decision in this PRD solves.
12. **Reading-level ("Easy") variant scope in the Rust rebuild**: v1's
    docs mention an "Easy" reading-level variant per KB entry
    (`09-knowledge-base-schema.md`'s `reading_level_variant`); this PRD's
    testing/i18n sections don't yet specify whether the FTL/`kb-content`
    architecture treats reading-level variants as a first-class dimension
    (like locale) or an ad hoc field — needs an explicit decision before
    MVP-Rust-v1's `kb-content` schema is finalized.
13. **PDF generation resource cost**: `headless_chrome`-driven PDF
    rendering (checklists/cover-letter drafts) requires running a real
    Chromium instance per render, which is meaningfully heavier
    (memory/CPU/attack-surface) than the rest of the otherwise-lightweight
    Rust stack. Whether this runs as part of `crates/api`'s request path,
    a separate `crates/jobs` worker, or an isolated sandboxed service is
    not yet decided, and matters for both the section 15.9 load-testing
    plan (a spike in PDF-generation requests during election week could
    behave very differently from the rest of the API's request profile)
    and for the modest-hosting-cost goal (risk 8).

---

## 23. Appendix: Glossary & Citation Cross-Reference

### 23.1 Glossary

| Acronym | Definition |
|---|---|
| ECI | Election Commission of India — the constitutional body administering elections nationally |
| ERO | Electoral Registration Officer — official responsible for electoral roll preparation/revision in an Assembly Constituency |
| BLO | Booth Level Officer — local official handling ground-level elector registration/verification |
| DEO | District Election Officer |
| CEO | Chief Electoral Officer — state/UT-level election administration head |
| EPIC | Elector's Photo Identity Card — the voter ID card (physical or e-EPIC digital form) |
| AC | Assembly Constituency |
| PC | Parliamentary Constituency |
| MCC | Model Code of Conduct — ECI's conduct rules in force from election-schedule announcement until results are declared |
| GIGW | Guidelines for Indian Government Websites/Apps (NIC/MeitY) |
| DPDP | Digital Personal Data Protection Act 2023 / its 2025 Rules |
| RPwD | Rights of Persons with Disabilities Act 2016 |
| NGSP | National Grievance Service Portal (2.0) — ECI's citizen-grievance tracking system |
| SVEEP | Systematic Voters' Education and Electoral Participation — ECI's voter-education program |
| UNPER | Unified National Photo Electoral Roll — ECI's common national elector database |
| ERONET | ECI's internal Electoral Roll Management System used by election officials |
| ECINET | ECI's broader unified digital platform for electoral services |
| CSV (Classified Service Voter) | A service elector who opts for a proxy voter instead of a postal ballot |
| FTL | Fluent (Project Fluent) — the localization file format/syntax used for UI and templated KB text |
| RBAC | Role-Based Access Control — the admin-app permission model (`contributor`/`reviewer`/`legal_reviewer`/`translator`/`analytics_viewer`/`superadmin`) |
| SSR | Server-Side Rendering — rendering HTML on the server per request, as `crates/web-app`/`crates/admin-app` (Leptos) do by default |

### 23.2 Citation Cross-Reference

This appendix intentionally does not duplicate the master citation index.
`docs/citations.md` remains the
single source-of-truth for every factual/legal citation referenced across
this PRD (ECI circulars, Gazette notifications, statute references, GIGW/
WCAG version citations). Where any section above asserts a specific ECI
procedural fact, the citation for that fact lives in `citations.md`, not
restated inline here — consult it directly rather than treating any
citation-shaped text in this appendix as authoritative on its own.
