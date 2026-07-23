# VoteAssist India — PRD v3: Comprehensive Expansion Addendum

**Status:** Draft v3.0 — extends and, in one important respect, corrects `PRD-V2-RUST-PLATFORM.md`. Pending team/legal review, same as v2.
**Relationship to PRD v2:** this is an ADDENDUM, not a replacement. Everything in `PRD-V2-RUST-PLATFORM.md` stands except where this document explicitly says otherwise. Read v2 first — this document assumes it. Section numbers here are prefixed "V" (V1–V9) to keep them visually distinct from v2's plain numbers (1–23) while making clear they slot into the same overall specification. A future consolidation pass may merge the two into one numbered document once the corrections and additions below are reviewed and accepted; until then, treat v2 + this addendum together as the current state of the plan.
**The single most important correction in this document:** PRD v2 implicitly scoped VoteAssist to the Election Commission of India. That scope was incomplete — Panchayat and Municipal (local body) elections in India are constitutionally administered by separate **State Election Commissions** (Article 243K/243ZA), not the ECI, with no unified national portal equivalent to voters.eci.gov.in. Section V3 below is the fix: a jurisdiction model, new reference data, and a decision-tree routing change so VoteAssist never sends a citizen asking about a local-body election to the wrong official system.
**What else this addendum adds:** a fact-verification pass on v2's research (V1); new open-source and government-platform research — Bhashini, DIGIT, India Stack/DEPA, and international prior art (V2); the election-jurisdiction and calendar model (V3); existing/"old"-voter parity as a co-equal pillar to new-voter registration (V4); a deeper end-to-end UX specification covering minimalism principles, cross-module cohesion ("deeply coupled"), a fringe-case registry, and a self-improving (human-in-the-loop) content pipeline (V5); an optional, strictly-opt-in accounts/saved-drafts/deletion/privacy architecture that does not weaken the anonymous-by-default posture (V6); concrete data/interoperability formats — calendar export, draft export, consent artifacts, audit-log shape, NGO data-export manifests (V7); an expanded feature/task backlog, EPICs 17–22, continuing PRD v2 Section 14's numbering (V8); and new open questions/risks specific to this addendum's scope (V9).
**Last updated:** 2026-07-23

## Table of Contents (this addendum)

V1. Fact-Verification Log
V2. New Open-Source & Government Platform Research
V3. Generalized Election Model — Every Election Type, Correct Jurisdiction
V4. Existing / "Old" Voter Parity
V5. Deep End-to-End UX & Information Flow v3
V6. Optional Accounts, Saved Drafts, Deletion & Privacy Architecture
V7. Format & Interoperability Specifications
V8. Expanded Feature & Task Backlog Addendum (EPICs 17–22)
V9. Updated Open Questions & Risks

---
## V1. Fact-Verification Log

### V1.0 Methodology note

This log is a **point-in-time snapshot**, produced by cross-checking PRD v2's
ECI/legal factual claims (`docs/PRD-V2-RUST-PLATFORM.md`, principally
Sections 2, 3, and 19) against the deeper, more targeted research compiled
for this v3 addendum. It is not, and is not a
substitute for, a live re-verification against primary official sources
(eci.gov.in, voters.eci.gov.in, ECINET, servicevoter.nic.in, MeitY/NIC
publications, the Gazette of India). Two things are true at once: (1)
nothing below was rubber-stamped — where the v3 research corpus did not
independently re-derive a fact from a primary source, that is stated
plainly rather than presented as fresh confirmation; (2) full, recurring,
live-source re-verification is itself a committed piece of product
infrastructure, not a one-off task for this addendum to discharge by hand.
Specifically:

- PRD v2 Section 6.8 already commits to a **KB re-verification-due digest**
  job (`crates/jobs`, `apalis-cron`): a nightly/scheduled check that flags
  any knowledge-base entry whose `lastVerifiedDate` has aged past a
  configurable threshold, surfacing it to a human reviewer. That job is the
  system's existing age-based re-verification backbone.
- This addendum's new **"official source change-detection" job**
  (Section V5's self-improving-system mechanism 1) tightens that
  loop with an event-driven trigger: a small, curated, rate-limited list of
  official pages (ECI forms/FAQ pages, and — per V1.6 below — GIGW's own
  published guidelines page) is polled at most daily for a content-hash
  change, and a hash change immediately flags the citing KB entries as
  `needs_reverification` rather than waiting for the 180-day age-based
  sweep to catch up.
- Together, these two jobs are the actual, ongoing, infrastructural answer
  to "is this still true" — not this document. This log exists to give the
  next content-review cycle a prioritized starting point, and to be honest,
  right now, about which of PRD v2's claims are solid, which need a stated
  caveat, and which are genuinely open.

Verdicts used below: **VERIFIED UNCHANGED** (still true, no new caveat
needed), **VERIFIED WITH NEW NUANCE** (true, but v3 research adds a caveat
that changes how it should be used in product copy or architecture), and
**FLAGGED FOR RE-CHECK** (genuinely uncertain from the material available
to this addendum; a specific official source is named for the next content
review cycle to check before publishing anything that depends on it).

### V1.1 Forms 6/6A/7/8/2/12D consolidation (2022 amendment) and current purposes

**VERIFIED UNCHANGED.** The Registration of Electors (Amendment) Rules 2022
(Gazette 17 June 2022, in force 1 August 2022) consolidation — Form 6 (new
registration), Form 6A (overseas/NRI), Form 7 (objection/deletion), Form 8
(shift/correction/EPIC replacement/PwD marking, absorbing the discontinued
Form 8A and Form 001), Form 2 (service electors, via servicevoter.nic.in),
and Form 12D (postal ballot/home voting for PwD and 85+ electors) — is
restated identically in this addendum's own
opening paragraph explicitly lists this as a v2 fact it is not revisiting or
contradicting). No new nuance surfaced. One scope caveat worth stating
explicitly here rather than leaving implicit: this entire form set belongs
to **ECI's** electoral-roll administration chain (ordinary electors, service
electors) for Lok Sabha/Assembly purposes. It has no direct bearing on
Panchayat/Municipal electoral rolls, which in at least some states run
through a different administrative chain entirely — see V1.8, the single
most significant correction in this log, for the full treatment. This is a
scope note, not a retraction: Forms 6/6A/7/8/2/12D remain exactly as
described in PRD v2 Section 2.3 for everything they actually cover.

### V1.2 Four qualifying dates (1 Jan / 1 Apr / 1 Jul / 1 Oct) for turning 18

**VERIFIED UNCHANGED.** Restated without contradiction in the v3 research
corpus. No new nuance. This remains the correct model for the decision
engine's age/qualifying-date branch node: compute against the nearest
*upcoming* of four candidate dates, not a single annual cutoff.

### V1.3 e-EPIC download process and legal equivalence to physical EPIC

