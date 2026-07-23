# Legal and Compliance Review

Status: Draft for legal counsel review. Nothing in this document is legal
advice; it is an internal working document intended to be reviewed and
corrected by qualified counsel before the product ships publicly. Where a
specific legal conclusion is asserted below without a named source, treat
it as a hypothesis requiring verification, not a settled fact.

## 1. Nature of the Content: Informational, Not Legal Advice

VoteAssist India provides procedural/administrative information about
electoral registration processes, sourced from and citing public official
material (ECI, SVEEP, CEO portals, Gazette notifications, RP Act/Rules).
It does not provide legal advice, does not represent any user, and does
not adjudicate any user's individual eligibility — it only maps stated
facts about a user's situation to publicly documented ECI procedures.

Required disclaimer language (must appear on every page, not just a
buried footer, and must be repeated at the point of the terminal outcome):

> "VoteAssist India provides general information to help you understand
> electoral registration procedures. It is not legal advice, is not
> affiliated with the Election Commission of India or any government
> body, and does not guarantee eligibility or outcome. For official
> action and current requirements, use voters.eci.gov.in, ECINET, the
> Voter Helpline app, or call the National Voter Helpline at 1800-11-1950."

## 2. Non-Impersonation of Government (Trademark / Passing-Off Risk)

Risks to actively guard against:

- Use of the ECI emblem, the State Emblem of India, the Government of
  India tricolor/ashoka-chakra iconography, or any CEO office logo. None
  of these may appear in VoteAssist India's branding, favicon, or UI
  chrome, under any circumstance.
- Visual similarity to eci.gov.in or voters.eci.gov.in (layout, color
  palette, typography, or domain-name patterns that could confuse a user
  into thinking they are on an official site). See `07-design-system.md`
  for the deliberately distinct visual language chosen partly for this
  reason.
- Naming: "VoteAssist India" must always be presented with a clear
  "independent" / "unofficial" qualifier in first mentions on any given
  page or screen, to avoid an implication of official status. Domain name
  and app store listing copy must avoid terms like "official," "govt,"
  or ECI-branded terms in a way that implies endorsement.
- No use of government letterhead styling, ministry seals, or any
  simulated "official notice" formatting for VoteAssist content.

This is a passing-off / trademark / possibly Emblems and Names
(Prevention of Improper Use) Act 1950 concern — flagged here for counsel
to confirm which specific statutes apply and what compliance steps (if
any beyond the design/naming precautions above) are required.

## 3. Data Protection: Digital Personal Data Protection Act 2023 (DPDP)

Working principles (verify final compliance posture with counsel once DPDP
rules and any sectoral guidance are finalized/in force):

- **Minimal collection**: collect only what the decision engine needs to
  answer the user's question in the current session (e.g., age bracket,
  residence type, state) — never collect name, EPIC number, Aadhaar
  number, or other direct identifiers unless a specific optional feature
  requires it (e.g., emailing oneself a checklist) and the user opts in.
- **Purpose limitation**: session data used only to produce that session's
  result and (in de-identified, aggregate form) product analytics — never
  repurposed for profiling, marketing, or sharing with third parties.
- **Consent**: any optional data collection beyond the anonymous session
  (e.g., email for checklist delivery, feedback with contact info) must
  have clear, specific, opt-in consent, not a bundled "by using this site
  you agree to..." blanket clause.
- **No sensitive document retention by default**: no upload/storage of
  Aadhaar, EPIC, passport, birth certificate, disability certificate, or
  any official document, under any default flow. If a future feature
  requires temporary processing of a document (e.g., client-side OCR),
  prefer doing that entirely client-side; if server-side processing is
  ever unavoidable, it must be encrypted at rest and in transit, subject
  to a strict TTL (delete-by-default), and gated by explicit,
  feature-specific consent (see `08-database-schema.md` and
  `16-security-threat-model.md`).
- **Right to erasure / access**: any user-identifiable data VoteAssist
  does hold (e.g., an email address provided for checklist delivery, or a
  feedback submission with contact info) must be deletable and
  retrievable on request via a documented process, published in
  `/about/privacy`.
- **No special-category inference**: caste, religion, political
  affiliation, and similar sensitive categories must never be collected,
  inferred, or stored, under any feature, ever — this is a stricter bar
  than DPDP strictly requires, adopted deliberately given the political-
  neutrality mission constraint.