**VERIFIED UNCHANGED.** e-EPIC (available since 25 January 2021, OTP-verified
download, legally equivalent to the physical card) is restated without
contradiction. No new nuance from the v3 research pass. Note this is a
claim PRD v2 already correctly treats as a strong, specific legal assertion
used directly in user-facing copy ("this digital copy is just as valid as
your physical card") — that kind of claim is exactly the category the new
event-driven change-detection job (V1.0 above) should prioritize watching
for any future ECI notification that qualifies or extends it (e.g., to a
new card format), even though nothing suggests that has happened.

### V1.4 PwD/85+ postal ballot via Form 12D, submitted within 5 days of election notification

**VERIFIED WITH NEW NUANCE.** The core fact is unchanged: Form 12D is
submitted to the Returning Officer within 5 days of a specific election's
notification, distinct from the one-time PwD-marking step on Form 8 (PRD v2
Section 2.3's persona-8/Deepa distinction stands exactly as written). The
new nuance from v3 research: Section 149 of the Representation of the
People Act, 1951 requires a **by-election (bypoll)** to be called within six
months of a seat falling vacant, and ECI runs a bypoll on the same
structural notification → nomination → scrutiny → withdrawal → polling
timeline as a general election, just for one (or a handful of)
constituencies. This means the Form 12D 5-day clock is not something a
PwD/85+ elector only needs to track around general elections — it re-arms
around **every** by-election notification affecting their constituency too,
and by-elections are frequent, localized, and easy to miss precisely
because they don't get general-election-level public attention. Concrete
product implication: the new election-calendar feature (V2 of this
addendum's sibling Section V3, and this addendum's "Election
calendar" concept) must model by-elections as first-class calendar events,
not an edge case, and any proactive "an election is coming, here's your
saved Form 12D reminder" surfacing (per the "existing/old voter parity"
principle established above) must fire for bypoll notifications
exactly as it does for general elections.

### V1.5 National Voter Helpline 1950, NGSP grievance portal, Voter Helpline App, cVIGIL, Saksham

**VERIFIED UNCHANGED.** The 1950 helpline (1800-11-1950, 8am-8pm), Book-a-Call
with BLO via ECINET, NGSP 2.0, the Voter Helpline App, cVIGIL, and Saksham
are all restated without contradiction in the v3 research corpus, and none
of the v3 architecture decisions (jurisdiction routing, Bhashini, DIGIT,
DEPA, DPDP, accounts) touch this fact set. No new nuance. The specific
"resolve within 48 hours" NGSP service-level figure carried in PRD v2
Section 2.5 was treated as given, verified research at the time this PRD
was written and is repeated here on that same basis — it is not
independently re-derived by this addendum from a primary source, and is a
reasonable candidate for the standard content-review cadence to re-confirm
against the NGSP portal's own published service standards, but that is
routine upkeep, not a new doubt raised by v3 research specifically.

### V1.6 GIGW 3.0 baseline (WCAG 2.1 AA + India-specific extensions)

**VERIFIED WITH NEW NUANCE.** GIGW 3.0 (maintained by NIC/MeitY, WCAG 2.1 AA
floor plus India-specific UX/multilingual requirements) is restated without
contradiction, and PRD v2's decision to adopt WCAG 2.2 AA as VoteAssist's
own voluntary baseline (stricter than GIGW's own WCAG 2.1 AA floor) stands
unchanged. The new nuance is procedural, not factual: GIGW is a
periodically-revised government guideline document (this is inherent to
"maintained by NIC/MeitY," not a specific announced revision this addendum
is aware of), which makes it a slightly unusual case for the KB
re-verification model described in V1.0 — it is not a `knowledge_entry`
citation with a `lastVerifiedDate` in the ordinary sense, it is a standard
the whole platform's accessibility posture is pinned to. Recommendation for
the next content-review cycle: add the GIGW guidelines page itself (on
guidelines.gov.in or MeitY's site — exact canonical URL is TODO: confirm)
to the small curated list the new official-source change-detection job (V1.0)
polls, alongside ECI's forms/FAQ pages, rather than leaving "is GIGW still
at version 3.0" as an implicit assumption nobody is watching.

### V1.7 DPDP Rules 2025 notification date/phased timeline, and the actual scope of data-principal rights

**VERIFIED WITH NEW NUANCE — read carefully, this corrects an imprecision in
PRD v2's wording.** The notification date and timeline are unchanged and
correct: DPDP Rules 2025 were notified 13 November 2025 (Gazette 14 November
2025), phased compliance through 13 May 2027. PRD v2 Section 19.1 and
Section 2.7, and the underlying `docs/06-legal-compliance-review.md` Section
3, are accurate on that timeline and on the core minimal-data-by-design
posture built around it.

Where PRD v2's wording needs tightening: `docs/06-legal-compliance-review.md`
Section 3 states VoteAssist-held user data "must be deletable and
**retrievable** on request via a documented process," and PRD v2 Section
19.1/2.7 (R-DPDP-5) restates this as a "documented right-to-erasure/access
process." Taken together, "deletable and retrievable" is loose enough to be
misread as implying something like a GDPR-style data-portability guarantee
(a structured export a user could hand to a different service). The v3
research corpus is specific that the DPDP Act's actual data-principal rights
are **narrower** than that reading suggests:

- A right to **correction and erasure** (erasure required once the purpose
  is served, consent is withdrawn, or the individual disengages past a
  retention period) — this part of PRD v2's framing is accurate.
- A right of **access** to information about what's processed and why —
  also consistent with PRD v2's framing.
- A **90-day** statutory response window for data-principal requests — this
  specific figure does **not** appear anywhere in PRD v2 or in
  `06-legal-compliance-review.md` at all; it is not contradicted, it is
  simply absent, and should be added explicitly to Section 19.1's language
  so the document states the actual statutory baseline it is voluntarily
  exceeding.
- **No explicit, GDPR-Article-20-style right to data portability** (the
  right to receive one's data in a structured, commonly-used, machine-
  readable format and transmit it to another controller). DPDP does not
  grant this.
- **No explicit right to object to processing** on grounds broader than
  withdrawing consent.
- **No explicit right against solely-automated decision-making** (no
  GDPR-Article-22 analogue).

None of this means PRD v2 stated something false — "retrievable... via a
documented process" is defensible as describing an access right, and PRD v2
never uses the word "portability." But the phrasing is loose enough that a
future reader (or a future feature proposal) could reasonably infer a
statutory portability guarantee that does not exist, and the new optional-
accounts feature (Section V6, written separately) leans directly on this
distinction: it offers one-click JSON export as a **product choice that
exceeds the legal minimum**, not as a discharge of a DPDP portability
mandate, precisely because no such mandate exists. **Correction needed in
PRD v2 Section 19.1 and `docs/06-legal-compliance-review.md` Section 3**:
replace "deletable and retrievable" language with explicit, separated
statements of (a) the actual statutory rights (access, correction, erasure,
90-day response window), and (b) VoteAssist's voluntary product commitments
that go beyond them (self-service export, faster-than-statutory deletion
SLA), so the privacy policy this feeds into (`/about/privacy`) never
implies a legal guarantee VoteAssist does not actually owe users under
DPDP. This is squarely a wording-precision fix, not a redesign — the
underlying architecture (minimal data, no document custody, self-service
delete) already comfortably clears the real bar.

### V1.8 voters.eci.gov.in/ECINET as primary/secondary deep-link targets — jurisdiction scope correction

**VERIFIED WITH NEW NUANCE — SIGNIFICANT, TREAT AS A CORRECTION, NOT A
FOOTNOTE.** This is the single most important correction this fact-
verification pass surfaces, and it should be read as a scope correction to
PRD v2's entire deep-linking model, not a narrow factual quibble.

PRD v2 (Sections 2.1, 2.9, 4, and the `states`/`assembly_constituencies`/
`parliamentary_constituencies` schema in Section 8.1) is accurate as far as
it goes: voters.eci.gov.in is correctly the primary deep-link target and
ECINET the correct secondary target for everything ECI actually
administers. The gap is that PRD v2 never states, and its schema and
routing model never encode, which elections ECI administers **and which it
does not**. Left as-is, the implicit assumption baked into "voters.eci.gov.in
is VoteAssist's primary deep-link target for essentially every terminal
outcome" (PRD v2 Section 2.1, verbatim) reads as "ECI = all Indian
elections." That is not accurate.

India has **two** separate constitutional election-administration bodies:

- **ECI** (Article 324): Lok Sabha, Rajya Sabha (indirect), State Legislative
  Assembly, State Legislative Council (partly indirect/nominated), the
  (indirect) Presidential/Vice-Presidential election, and by-elections to
  any of the above.
- **State Election Commissions** (SECs, one per state, Article 243K for
  Panchayati Raj Institutions, mirrored by Article 243ZA for Urban Local
  Bodies): **all** elections to Gram Panchayats, Panchayat Samitis/Blocks,
  Zilla Parishads, Municipal Corporations, Municipal Councils, and Nagar
  Panchayats — including "superintendence, direction and control of the
  preparation of electoral rolls" for these bodies, per the Constitution's
  own text.

voters.eci.gov.in and ECINET **do not cover Panchayat or Municipal elections
at all.** A VoteAssist user asking "how do I register/check my status for my
Gram Panchayat election" who is routed to voters.eci.gov.in is being sent to
the wrong authority entirely — not a suboptimal path, a wrong one. There is
no unified national SEC portal analogous to voters.eci.gov.in; each state's
SEC runs its own site (e.g., State Election Commission Punjab, Maharashtra
SEC), so this is also not a simple "add one more link" fix — it requires a
per-state SEC portal reference table analogous to (and currently missing
alongside) the CEO-portal-URL gap PRD v2 Section 2.9 already flags.

One further genuinely open question, not resolved by this correction and
worth its own explicit **FLAGGED FOR RE-CHECK**: whether a given state's
Panchayat/Municipal electoral roll is derived from the same underlying
ECI/ERONET/UNPER roll data or maintained as a wholly separate roll by that
state's SEC is **state-specific** — state legislation on local-body roll
preparation varies, and no uniform national answer exists in the research
compiled for this addendum. This must not be asserted uniformly one way or
the other in product copy. **Recommended sources for the next content-
review cycle to check on a per-state basis**: each state's Panchayati Raj
Act and Municipal Corporation/Municipalities Act (for the roll-preparation
provision specifically), and the relevant State Election Commission's own
published rules — starting with the highest-population states first given
limited review bandwidth.

**This is not re-litigated in this section.** The full design response
(jurisdiction-routing model in the decision engine, the SEC reference-data
extension, per-state SEC portal table, and how this interacts with the
existing national-tree-with-state-varying-deep-links architecture from PRD
v2 Section 2.9) is specified in this addendum's **Section V3
(election-jurisdiction)**, written separately. This entry's job is only to
make the correction explicit and point clearly at the fix: PRD v2's routing
model must add a jurisdiction-authority determination (ECI vs. the user's
state SEC) as a first-class step for any election-related question, before
it ever selects a deep-link target — it is a routing-logic gap, not a
missing link.

---

## V2. New Open-Source & Government Platform Research

VoteAssist's Rust-native architecture (PRD v2 Section 6) and its
deep-link-only, no-scraping, minimal-data posture (PRD v2 Section 3) were
already settled decisions before this addendum. Nothing below revisits
those decisions. This section evaluates four categories of adjacent
platform/prior-art research against that already-settled architecture:
government-backed language infrastructure (Bhashini), a government-service
delivery platform (DIGIT), India's consent-architecture pattern (India
Stack/DEPA), and international civic-tech prior art. In every case the
question asked is the same one PRD v2 Section 3 already establishes as the
operating discipline: what's the concrete architectural touchpoint, if any,
and what's explicitly declined and why.

### V2.1 Platforms and projects reviewed, at a glance

| Platform / Project | Category | What it actually is | VoteAssist's concrete touchpoint | Verdict |
|---|---|---|---|---|
| **Bhashini** | Government language-AI platform (MeitY) | Free ASR/MT/TTS API platform, 300+ models, 22+ voice languages, 36+ text languages, ULCA-based | New `crates/bhashini-client`; MT-assist in admin Translation Management (§11.5); candidate STT/TTS for future `crates/ivr-gateway` | **ADOPT** (as an additive API integration, not a replacement for the human-review gate or for Exotel telephony) |
| **DIGIT** (eGovernments Foundation) | Government service-delivery microservices platform | MIT-licensed, Java/Node, modular workflow engine + MDMS, deployed in 5+ states, 40M+ urban citizens | Two design patterns only: config-driven multi-tenant workflow model; master-data-registry pattern | **PATTERN-ONLY** (explicitly not a dependency) |
| **India Stack / DEPA** (DigiLocker, eSign, Account Aggregator) | Consent-management architecture | Data-blind Consent Manager pattern underlying finance-sector data sharing | Consent-discipline pattern borrowed for the new optional-accounts feature (§V6) | **PATTERN-ONLY** (explicitly no AA/DigiLocker network integration) |
| **Democracy Works / TurboVote Elections API** | US civic-tech nonprofit | Address-scoped "upcoming elections near me" API + reminders | UX pattern for the new election-calendar feature | **REUSE-AS-PATTERN** |
| **Google Civic Information API** | Commercial/government-adjacent civic API | Elections/Divisions data; Representatives-lookup endpoint sunset April 2025 | Cautionary data point only | **AVOID as a dependency; CITE as a caution** |
| **OpenElections** | US civic-tech data project | Structured, versioned historical election **results** schema | Schema-discipline inspiration only (VoteAssist does no results reporting) | **REUSE-AS-PATTERN** (schema discipline only) |
| **VotingWorks** | US civic-tech nonprofit | Open-source voting-machine/audit software, accessible at-home voting product | Credibility nod for accessible-voting-as-fundable-pattern; no code/API relevance | **AVOID** (out of scope), one-line credibility nod |
| **Ushahidi** | Crowdsourced incident-reporting platform | Election-monitoring deployments in 27+ countries (e.g., Kenya's Uchaguzi) | None proposed | **AVOID**, explicitly, on neutrality grounds (see V2.5) |

### V2.2 Bhashini

**What it is.** Bhashini (bhashini.gov.in) is MeitY's National Language
Translation Mission platform (launched 2022): a free, government-funded,
open API layer exposing 300+ pre-trained AI models — Automatic Speech
Recognition (ASR), Machine Translation (MT), and Text-to-Speech (TTS) —
across 22+ Indian languages for voice and 36+ for text, built by a
consortium of IITs, IIITH, and CDAC research groups, and exposed to
developers via the ULCA (Universal Language Contribution API) platform.
Access is free for developers at the API level (rate limits and terms of
use apply and should be confirmed against Bhashini's current developer
terms before production integration — TODO: verify current rate-limit and
ToS specifics against bhashini.gov.in's developer documentation, not
assumed unlimited).

**Concrete architectural proposal.** Add a new crate, **`crates/bhashini-
client`**, following exactly the same pattern PRD v2 already established
for `crates/bot-whatsapp` and `crates/ivr-gateway`: a thin `reqwest`-based
wrapper over Bhashini's REST/JSON API, holding no business logic, doing
translation/transcription/synthesis calls only. This is a deliberate
consistency choice — VoteAssist's architecture already has a standing
pattern for "no mature Rust-native SDK exists for this external service, so
wrap it thinly rather than adopt a heavier dependency," and Bhashini fits
that pattern exactly (no Rust SDK is known to exist for Bhashini/ULCA at
time of writing — TODO: re-check before implementation in case one has
since appeared). Proposed surface, deliberately minimal:

```
pub struct BhashiniClient { /* base_url, api_key, http: reqwest::Client */ }

impl BhashiniClient {
    pub async fn translate(&self, text: &str, source_lang: &str, target_lang: &str) -> Result<TranslationDraft>;
    pub async fn transcribe(&self, audio: AudioPayload, lang: &str) -> Result<TranscriptDraft>;
    pub async fn synthesize(&self, text: &str, lang: &str, voice: Option<&str>) -> Result<AudioPayload>;
}
```

Exact ULCA endpoint paths, auth header format, and model/pipeline-ID
discovery mechanism are TODO: verify against Bhashini's current published
API reference before implementation — the shape above is a integration
contract proposal, not a claim about Bhashini's actual wire format.

**Where it plugs in — two integration points, both additive:**

1. **Admin Translation Management (PRD v2 Section 11.5, admin page 5).**
   PRD v2 Section 11.5 already specifies a generic "Machine-translation-
   assist button" per string/entry that "calls a machine-translation
   service" and populates a clearly-labeled, unapproved draft. This
   addendum's concrete proposal is to make **Bhashini the named provider**
   behind that existing button, via `bhashini-client`'s `translate()` call.
   This changes nothing about the workflow PRD v2 already specified: MT
   output is still never auto-published, the draft-label flag is still
   server-side-enforced and cannot be cleared except by human edit or
   explicit reviewer acknowledgment, and KB-body/decision-tree-text
   approval is still hard-gated to `legal_reviewer`/`superadmin` (PRD v2
   Section 11.5's guardrails, unchanged). Bhashini is an *input* to an
   already-specified human-review gate, not a new gate and not a
   replacement for one. Its relevance is specifically for the 21
   under-resourced Eighth Schedule languages still on the translation
   roadmap (PRD v2 Section 17.1) — a free, government-backed MT baseline
   materially lowers the cost of getting a first, human-reviewable draft
   in front of a translator for languages where volunteer translator
   capacity is thin.
2. **Future IVR/voice-mode channel (PRD v2 Section 6.7's
   `crates/ivr-gateway`).** PRD v2 already chose Exotel for the telephony
   layer specifically because "Exotel's voice-streaming platform is
   vernacular-first with Indic STT/TTS support from day one" (Section 6.7,
   verbatim) — i.e., PRD v2's existing rationale for Exotel already rests
   partly on Exotel's *own* language capability, not only its telephony
   capability. This addendum's job is to make that an explicit, evaluated
   choice rather than an implicit one: **Bhashini and Exotel's own
   STT/TTS should be evaluated side by side** for the language layer,
   not assumed to be mutually exclusive. Concretely:

   | Dimension | Exotel's own STT/TTS | Bhashini |
   |---|---|---|
   | Telephony (place/receive calls, DTMF, call routing) | Yes — this is infrastructure Bhashini does not provide at all | No — Bhashini is not a telephony platform |
   | Cost | Commercial, bundled with telephony usage | Free at the API level (subject to ToS/rate limits, TODO: verify) |
   | Language coverage | India-native, vernacular-first per Exotel's own positioning; exact language list and quality per-language is a vendor claim, TODO: verify against Exotel's current published language list before committing | 22+ voice languages, 36+ text languages, government-published model roster |
   | Governance/credibility | Commercial vendor SLA | Government-backed, aligned with MeitY's language-access mission — a credibility asset for a civic-tech product |
   | Integration complexity | Single vendor relationship already chosen for telephony; using its STT/TTS too means one fewer integration | A second vendor/API relationship alongside Exotel |

   The tradeoff is not "replace Exotel" — Exotel (or an equivalent India-
   native telephony vendor) remains necessary regardless of which STT/TTS
   is used, because Bhashini has no telephony capability whatsoever;
   something has to actually place and receive the phone call. The
   realistic architectures are: (a) Exotel handles both telephony and
   language processing end-to-end, if its own Indic STT/TTS proves
   sufficient in evaluation — in which case Bhashini adds no value on this
   integration point and is not adopted here; or (b) Exotel handles
   telephony and streams audio to `bhashini-client` for STT/TTS, giving
   VoteAssist a government-backed, free, and potentially broader-coverage
   language layer decoupled from a single commercial vendor's language
   roadmap. **This addendum does not resolve which of (a) or (b) is
   correct** — that requires an actual side-by-side quality/latency
   evaluation once `ivr-gateway` moves from scaffolded interface (PRD v2
   Section 6.7's stated MVP-Rust-v1 status) to real implementation, which
   PRD v2 already places at v2/v3, i.e., not urgent. It is recorded here as
   an explicit open question for that future evaluation, not a decision.

**What VoteAssist does not do with Bhashini.** No wholesale replacement of
the FTL/Fluent-based human translation workflow (PRD v2 Section 6.11,
17.3); no use of Bhashini output as final, unreviewed, published content
under any circumstance; no dependency on Bhashini for the currently-shipped
English/Hindi content (which is fully human-translated and reviewed
already, per PRD v2 Section 17.2) — Bhashini is relevant to the roadmap
languages, not to what's already shipped.

### V2.3 DIGIT (eGovernments Foundation)

**What it is.** DIGIT is an MIT-licensed, modular microservices platform for
government service delivery — a workflow engine, a Master Data Management
Service (MDMS), a notification service, ID generation, and multi-tenancy
support — deployed across 5+ Indian states, serving 40M+ urban citizens for
municipal service-delivery use cases (property tax, water/sewerage
connections, grievance redressal, and similar).

**Why VoteAssist does not adopt it as a dependency.** This is a deliberate
non-adoption, stated plainly rather than left as an unexamined gap: DIGIT
is a large Java/Node platform with its own operational footprint (its own
deployment model, its own database conventions, its own service mesh
expectations), and adopting it would directly contradict the Rust-native
architecture decision already made and extensively justified in PRD v2
Section 1.2/6 ("use rust system," per explicit prior direction). Pulling in
a large polyglot platform to solve problems VoteAssist's much smaller,
purpose-built Rust workspace already solves adequately (a workflow/decision
engine in `core-domain`, reference tables in `sqlx` migrations) would be a
scale and complexity mismatch, not a capability gap DIGIT is uniquely
positioned to fill.

**What is adopted: two design patterns, as inspiration, not integration.**

1. **Config-driven, multi-tenant workflow model.** DIGIT's core architectural
   idea — that different government workflows (property tax assessment,
   water-connection approval, grievance handling) are expressed as
   *configuration* against a shared workflow engine, rather than as
   separate hand-coded code paths per workflow type — is directly relevant
   to this addendum's V3 (election-jurisdiction) goal of generalizing
   VoteAssist's decision engine to handle multiple election types (Lok
   Sabha/Assembly via ECI, Panchayat/Municipal via SECs) without a
   proliferation of type-specific branching logic in `core-domain`.
   Concretely: an "election type" (with its administering authority, its
   applicable form set, its jurisdiction-routing rules) should be a
   **data** record the decision engine reads, following DIGIT's precedent,
   not a `match` arm hard-coded per election type in Rust. This is a
   pattern borrowed from DIGIT's MDMS-plus-workflow-engine design, not code
   or a library.
2. **Master-data-registry pattern.** DIGIT's MDMS concept — a single,
   versioned, referenceable source of master/reference data (localities,
   departments, workflow definitions) that every service consults rather
   than duplicating — is the same shape of problem PRD v2 Section 8.1's
   `states` / `assembly_constituencies` / `parliamentary_constituencies`
   tables already partially solve. This addendum's concrete extension
   (detailed fully in Section V3, not repeated here): those tables need
   new siblings for **State Election Commissions** (one row per state, SEC
   name, SEC portal URL — the SEC-portal-URL table flagged as missing in
   V1.8 above), **Panchayat/Municipal jurisdiction levels** (Gram
   Panchayat / Panchayat Samiti / Zilla Parishad / Municipal Corporation /
   Municipal Council / Nagar Panchayat, per state, where that
   state-specific hierarchy is knowable), and **election-type metadata**
   (which authority administers it, which form set applies, whether it's a
   by-election variant). This is the same lightweight-Postgres-table
   approach PRD v2 already uses, extended in scope — not a MDMS service
   adopted wholesale.

**Explicit boundary.** No DIGIT service is deployed, called, or depended on
anywhere in this architecture. If a future integration need genuinely
required interoperating with a state government's existing DIGIT-based
municipal-services deployment (e.g., a state that runs its grievance
redressal on DIGIT), that would be evaluated as its own proposal against
the Significant-Data-Fiduciary-avoidance and minimal-data goals (PRD v2
Section 19.2) at that time — nothing here proposes it now.

### V2.4 India Stack / DEPA consent architecture

**What it is.** India's Data Empowerment and Protection Architecture (DEPA)
is the consent-management design underlying DigiLocker (verified document
storage/sharing), eSign (Aadhaar-based digital signatures), and the Account
Aggregator (AA) framework used in the financial sector for consented data
sharing between regulated entities. Its central architectural idea is the
**"Consent Manager"**: an intermediary that is *data-blind* — it cannot read
the content it relays, only the consent artifact governing whether a
transfer between two other parties is authorized — combined with a
discipline that every consent grant is explicit, purpose-scoped, time-
bound, revocable, and logged as an auditable artifact a user can inspect.

**What VoteAssist borrows, and what it explicitly does not.** VoteAssist
has no financial data-sharing need and no cross-institution document-
transfer need — it does not handle documents at all, by design (PRD v2
Section 4, non-goal 3: "no document custody by default"). It therefore
does **not** integrate with the actual AA network, DigiLocker's API, or
eSign — there is no concrete use case today that would justify that
integration surface, and adding one would be exactly the kind of scope
expansion Section 19.2's Significant-Data-Fiduciary-avoidance goal exists
to gate. What it *does* borrow is the **consent discipline itself**, as a
design philosophy applied to the new optional-accounts feature specified
in this addendum's Section V6 (written separately — not duplicated here):
explicit, purpose-scoped consent captured at account creation rather than
folded into a blanket Terms-of-Service checkbox; a visible, literal
rendering of what an account holds back to the user (an account settings
page that shows the actual stored data, the DEPA-style "consent artifact
made visible" idea, not a prose description of it); and a revocation path
(account deletion) that is one action, immediate, and logged.

**A plausible future use case, explicitly not built now.** The one scenario
where an actual DigiLocker integration might someday make sense — a user
pulling their own address-proof document from their own DigiLocker to
attach when filing Form 6/8 — happens entirely on **ECI's own site**
(voters.eci.gov.in), not VoteAssist's, and is consistent with VoteAssist
never touching document custody. This is named here only to be explicit
that the "no AA/DigiLocker integration" position is a scoping choice
grounded in absence of a current use case, not a blanket architectural
objection to ever touching India Stack infrastructure.

### V2.5 International prior art

| Project | What it is | Relevant to VoteAssist | Verdict + reasoning |
|---|---|---|---|
| **Democracy Works (TurboVote) Elections API** | US civic-tech nonprofit; an address-scoped "what elections apply to me" lookup API plus opt-in voting reminders | Directly analogous *problem shape* to VoteAssist's new election-calendar feature: jurisdiction-scoped election lookup, opt-in reminders | **REUSE-AS-PATTERN, not code or data.** US-specific data, no India coverage, no integration possible or desirable. The UX pattern — resolve a user's address/jurisdiction to the specific elections that apply to them, then let them opt in to a reminder — is directly reusable for VoteAssist's own election-calendar UX (a curated-content model, not a live external API), including its ECI-vs-SEC jurisdiction split from V1.8/Section V3. |
| **Google Civic Information API** | Google's civic-data API; the Representatives-lookup endpoint was sunset April 2025, Elections/Divisions endpoints remain (US-focused) | Cautionary data point for the same election-calendar feature | **AVOID as a dependency (no India coverage regardless); CITE as a caution.** Even a well-resourced, Google-operated civic API can be deprecated with real disruption to downstream integrators. This directly reinforces the decision, made elsewhere in this addendum, to treat "upcoming elections" as admin-curated content rather than a hard dependency on any single external API, ECI-published or otherwise — the Google case is concrete evidence for why that caution is warranted generally, not specific to Google. |
| **OpenElections** | US project standardizing historical election **results** data with a documented, versioned schema | VoteAssist does no results reporting at all — this is schema-discipline inspiration only | **REUSE-AS-PATTERN, narrowly.** The only transferable idea is "structured, versioned, citable election data deserves its own disciplined schema, not ad hoc fields" — relevant to how the new `elections` KB concept (curated calendar entries) and the DIGIT-inspired election-type metadata (V2.3) should be modeled: dated, versioned, source-cited, not freeform text fields. Not relevant to anything about results, which VoteAssist never touches. |
| **VotingWorks** | US nonprofit open-source voting-machine and risk-limiting-audit software, including an accessible at-home voting product | Out of scope: VoteAssist never touches vote-casting infrastructure (non-goal, unchanged) | **AVOID**, unambiguously — different problem domain entirely (VoteAssist is a guidance layer, not voting infrastructure). Worth exactly a one-line credibility nod: VotingWorks' funded, accessible-voting-focused product line is evidence that PwD-first, accessibility-first design for voting-adjacent civic tools is an established and fundable pattern in this space, which is a useful data point for VoteAssist's own funding/credibility conversations, not an architectural or code relevance. |
| **Ushahidi** | Crowdsourced incident-reporting platform, used in election-monitoring deployments across 27+ countries (e.g., Kenya's Uchaguzi) | Superficially adjacent — "citizen reporting during elections" sounds like it overlaps with VoteAssist's civic-participation space | **AVOID, explicitly, and for a reason specific to VoteAssist's mission, not a generic "not invented here."** Crowdsourced political/election-incident reporting requires moderating politically-charged submissions — exactly the kind of judgment call VoteAssist's strict political-neutrality non-goal (PRD v2 Section 4, non-goal 4) is designed to keep the product out of making. This need is also already served: ECI's own official cVIGIL app exists precisely for citizen reporting of Model Code of Conduct violations (PRD v2 Section 2.5), and PRD v2 already establishes that VoteAssist's only relationship to that space is a factual "if you want to report an MCC violation, that's a separate ECI tool" note, not active promotion or a parallel feature (PRD v2 Section 3's table, `cVIGIL` row). Building an Ushahidi-style reporting feature would duplicate an existing official tool while taking on exactly the neutrality risk VoteAssist is built to avoid. This verdict is unchanged from v2 and is restated here only because Ushahidi itself (as opposed to cVIGIL) was not previously named explicitly in the PRD v2 landscape table. |

### V2.6 Consolidated new open questions from this section

- Bhashini's current developer ToS/rate limits (TODO: verify against
  bhashini.gov.in's developer documentation before any production
  integration is scheduled).
- Whether a Rust-native ULCA/Bhashini SDK crate exists at implementation
  time (none identified in this research pass; re-check before starting
  `bhashini-client`).
- The Exotel-native-STT/TTS vs. Bhashini side-by-side evaluation for
  `ivr-gateway` (V2.2) — explicitly unresolved, deferred to the v2/v3
  implementation window PRD v2 Section 6.7 already assigns to
  `ivr-gateway`.
- The canonical GIGW guidelines URL to add to the official-source
  change-detection job's watch list (V1.6).
- Per-state Panchayat/Municipal electoral-roll administration chain
  (ECI-derived vs. SEC-independent) — genuinely state-specific, flagged in
  V1.8, to be resolved incrementally per state rather than asserted
  uniformly.
## V3. Generalized Election Model — Every Election Type, Correct Jurisdiction

### V3.1 Why this section exists

PRD v2 implicitly scoped VoteAssist to "the Election Commission of India."
That scoping is incomplete in a way that matters legally, not just
cosmetically: **India has two separate constitutional election-
administration bodies**, and a platform that generalizes to "all types of
elections" without modeling that split will confidently send a citizen to
the wrong official system for a real fraction of the elections that affect
them. This section fixes that at the architecture level, before any code
is written against it.

### V3.2 The jurisdiction split

| Election type | Administering body | Constitutional basis | VoteAssist's role |
|---|---|---|---|
| Lok Sabha (House of the People) | Election Commission of India (ECI) | Article 324 | Full guidance + deep links (existing v2 scope) |
| Rajya Sabha (Council of States) | ECI | Article 324 | Informational only — this is an indirect election by sitting MLAs, not by ordinary citizens; VoteAssist has no registration-guidance role here, but should be able to correctly answer "can I vote in a Rajya Sabha election?" (answer: no, not directly) rather than staying silent and letting a user assume the Lok Sabha flow applies |
| State Legislative Assembly (Vidhan Sabha) | ECI | Article 324 | Full guidance + deep links (existing v2 scope) |
| State Legislative Council (Vidhan Parishad), where it exists | ECI (elected seats) / Governor (nominated seats) | Article 324 (elected portion) | Informational only, same reasoning as Rajya Sabha — largely indirect/nominated, not a direct-citizen-vote registration scenario |
| President of India | Electoral college (MPs + MLAs), administered by ECI | Article 324 | Informational only — no ordinary citizen registration applies |
| Vice President of India | Electoral college (MPs only), administered by ECI | Article 324 | Informational only |
| By-elections (bypolls) to any ECI-administered seat | ECI | Representation of the People Act 1951, s.149 (6-month rule) | Full guidance — same registration/roll mechanics as a general election to that seat type, just narrower in geographic scope; see V3.4 |
| Panchayat elections (Gram Panchayat, Panchayat Samiti/Block, Zilla Parishad) | **State Election Commission (SEC)** of the relevant state | Article 243K | Guidance limited to what's genuinely common across states (see V3.5) plus a jurisdiction-routing terminal to the correct state SEC — VoteAssist does NOT attempt a unified national Panchayat-election decision tree, because there is no unified national portal or rule set to route to |
| Municipal/Urban Local Body elections (Municipal Corporation, Municipal Council, Nagar Panchayat) | **State Election Commission (SEC)** of the relevant state | Article 243ZA | Same posture as Panchayat elections above |

The practical consequence for the decision engine: **every session must
resolve, as early as is natural in the conversation, which body's rules
apply** — and for Panchayat/Municipal questions, resolve which STATE's SEC
applies, since there is no national equivalent of voters.eci.gov.in for
local-body elections.

### V3.3 New data model: jurisdictions and election types

Extending PRD v2 Section 8's reference tables:

```sql
CREATE TYPE administering_body AS ENUM ('eci', 'state_election_commission');

CREATE TYPE election_type AS ENUM (
    'lok_sabha', 'rajya_sabha', 'legislative_assembly', 'legislative_council',
    'president', 'vice_president', 'by_election',
    'panchayat_gram', 'panchayat_block', 'panchayat_zilla',
    'municipal_corporation', 'municipal_council', 'nagar_panchayat'
);

CREATE TABLE state_election_commissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    state_id UUID NOT NULL REFERENCES states(id),
    official_name TEXT NOT NULL,
    portal_url TEXT NOT NULL,
    -- Whether this state's SEC maintains a distinct local-body electoral
    -- roll from the ECI/ERONET chain, or derives it from the ECI roll.
    -- Genuinely varies by state law; NULL until individually verified.
    roll_derivation_note TEXT,
    roll_derivation_verified_date DATE,
    last_verified_date DATE NOT NULL,
    CONSTRAINT uq_sec_per_state UNIQUE (state_id)
);

CREATE TABLE election_type_jurisdiction (
    election_type election_type PRIMARY KEY,
    administering_body administering_body NOT NULL,
    is_direct_citizen_vote BOOLEAN NOT NULL,
    voteassist_scope TEXT NOT NULL CHECK (voteassist_scope IN ('full_guidance', 'informational_only', 'jurisdiction_routing_only'))
);

-- Seed data reflects the table in V3.2 exactly; this is reference data,
-- not user data, and changes only on constitutional amendment or a fresh
-- ECI/SEC jurisdictional ruling — an exceptionally rare event, but the
-- table exists so the mapping is data, not a hardcoded match statement
-- buried in `core-domain`.
```

A new `crates/jurisdiction` module (or a submodule of `core-domain`) owns
exactly one responsibility: given a user's stated election-type interest
(or, more commonly, given that the decision tree doesn't ask "which
election?" up front because most users arrive with a registration-status
question, not an election-type question — see V3.4), resolve which body's
guidance applies and which deep link to offer. This keeps jurisdiction
logic out of the main decision-tree content (which stays about registration
mechanics) and in one small, testable place.

### V3.4 How this changes the decision tree's start of flow

The existing MVP/v2 tree opens with "Are you already registered as a
voter?" — that question is correct and unchanged for ECI-administered
elections (Lok Sabha/Assembly/by-elections all share the same registration
mechanics via Forms 6/6A/7/8/2). It does NOT make sense as the entry point
for a Panchayat/Municipal question, because "registered" means something
state-specific there. Rather than bolt a confusing "which election are you
asking about?" question onto the very first screen (bad UX — most users
don't think in terms of election type, they think in terms of "I need to
register" or "I need to check something"), the recommended flow is:

1. Keep the existing entry question set unchanged for the common case
   (registration status, corrections, EPIC issues) — this covers Lok
   Sabha/Assembly/by-election needs correctly, since that's what "the
   electoral roll" means to nearly every user by default.
2. Add a SEPARATE, clearly-labeled entry point ("I have a question about
   my local Panchayat/Municipal election" or similar, surfaced as an
   option on the home screen alongside the main "find out what I need to
   do" button, not buried inside the existing tree) that immediately asks
   for the user's state and routes to a jurisdiction-routing terminal: a
   short explanation that local-body elections are run by that state's own
   Election Commission, a deep link to the state SEC portal, and — only
   where the content team has verified it for that specific state — a note
   on whether the same electoral roll applies or a separate one is
   maintained.
3. Never let the main "registered/not registered" tree silently produce a
   Panchayat/Municipal answer — if a user's free-text search or a bot
   conversation surfaces intent that looks like a local-body question
   (e.g., typed "gram panchayat" into KB search), the search/bot layer
   should offer the jurisdiction-routing terminal as a suggested result,
   not force it through the ECI-shaped tree.
4. For by-elections specifically: no new tree branch is needed. A bypoll
   uses identical Form 6/6A/7/8 mechanics to a general election for that
   seat — the ONLY new product surface a bypoll needs is the election-
   calendar feature (V5) surfacing "there's a by-election in your
   constituency on [date]" so an affected user finds out at all, since
   bypolls are easy to miss compared to a general election.

### V3.5 What VoteAssist can honestly say about Panchayat/Municipal elections nationally

Because there are 28 states' worth of SECs, each with their own portal,
own local-body-specific forms, and (per V3.2) potentially their own roll
maintenance rules, VoteAssist's v1/v2-Rust-roadmap commitment here is
deliberately modest and honest, mirroring the same "flag for local
verification" pattern already used in PRD v2 Section 10 for tribal/remote-
area address-proof questions:

- **What ships at MVP-Rust-v1/v1 (national, verifiable once, true
  everywhere)**: the constitutional/jurisdictional facts in V3.2 (who runs
  what), a generic explanation of the Panchayat/Municipal tier structure,
  and the jurisdiction-routing terminal described in V3.4.
- **What requires per-state legal/content research before shipping (v2
  roadmap item, same discipline as PRD v2's per-state tribal/urban-slum
  review program)**: each state SEC's portal URL and current online-
  service availability (many SEC portals are far less digitized than
  voters.eci.gov.in), whether that state's local-body roll is
  ECI-roll-derived or separately maintained, and any state-specific local-
  body registration/correction forms. This is tracked as a per-state
  checklist in the admin Knowledge Base (one `knowledge_entries` row per
  state, topic `local-body-elections`, reviewStatus starting at `draft`
  until a state's specifics are actually confirmed) — VoteAssist does not
  claim national local-body coverage until each state's row reaches
  `verified`, and the product UI must show, per state, whether that state's
  local-body information is verified yet or still pending, rather than
  presenting silence or a placeholder as if it were confirmed information.

### V3.6 Election-calendar / upcoming-elections tracking

New concept, addressed in more product-flow depth in Section V5, specified
here at the data-model level:

```sql
CREATE TABLE tracked_elections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    election_type election_type NOT NULL,
    state_id UUID REFERENCES states(id), -- NULL for a Lok Sabha general election spanning all states
    constituency_ids UUID[], -- specific ACs/PCs for a by-election; NULL/empty for a general election
    schedule_announced_date DATE,
    nomination_start_date DATE,
    nomination_end_date DATE,
    polling_date DATE,
    counting_date DATE,
    mcc_window_start DATE, -- typically = schedule_announced_date, see PRD v2 admin page 8
    mcc_window_end DATE,   -- NULL until results are declared, per PRD v2's MCC handling
    source_notification_url TEXT NOT NULL, -- the Gazette/PIB/ECI or SEC press release this was entered from
    entered_by UUID NOT NULL REFERENCES admin_users(id),
    entered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_verified_date DATE NOT NULL
);
```

This table is deliberately **admin-curated content**, not a live feed from
an external API — research for this PRD found no equivalent to Democracy
Works' US Elections API for India, and even that US API's sibling
(Google's Civic Information Representatives-lookup endpoint) was sunset in
April 2025, which is exactly the risk of over-depending on any single
external election-data API. An admin (or, longer-term, a semi-automated
ingestion assist reading ECI/SEC press-release feeds and proposing a draft
row for human confirmation — never auto-publishing, consistent with every
other content pipeline in this system) enters each election as it's
officially announced. This feeds: (a) the MCC Control Panel from PRD v2
admin page 8 (auto-suggesting an MCC window rather than requiring separate
manual entry), (b) a public "is there an election coming up where I live"
banner/notice on the web app and bot channels, and (c) opt-in reminder
notifications for users with a saved account (Section V6) who've
indicated a state of interest.
## V4. Existing / "Old" Voter Parity

### V4.1 Why this is a first-class pillar, not an afterthought

PRD v2's decision tree (Section 10) already contains substantial existing-voter
coverage — the entire `registered_action` branch (moved, PwD, corrections,
lost EPIC, object/delete, polling-station-changed) exists precisely because an
already-registered elector's ongoing needs were in scope from the MVP. What
has skewed toward new-voter framing is not the tree's *logic* but the
product's *presentation and lifecycle posture*: the home screen, the roadmap
narrative in `19-roadmap.md`, and the persona coverage matrix (Section 10.15)
all read as "help a citizen get registered," with existing-voter maintenance
implicitly bundled in as one branch among several rather than named as a
co-equal pillar. Ground-truth v3 is explicit that this must change: an
already-registered elector correcting details years later, re-confirming
registration before an election they didn't realize was coming, requesting
home voting again for a new election despite having done it once before, and
checking roll status after a Special Summary Revision are all to be treated
as **equally first-class to new-voter registration, not an afterthought**.

This section does not propose new decision-tree nodes or terminals — Section
10's existing terminals (`terminal_form8_shift`, `terminal_form8_correction`,
`terminal_eepic_download`, `terminal_form8_epic_replacement`,
`terminal_pwd_marking`, `terminal_senior_postal_ballot`,
`terminal_polling_station_changed`, `terminal_roll_search`) already cover the
mechanics. What this section adds is: (1) home-screen information
architecture that gives existing-voter needs equal visual weight, (2) a
proactive nudge system built on the V3.6 `tracked_elections` table and the V6
account layer, (3) explicit product handling of the fact that some existing-
voter services (Form 12D, roll re-verification) are *recurring*, not
one-time, and (4) a copywriting/IA rule that lets the same engine present
itself differently depending on whether the person arriving is more likely
mid-registration or maintaining an existing entry.

### V4.2 Home-screen entry paths: two co-equal primary actions

**Current state (implicit in PRD v2):** the tree's `start` node functionally
asks "are you already registered as a voter?" (per V3.4's description of the
existing entry flow), branching to `registered_action` (yes) or
`not_registered_age`/`citizenship_check` (no). But this is a *question inside
the tree*, reached after a generic "find out what you need to do" call to
action on the home screen. The distinction between new and existing voters
therefore lives one screen deep, not on the home screen itself.

**v3 requirement:** the home screen must present two co-equal primary
actions, not one primary action followed by a branching question:

| Primary action | Home-screen label (en) | Deep-links directly into | Skips |
|---|---|---|---|
| A | "Find out what you need to do" (new/unsure) | `not_registered_age` (or `citizenship_check` if the user's account/draft already indicates adult citizenship — see V4.7) | The generic registration-status gate question |
| B | "Check on your existing registration" (returning/existing voter) | `registered_action` | The generic registration-status gate question |

Both buttons are rendered with identical visual weight (same size, same row,
neither styled as "primary" and the other as "secondary" via color/size —
this is a deliberate IA decision, not a visual-design afterthought delegated
to a later design pass). Concretely, in `crates/web-app`'s Leptos SSR home
template, this means:
- Two equally-sized CTA cards/buttons side by side (stacking vertically on
  narrow viewports, in the same visual order, neither demoted below a fold).
- Each deep-links the anonymous session directly to the corresponding tree
  node, bypassing whatever gate question `start` currently asks — this is a
  data/routing simplification, not a tree change: the two home-screen buttons
  effectively *pre-answer* the gate question the same way a returning user's
  saved context pre-answers questions per V6.3's replay logic and per the
  minimalism rule in V5.1 rule 4 ("a returning user's saved context shortens
  the flow, never lengthens it").
- Neither button requires an account. Button B ("check on your existing
  registration") works identically for a fully anonymous user and a
  logged-in user with a saved draft — the only difference for a logged-in
  user is that V4.7's copy-tone shift applies and, if a saved draft exists,
  a "resume where you left off" affordance appears above the two buttons
  (not replacing them — a returning user might still want to start a fresh
  session for a different need, e.g., helping a family member).
- Under Button B specifically, add a secondary row of quick-links for the
  most common returning-voter needs that don't require going through the
  full `registered_action` question first: "Download your e-EPIC" (deep-
  links directly to `terminal_eepic_download`, see V4.6), "Find my polling
  station" (deep-links to `terminal_roll_search` /
  `terminal_polling_station_changed` framing), and, when a tracked election
  exists for the user's stated or detected state (V3.6), "Check if there's
  an election coming up" (deep-links to the election-calendar notice
  described in V4.3). These quick-links exist precisely so a returning voter
  whose actual need is "just let me get my e-EPIC" or "just tell me where to
  vote" never has to answer a question whose only purpose is routing them to
  a terminal they could reach directly.

### V4.3 Proactive re-verification nudges

**Trigger conditions.** A nudge fires for an account (per V6) only when
**both** of the following hold: (a) `user_accounts.notifications_opt_in =
true`, and (b) `notification_state_interest` is set to a specific state. No
nudge ever fires for an account that has not explicitly opted in and named a
state — this matches V6.5's rule that notification consent is a single,
specific, opt-in toggle with no vague "send me updates" default.

Given those preconditions, a nudge fires on exactly two kinds of events, not
on a fixed calendar cadence:

1. **A new `tracked_elections` row (V3.6) is entered for the account's
   `notification_state_interest`** (a general election, a by-election, or,
   pending the V3.5 per-state local-body content program, a Panchayat/
   Municipal election once that content reaches `verified`). This is an
   *event-driven* trigger, not periodic — it fires once, the first time the
   admin team curates and publishes a `tracked_elections` row for that
   state, not on every subsequent day the row exists.
2. **A Special Summary Revision (SSR) window opens for the account's state**
   (see V4.4 for what is and is not verified about SSR timing). This
   requires a new lightweight admin-curated table, structurally identical in
   spirit to `mcc_windows` (PRD v2 admin page 8) and `tracked_elections`
   (V3.6) — admin-entered upon ECI's public SSR announcement for a state,
   never auto-detected, since (per V3.6's reasoning about the absence of a
   reliable programmatic ECI feed) there is no live API to poll for this
   either:

```sql
-- [NEW TABLE] mirrors the admin-curated, never-auto-detected posture of
-- mcc_windows (PRD v2 admin page 8) and tracked_elections (V3.6).
CREATE TABLE ssr_windows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    state_id UUID NOT NULL REFERENCES states(id),
    window_start DATE NOT NULL,
    window_end DATE, -- nullable until the final roll is published
    source_notification_url TEXT NOT NULL,
    entered_by UUID NOT NULL REFERENCES admin_users(id),
    entered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_verified_date DATE NOT NULL
);
```

**Cadence discipline — explicitly NOT more often than this.** An account
receives **at most one nudge per distinct trigger event** (one per new
`tracked_elections` row, one per `ssr_windows` row opened for their state),
never a repeating reminder for the same event, and never a nudge with no
underlying event at all (there is no "just checking in every 90 days"
background cadence). This is enforced by a small log table so the job is
idempotent and auditable:

```sql
-- [NEW TABLE]
CREATE TABLE proactive_nudges_sent (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES user_accounts(id) ON DELETE CASCADE,
    trigger_type TEXT NOT NULL CHECK (trigger_type IN ('tracked_election', 'ssr_window', 'form12d_reapplication')),
    trigger_ref_id UUID NOT NULL, -- tracked_elections.id or ssr_windows.id
    channel TEXT NOT NULL, -- 'email', 'whatsapp', 'telegram' -- matches V6/bot channel scope
    sent_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (account_id, trigger_type, trigger_ref_id)
);
```

The `UNIQUE (account_id, trigger_type, trigger_ref_id)` constraint is the
actual cadence enforcement mechanism, not a policy an operator has to
remember: the nudge job (a new `crates/jobs` apalis task, e.g.
`proactive-nudge-dispatcher`, running on a modest daily schedule to check for
new un-notified rows) attempts an insert-then-send, and the database
constraint itself makes a duplicate send for the same event structurally
impossible, independent of any bug in the dispatch logic.

**Nudge content and deep link.** The nudge copy is a single sentence plus a
one-click deep link, not a paragraph: "It's a good time to check your
registration details are still correct" (SSR trigger) or "There's an
election coming up in [state] — here's what to check" (tracked-election
trigger), each linking directly into the roll-search terminal
(`terminal_roll_search`) pre-filled with whatever the account's most recent
saved draft already indicates (state, and EPIC number if the user previously
entered one in a prior session — never requested fresh at nudge time if
already known, per the same "don't re-ask what you already know" principle
as V4.2/V4.7).

**Channel and MCC/template interaction.** Because an SSR window or a
tracked-election announcement frequently coincides with (or shortly
precedes) an active MCC window for the same state, this nudge is exactly the
case PRD v2's MCC handling describes as "pure administrative
guidance" that should continue, with an added neutral disclaimer, rather
than being suppressed outright — the nudge is opt-in, user-requested,
administrative ("check your own registration"), and time-sensitive, not an
announcement or endorsement. Concretely: the nudge dispatcher checks
`mcc_windows` (PRD v2 admin page 8) for the account's state before sending;
if active, it still sends, but the WhatsApp/Telegram/email template used
includes the same neutral-disclaimer copy block the MCC panel's "Effect
preview" already describes for other proactive messaging, not a suppressed
send. On WhatsApp specifically, this nudge must use a Meta-pre-approved
message template (per admin page 9's constraint that only approved templates
may be used for proactive outbound messages outside a user-initiated 24-hour
window) — two distinct templates should be pre-submitted for Meta approval:
one for the SSR trigger, one for the tracked-election trigger, both including
the neutral-disclaimer variant as a template parameter rather than as two
entirely separate template pairs (Meta-approved templates are expensive to
get re-approved on every wording change, so parameterizing the disclaimer
in/out is preferable to maintaining four near-duplicate templates).

### V4.4 Special Summary Revision (SSR) — what is and is not verified here

**What can be stated with reasonable confidence from public ECI practice,**
without inventing a specific current-cycle schedule: ECI conducts periodic
revisions of the electoral roll, and a "Special" Summary Revision is an
intensified version of the ordinary annual summary revision — typically
timed ahead of a major (usually state Assembly or general) election in the
relevant state(s), during which the draft roll is published, a defined
window for claims and objections (Forms 6/7/8, per the amended 2022 Rules)
receives heightened processing attention (often including door-to-door BLO
verification in some cycles), and a final roll is subsequently published.
This structural pattern (draft roll → claims/objections window → final roll)
is consistent, cycle to cycle, with the ordinary roll-maintenance rhythm
already described in Section 2 of PRD v2, just compressed and intensified
around a specific state's upcoming election.

**TODO: verify SSR cycle timing and current-cycle status against
eci.gov.in.** This PRD deliberately does **not** assert: (a) a fixed
calendar month/window in which SSRs occur nationally (they are announced
per state/cycle, tied to that state's election calendar, not a single
national annual date); (b) whether any specific state currently has an SSR
in progress as of this document's writing; (c) the exact current claims/
objections window length or whether door-to-door BLO verification is
mandated in every cycle or only some. All of this must be confirmed against
current ECI/CEO-office announcements before the `ssr_windows` admin UI or
the nudge copy in V4.3 asserts specifics beyond "an SSR is a good time to
double-check your registration" — the product should not hardcode a claimed
SSR calendar it has not verified.

**Product implication, stated plainly regardless of the timing specifics
above:** an SSR window is exactly the moment when VoteAssist's proactive
nudge (V4.3) is most valuable and least likely to be dismissed as noise,
because the claims/objections action is genuinely time-boxed and consequential
during that window in a way an arbitrary "just checking in" reminder is not.
This is the core reasoning for tying the nudge's cadence to real events
(SSR windows, tracked elections) rather than a fixed periodic schedule — a
nudge sent because "it's been 90 days" competes with every other
notification a user gets and trains them to dismiss VoteAssist's messages;
a nudge sent because "your state's SSR claims window just opened" is
self-evidently relevant and time-sensitive, which is the entire design
rationale for the event-driven trigger in V4.3 rather than a calendar-driven
one.

### V4.5 Repeat-service scenarios: Form 12D is recurring, not one-time

PRD v2 Section 10.6 already correctly separates the one-time PwD roll-marking
step (`terminal_pwd_marking`, via Form 8) from the per-election home-voting
request (Form 12D, submitted "within 5 days of election notification"). What
neither `terminal_pwd_marking` nor `terminal_senior_postal_ballot`'s existing
checklist copy states explicitly is that **Form 12D must be resubmitted for
every subsequent election** — a PwD or 85+ elector who successfully used home
voting once does not have a standing arrangement; the next election requires
a fresh Form 12D submission within that election's own 5-day window. Left
implicit, this is exactly the kind of gap a real user hits once and reports
as a fringe case (V5.3) — better to close it now.

**v3 content extension (both existing terminals, no new node needed):**

- `terminal_pwd_marking` checklist gains one bullet: "Your PwD marking on the
  roll (Form 8) is permanent and does not need to be repeated. However, the
  home-voting/postal-ballot request itself (Form 12D) is per-election — you
  must submit a new Form 12D within 5 days of every election's notification,
  even if you used home voting in a previous election."
- `terminal_senior_postal_ballot` checklist gains the equivalent bullet
  (age-85+ eligibility itself never needs re-confirming, but the Form 12D
  request is per-election, identical framing).

**Product-level treatment as a recurring need, not a one-time terminal
outcome:** this is where the pattern connects to the election-calendar
feature (V3.6) rather than remaining static PRD copy. For an account with a
saved draft/frozen terminal snapshot whose `recommendedForms` includes
`form-12d` (i.e., the account previously reached `terminal_pwd_marking` or
`terminal_senior_postal_ballot`), the nudge dispatcher from V4.3 gains a
third trigger type, already reflected in the `proactive_nudges_sent` schema
above (`trigger_type = 'form12d_reapplication'`): whenever a new
`tracked_elections` row is entered for that account's state (or, once
constituency-level `tracked_elections.constituency_ids` targeting is
populated, matching a by-election in their specific constituency), and the
account's most recent relevant draft indicates a Form 12D outcome, the
dispatcher sends a distinctly-worded nudge: "There's an election coming up
in [state/constituency] — if you want home voting or a postal ballot again,
you'll need to submit a new Form 12D within 5 days of the official
notification. It doesn't carry over from last time." This is deliberately a
different message from the generic SSR/election nudge in V4.3 — it names the
specific recurring obligation rather than a generic "check your details"
prompt, because the underlying user need (don't miss a 5-day window, again)
is more specific and more time-critical than a general registration check.

**Same logic for anyone whose home-voting eligibility might need
re-confirming:** the PwD-marking terminal's roll entry does not expire, but a
user's actual disability status or circumstances could change (improve,
worsen, or a temporary condition resolve) between elections. This PRD does
not propose a forced re-confirmation step (there is no ECI requirement for
one, and inventing one would be exactly the kind of fabricated-specificity
this project's style rules prohibit) — but the same `form12d_reapplication`
nudge is the natural, non-intrusive moment to remind a user that eligibility
conditions are worth a quick self-check, phrased as a single added sentence
in the nudge copy: "If your situation has changed since you last registered
for home voting, you can update your PwD marking (Form 8) or check current
eligibility criteria before submitting Form 12D."

### V4.6 EPIC vs. e-EPIC parity: direct discoverability for returning voters

**The gap.** `terminal_eepic_download` already exists in the MVP tree and is
already legally accurate (e-EPIC has been available since 25 Jan 2021, is
OTP-verified, and is legally equivalent to the physical card). But per
Section 10's cluster-12 coverage table, it is reached via
`registered_action.lost_epic` → `lost_or_damaged_epic` — i.e., the only
documented path to it is framed around having *lost or damaged* a physical
card. A large population of existing voters have a perfectly intact physical
EPIC, have never lost anything, and simply don't know a digital e-EPIC
exists as an option at all — for them, the "lost/damaged EPIC" framing at
`registered_action` never gets clicked, because nothing is lost or damaged,
so they never reach `terminal_eepic_download`.

**v3 fix — routing, not new content.** No new terminal or node is required;
`terminal_eepic_download`'s existing content stands unchanged. What changes
is discoverability:

1. The home-screen quick-link "Download your e-EPIC" specified in V4.2
   deep-links directly to `terminal_eepic_download`, independent of the
   `lost_or_damaged_epic` question node — a user who clicks it never sees or
   needs to answer "is your EPIC lost or damaged," since that framing is
   simply inapplicable to them.
2. `registered_action`'s option list (or its immediate follow-on framing)
   should be reviewed so the "lost/damaged EPIC" option's label is not the
   *only* surfaced path to e-EPIC-adjacent content — e.g., relabeling that
   option's helpText or adding a short one-line note near it: "Have your
   physical EPIC but want a digital copy too? [Download your e-EPIC]
   directly — you don't need to report anything as lost." This keeps the
   existing `lost_or_damaged_epic` node's own framing intact for users who
   genuinely lost their card, while giving everyone else a way in that
   doesn't require self-identifying into a "something is wrong" framing
   that doesn't match their actual situation.
3. This is explicitly flagged as a discoverability/routing fix, not a legal-
   content change — `terminal_eepic_download`'s citations (`e-epic` KB
   entry) and copy require no edit.

### V4.7 Copywriting/IA guideline: same engine, different framing

**The principle.** The decision engine (`core-domain`), the tree content, the
citations, and the terminals are identical regardless of who is asking or
how they arrived. What changes for a returning/logged-in user (has a saved
draft or account, per V6) or a user who clicked the "Check on your existing
registration" home-screen entry (V4.2) is **wrapper copy tone only** — a
presentational layer, never a logic branch, never a different set of nodes
or citations.

| Context | New-voter framing (default, anonymous/first-time) | Existing-voter framing (returning entry point or logged-in with a saved draft) |
|---|---|---|
| Home-screen headline | "Let's figure out what you need to do" | "Let's check on what you've already got" |
| Primary CTA | "Get started" | "Check my registration" |
| Question-screen lead-in | "Answer a few questions to get your personalized checklist" | "A few quick checks based on what you told us" |
| Resuming a session | (n/a — no prior session) | "Welcome back — pick up where you left off, or start something new" |
| Terminal-outcome framing | "Here's what you need to do next" | "Here's what to double-check" |

**Implementation rule.** This is implemented the same way the Translation
Management page (PRD v2 admin page 5) already handles FTL plural/gender
variants: a small, fixed set of copy-variant keys (e.g.,
`home.headline.new` vs. `home.headline.returning`) selected by a single
presentational flag threaded through page rendering — never a second copy
of any question, option, citation, or terminal body text. The flag itself is
derived from **self-identification** (which home-screen button was clicked,
or whether a saved draft/account exists), never from any claim about a
user's actual verified ECI registration status, which VoteAssist has no way
to know. The copy must not imply otherwise — e.g., "Let's check on what
you've already got" is honest (it describes the product's posture), while a
hypothetical "Welcome back, registered voter" would overclaim knowledge the
product does not have.

**Explicit non-goal:** no persona-detection heuristic, no assumption based
on age/state/device, and no dark-pattern nudging toward account creation to
"unlock" the existing-voter framing — an anonymous user who simply clicks
"Check on your existing registration" gets the existing-voter copy tone
immediately, with zero account requirement, exactly matching V6.1's test
that account features must never gate or degrade the anonymous path.

---

## V5. Deep End-to-End UX & Information Flow v3

### V5.1 Minimalism & clarity principles

These are deliberately specific and checkable against actual node copy in
PRD v2 Section 10 — not generic UX platitudes. Each rule below states the
rule, why it matters for this specific product, and one concrete before/
after example against an existing node.

**Rule 1 — Every screen answers exactly one question.**
A screen that asks two things at once forces a user to hold two decisions in
mind before they can act on either. This product's own design already gets
this right in most places (Section 10's `moved_service_voter_check` is a
single yes/no about service-voter status; `moved_same_or_diff_ac` is a
single question about AC boundary) — this rule exists to hold future
additions to the same standard.
- *Before (hypothetical anti-pattern this rule forbids):* a single combined
  screen asking "Is this move because of an Armed Forces/notified-service
  posting, AND is your new address in the same Assembly Constituency as your
  old one?" as one compound question with four combined answer options
  (yes/yes, yes/no, no/yes, no/no).
- *After (what Section 10 actually specifies, and what this rule requires
  future additions to match):* two sequential single-question nodes,
  `moved_service_voter_check` then `moved_same_or_diff_ac`, each with a
  binary/simple option set, each independently answerable without holding
  the other question in mind.

**Rule 2 — Never show a form field before explaining why it's needed.**
The explanatory `helpText` must render *before* the input control it
clarifies, not after or beside it as an afterthought.
- *Before (an ordering bug this rule forbids, even though the spec text
  happens to list `helpText` after the options in Section 10.10's document
  layout):* the `documents_checklist` multi-select renders its eight
  document-type checkboxes first, with the reassurance "You do not need all
  of these — this just helps us give you the most relevant guidance" only
  visible if the user scrolls past the checklist or notices small print
  beneath it.
- *After:* the reassurance sentence renders as the first thing on the
  screen, above the checkbox list, so a user never sees "select which of
  these 8 things you have" as an unexplained gate before understanding that
  partial selection (or none) is fine.

**Rule 3 — The shortest correct path is always the default path; optional
detours are opt-in expansions, never forced reading.**
Clarifying detail that most users don't need to complete the flow correctly
belongs behind an explicit, reversible expansion control, not inline forced
text on the primary path.
- *Before:* `moved_same_or_diff_ac`'s full extended helpText — "Moving to a
  different state always means a different Assembly Constituency, but
  moving within the same state or even the same city can also mean a
  different AC — and the legal process (Form 8) is identical either way.
  There is no separate 'interstate' form or procedure" — rendered as
  mandatory paragraph text every user must read before answering, even users
  who already understand this and just want to answer "same" or
  "different."
- *After:* the question itself stays terse ("Is your new address in the
  same Assembly Constituency, or a different one?") with a small "Why does
  this matter? / Not sure?" inline expansion control that reveals the fuller
  explanation only for users who click it — the shortest correct path (read
  question, pick same/different, proceed) never requires reading the
  clarifying paragraph, but it remains one click away for anyone who needs
  it, satisfying the "opt-in expansion, never forced reading" standard.

**Rule 4 — A returning user's saved context shortens the flow, never
lengthens it.**
Anything the product already knows about a user's current session (a saved
draft's prior answers, an account's `notification_state_interest`, the
home-screen entry point clicked per V4.2) must be used to skip redundant
questions, never merely displayed alongside a question that gets asked
again anyway.
- *Before (the anti-pattern this rule forbids):* a user resumes a saved
  draft that already answered `residence_type` with `tribal_remote` in a
  prior session; the resumed flow re-presents `residence_type` from
  scratch, with no pre-fill or skip, requiring the user to re-answer a
  question they already answered.
- *After (matching V6.3's specified replay behavior):* the resumed session
  replays every prior answer against the current tree version automatically
  and lands the user on the first node their prior answers don't already
  resolve — `residence_type` is never re-shown if it was already answered
  and the tree version hasn't restructured that node. The same principle
  applies to the home-screen entry points in V4.2: clicking "Check on your
  existing registration" must never re-ask the generic "are you already
  registered" gate question the button itself already answered.

**Rule 5 — A terminal outcome states one action, not a menu of contingent
actions.**
Where a terminal's guidance genuinely depends on a condition, that condition
belongs in an upstream question node, not folded into the terminal as an
"it depends" list.
- *Before (the anti-pattern):* a hypothetical terminal reading "Depending on
  your situation, you may need to (a) search the roll, (b) file Form 8, (c)
  contact your BLO, or (d) do nothing — see below for which applies to you,"
  followed by a nested conditional explanation the user has to self-apply.
- *After (what Section 10 already does correctly, and what this rule holds
  future terminals to):* `terminal_polling_station_changed` states exactly
  one action plainly — "search the electoral roll using your EPIC number to
  see your current polling station" — with the one genuine contingency
  ("if you find your address or constituency actually did change, treat
  that as the 'I moved' case instead") stated as a single clear fallback
  sentence, not a branching menu the user has to parse.

**Rule 6 — Never ask a question whose disambiguation the system could
already infer from context already given.**
A new question node is justified only when the system genuinely cannot
resolve the next step from information already collected in this session (or
already known from a saved account/draft, per Rule 4).
- *Before (the anti-pattern):* asking a user who arrived via the V4.2
  "Check on your existing registration" entry point a redundant "are you
  already a registered voter?" gate question when that fact is exactly what
  the entry point itself established.
- *After:* `duplicate_or_deleted_check` is a good existing example of a
  *justified* new question, because "duplicate," "someone else's entry,"
  and "my own entry was deleted" are genuinely three different real-world
  situations the system cannot infer from the single fact that the user
  clicked "object/delete" — this rule is a check against adding *un*justified
  disambiguation, not against disambiguation generally.

**Rule 7 — Never introduce an acronym or form number before it's defined in
that screen's own context.**
"Form 8," "AC," "EPIC," "SSR," and similar terms must be spelled out or
briefly glossed on first use per screen, even if a glossary page exists
elsewhere (per PRD v2 Section 23's Appendix glossary) — a user should never
have to leave the flow to look up a term used in a question they're being
asked to answer right now.
- *Before:* a hypothetical nudge (V4.3) reading "Your SSR window is open —
  check your details" with no expansion of what SSR means for a first-time
  reader.
- *After:* "It's Special Summary Revision time in [state] — a periodic
  window when the electoral roll is updated and corrections get special
  attention. It's a good time to check your registration details are still
  correct," spelling the term out inline rather than assuming prior
  familiarity.

### V5.2 "Deeply coupled and self-improving," made concrete

Ground-truth v3 uses "deeply coupled and self-improving" as a design
description; this subsection converts it into specific UI/data requirements
on the **existing** admin pages from PRD v2 Section 11 — no new admin page is
introduced by this subsection (the fringe-case registry in V5.3 is the one
genuinely new admin surface, and even that is argued as a tab on an existing
page, not a new one).

**Interpretation: "deeply coupled" = high internal cohesion.** Every module
cross-references and stays visibly consistent with every other module's
state, surfaced inline at the point of use, rather than requiring a content
editor to manually cross-reference a separate dashboard. Concretely:

1. **KB Content Editor (PRD v2 admin page 3) → Decision Tree Visual Editor
   (admin page 4), citation freshness.** Today, admin page 4's node
   inspector panel (Section 11.4) shows a node's citations as links to
   `knowledge_entries`, and admin page 3 separately tracks `review_status`
   and `last_verified_date` per entry. v3 requires: **every node on the
   Decision Tree Visual Editor's canvas that cites a KB entry must render an
   inline freshness badge on the node itself** (not only inside the node
   inspector side panel, which requires a click to see) — green if the cited
   entry is `verified` and within the re-verification age threshold, amber if
   `verified` but approaching the threshold (same threshold value as PRD v2
   admin page 2's Dashboard tile), red if `needs_reverification`. This badge
   must update the moment a `reviewer`/`legal_reviewer` changes the cited
   entry's status or `last_verified_date` on admin page 3 — a tree editor
   opening the canvas the next day should see the change reflected without
   any manual sync step, because both pages read the same live
   `knowledge_entries.review_status`/`last_verified_date` columns, not a
   cached snapshot taken at some earlier point.
2. **Fringe-case registry (V5.3) → both KB Content Editor and Decision Tree
   Visual Editor.** Any `fringe_case` row with a non-null
   `linked_tree_node_id` must render as a badge directly on that node in the
   canvas (a small counter icon, e.g. "2 open reports"), linking through to
   the filtered fringe-case list scoped to that node. Any row with a non-
   null `linked_kb_entry_id` must render equivalently on that entry's detail
   view in admin page 3. This is the concrete meaning of "a fringe-case
   report should link directly to the specific tree node/KB entry it
   concerns" — the link is bidirectional and visible from both ends, not a
   one-way pointer buried in the fringe-case row itself.
3. **Analytics (PRD v2 admin page 12) → Decision Tree Visual Editor.**
   Today, drop-off analytics live entirely on admin page 12, keyed by
   decision-tree version id and node id, requiring a content editor to open
   a separate page, find the relevant node's row in a ranked list, and
   mentally map it back to the node they're editing on admin page 4. v3
   requires: **the node inspector panel on admin page 4 must show that
   specific node's current-period drop-off rate and trend inline**, sourced
   from the same rollup tables admin page 12 reads (Section 11.12's hourly/
   daily rollups), scoped to the tree version currently open in the editor.
   This is a read-only inline display, not a new analytics computation —
   the requirement is inline surfacing at the point of editing, not a new
   metric.
4. **Reverse citation index, KB → tree.** Admin page 3's entry detail view
   must show a "Cited by these decision-tree nodes" panel — a reverse lookup
   from `knowledge_entries.id` to every `decision_trees` node (across every
   tree, current draft and currently-published version) whose `citations`
   field references that entry. This is the concrete meaning of "a KB
   entry's `lastVerifiedDate` change should be visible from the decision-
   tree editor if that entry is cited by a tree node" stated from the other
   direction: a contributor editing a KB entry should immediately see which
   tree nodes depend on it, so they understand the blast radius of a content
   change before saving, not after a tree editor separately discovers a
   citation broke.

**Concrete data requirement underlying all four items above:** a
lightweight, always-fresh reverse index from `knowledge_entries.id` to
citing tree nodes (recomputed whenever a tree draft is saved, since
citations only change on tree edits) and from tree node id to open
`fringe_case` rows (a simple indexed query on `fringe_case.linked_tree_node_id`,
no materialization needed given expected low-thousands scale). Neither
requires a new service or a new page — both are inline query/render
additions to admin pages 3 and 4 exactly as specified above.

**"Tree health" as the aggregate view of the same coupling.** Section V5.4
below specifies the "tree health" report itself; the point made here is
narrower and purely about *where it must surface*: the report's output must
appear as a tile on the existing Dashboard (admin page 2), linking through to
the existing Analytics Dashboard (admin page 12) for the full ranked list —
it is not a new standalone page, and its per-node data is the same data
surfaced inline per node on admin page 4 per item 3 above. There is exactly
one source of truth for "this node's drop-off rate," read from three places
(Dashboard tile, Analytics Dashboard, Tree Editor node inspector), not three
independently computed numbers that could drift apart.

### V5.3 Fringe-case registry

**Why a registry, distinct from ordinary feedback (PRD v2 admin page 7).**
Ordinary user feedback (bug reports, "was this helpful," general comments)
is triaged to `resolved`/`wontfix` and, once closed, has served its purpose —
it is not designed to track whether the *underlying tree/KB coverage gap* it
revealed has actually been addressed. A fringe case is specifically about
**decision-tree coverage** — "the tree doesn't yet handle X" — and needs a
lifecycle that persists past a single feedback item's closure, because the
same gap might be reported by three different users over six months before
anyone gets around to fixing it, and without a registry, that pattern is
invisible (three separate `feedback` rows, each independently marked
`resolved`, with no connective tissue showing they're the same underlying
gap).

**Schema.**

```sql
CREATE TYPE fringe_case_reporter_channel AS ENUM (
    'web_feedback', 'telegram_bot', 'whatsapp_bot', 'ivr', 'manual_observation', 'admin_escalated_from_feedback'
);

CREATE TYPE fringe_case_status AS ENUM (
    'new', 'triaged', 'added_to_tree', 'wontfix', 'needs_legal_review'
);

CREATE TABLE fringe_case (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    description TEXT NOT NULL,
    reporter_channel fringe_case_reporter_channel NOT NULL,
    linked_feedback_id UUID REFERENCES feedback(id), -- nullable: not every fringe case originates from a feedback row (e.g. manual_observation)
    status fringe_case_status NOT NULL DEFAULT 'new',
    linked_kb_entry_id UUID REFERENCES knowledge_entries(id), -- nullable until triage identifies a specific entry
    linked_tree_node_id TEXT, -- nullable; node id within decision_trees content, not a separate FK table (node ids are content-scoped, not globally unique rows)
    resolution_note TEXT, -- required (enforced at the application/API layer, not a DB CHECK, since it depends on status) once status moves to added_to_tree, wontfix, or needs_legal_review
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ -- set when status first reaches a terminal value (added_to_tree, wontfix); needs_legal_review is not terminal, see workflow below
);

CREATE INDEX idx_fringe_case_status ON fringe_case (status);
CREATE INDEX idx_fringe_case_linked_tree_node ON fringe_case (linked_tree_node_id) WHERE linked_tree_node_id IS NOT NULL;
CREATE INDEX idx_fringe_case_linked_kb_entry ON fringe_case (linked_kb_entry_id) WHERE linked_kb_entry_id IS NOT NULL;
```

**Where it lives in the admin app — argued explicitly, not asserted.** This
belongs as a **new tab on the existing Feedback & Grievance Triage page**
(PRD v2 admin page 7), not a standalone new page, for three concrete
reasons: (1) the overwhelming majority of fringe cases originate as an
ordinary feedback submission that a triager recognizes as "this isn't a bug
report, it's a coverage gap" — keeping both on one page means the triager
never has to leave their working context to escalate what they're already
looking at; (2) the RBAC audience is identical (`reviewer`, `legal_reviewer`,
`superadmin` — the same roles who already have access to admin page 7, per
Section 11.7, and no role needs fringe-case access without also having
feedback-triage access); (3) introducing a new top-level nav item for a
feature whose primary input channel is "escalated from feedback" adds
navigation overhead without adding capability a tab can't provide. The one
place this page's design must differ from a plain feedback list is its
status vocabulary and linking fields, which the tab's own dedicated list
view (filterable by `status`, `reporter_channel`, and "has linked tree
node"/"has linked KB entry") reflects, distinct from admin page 7's own
`new`/`triaged`/`resolved`/`wontfix` feedback-status filters.

**Admin UI on the tab.**
- **List view**: one row per `fringe_case`, columns: description (truncated),
  reporter channel, status badge, linked KB entry (if any, clickable through
  to admin page 3), linked tree node (if any, clickable through to admin
  page 4's canvas centered on that node), created-at, resolved-at (if
  applicable). Filters: status, reporter channel, has-linked-entity
  (yes/no), date range.
- **Escalate-from-feedback action**, on admin page 7's ordinary feedback
  inbox (Section 11.7): a triager reviewing a feedback item that reveals a
  coverage gap clicks "Escalate as fringe case," which opens a small form
  pre-filled with the feedback's message as a starting `description` (editable
  before saving — the fringe case's description should be the triager's own
  restatement of the gap, not a verbatim copy of user-submitted text, since
  the user's original phrasing may be unclear or specific to their situation
  in ways a generalized coverage-gap description shouldn't be), sets
  `reporter_channel = 'admin_escalated_from_feedback'`,
  `linked_feedback_id` to the source row, and `status = 'new'`. This does
  not change the underlying `feedback` row's own status — a feedback item
  can be marked `resolved` on admin page 7 (its own lifecycle: was the
  submitter's immediate concern acknowledged) independently of whether the
  fringe case it spawned is still open (its own lifecycle: has the
  underlying tree/KB gap actually been fixed) — these are deliberately
  decoupled statuses tracking two different questions.
- **Detail/triage view**: `new` → `triaged` (any `reviewer`+; sets
  `linked_kb_entry_id`/`linked_tree_node_id` if a specific entity is
  identified, or leaves both null if the gap is broader than one entity);
  `triaged` → `added_to_tree` (requires `resolution_note` describing what
  changed and, where practical, the specific tree version number or KB
  revision id the fix shipped in — a `legal_reviewer`+ action if the
  resolution involves a legal/procedural content change per the same sign-
  off gate as admin page 3/4's `verified`/publish actions, since a fringe-
  case resolution that edits a KB entry or tree node goes through those
  pages' own normal state machines; this status change on the fringe-case
  row itself is a record of that having happened, not a separate content-
  editing action); `triaged` → `wontfix` (any `reviewer`+, requires
  `resolution_note` explaining why — e.g., "out of scope, adjacent to
  candidate-data territory the project explicitly avoids," mirroring the
  reasoning already used elsewhere in this PRD for scoped-out adjacent
  domains); `triaged` → `needs_legal_review` (routes the item into
  `legal_reviewer`'s queue specifically — used for exactly the kind of gap
  PRD v2 Section 10 already flags as "NEW KB ENTRY NEEDED," see below).

**Workflow, end to end:** raw feedback or a direct support-channel report
(bot free-text fallback, a manually observed pattern, a support-channel
transcript) → triager reviews it on admin page 7 → recognizes a coverage gap
→ escalates to a `fringe_case` row (`status = new`) → a `reviewer`+ triages
it, linking the relevant KB entry/tree node if identifiable (`status =
triaged`) → either (a) the team makes the KB/tree change through the normal
admin page 3/4 pipelines and the fringe case is marked `added_to_tree` with a
`resolution_note` citing the resulting revision/version, (b) the team
decides not to act and marks it `wontfix` with a `resolution_note` explaining
why, or (c) the gap requires legal/content research before any change can be
made, and it's marked `needs_legal_review`, sitting in that state until a
`legal_reviewer` either resolves the research question (moving it to
`triaged` again with findings, then on to `added_to_tree`/`wontfix`) or
determines it should stay flagged long-term as a known, honestly-labeled
limitation.

**Retroactive seeding.** PRD v2 Section 10 already contains four items that
are, in substance, exactly what this registry is designed to track, having
been discovered through the PRD-authoring process itself rather than through
live user feedback: the tribal/remote/urban-slum accepted-document research
gap (Section 10.7's "NEW KB ENTRY NEEDED"), the transgender gender-marker KB
research gap (Section 10.9's "NEW KB ENTRY NEEDED"), the wrongful-deletion-
contest instrument question (Section 10.12's `terminal_deleted_entry_contest`
caution), and the retired-service-voter transition open question (Section
10.5's extension to `terminal_service_voter`). When this registry ships,
these four should be backfilled as pre-seeded `fringe_case` rows at
`needs_legal_review` (the first three) or `triaged` (the fourth, since it's
an open question rather than a missing KB entry per se) status, with
`reporter_channel = 'manual_observation'` and `linked_tree_node_id`/
`linked_kb_entry_id` set per the relevant cluster — rather than leaving them
as inline PRD prose that a future team has to remember to re-discover.

### V5.4 Self-improving system, in full

**Restated up front, and true of both mechanisms below without exception:
neither mechanism ever auto-publishes a content or tree change. Both only
ever produce a flagged item for a human (`contributor`, `reviewer`, or
`legal_reviewer`, per PRD v2's RBAC) to act on through the existing,
unchanged admin pipelines in Section 11.** Nothing in this subsection
introduces a code path that edits `knowledge_entries` content, promotes a
`review_status`, or publishes a `decision_trees` version without a human
clicking the corresponding button on admin page 3 or 4 exactly as already
specified there.

#### Mechanism 1: event-driven change-detection job

**New `crates/jobs` task: `citation-change-detector`** (apalis-cron,
alongside the existing nightly dead-citation-link checker and the 180-day
age-based re-verification digest from Section 6.8/11.2 — this job
*supplements*, does not replace, either of those).

- **What it watches.** The same bounded, curated set of URLs already tracked
  in `link_check_results` — the cited sources actually attached to
  `verified`/`in_review` `knowledge_entries` in the live product (on the
  order of a few hundred distinct URLs given expected KB size, not an open-
  ended crawl target). No new URL discovery, no following of links found on
  a watched page, no expansion of scope beyond what a human contributor has
  already curated as a citation.
- **How it detects change without being an aggressive crawler.**
  - Fetches each watched URL at most **once per day**, off-peak (e.g., a
    fixed early-morning IST window), sequentially with a deliberate delay
    between requests rather than a parallel burst — the same courteous-
    crawling posture already required of the "recheck all broken" bulk
    action on admin page 6 (Section 11.6's guardrail against the tool
    itself looking like an abusive crawler).
  - Computes a content hash over a **normalized text extraction** of the
    fetched page (stripped of markup, scripts, and known-volatile elements
    such as session tokens, ad-injection fragments, or timestamps embedded
    in page chrome), not a raw-HTML hash — a raw-HTML hash would produce
    false-positive "changed" flags on every fetch due to incidental
    markup/whitespace noise unrelated to the cited content itself.
  - Compares the new hash against the most recently stored hash for that URL
    (an added `content_hash` + `content_hash_checked_at` column pair on
    `link_check_results`, or a small companion `content_hash_history` table
    keyed by URL if retaining more than the single most-recent hash proves
    useful for later diffing — an implementation detail left to the team,
    not load-bearing for the product behavior specified here).
  - Sends an explicit, identifying User-Agent string on every request (e.g.
    `VoteAssistIndiaBot/1.0 (+https://voteassist.example/bot-policy)`)
    linking to a public page describing the bot's purpose, its modest daily-
    scale (not real-time) fetch frequency, and a contact address for any
    site operator who wants to request exclusion.
  - Respects `robots.txt` for every watched domain, checked and cached per
    domain rather than re-fetched on every run.
- **What "flags for re-verification" concretely does downstream.** If a
  watched URL's content hash has changed since the last check, the job
  immediately transitions every `knowledge_entries` row that cites that URL
  to `review_status = needs_reverification` — reusing the exact existing
  state-machine transition already specified in Section 11.3 ("any status →
  `needs_reverification`: any `reviewer`, `legal_reviewer`, or `superadmin`
  can flag an entry at any time"), just triggered by the system instead of a
  human noticing. This means: (a) it appears on the Dashboard's existing
  "KB entries needing re-verification" tile (admin page 2) exactly as a
  manually-flagged entry would, no new UI element required there; (b) it
  writes a `knowledge_entry_revisions` row and an `audit_log` row exactly as
  a human-triggered flag would, with the actor recorded distinctly (e.g.
  `actor_admin_user_id = NULL`, a new `actor_system` column or a reserved
  system-actor row identifying `citation-change-detector` specifically) so a
  reviewer opening the entry sees *why* it's flagged ("flagged automatically:
  content change detected at [URL] on [date]") rather than an unexplained
  status change; (c) it does not touch the tree — a KB entry's status change
  alone does not retract a live tree citation, since Section 11.4 already
  establishes that only `verified` KB entries are citable and that saving an
  edit to a `verified` entry demotes it to `in_review` automatically; the
  same demotion-on-change discipline applies here, just triggered by an
  external content change instead of an internal edit.
- **Relationship to the existing 180-day age-based job, stated explicitly
  per this addendum:** this event-driven job *tightens* the existing
  re-verification loop; it does not replace the age-based backstop. An entry
  whose cited source never changes at the byte level but has simply aged
  past the configured threshold (Section 11.2/11.14's open threshold
  question) still gets flagged by the existing nightly age-based digest,
  independent of whether `citation-change-detector` ever fires for it.

#### Mechanism 2: analytics-informed "tree health" report

**Output format — a ranked list, not a dashboard of unrelated charts:**

| Column | Source |
|---|---|
| `node_id` | `decision_trees` content (current published version) |
| `node_type` | question / terminal, from tree content |
| `drop_off_rate` (current period) | Existing analytics rollup tables (Section 11.12), same aggregation already computing "top drop-off nodes" |
| `trend_vs_prior_period` | Current-period rate minus prior-period rate for the same node, same time-range granularity as the Analytics Dashboard's existing time-range picker |
| `open_fringe_case_count` | `COUNT(*) FROM fringe_case WHERE linked_tree_node_id = node_id AND status NOT IN ('added_to_tree', 'wontfix')` |
| `feedback_mentions` | `COUNT(*) FROM feedback WHERE linked_decision_session id's terminating node = node_id` (or, more simply, feedback rows with an explicit link to this node/entry captured at submission time, per Section 11.7's existing "linked KB entry or decision-session id if the feedback was submitted in-context" field) — deliberately restricted to feedback with an explicit link, not a free-text/NLP correlation, since this PRD does not specify any NLP capability elsewhere and should not silently introduce one here |

Sorted descending by `drop_off_rate` by default (with a secondary sort
option by `open_fringe_case_count` for a "what's both leaking users and
actively reported broken" view). This is computed by a `crates/jobs` rollup
(alongside the existing hourly/daily analytics rollups) into a small
**[NEW TABLE]** `tree_health_rollup` (`tree_version_id`, `node_id`,
`period_start`, `period_end`, `drop_off_rate`, `prior_period_drop_off_rate`,
`open_fringe_case_count`, `feedback_mentions`, computed on the same schedule
as the existing hourly/daily analytics rollups) rather than computed live on
every page load, consistent with the Dashboard's existing performance
guardrail (Section 11.2, "served from rollup tables... rather than computed
live").

**Where it surfaces, per V5.2's coupling requirement:**
- **Dashboard (admin page 2):** a new tile, "Tree health — top nodes needing
  attention," showing the top 3-5 rows by `drop_off_rate`, each linking
  through to the node's detail view.
- **Analytics Dashboard (admin page 12):** the full ranked list, filterable/
  sortable exactly as any other table on that page, alongside the existing
  "top drop-off nodes" visualization it already specifies — this is the
  same underlying metric, extended with the fringe-case/feedback columns
  the existing page doesn't yet join against.
- **Decision Tree Visual Editor (admin page 4):** per V5.2 item 3, the
  specific node's drop-off rate and trend render inline in the node
  inspector panel when that node is selected — a content editor never has
  to separately open the Analytics Dashboard to see whether the node
  they're currently editing is a known problem area.

**The system never edits its own content; it gets better at telling humans
where to look** — this is the entire scope of mechanism 2. No automatic
action follows from a node appearing at the top of the tree-health ranking
beyond it being visible, prioritized, and linked to whatever fringe cases or
feedback already exist about it; a `contributor`/`reviewer`/`legal_reviewer`
still does the actual work of deciding what to change and executing that
change through admin pages 3/4's normal editing and publish pipelines.

### V5.5 End-to-end journey maps, generalized (v3 additions)

Same format as `docs/03-user-journeys.md` and PRD v2 Section 10's persona
treatment: stages, touchpoints, emotional valence, pain points, VoteAssist's
role at each stage, and deep-links. These four are new for v3 and
specifically combine dimensions introduced in this addendum (existing-voter
proactive nudges, jurisdiction routing, the fringe-case loop, and — noted
but not duplicated — Bhashini-assisted translation from Section V2).

---

#### Journey 9: Returning voter gets a proactive SSR nudge and resumes a saved draft (Ramesh)

| Stage | Touchpoint | Emotion | Pain point | VoteAssist role | Deep-link |
|---|---|---|---|---|---|
| Prior context | Used VoteAssist eight months ago to shift his registration after moving; created an account to save the draft and opted into reminders for his state | Neutral (recalled from memory) | — | Account/draft exists per V6; `notification_state_interest` set to his state | — |
| Trigger | An SSR window opens for his state; admin has entered the `ssr_windows` row | Unaware (has not opened the app) | Wouldn't otherwise know an SSR is happening or that it's relevant to him | `proactive-nudge-dispatcher` job (V4.3) matches his account, fires a WhatsApp nudge using a Meta-approved template | WhatsApp (opt-in channel) |
| Notification | Receives: "It's Special Summary Revision time in [state] — a good time to check your registration details are still correct" | Mildly curious | Might dismiss as noise if it felt generic | One-click deep link straight into `terminal_roll_search`, pre-filled with his last-known EPIC number from his saved draft | Deep link into `crates/web-app` |
| Resume | Clicks through; the app also surfaces "Welcome back — pick up your saved draft?" per V4.7's returning-user framing | Relieved | Doesn't want to re-enter everything from scratch | Draft replay per V6.3: prior answers auto-applied, landing him only on any node the tree version has since changed | — |
| Check | Confirms his roll entry still shows the correct address and AC | Confident | — | `terminal_roll_search` outcome, existing-voter copy tone ("here's what to double-check," per V4.7) | voters.eci.gov.in (roll search) |
| Outcome | No action needed this cycle; declines to open a new session | Satisfied | — | Nudge is not repeated for this same `ssr_windows` row (enforced by `proactive_nudges_sent`'s unique constraint) | — |

---

#### Journey 10: First-time Panchayat-election question, correctly jurisdiction-routed (Lakshmi)

| Stage | Touchpoint | Emotion | Pain point | VoteAssist role | Deep-link |
|---|---|---|---|---|---|
| Trigger | Hears her village's Gram Panchayat election is coming up and wants to check her registration | Curious | Assumes this works the same way as checking her Lok Sabha registration | Home screen's separate, clearly-labeled Panchayat/Municipal entry point (V3.4) sits alongside, not folded into, the two main entry buttons from V4.2 | — |
| Orientation | Clicks the local-body entry point; asked only for her state | Neutral | Doesn't know Panchayat elections aren't run by ECI at all | Explains the ECI/SEC jurisdiction split plainly (V3.2) before asking anything else, so she understands why a *different* portal is coming | — |
| Routing | Told her state's SEC administers this, given the SEC's portal link | Slightly surprised, then informed | Worried she's being sent somewhere illegitimate | Jurisdiction-routing terminal (V3.4) states this is the correct, official body for local-body elections, not a workaround | State SEC portal (state-specific) |
| Roll-continuity check | Asks whether her ordinary ECI roll entry is the same one used for the Panchayat election | Uncertain | Genuinely varies by state (per V3.2/V3.3's `roll_derivation_note`) | If her state's `state_election_commissions.roll_derivation_note` is verified, states it plainly; if not yet verified, says so honestly rather than guessing | — |
| Action | Follows the SEC portal link to check her local-body roll status | Determined | SEC portal may be less digitized than voters.eci.gov.in | Sets expectations honestly per V3.5 rather than assuming ECI-level portal maturity | State SEC portal |
| Confirmation | Confirms she's on the correct local roll | Relieved | — | Optional feedback prompt, same as any other terminal | — |

---

#### Journey 11: A fringe case flows from feedback through triage to a shipped content fix (single timeline)

| Stage | Touchpoint | Emotion | Pain point | VoteAssist role | Deep-link |
|---|---|---|---|---|---|
| User hits the gap | A citizen whose EPIC was deleted during a roll-cleanup exercise tries the tree and reaches `terminal_deleted_entry_contest`'s honest "contact your BLO/ERO directly" outcome — the exact open question flagged in Section 10.12 | Frustrated | Wanted a specific procedure, got an honest "we don't have verified guidance yet" | The terminal is honest rather than fabricated, but logs the gap as a real product limitation (Section 10.17, open question #1) | 1950 helpline (fallback) |
| Feedback submitted | Submits feedback via the web app's in-context "was this helpful?" prompt attached to that terminal, describing the specific gap | Still frustrated but engaged | No submitter-facing reply channel exists (by design, per admin page 7) | Feedback lands in `feedback` with `linked_kb_entry_id`/decision-session context intact | Admin: Feedback & Grievance Triage inbox |
| Triage | A `reviewer` reads the feedback, recognizes it matches the already-known open question rather than a one-off | Neutral (routine triage work) | Without a registry, this could be marked `resolved` and forgotten as an isolated case | Clicks "Escalate as fringe case" (V5.3); since a fringe case for this exact gap was already pre-seeded at ship-time (V5.3's retroactive-seeding recommendation), the reviewer links this new feedback item to the *existing* `fringe_case` row via `linked_feedback_id` rather than creating a duplicate | Admin: Feedback & Grievance Triage, Fringe-Case tab |
| Legal research | The fringe case is already `needs_legal_review`; a `legal_reviewer` researches whether Form 7, a distinct claims/appeal mechanism, or Form 6 re-registration is the correct instrument, per Section 10.17's open question #1 | Focused (research work) | Requires actually contacting ECI/CEO offices or reviewing current Gazette notifications, not guesswork | Legal reviewer resolves the open question with a verified answer and a citable source | — |
| Content fix ships | A `contributor` drafts an updated `terminal_deleted_entry_contest` outcome with the now-verified procedure and a real citation; `reviewer` advances it; `legal_reviewer` promotes it to `verified` and publishes the tree version containing the updated terminal | Satisfied (team) | Must pass the same citation-completeness/`proptest` gates as any other tree content (Section 10.0) | Admin pages 3 (KB edit) and 4 (tree publish), normal pipelines, unchanged | — |
| Fringe case closed | The fringe case is moved to `added_to_tree` with a `resolution_note` citing the new tree version number and KB revision id | Resolved | — | Closes the loop visibly — the node's badge on admin page 4's canvas (V5.2 item 2) drops to zero open reports | — |
| Future users benefit | A new user reaching the same terminal now gets the verified procedure instead of "contact your BLO directly" | Relieved | — | The specific gap that frustrated the original reporter no longer exists for anyone reaching that path | voters.eci.gov.in / the now-correct form |

---

#### Journey 12: Bhashini-assisted draft translation, human-reviewed and approved (Manoj)

*(This journey ties to Section V2's Bhashini integration, specified separately; it is included here only in its end-to-end product-flow form, per this section's brief, and does not restate V2's technical integration detail.)*

| Stage | Touchpoint | Emotion | Pain point | VoteAssist role | Deep-link |
|---|---|---|---|---|---|
| Gap identified | A content contributor notices a KB entry has no Bengali translation yet, and Bengali is one of the 21 roadmap languages with active but incomplete coverage | Neutral (routine content-ops work) | Fully human-translating every entry is slow, and Bengali coverage is currently 0% for this specific entry per admin page 5's per-locale completeness dashboard | Translation Management page (admin page 5) shows the gap plainly, not as a misleadingly-styled 0% bar (Section 11.5) | Admin: Translation Management |
| MT-assist draft | Contributor clicks "MT draft" on the entry, which calls Bhashini's translation API (V2) | Curious whether the draft is usable | MT output for civic/legal terminology (e.g., "Form 8," "Assembly Constituency") can be inaccurate if used unreviewed | Draft populates the target field, visually flagged "MACHINE-TRANSLATED DRAFT — REQUIRES HUMAN REVIEW," not approvable in that state (Section 11.5 guardrail, unchanged by which MT provider is used) | — |
| Human review | A `translator` fluent in Bengali reviews the MT draft against the English source, correcting terminology and phrasing | Focused | Needs to catch subtle legal/procedural mistranslations MT alone would miss (e.g., a deadline, a form number, a conditional eligibility clause) | Side-by-side source/target workbench (Section 11.5); editing the text clears the "unreviewed MT draft" flag automatically | — |
| Sign-off | Because this is KB body text (not UI chrome), approval requires `legal_reviewer`, not just `reviewer` | — | The same legal/procedural-accuracy bar as the English/Hindi source content | `legal_reviewer` reviews and approves per Section 11.5's content-type-specific approval gate | — |
| Publish | The Bengali translation is now live for this entry; if this pushes Bengali's overall completeness across the exposure threshold, the language switcher in `crates/web-app` newly lists Bengali as available | Satisfied (team) | A half-translated language must never appear as "available" prematurely (Section 11.5 guardrail) | Public-site language switcher gated on the completeness threshold, unchanged mechanism | — |
| User benefit | Manoj, a Bengali-speaking first-time voter, now gets this specific answer in Bengali instead of falling back to Hindi/English | Relieved | Previously had to rely on a family member to read English/Hindi content aloud and translate informally | The specific answer he needed is now available in his own language, reviewed and legally sound, not a raw MT string | voters.eci.gov.in |
## V6. Optional Accounts, Saved Drafts, Deletion & Privacy Architecture

### V6.1 Reconciling this with the anonymous-by-default non-goal

PRD v2 (Section 4) makes anonymity the default and states that no account
is required for core guidance. That does not change here. What v3 adds is
a **strictly optional** account layer for users who want one specific
convenience: continuity — picking up a saved decision-tree session on a
different device, keeping a checklist to show a family member, or opting
in to a reminder before an election they care about. The design test for
every feature in this section is: **does the anonymous, no-account path
still work exactly as it did in PRD v2, with zero degradation?** If a
proposed account feature would make the anonymous path worse, slower, or
nudge users toward creating an account they don't need, it fails that test
and does not belong in this product. Accounts are additive, never a
gate.

### V6.2 What an account is, precisely

- **Signup**: email address or phone number + OTP verification. No
  Aadhaar, no EPIC number, no government ID of any kind is ever requested
  at signup — an account is a login credential, nothing else, deliberately
  decoupled from the citizen's actual electoral identity. (VoteAssist
  never needs to know if you're the same person as your EPIC record; it
  only needs to remember what you told the decision engine.)
- **What it stores** (and the exhaustive list of what it does NOT store):

| Stored | Never stored |
|---|---|
| Login credential (email/phone hash + OTP verification state) | Aadhaar number |
| Saved draft sessions (Section V6.3) | EPIC number |
| Notification preferences (opt-in only, default off) | Uploaded documents of any kind |
| Account creation/last-login timestamps | Caste, religion, political-party, or any sensitive-category data (never collected from anyone, account or not) |
| A user-chosen display label per draft (e.g., "my registration," "amma's address change") | Full name (a display label is user-chosen free text, not verified identity) |

- **Where it lives**: a new `user_accounts` table in the same Postgres
  instance, but logically and, in a v2-roadmap deployment, PHYSICALLY
  separated from `admin_users` (different schema/least-privilege DB role,
  since these are two entirely different trust boundaries — a compromised
  public-account credential store must never be a path to admin access,
  and vice versa).

```sql
CREATE TABLE user_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Store a salted hash of the contact identifier, not the plaintext
    -- email/phone, wherever the login flow allows it (OTP delivery still
    -- needs the real address transiently, but it should not be the primary
    -- key or appear in most queries in plaintext).
    contact_identifier_hash TEXT NOT NULL UNIQUE,
    contact_channel TEXT NOT NULL CHECK (contact_channel IN ('email', 'phone')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login_at TIMESTAMPTZ,
    notifications_opt_in BOOLEAN NOT NULL DEFAULT false,
    notification_state_interest UUID REFERENCES states(id) -- only if opted in
);

CREATE TABLE saved_drafts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES user_accounts(id) ON DELETE CASCADE,
    label TEXT, -- user-chosen, optional, free text, never validated against real identity
    decision_tree_version INT NOT NULL,
    answer_history JSONB NOT NULL, -- ordered list of {questionId, value}, same shape as core-domain's Answer[]
    -- If the session reached a terminal, freeze the result AS IT WAS SHOWN,
    -- so a later KB edit doesn't silently rewrite what the user saved.
    frozen_terminal_snapshot JSONB,
    frozen_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

`ON DELETE CASCADE` is deliberate and load-bearing: deleting an account
must delete every draft in the same transaction, not leave orphaned rows
a later export/audit could resurrect.

### V6.3 The draft/resume experience

A user answering questions in `crates/web-app` sees a low-friction, non-
blocking "Save my progress" affordance at any point after the first
answer — NOT a signup wall before they can start (that would violate
V6.1's test immediately). Clicking it prompts account creation/login only
at that moment, exactly like a "save for later" pattern in any well-
designed form, never earlier. Returning to a saved draft: (a) if the
underlying decision-tree version hasn't changed, resume exactly where they
left off; (b) if it has changed (a new tree version was published per PRD
v2 Section 11's tree-editor workflow), replay the saved answers against
the new tree where every prior answer's node id/value still exists, and if
any answer no longer maps cleanly (a node was restructured), stop at the
last still-valid point and tell the user plainly why ("this question has
changed since you last answered it, please continue from here") rather
than silently producing a possibly-wrong result from stale answers matched
against new content.

For a saved terminal outcome specifically: show the frozen snapshot by
default (what they saw when they saved it) with a visible, dated "Saved on
[date] — refresh to see the current guidance?" control, rather than either
(a) silently showing possibly-outdated content as if current, or (b)
silently overwriting their saved reference with new content they didn't
ask for. The user controls when their saved reference updates.

### V6.4 Deletion

- **Delete a single draft**: one action, immediate, irreversible in the
  live system (no soft-delete/undo window in the primary database — see
  the backup-retention note below for the one narrow exception).
- **Delete the whole account**: one action (with a single, clear
  confirmation step — not a multi-page "are you sure, are you REALLY
  sure" dark pattern, and not a "contact support to delete" friction
  pattern either), cascades to every draft, and additionally deauthorizes
  any active session tokens immediately.
- **Response-time commitment**: research for this PRD established
  that the DPDP Act itself gives data principals a 90-day statutory
  response window for such requests — but because VoteAssist's account
  deletion is a fully self-service, immediate database operation (not a
  request routed to a human process), the actual user experience is
  "instant," not "within 90 days." The privacy policy should say plainly:
  self-service deletion completes immediately; this significantly exceeds
  (in the user's favor) what the law requires, and that's a deliberate
  product commitment, not a legal minimum being marketed as more than it
  is.
- **Backup-retention exception, disclosed not hidden**: encrypted
  disaster-recovery backups may retain deleted data for a short, fixed
  window (recommend 30 days) purely for operational restore purposes,
  never queried or restored selectively for any purpose other than whole-
  system disaster recovery, and this fact is stated plainly in the privacy
  policy rather than glossed over — a "we deleted it everywhere instantly"
  claim that isn't quite true is worse for trust than an honest "deleted
  from the live system immediately; purged from encrypted backups within
  30 days" statement.

### V6.5 Privacy features — the account settings page

Borrowing the design discipline (not the technical integration) of India's
own DEPA/consent-manager model researched for this PRD — visible,
explicit, revocable consent, never a buried assumption — the account
settings page in `crates/web-app` shows, literally rendered back to the
user rather than only described in prose:

- Every field in the `user_accounts`/`saved_drafts` tables that pertains to
  them, in a genuinely readable format (not a raw JSON dump — a formatted
  view of "here is what we have: your saved drafts, when you created this
  account, your notification setting").
- A one-click **export** producing a downloadable JSON bundle of the exact
  same data, both because this is good practice regardless of whether
  DPDP mandates full portability (research for this PRD found DPDP does
  NOT include an explicit GDPR-style portability right — VoteAssist offers
  this anyway, as a deliberate trust-building commitment beyond the legal
  floor, and the privacy policy should say so honestly rather than imply
  it's purely a compliance requirement).
- A one-click **delete draft** and **delete account** action (V6.4).
- **Notification consent** as a single, specific, opt-in toggle ("remind me
  before elections in [state]") — default OFF, and turning it on requires
  picking the specific state of interest (no vague blanket "send me
  updates" consent).
- An honest, dated note on what is NEVER collected regardless of account
  status (the "never stored" column from V6.2's table), so a user
  evaluating whether to create an account can see the actual boundary, not
  just trust a marketing claim.

### V6.6 Threat-model additions (extends PRD v2 Section 13)

- **New spoofing surface**: account login (OTP interception, phishing of
  the login flow). Mitigation: rate-limited OTP attempts, short OTP
  validity windows, no password-based fallback that could be weaker than
  OTP (if a password option is ever added, it must meet the same argon2
  standard as admin accounts).
- **New information-disclosure surface**: `saved_drafts.answer_history`
  and `frozen_terminal_snapshot` are the most sensitive data this product
  has ever stored, since a draft's answers can reveal a user's living
  situation (hostel vs. family address, disability status if they went
  down the PwD path, NRI/service-voter status). This is exactly why V6.2
  keeps the account layer entirely separate from any real-world identity
  (no name, no EPIC, no Aadhaar) — a breach of `saved_drafts` discloses
  "someone with this email chose the student-hostel path," not "citizen
  X, EPIC number Y, is disabled." Encrypt `answer_history`/
  `frozen_terminal_snapshot` at rest (column-level encryption, not just
  disk-level, given the sensitivity) as a v1-roadmap hardening item.
- **New elevation-of-privilege boundary**: `user_accounts` must use a
  distinct, least-privilege Postgres role from `admin_users`/content
  tables — an application bug or injection in the public account
  endpoints must not be able to reach admin tables, and vice versa.

### V6.7 What this section deliberately does NOT add

To keep this addition honest about its own scope: this is not a "user
profile" system, not a social feature, not a way for VoteAssist to build a
picture of a user across sessions for any purpose beyond the specific
draft-resume/reminder conveniences above, and not a login requirement for
anything. No feature in the rest of this PRD (the decision engine, the
knowledge base, the multi-channel bots) requires an account to function.
If a future proposal for this system involves an account unlocking
guidance content that anonymous users can't see, that proposal should be
rejected on sight — it would invert the entire premise of V6.1.
## V7. Format & Interoperability Specifications

This section fixes concrete, buildable wire/file formats for the five
artifacts this addendum introduces. Each is versioned or otherwise made
forward-compatible, since every one of them leaves VoteAssist's own
systems (a user's phone calendar, a user's downloaded export, a third
party's file-integrity check) and therefore cannot be silently reshaped
later without breaking something already in the wild.

### V7.1 Election-calendar export — iCalendar (.ics)

**Why .ics, not a proprietary format.** iCalendar (RFC 5545) is read by
every mainstream phone/desktop calendar app with zero VoteAssist-side
client code required. Two delivery modes are specified:

1. **One-time download**: `GET /v1/calendar/tracked-elections/{id}.ics` —
   a single `VEVENT` for one `tracked_elections` row, for a user who wants
   "add this one election to my calendar" from a terminal/checklist page
   or the election-calendar banner (Section V3.6).
2. **Subscribable feed**: `GET /v1/calendar/subscribe.ics?state_id={id}`
   (also reachable via a `webcal://` URL for one-tap subscription on iOS/
   Android/desktop calendar apps) — a live-updating multi-`VEVENT` feed of
   every `tracked_elections` row for that state, so a subscribed user's
   calendar app re-fetches and picks up admin corrections automatically
   (subject to that app's own refresh interval — see Section V9, risk 10,
   for the honest limits of this).

**Field mapping** (`tracked_elections` row → `VEVENT`):

| iCalendar field | Source | Notes |
|---|---|---|
| `UID` | `tracked_elections.id` + `@voteassist.in` | Stable identifier; unchanged across edits to the same row so calendar apps update in place rather than duplicating. |
| `DTSTAMP` | export-time `now()` | Per RFC 5545 requirement, regenerated on every export. |
| `DTSTART;VALUE=DATE` | `polling_date` | All-day event — a polling day, not a specific hour, since polling hours vary by state/booth and VoteAssist does not claim to know a user's exact booth timing. |
| `DTEND;VALUE=DATE` | `polling_date + 1 day` | RFC 5545 all-day convention: `DTEND` is exclusive, so a one-day event's end is the following calendar date. |
| `SUMMARY` | Derived: `"{election_type_label} — Polling Day"` for a general election, or `"By-election: {constituency_name} — Polling Day"` for a by-election (`constituency_ids` populated) | Never includes a candidate or party name — pure administrative fact, consistent with the platform's neutrality mandate. |
| `DESCRIPTION` | Templated (see below) | Includes a deep link back to VoteAssist and the official ECI/SEC source URL. |
| `URL` | `https://voteassist.in/elections/{tracked_elections.id}` | A VoteAssist page rendering the same election's full detail (nomination dates, MCC window, source citation) — most calendar apps render this as a tappable link. |
| `CATEGORIES` | `"Elections"` | Fixed value, lets a user filter/color-code in calendar apps that support it. |
| `VALARM` (nested) | `TRIGGER:-P7D` (7 days before `DTSTART`) | A single reminder, 7 days ahead — long enough to still act (check registration, plan travel) but not so early it's ignored as noise. Not configurable per-row in v1; a per-user configurable lead time is a plausible v2 refinement, not built here. |
| (not emitted) | `nomination_start_date`/`nomination_end_date`/`mcc_window_*` | These are candidate-nomination and MCC-compliance fields, not voter-facing calendar facts — folded into `DESCRIPTION` text instead of separate `VEVENT`s, to avoid cluttering a voter's calendar with events meant for candidates or admins. |

**`DESCRIPTION` template** (plain text, RFC 5545 requires literal `\n` for
line breaks and escaped commas/semicolons — shown here unescaped for
readability, escaping applied at generation time):

```
This date comes from an official source, not a live government feed.
Verify against the official notification before making plans:
{source_notification_url}

More detail and official links: https://voteassist.in/elections/{id}
Last confirmed by VoteAssist: {last_verified_date}

VoteAssist is an independent, non-governmental guidance tool. It is not
affiliated with the Election Commission of India or any State Election
Commission.
```

**Worked example** — a Punjab Panchayat by-election, one `VEVENT`:

```
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//VoteAssist India//Election Calendar//EN
CALSCALE:GREGORIAN
METHOD:PUBLISH
BEGIN:VEVENT
UID:7c1e3a9a-2f4b-4e11-9c2a-1a2b3c4d5e6f@voteassist.in
DTSTAMP:20260723T090000Z
DTSTART;VALUE=DATE:20260915
DTEND;VALUE=DATE:20260916
SUMMARY:By-election: Ludhiana East (Punjab Assembly) — Polling Day
DESCRIPTION:This date comes from an official source\, not a live govern
 ment feed. Verify against the official notification before making pla
 ns:\nhttps://ceopunjab.gov.in/notifications/2026-ludhiana-east-bye-ele
 ction\n\nMore detail and official links: https://voteassist.in/electi
 ons/7c1e3a9a-2f4b-4e11-9c2a-1a2b3c4d5e6f\nLast confirmed by VoteAssist
 : 2026-07-20\n\nVoteAssist is an independent\, non-governmental guidan
 ce tool. It is not affiliated with the Election Commission of India or
  any State Election Commission.
URL:https://voteassist.in/elections/7c1e3a9a-2f4b-4e11-9c2a-1a2b3c4d5e6f
CATEGORIES:Elections
BEGIN:VALARM
ACTION:DISPLAY
DESCRIPTION:Election reminder
TRIGGER:-P7D
END:VALARM
END:VEVENT
END:VCALENDAR
```

The subscribable feed (`/v1/calendar/subscribe.ics?state_id=...`) is the
same `VCALENDAR` wrapper with one `VEVENT` block per matching
`tracked_elections` row, `METHOD:PUBLISH`, and a `REFRESH-INTERVAL;VALUE=
DURATION:P1D` property as a hint to calendar clients that honor it
(most don't reliably — see Section V9).

### V7.2 Saved-draft/checklist export bundle (account settings "export" button)

A single downloadable JSON file, both human-readable (pretty-printed,
UTF-8, no minification) and machine-parseable. Top-level `formatVersion`
follows semver-ish `MAJOR.MINOR`; a MINOR bump adds fields only, a MAJOR
bump may remove/rename fields, so older tooling that only reads fields it
recognizes stays functional across MINOR bumps.

```json
{
  "formatVersion": "1.0",
  "exportedAt": "2026-07-23T14:32:00Z",
  "generator": "voteassist-api/0.9.2",
  "account": {
    "id": "b3f1c9d2-1234-4a5b-8c9d-0e1f2a3b4c5d",
    "contactChannel": "phone",
    "createdAt": "2026-03-01T09:12:00Z",
    "lastLoginAt": "2026-07-20T18:03:11Z",
    "notificationsOptIn": true,
    "notificationStateInterest": {
      "stateId": "8f3e2d1c-0000-4444-8888-abcdefabcdef",
      "stateName": "Punjab"
    }
  },
  "drafts": [
    {
      "id": "9a8b7c6d-5e4f-4321-a1b2-c3d4e5f60718",
      "label": "amma's address change",
      "decisionTreeVersion": 7,
      "createdAt": "2026-06-10T11:00:00Z",
      "updatedAt": "2026-06-10T11:14:22Z",
      "answerHistory": [
        { "questionId": "already_registered", "value": "yes" },
        { "questionId": "moved_recently", "value": "yes_same_state" },
        { "questionId": "move_type", "value": "intrastate" }
      ],
      "frozenTerminalSnapshot": {
        "terminalId": "terminal_form8_correction",
        "capturedTreeVersion": 7,
        "checklistItems": [
          "File Form 8 at voters.eci.gov.in for address correction.",
          "Keep your old and new address proof ready to upload."
        ],
        "citations": ["kb:form-8", "kb:ordinary-residence-student"],
        "deepLinks": ["https://voters.eci.gov.in/form-8"]
      },
      "frozenAt": "2026-06-10T11:14:22Z"
    },
    {
      "id": "1a2b3c4d-5e6f-4708-9a0b-1c2d3e4f5061",
      "label": null,
      "decisionTreeVersion": 6,
      "createdAt": "2026-05-02T08:00:00Z",
      "updatedAt": "2026-05-02T08:04:00Z",
      "answerHistory": [
        { "questionId": "already_registered", "value": "no" }
      ],
      "frozenTerminalSnapshot": null,
      "frozenAt": null
    }
  ],
  "consentArtifacts": [
    {
      "id": "4d5e6f70-8192-4a3b-8c9d-0e1f2a3b4c5d",
      "purpose": "notifications_state_interest",
      "action": "granted",
      "scope": { "stateId": "8f3e2d1c-0000-4444-8888-abcdefabcdef" },
      "timestamp": "2026-03-01T09:13:00Z",
      "source": "account_settings_page"
    }
  ]
}
```

Notes on the shape:
- `drafts[].frozenTerminalSnapshot` is `null` for any draft that never
  reached a terminal (in-progress draft) — the field is always present
  for schema stability, just nullable, rather than conditionally omitted.
- `answerHistory` reuses the exact `{questionId, value}` shape
  `core-domain::Answer` already uses internally (Section V6.2), so the
  export is a direct serialization, not a translation layer that could
  drift out of sync with the live data model.
- `consentArtifacts` is included here for completeness of "everything an
  account holds," and is the same array a request to `GET /v1/account/
  consent-artifacts` would return (Section V7.3) — export is not a
  separate data path from what the account settings page renders live.

### V7.3 Consent-artifact format

Scoped narrowly to VoteAssist's own needs (Section V6.5, borrowing DEPA's
discipline, not its network protocol): a new append-only table, one row
per grant/revoke of any specific consent, chained so a user's full
consent history for a purpose is reconstructable.

```sql
CREATE TYPE consent_purpose AS ENUM ('notifications_state_interest');
-- Deliberately a single purpose today (state-election reminders) — the
-- enum exists (rather than a free-text column) so a future new proactive-
-- messaging purpose must be added here explicitly and reviewed, not
-- silently introduced as an unenumerated string.

CREATE TYPE consent_action AS ENUM ('granted', 'revoked');

CREATE TABLE consent_artifacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES user_accounts(id) ON DELETE CASCADE,
    purpose consent_purpose NOT NULL,
    action consent_action NOT NULL,
    scope JSONB, -- e.g. {"stateId": "..."} for notifications_state_interest; NULL for a purpose with no sub-scope
    timestamp TIMESTAMPTZ NOT NULL DEFAULT now(),
    source TEXT NOT NULL -- which UI control produced this, e.g. "account_settings_page", "signup_flow"
);
```

`ON DELETE CASCADE` matches `saved_drafts`: deleting an account purges its
consent history along with everything else (Section V6.4) — there is no
"we keep your consent log after you delete your account" carve-out, since
that would itself be data the user asked to have erased.

Worked example — a user granting, then later revoking, notification
consent:

```json
[
  {
    "id": "4d5e6f70-8192-4a3b-8c9d-0e1f2a3b4c5d",
    "accountId": "b3f1c9d2-1234-4a5b-8c9d-0e1f2a3b4c5d",
    "purpose": "notifications_state_interest",
    "action": "granted",
    "scope": { "stateId": "8f3e2d1c-0000-4444-8888-abcdefabcdef" },
    "timestamp": "2026-03-01T09:13:00Z",
    "source": "account_settings_page"
  },
  {
    "id": "6a7b8c9d-0123-4e5f-9a1b-2c3d4e5f6071",
    "accountId": "b3f1c9d2-1234-4a5b-8c9d-0e1f2a3b4c5d",
    "purpose": "notifications_state_interest",
    "action": "revoked",
    "scope": null,
    "timestamp": "2026-07-15T19:40:00Z",
    "source": "account_settings_page"
  }
]
```

`scope` is intentionally `null` on a `revoked` row (revocation withdraws
the purpose entirely; it doesn't need to restate what was revoked, the
prior `granted` row already recorded that). This table is itself exported
(Section V7.2) and deletable, exactly like every other account-linked
table — no consent artifact is exempt from the account's own
export/delete rights, which is a deliberate difference from `audit_log`'s
admin-side entries (Section V7.4), which are never user-deletable because
they document platform-operator actions, not the user's own data.

### V7.4 Audit-log entry format

PRD v2 Section 11.11 already defines the outer `audit_log` table shape
(`actor_admin_user_id`, `action_type`, `entity_type`, `entity_id`,
`before_value`, `after_value`, `timestamp`, `ip_address`, `user_agent`).
What v2 leaves unspecified — and what this addendum fixes, since every
new admin surface in EPICs 17-21 below writes to this same table — is the
**internal shape of `before_value`/`after_value`**, so the Audit Log
Viewer's diff renderer (11.11: "shows before/after diff rendered
readably") can work uniformly across subsystems instead of each one
inventing its own payload convention.

**Convention**: `before_value`/`after_value` are always either `null`
(nothing existed / nothing exists — a create or a delete) or a flat JSON
object mapping **only the fields that changed** to their value —
a sparse diff, not a full-entity snapshot. Rationale: full snapshots of a
KB entry's body text or a decision tree's node graph would make
`audit_log` rows unboundedly large and would duplicate data already
versioned elsewhere (`knowledge_entry_revisions`, tree version history);
a sparse diff keeps every audit row small and keeps the Audit Log Viewer
fast, while the entity-scoped revision-history views (11.3/11.4) remain
the place to see a full historical body if needed.

`action_type` values are extended (additively, alongside v2's existing
enumerated list in 11.11) for the new subsystems this addendum adds:
`tracked_election_added`, `tracked_election_edited`, `tracked_election_
closed`, `local_body_kb_entry_status_change` (reuses the general
`kb_entry_status_change` type in practice — listed here only to confirm
per-state local-body entries are not a special case), `fringe_case_
logged`, `fringe_case_status_changed`, `consent_changed`, `draft_
deleted`, `account_deleted`, `bhashini_credential_rotated`.

Four worked examples spanning different subsystems, showing the uniform
envelope with subsystem-specific payloads inside:

```json
{
  "id": "e1f2a3b4-c5d6-4718-9a0b-1c2d3e4f5061",
  "actorAdminUserId": "5f6e7d8c-9b0a-41c2-83d4-e5f60718293a",
  "actionType": "tracked_election_added",
  "entityType": "tracked_elections",
  "entityId": "7c1e3a9a-2f4b-4e11-9c2a-1a2b3c4d5e6f",
  "beforeValue": null,
  "afterValue": {
    "electionType": "by_election",
    "stateId": "8f3e2d1c-0000-4444-8888-abcdefabcdef",
    "pollingDate": "2026-09-15",
    "sourceNotificationUrl": "https://ceopunjab.gov.in/notifications/2026-ludhiana-east-bye-election"
  },
  "timestamp": "2026-07-20T10:02:00Z",
  "ipAddress": "203.0.113.4",
  "userAgent": null
}
```

```json
{
  "id": "0a1b2c3d-4e5f-4061-8283-94a5b6c7d8e9",
  "actorAdminUserId": "5f6e7d8c-9b0a-41c2-83d4-e5f60718293a",
  "actionType": "kb_entry_status_change",
  "entityType": "knowledge_entries",
  "entityId": "local-body-elections-punjab",
  "beforeValue": { "reviewStatus": "draft" },
  "afterValue": { "reviewStatus": "verified" },
  "timestamp": "2026-08-02T13:20:00Z",
  "ipAddress": "203.0.113.4",
  "userAgent": null
}
```

```json
{
  "id": "1b2c3d4e-5f60-4172-8394-a5b6c7d8e9f0",
  "actorAdminUserId": "5f6e7d8c-9b0a-41c2-83d4-e5f60718293a",
  "actionType": "fringe_case_status_changed",
  "entityType": "fringe_case",
  "entityId": "42",
  "beforeValue": { "status": "triaged" },
  "afterValue": { "status": "added_to_tree", "linkedTreeVersion": 8 },
  "timestamp": "2026-09-01T09:15:00Z",
  "ipAddress": "203.0.113.9",
  "userAgent": null
}
```

```json
{
  "id": "2c3d4e5f-6071-4283-94a5-b6c7d8e9f0a1",
  "actorAdminUserId": null,
  "actionType": "account_deleted",
  "entityType": "user_accounts",
  "entityId": "b3f1c9d2-1234-4a5b-8c9d-0e1f2a3b4c5d",
  "beforeValue": { "draftCount": 2, "notificationsOptIn": true },
  "afterValue": null,
  "timestamp": "2026-07-23T14:35:00Z",
  "ipAddress": "203.0.113.201",
  "userAgent": "Mozilla/5.0 (self-service deletion, public web-app)"
}
```

Note the last example: a user's own self-service account deletion is
still an `audit_log` row (the platform must be able to account for "an
account existed and was deleted" for its own operational/compliance
history), but `actorAdminUserId` is `null` (no admin performed it — the
Audit Log Viewer must render a `null` actor as "user self-service", not
as a blank/error state) and the row itself contains no field that could
re-identify which user, beyond the already-erased `entityId` UUID — the
row documents that a deletion happened, not what was in the deleted
account.

### V7.5 Data-export bundle format for third parties/NGOs

Extends PRD v2 Section 11.13's "Export full knowledge base" concept from
a single downloadable archive into a **manifest-verified bundle**, so a
third party (a researcher, another civic-tech project, an archival
effort — the "open API for NGOs" deliverable referenced in Section 14's
EPIC 16, at the file-export level rather than the live-REST-API level of
Sections 4/10) can verify they received a complete, unmodified export
without needing any VoteAssist infrastructure access.

**Bundle layout** (a `.zip` or `.tar.gz`, either is fine — the manifest
format below is what matters, not the container):

```
voteassist-kb-export-2026-07-23/
  manifest.json
  entries/
    form-6.json
    form-6a.json
    form-7.json
    form-8.json
    service-voter.json
    ... (one file per knowledge_entries row)
    local-body-elections-punjab.json
  forms/
    form-6.json
    ...
  glossary/
    glossary.json
```

**`manifest.json`**:

```json
{
  "formatVersion": "1.0",
  "generatedAt": "2026-07-23T15:00:00Z",
  "generator": "voteassist-api/0.9.2",
  "exportScope": "verified_only",
  "kbSchemaVersion": "2.1",
  "totalEntries": 47,
  "license": "CC-BY-4.0",
  "files": [
    {
      "path": "entries/form-8.json",
      "entryId": "form-8",
      "schemaVersion": "2.1",
      "reviewStatus": "verified",
      "lastVerifiedDate": "2026-06-30",
      "sizeBytes": 4821,
      "sha256": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
    },
    {
      "path": "entries/local-body-elections-punjab.json",
      "entryId": "local-body-elections-punjab",
      "schemaVersion": "2.1",
      "reviewStatus": "verified",
      "lastVerifiedDate": "2026-08-02",
      "sizeBytes": 3110,
      "sha256": "1c4a2e6b7f9d0a3c5e8b1d4f6a9c2e5b8d1f4a7c0e3b6d9f2a5c8e1b4d7f0a3c"
    }
  ]
}
```

**Verification contract**: a consumer computes SHA-256 over each listed
file and compares against `files[].sha256`; a mismatch or a missing file
means the bundle is incomplete or corrupted, not that the content itself
is wrong — this is an integrity check, not a content-correctness check
(content correctness is what `reviewStatus: verified` + the KB's own
citation discipline already vouches for). `exportScope` is always present
and honestly reflects Section 11.13's existing `verified-only` default
vs. the internal-mirroring toggle that includes draft content (in which
case `exportScope` reads `"includes_unverified_draft_content"` and the
manifest itself, not just the archive's README, carries this warning so
a consumer inspecting only `manifest.json` still sees it). `license` is
carried in the manifest so a consumer doesn't have to separately locate
the project's licensing terms to know what they can do with the export —
**TODO: confirm the project's actual chosen content license** (this PRD
does not resolve that decision; `CC-BY-4.0` above is illustrative, not a
commitment).

---

## V8. Expanded Feature & Task Backlog Addendum

Continues PRD v2 Section 14's exact numbering convention
(`EPIC N` / `Feature N.M` / `Task E<epic>.F<feature>.T<task>`, phase tags
**[MVP-Rust-v1]** / **[v1]** / **[v2]** / **[v3]**, unmarked = MVP-Rust-v1)
starting at **EPIC 17**, since PRD v2 ends at EPIC 16.

### EPIC 17 — Election Jurisdiction & Calendar Engine

**Feature 17.1 — Jurisdiction data model & `crates/jurisdiction`**
- E17.F1.T1 Add the `administering_body`/`election_type` enums and
  `election_type_jurisdiction`/`state_election_commissions` migrations
  (Section V3.3).
- E17.F1.T2 Seed `election_type_jurisdiction` with the exact mapping in
  Section V3.2 (13 election types, each with `administering_body`,
  `is_direct_citizen_vote`, `voteassist_scope`).
- E17.F1.T3 Scaffold `crates/jurisdiction` (or a `core-domain` submodule,
  per Section V3.3) with a single `resolve_jurisdiction(election_type,
  state_id: Option<Uuid>) -> JurisdictionResult` function returning the
  administering body, scope, and (for SEC-administered types) the
  relevant `state_election_commissions` row.
- E17.F1.T4 `cargo nextest` unit tests covering all 13 `election_type`
  values, including the "no `state_id` supplied for an SEC-administered
  type" error case.
- E17.F1.T5 Seed `state_election_commissions` with official name + portal
  URL for every state/UT; leave `roll_derivation_note` /
  `roll_derivation_verified_date` `NULL` pending per-state research
  (**[v1]**, feeds EPIC 22).

**Feature 17.2 — Decision-tree jurisdiction routing**
- E17.F2.T1 Add the separate home-screen entry point "I have a question
  about my local Panchayat/Municipal election" alongside the existing
  "find out what I need to do" button (Section V3.4, item 2) — not nested
  inside the existing tree.
- E17.F2.T2 Implement a new `TerminalNode` variant, `jurisdiction_
  routing`, rendering: a plain-language explanation that local-body
  elections are run by the state's own Election Commission, the state
  SEC's portal deep link, and (only once verified per-state, Section
  V3.5) the `roll_derivation_note`.
- E17.F2.T3 Wire KB search and both bot channels' free-text intent
  matching to surface the `jurisdiction_routing` terminal for Panchayat/
  Municipal-flavored queries (e.g., "gram panchayat," "municipal ward")
  instead of forcing them through the ECI-shaped registration tree
  (Section V3.4, item 3).
- E17.F2.T4 Add short informational-only answer nodes for Rajya Sabha,
  Legislative Council (nominated portion), President, and Vice President
  questions ("can I vote in a Rajya Sabha election?" → correctly answered
  "no, not directly" rather than silently falling through to the Lok
  Sabha flow, Section V3.2).
- E17.F2.T5 `proptest`: assert no path through the existing "registered/
  not registered" tree ever emits an SEC deep link, and the `jurisdiction_
  routing` terminal never emits an ECI deep link (a jurisdiction-crossing
  regression test, run in CI alongside EPIC 2's existing tree property
  tests).

**Feature 17.3 — Per-state local-body content checklist**
- E17.F3.T1 Create the `local-body-elections` KB topic and one draft
  `knowledge_entries` row per state/UT (Section V3.5), `reviewStatus =
  draft` until individually verified.
- E17.F3.T2 Add a "local-body coverage map" widget to the Knowledge Base
  Content Editor (Section 11.3) — a small state-by-state
  verified/pending indicator specific to this topic, distinct from the
  general KB list/filter view.
- E17.F3.T3 Surface the same per-state verified/pending status on the
  public `jurisdiction_routing` terminal (Section V3.5: "the product UI
  must show, per state, whether that state's local-body information is
  verified yet or still pending, rather than presenting silence ... as if
  it were confirmed").
- E17.F3.T4 **[v2]** Per-state legal/content research and verification —
  tracked individually per state, see EPIC 22.

**Feature 17.4 — Election calendar & `.ics` export**
- E17.F4.T1 Add the `tracked_elections` migration (Section V3.6).
- E17.F4.T2 Implement an admin "Election Calendar" page (new admin page,
  extending Section 11's page list) with a `tracked_elections` entry
  form requiring `source_notification_url` (non-empty, validated as a
  URL) before save — no row may be entered without a citable source.
- E17.F4.T3 Implement `GET /v1/calendar/tracked-elections/{id}.ics`
  (Section V7.1's single-event mapping).
- E17.F4.T4 Implement `GET /v1/calendar/subscribe.ics?state_id={id}`
  (Section V7.1's subscribable multi-event feed).
- E17.F4.T5 Implement the public "there's an election coming up where you
  live" banner on `web-app` and both bot channels, reading
  `tracked_elections` (Section V3.6.b).
- E17.F4.T6 Wire a new `tracked_elections` row to auto-suggest (not
  auto-create) a draft `mcc_windows` row on the MCC Control Panel
  (Section 11.8), pre-filling `window_start` from
  `schedule_announced_date` for `superadmin` confirmation, rather than
  requiring fully independent manual MCC entry (Section V3.6.a).
- E17.F4.T7 **[v2]** Evaluate a semi-automated ECI/SEC press-release
  ingestion assist that proposes (never auto-publishes) a draft
  `tracked_elections` row for human confirmation.
- E17.F4.T8 `cargo nextest` coverage of `.ics` generation: valid RFC 5545
  output (verified via a parsing round-trip, e.g. against the `icalendar`
  crate or an equivalent parser used only in tests), correct all-day
  `DTEND` exclusivity, correct `VALARM` offset.

### EPIC 18 — Optional Accounts, Drafts & Privacy

**Feature 18.1 — Account data & auth**
- E18.F1.T1 Add the `user_accounts` migration (Section V6.2).
- E18.F1.T2 Implement OTP-based signup/login (email or phone channel,
  Section V6.2) — rate-limited attempts, short OTP validity window, no
  password-based fallback weaker than OTP (Section V6.6).
- E18.F1.T3 Implement least-privilege Postgres role separation: a
  distinct DB role for account/draft-endpoints with no grants on
  `admin_users`/content tables, and vice versa (Section V6.6); add a CI
  check (`xtask check-db-roles` or equivalent) that fails the build if
  the account-service role's grants include any admin table.
- E18.F1.T4 Implement session/token issuance for account holders,
  distinct from `admin-app`'s `tower-sessions` middleware (Section
  V6.2's trust-boundary separation), including immediate token
  invalidation on account deletion (Section V6.4).

**Feature 18.2 — Saved drafts**
- E18.F2.T1 Add the `saved_drafts` migration with `ON DELETE CASCADE`
  (Section V6.2).
- E18.F2.T2 Implement the non-blocking "Save my progress" affordance in
  the `web-app` question/answer island, surfaced only after the first
  answer, never as a pre-start signup wall (Section V6.3).
- E18.F2.T3 Implement draft-resume logic: same-tree-version exact replay,
  or new-tree-version replay-with-stop-at-first-invalid-node plus an
  explicit "this question has changed since you last answered it, please
  continue from here" message (Section V6.3).
- E18.F2.T4 Implement `frozen_terminal_snapshot` capture at save time and
  the "Saved on [date] — refresh to see current guidance?" control
  (Section V6.3).
- E18.F2.T5 **[v1]** Column-level encryption at rest for
  `answer_history`/`frozen_terminal_snapshot` (Section V6.6 hardening
  item).

**Feature 18.3 — Account settings, export, deletion**
- E18.F3.T1 Implement the account settings page rendering every stored
  field back to the user in readable form (not a raw JSON dump), per
  Section V6.5.
- E18.F3.T2 Implement the one-click JSON export button producing the
  bundle format in Section V7.2.
- E18.F3.T3 Implement one-click "delete this draft" and "delete my
  account" actions (Section V6.4), each writing the corresponding
  `draft_deleted`/`account_deleted` `audit_log` entry (Section V7.4).
- E18.F3.T4 Implement the notification-consent toggle (state-scoped
  opt-in, default off, no vague blanket consent — Section V6.5).
- E18.F3.T5 Implement the disclosed 30-day encrypted-backup purge job for
  deleted accounts/drafts beyond their immediate live-system deletion
  (Section V6.4), and the corresponding privacy-policy language.

**Feature 18.4 — Consent-artifact logging**
- E18.F4.T1 Add the `consent_artifacts` migration (Section V7.3).
- E18.F4.T2 Wire every grant/revoke of notification consent to write a
  `consent_artifacts` row.
- E18.F4.T3 Surface consent-artifact history on the account settings page
  (Section V6.5's "visible consent artifact" idea).
- E18.F4.T4 Include `consent_artifacts` in both the account export bundle
  (Section V7.2) and the account-delete cascade (Section V6.4) — no
  consent record outlives the account it belongs to.

### EPIC 19 — Existing-Voter Parity Features

**Feature 19.1 — Returning-voter UX**
- E19.F1.T1 Add a distinct home-screen entry path for an already-
  registered elector checking on something (correction, re-verification,
  polling-station change) alongside the new-registration entry path, so
  neither reads as the "default"/primary path and the other as an
  afterthought (this addendum's "existing/old voter parity"
  principle).
- E19.F1.T2 Implement a proactive re-verification nudge: for a logged-in
  user with a saved draft and notification opt-in for a state, a
  `tracked_elections` row for that state triggers a "there's an election
  coming up — here's what to check" notification (ties E17.F4, E18.F2,
  and the WhatsApp template-approval constraint noted in Section 11.9 —
  this is exactly the kind of proactive nudge that must go through an
  approved template on WhatsApp).
- E19.F1.T3 Implement a repeat postal-ballot/home-voting reapplication
  reminder: PwD/85+ users who previously completed the Form 12D path
  (Section V4/existing MVP terminal) are reminded, per new
  `tracked_elections` row affecting their constituency, that Form 12D
  must be resubmitted for each election — it is not a one-time
  registration.
- E19.F1.T4 Apply the copywriting-tone guidelines (Section V4) across
  every existing MVP-v1 terminal node as a dedicated content-audit-and-
  rewrite pass, requiring `legal_reviewer` re-sign-off per touched node
  since terminal wording is legally-reviewed content (not a free-form
  copy-editing pass — see Section V9, risk 12, on the review-capacity
  implication of this).
- E19.F1.T5 Add an automated tone-guideline lint (a banned-phrase/
  required-disclaimer-pattern check against terminal node copy) as a CI
  gate, so future terminal-copy edits can't silently regress the
  guideline established in E19.F1.T4.

### EPIC 20 — Self-Improving Content Operations

**Feature 20.1 — Courteous change-detection job**
- E20.F1.T1 Add a `citation_source_hashes` table (`source_url`,
  `last_content_hash`, `last_checked_at`, `citing_kb_entry_ids`) recording
  the last-known content hash of each distinct cited official URL.
- E20.F1.T2 Implement a rate-limited (a small, curated list of official
  pages, fetched at most once daily per URL),
  `robots.txt`-respecting fetch job in `crates/jobs` that recomputes each
  URL's content hash and, on a change, immediately flags every citing
  `knowledge_entries` row `needs_reverification` — an event-driven
  supplement to, not a replacement for, the existing 180-day age-based
  digest (E11.F2.T2/E5.F3.T3).
- E20.F1.T3 Add a Dashboard tile (Section 11.2) — "N entries flagged by
  change-detection since last review" — reported separately from the
  existing age-based re-verification counter, since the two signals mean
  different things operationally (age-based = "due for a routine look,"
  change-detected = "something upstream may already have moved").
- E20.F1.T4 Implement backoff/failure handling so a source site's
  transient outage or temporary block does not spuriously flag every
  citation pointing at it — require N consecutive successful-fetch
  mismatches (not a single failed/differing fetch) before flagging.

**Feature 20.2 — Fringe-case registry**
- E20.F2.T1 Add the `fringe_case` migration: `id`, `description`,
  `reporter_channel` (e.g. `feedback_form`, `admin_observation`,
  `telegram`, `whatsapp`), `status` (`new` / `triaged` / `added_to_tree` /
  `wontfix` / `needs_legal_review`), `linked_kb_entry_id` (nullable FK),
  `linked_tree_version` (nullable INT), `created_at`, `updated_at`.
- E20.F2.T2 Implement an admin "Fringe Cases" triage page (new page,
  extending Section 11's list) — list/filter by status/reporter channel,
  a status-transition action, and a "link to KB entry/tree version once
  resolved" action.
- E20.F2.T3 Add a "flag as fringe case" action from the existing Feedback
  & Grievance Triage page (Section 11.7), distinct from that page's
  ordinary feedback-resolution actions — logging a coverage gap is not
  the same event as closing a support ticket.
- E20.F2.T4 Write `audit_log` entries (`fringe_case_logged`,
  `fringe_case_status_changed`) for every creation and status transition
  (Section V7.4).

**Feature 20.3 — Analytics-informed "tree health" report**
- E20.F3.T1 Extend the Analytics Dashboard (Section 11.12) with a "Tree
  Health" report: the highest-drop-off nodes cross-referenced with
  feedback-category correlation (nodes where both drop-off rate and
  negative-feedback rate are elevated, ranked as a prioritized human
  review list).
- E20.F3.T2 Implement the correlation as a precomputed scheduled-job
  report table (`crates/jobs`, feeding off the existing hourly/daily
  rollups plus the `feedback` table), not a live heavy join computed on
  page load, consistent with Section 11.12's existing "reads only from
  rollup tables" performance guardrail.
- E20.F3.T3 Add a one-click "send to fringe-case registry" action from a
  Tree Health report row, pre-filling `description`/`reporter_channel =
  admin_observation` and linking the triggering node/analytics window.

### EPIC 21 — Bhashini Integration

- E21.F1.T1 Scaffold `crates/bhashini-client` — a thin `reqwest` wrapper
  over the Bhashini/ULCA APIs, following the same adapter pattern as
  `crates/bot-whatsapp`/`crates/ivr-gateway`.
- E21.F1.T2 Implement an MT-assist call for draft machine translation of
  KB body text, wired as a button in the Translation Management page
  (Section 11.5); output always lands in `draft` status and is never
  auto-published, per the existing MT-is-draft-only rule (PRD v2 Section
  17.5) — Bhashini output gets exactly the same human-review gate as any
  other machine translation source, no exception.
- E21.F1.T3 Add masked-credential config for Bhashini API access,
  following the same write-only-secret UX as Bot Channel Management
  (Section 11.9) — either as a new minimal config panel or an added
  section of that existing page.
- E21.F1.T4 **[v2/v3]** Evaluate Bhashini ASR/TTS as an STT/TTS option for
  the future IVR channel (`crates/ivr-gateway`, PRD v2 Section 6.7/8.3)
  alongside Exotel's own Indic STT/TTS — a side-by-side comparison of
  language coverage, latency, and reliability, before committing to
  either or both (see Section V9, risk 2, on why this must not be
  assumed settled by Bhashini's existence alone).
- E21.F1.T5 Add an integration test against Bhashini's sandbox/free tier
  if one exists at implementation time; otherwise a mocked-response
  contract test asserting `crates/bhashini-client`'s request/response
  parsing against a recorded fixture.
- E21.F1.T6 **[v2]** Evaluate Bhashini's rate limits, uptime history, and
  terms of service for production-scale suitability before any wider
  rollout beyond the Translation Management MT-assist button (Section
  V9, risk 2).

### EPIC 22 — Multi-Jurisdiction Content Expansion [v2/v3]

This is a large, genuinely multi-year content-operations effort, distinct
in kind from the engineering epics above — see Section V9, risk 1, on
whether it is realistically resourceable as scoped.

- E22.F1.T1 **Template task pattern** (apply once per state/UT): for
  state `<X>`, research and verify (a) the State Election Commission's
  official portal URL and current online-service availability, (b)
  whether `<X>`'s local-body electoral roll is ECI-roll-derived or
  separately maintained (populate `roll_derivation_note`/
  `roll_derivation_verified_date` in `state_election_commissions`), (c)
  any state-specific local-body registration/correction forms, (d) write
  and move the `local-body-elections-<x>` KB entry to `reviewStatus =
  verified` with `legal_reviewer` sign-off, (e) update the
  `state_election_commissions` row's `last_verified_date` accordingly.
- E22.F1.T2 Apply the template to **Maharashtra** (State Election
  Commission, Maharashtra).
- E22.F1.T3 Apply the template to **Uttar Pradesh** (UP State Election
  Commission).
- E22.F1.T4 Apply the template to **Tamil Nadu** (Tamil Nadu State
  Election Commission).
- E22.F1.T5 Apply the template to **Karnataka** (Karnataka State Election
  Commission).
- E22.F1.T6 Apply the template to **West Bengal** (West Bengal State
  Election Commission).
- E22.F1.T7 Apply the template to **Punjab** (State Election Commission,
  Punjab — the worked example already used in Section V3.4/V7.1).
- E22.F1.T8 **[ongoing, not a fixed sprint]** Apply the same template
  task pattern to each remaining state/UT (approximately 22 more) as
  volunteer or funded capacity allows; tracked as a standing content-ops
  backlog with one task per state, not scheduled against any committed
  roadmap phase, since there is currently no committed resourcing for it
  (Section V9, risk 1).

### Cross-cutting acceptance criteria addendum

Extends PRD v2 Section 14's five-item "MVP-Rust-v1 done" checklist with a
sixth item specific to this addendum's most sensitive data category:

6. **Privacy/data-minimization review checklist** — any task in EPIC 18
   (or any other epic) that adds or modifies a column, endpoint, or export
   path touching `user_accounts`, `saved_drafts`, or `consent_artifacts`
   is only complete once it has passed the following checklist, signed
   off by a `legal_reviewer`:
   - [ ] No new column stores a government ID, uploaded document, or any
     sensitive-category field (the "never stored" list, Section V6.2).
   - [ ] Every new field has a documented purpose and is rendered back to
     the user on the account settings page (Section V6.5) — nothing is
     stored that isn't also shown back.
   - [ ] Every new field is included in both the export bundle (Section
     V7.2) and the account/draft delete-cascade path — no field can be
     added to these tables that escapes export or deletion.
   - [ ] Any new proactive-messaging trigger reads its consent from
     `consent_artifacts`/`notifications_opt_in` — never a separate,
     ungated flag introduced elsewhere in the codebase.
   - [ ] If the field could reveal a sensitive inference even indirectly
     (disability status via the PwD path, NRI/service-voter status, a
     hostel-vs-family-address choice), it is covered by the column-level
     encryption plan (Section V6.6/E18.F2.T5) or an equivalent documented
     mitigation, not left as plaintext on the assumption that the account
     layer's identity-decoupling alone is sufficient.

---

## V9. Updated Open Questions & Risks (addendum to PRD v2 Section 22)

Continuing PRD v2 Section 22's numbering (items 14+), specific to this
addendum's scope — genuinely unresolved, not restated caveats:

14. **Per-state SEC content research is ~28 separate, unstandardized
    efforts with no unified source to draw from.** PRD v2 risk 5 already
    flags that the tribal/urban-slum/homeless per-state legal review
    program is unresourced; this is a **distinct and additional** risk —
    even a well-resourced team would face 28 states' worth of SEC
    portals, each with different levels of digitization, different
    officially-published forms, and no ECI-equivalent single national
    entry point to research from centrally (Section V3.5). EPIC 22 names
    six example states plus an "ongoing" bucket for the rest precisely
    because there is no credible way to commit this to a fixed timeline
    today; whether it is ever realistically completed depends entirely on
    volunteer-contributor or funded-partner capacity that does not
    currently exist, and this should be tracked as a standing
    organizational risk, not implied to be "in progress" by EPIC 22's
    existence.
15. **Bhashini's production suitability is unverified, not assumed.**
    Bhashini is free and government-backed, which makes it attractive,
    but this PRD has not verified its API uptime history, rate limits at
    realistic VoteAssist-scale usage, or its terms of service's
    compatibility with an independent (non-governmental) civic-tech
    product redistributing MT output as draft translations. E21.F1.T6
    names this evaluation as a gating task before wider rollout; it
    should not be treated as a formality — if Bhashini's real-world
    reliability or ToS terms turn out to be unsuitable for a production
    dependency, EPIC 21's IVR-evaluation task (E21.F1.T4) and even the
    Translation-Management MT-assist button (E21.F1.T2) may need to fall
    back to Exotel-only STT/TTS and human-only translation drafting,
    respectively — a real possibility, not a remote one, given how new
    and fast-evolving this platform is.
16. **Offering accounts at all — even fully optional — may itself be a
    trust/perception risk worth testing before wide rollout.** Section
    V6.1's design test ("does the anonymous path still work exactly as
    before") addresses functional regression, but does not address a
    separate perception question: a user who has internalized "this is a
    neutral, non-governmental guidance tool, not an official system" may
    reasonably ask "why would a neutral guidance tool need an account at
    all?" — and the mere presence of a signup affordance, however
    optional, could itself read as mission creep or as the platform
    building a user base for some other purpose, undermining the
    same trust posture Section 19.6/PRD v2 risk 7 works hard to protect.
    This is distinct from PRD v2 risk 7 (which is about disclaimer-banner
    confusion with an official ECI tool) — this risk is about the
    optional-account *feature's existence* creating doubt, independent of
    any branding confusion. Recommend targeted user testing of the
    "Save my progress" affordance's framing and placement specifically
    for this reaction, before EPIC 18 ships beyond an initial soft
    rollout.
17. **The election-calendar feature's admin-curated (non-live-API) nature
    means it will always lag real-world announcements by some amount —
    and that lag is not equally acceptable across every use of the same
    data.** For the public "election coming up" banner and `.ics` export,
    a lag of hours to a few days after an official announcement is a
    minor inconvenience. But `tracked_elections` is also wired
    (E17.F4.T6) to auto-suggest MCC window entries on the safety-critical
    MCC Control Panel (PRD v2 Section 11.8) — and MCC suppression of
    proactive messaging is meant to begin **the moment** ECI announces a
    schedule, per PRD v2 Section 2.6. If the admin team's routine
    cadence for entering a `tracked_elections` row is slower than that
    (e.g., entered the next business day rather than same-day), the
    platform could keep sending proactive broadcasts during the early
    hours/days of an active MCC window purely because no one had entered
    the row yet. This PRD does not yet specify an **urgent/out-of-cadence
    MCC-entry process** (e.g., an on-call rotation, a lower-friction
    "just flip MCC live for this state right now, backfill the
    `tracked_elections` row later" fast path) distinct from the routine
    admin-curation cadence assumed elsewhere in this section — this gap
    should be closed before EPIC 17 is treated as satisfying MCC's
    safety-critical timing requirement, not merely noted as a nice-to-
    have.
18. **The V3.2 jurisdiction table may not be the full picture of
    state-to-state legal variation.** V3.2/V3.3 model a clean two-body
    split (ECI vs. SEC) and V3.5 already flags that per-state roll-
    derivation and portal specifics need individual verification — but
    this is a narrower caveat than the deeper, unresolved question of
    whether the table's **category boundaries themselves** hold
    uniformly across all states. State legislation on local-body
    elections varies enough (some states' Panchayati Raj Acts, election-
    tribunal structures, or ward-delimitation processes may not map
    cleanly onto the ECI/SEC binary assumed here) that this PRD cannot
    claim V3.2 is a complete taxonomy until legal review confirms it —
    this is a **research-completeness risk about the model itself**, not
    only about filling in per-state facts under an assumed-correct model,
    and is distinct from the general "review is needed" caveat already
    present throughout Sections V3 and 10.
19. **OTP delivery introduces a new third-party dependency the anonymous
    product never needed.** Email/phone OTP delivery (Section V6.2)
    requires an SMS gateway and/or transactional-email provider —
    infrastructure the fully-anonymous decision-engine flow has no need
    for today. Who operates and pays for this (ties into PRD v2 risk 8's
    funding-model gap), and whether an SMS gateway vendor could itself
    become a point of phone-number exposure or correlation risk that the
    account design otherwise goes out of its way to avoid (Section V6.2's
    "store a salted hash of the contact identifier" design), is not yet
    evaluated against a specific vendor choice — this PRD names the OTP
    requirement but does not commit to a specific SMS/email provider or
    its own privacy posture.
20. **`consent_artifacts` and `audit_log` are themselves data that need a
    retention policy, not an assumed-indefinite exemption.** PRD v2
    Section 11.13 exempts MCC- and role-management-related `audit_log`
    rows from general purge policy because they matter for external
    accountability indefinitely — but this addendum's new
    `consent_artifacts` table (Section V7.3) and the new user-account-
    related `audit_log` entries (`account_deleted`, `draft_deleted`,
    `consent_changed`) are a different kind of record: an append-only
    history of one specific person's consent choices, which, if retained
    indefinitely by default, risks becoming exactly the kind of "shadow
    profile" the account design otherwise avoids. Section V7.3 states
    `consent_artifacts` is deleted on account deletion (closing the main
    gap), but the **retention window for `consent_artifacts` on an
    account that is never deleted** — years of a still-active user's
    consent-change history accumulating — is not yet addressed, and
    should not simply inherit the MCC/role-management "keep forever"
    posture by default.
21. **Column-level encryption is a real key-management commitment this
    PRD does not yet size.** E18.F2.T5's column-level encryption for
    `answer_history`/`frozen_terminal_snapshot` (Section V6.6) is the
    right call given what that data can reveal (Section V6.6), but it
    introduces key-management questions (who holds the encryption key,
    how is it rotated, what happens to encrypted rows if a key rotation
    is botched) that are non-trivial operational burden for a project
    PRD v2 risk 8/10 already flags as volunteer-heavy and
    funding-unresolved. This item should not be treated as a checkbox
    task equivalent in effort to an ordinary schema migration — it may
    slip past MVP-Rust-v1/v1 if key-management capacity isn't separately
    planned for, and this PRD does not yet commit to who owns that.
22. **The fringe-case registry surfaces gaps faster than the project can
    resolve them.** EPIC 20's `fringe_case` table (Section V5) gives the
    team visible tracking of "the tree doesn't yet handle X," which is a
    genuine improvement over losing that signal in the general feedback
    inbox — but visibility is not the same as resolution capacity. Many
    fringe cases will require the same scarce legal/content-research
    effort already named as unresourced in PRD v2 risk 5 and this
    section's risk 14; without a triage-to-resolution throughput
    commitment, the registry risks becoming a large, honestly-tracked,
    permanently-growing backlog rather than a mechanism that actually
    closes gaps — worth naming now rather than discovering after a year
    of `status = triaged` rows with no `added_to_tree` movement.
23. **A subscribed `.ics` calendar feed only updates as often as the
    user's own calendar app chooses to refetch it — and anonymous users
    have no correction channel at all if a date changes.** Section
    V7.1's `REFRESH-INTERVAL` hint is honored inconsistently across
    calendar clients (many refresh daily at best, some only on manual
    pull-to-refresh); if an admin corrects a `tracked_elections` row
    (e.g., a postponed polling date) after a user has already subscribed
    or downloaded a one-time `.ics` file, that user's phone calendar may
    silently show a now-wrong date for some period. A logged-in,
    notification-opted-in user would additionally receive a push/
    WhatsApp/Telegram correction (Section V8, EPIC 19) — but a purely
    anonymous `.ics` user, which is the majority-expected case given
    Section V6.1's anonymous-by-default design, has no correction channel
    whatsoever beyond their own calendar app's refresh behavior. This is
    an honest limitation of choosing an open, account-free calendar
    format and is not solved by anything currently specified in Section
    V7.1 or EPIC 17.
24. **An easy MT-assist button could paradoxically reduce translation
    quality for scarce-translator languages, not just supplement them.**
    PRD v2 risk 11 already flags a scarcity of qualified volunteer
    translators for lower-resource Eighth Schedule languages. Adding a
    one-click Bhashini MT-assist draft (E21.F1.T2) is meant to give those
    scarce translators a faster starting point, but the same feature
    could instead invite a rubber-stamp failure mode — a time-pressed
    volunteer translator approving a plausible-looking machine draft with
    less scrutiny than they'd apply to translating from scratch. This
    PRD does not yet specify a review-workflow safeguard (e.g., a visibly
    different UI treatment for "started from MT" vs. "written from
    scratch" translations, or a spot-check/second-reviewer sampling rule
    for MT-originated approvals) to guard against this, and should before
    E21.F1.T2 ships broadly.
25. **Retrofitting copywriting-tone guidelines onto every existing
    terminal node is a nontrivial draw on already-committed legal-review
    capacity, not a free copy pass.** PRD v2 Section 15.1 already commits
    to a one-time external legal review of the full decision-tree v2
    content before any public launch beyond MVP-Rust-v1 parity. EPIC 19's
    E19.F1.T4 requires `legal_reviewer` re-sign-off on every terminal node
    touched by the tone-guideline retrofit, since terminal wording is
    legally-reviewed content — this is additional legal-review load
    layered on top of, not instead of, that existing commitment, and this
    PRD does not yet reconcile the two against the same
    already-scarce-per-risk-5/14 legal-review capacity.

---