## 4. Political Neutrality and Model Code of Conduct (MCC) Exposure

- No content may reference or imply support/opposition for any party,
  candidate, or ideological position. Content review (section 7 below)
  must explicitly check for this on every content change, not just at
  launch.
- During the Model Code of Conduct period for any election VoteAssist
  content touches (i.e., whenever MCC is in force for a state or the
  Union), the product should:
  - Suspend any feature that could be construed as voter mobilization
    messaging beyond neutral procedural guidance (e.g., no "get out the
    vote" push notifications, no reminders framed as encouragement to
    vote for turnout's sake beyond stating factual deadlines).
  - Ensure all deep-links and cited information reflect the specific,
    current election's notified dates/timelines (e.g., the Form 12D
    5-day window is measured from that election's notification date).
  - Have a named compliance owner (section 6) empowered to pull or edit
    content quickly if an MCC concern is raised.
- This is a genuinely live legal risk area (MCC guidance and its
  application to third-party digital tools is not something this document
  can settle) — flagged for direct legal review before any election-period
  operation, not assumed safe by default.

## 5. Accessibility Law

- Rights of Persons with Disabilities Act 2016 (India) — establishes
  accessibility obligations relevant to a public-facing civic information
  service; specific applicability to a non-government open-source project
  should be confirmed with counsel, but WCAG 2.2 AA is adopted as the
  MVP technical baseline regardless of the precise legal applicability,
  because it is good practice for this product's mission (see
  `12-accessibility-spec.md`).
- WCAG 2.2 is referenced as the international technical standard;
  RPWD Act 2016 is the relevant domestic legal framework — treat these as
  complementary, not identical, and have counsel confirm any
  RPWD-specific documentation or certification expectations.

## 6. Governance: Compliance Owner Role

Even as an open-source project, VoteAssist India needs a named person (not
just "the community") accountable for:

- Reviewing and approving any change to legal/procedural claims before it
  ships (section 7).
- Being the point of contact for takedown/correction requests, including
  from ECI or CEO offices, or from users who spot an error.
- Monitoring MCC periods and triggering the election-period content
  posture described in section 4.
- Maintaining `/about/legal`, `/about/privacy`, and this document as living
  artifacts.

Recommendation: define this explicitly in a `GOVERNANCE.md` and
`CONTRIBUTING.md` at the repository root (not created in this doc pass —
tracked as a follow-up), naming the role "Compliance Owner," distinct from
"Maintainer," and requiring the role be filled before any public,
non-preview deployment.

## 7. Content Review Process (required before any legal/procedural claim ships)

Proposed minimum process for any change to a KB entry, decision-tree
terminal node, or form-related copy:

1. Author drafts the change with a citation (or an explicit
   "community guidance — pending verification" label).
2. At least one reviewer other than the author checks the citation against
   the live official source (not a cached/assumed version) and confirms
   the `last_verified_date` is updated.
3. Compliance Owner (or delegate) signs off specifically on: (a) no
   political content, (b) no impersonation risk, (c) no fabricated
   specifics beyond what the source supports.
4. Change is merged with the review trail preserved (PR history serves as
   the audit log in the open-source repo).
5. A recurring cadence (recommended: quarterly, and immediately after any
   ECI circular affecting covered forms) re-verifies all published content
   regardless of whether a change was requested — see `19-roadmap.md`.

## 8. Liability Disclaimers

- General disclaimer of warranty and liability in `/about/legal`, standard
  open-source-adjacent language ("provided as-is, without warranty of
  accuracy or fitness for a particular purpose") combined with the
  plain-language non-affiliation/non-legal-advice statement from section 1.
- Should be reviewed by counsel for enforceability under Indian consumer
  protection and IT Act frameworks before public launch; not assumed
  sufficient as drafted here.

## 9. Summary of Follow-Up Actions for Counsel

1. Confirm applicable statutes for the impersonation/passing-off concerns
   in section 2 and any required disclaimers beyond what's drafted.
2. Confirm DPDP Act 2023 compliance posture once rules are finalized,
   especially for any optional email-based features.
2. Confirm MCC exposure and any additional restrictions during election
   periods, especially for push notifications or app-store presence
   framed as "vote" tooling.
3. Confirm RPWD Act 2016 applicability and any documentation expectations
   beyond WCAG 2.2 AA.
4. Review and correct the liability/disclaimer language in section 8 and
   `/about/legal` before public launch.
