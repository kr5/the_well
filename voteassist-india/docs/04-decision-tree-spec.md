# Decision Tree Specification

This document specifies the v1 (MVP) decision engine tree: node types, the
logical structure, and the terminal outcome contract. It is explicitly a
**v1 subset**. Full "every legal scenario" coverage — every state's special
provisions, every service-voter subcategory, every edge case in tribal or
cross-border constituencies — is a roadmap item requiring further legal
review per state (see `19-roadmap.md` and `06-legal-compliance-review.md`).
Nothing in this tree should be read as a claim of completeness.

## 1. Node Types

| Node type | Purpose | Contract |
|---|---|---|
| `question` | Presents a single question with a small, fixed set of answer options | Must have exactly one edge per answer option; no free-text branching logic in v1 |
| `branch` | Internal routing node with no user-facing question (derived from prior answers, e.g., combining age + date) | Deterministic function of prior answers only; no side effects |
| `info` | Non-branching explanatory interstitial (e.g., "here's what a qualifying date is") shown before a question, not itself a decision point | Optional; must not be required to reach a terminal node without content |
| `terminal` | An end state of the tree: a specific recommended action | Must include: recommended form, required document categories, citation(s), deep-link target, and a "verify current requirements" caution (see section 3) |

Engine invariants (see also `17-test-plan.md`):

- Every path from the root must reach a `terminal` node in a bounded number
  of steps (no infinite loops; enforced by property-based tests).
- Every `terminal` node must carry at least one citation or be explicitly
  labeled "community guidance — pending verification."
- No `question` node may collect sensitive categories (caste, religion,
  party preference) under any circumstance.
- The tree is versioned (see `13-technical-architecture.md`); a session
  records which tree version it ran against.

## 2. Tree Structure (v1, outline form)

```
ROOT: "What best describes your situation?"
├── A. "I'm not sure if I'm registered to vote"
│   └── [question] Direct to roll search
│       └── TERMINAL: Roll search guidance
│           - Action: search the electoral roll by name/EPIC/details
│           - Deep-link: voters.eci.gov.in (roll search), Voter Helpline app
│           - Note: outcome of search determines next step; app should
│             offer to re-enter the tree at branch B or C based on result
│
├── B. "I've never registered before" (new elector)
│   └── [question] "Do you currently live in India, or outside India?"
│       ├── B1. Outside India
│       │   └── [question] Confirm Indian citizenship retained (not
│       │       acquired foreign citizenship)
│       │       ├── Yes
│       │       │   └── TERMINAL: Form 6A (overseas elector registration)
│       │       │       - Docs (general categories): passport, proof of
│       │       │         the address in India to be registered against
│       │       │       - Citation: Registration of Electors Rules 1960
│       │       │         (as amended); voters.eci.gov.in overseas elector
│       │       │         guidance
│       │       │       - Deep-link: voters.eci.gov.in (Form 6A / NRI
│       │       │         registration)
│       │       │       - Caution: verify current overseas
│       │       │         registration/voting-method rules before
│       │       │         asserting specifics; this is an evolving policy
│       │       │         area
│       │       └── No (acquired foreign citizenship)
│       │           └── TERMINAL: Not eligible to register as an Indian
│       │               elector
│       │               - Deep-link: N/A; suggest contacting nearest
│       │                 Indian mission for citizenship-related queries
│       │               - Caution: this is outside VoteAssist's scope;
│       │                 do not speculate on citizenship law
│       │
│       └── B2. Inside India
│           └── [branch] Age / qualifying-date check
│               ├── [question] "What is your date of birth?"
│               │   └── [branch] Compute: is DOB >= 18 years before today?
│               │       ├── Not yet 18
│               │       │   └── TERMINAL: Not yet eligible
│               │       │       - Action: explain the four qualifying
│               │       │         dates (1 Jan, 1 Apr, 1 Jul, 1 Oct) and
│               │       │         show the user's likely future
│               │       │         qualifying date
│               │       │       - Citation: RP Act qualifying-date
│               │       │         provision; ECI/PIB announcement
│               │       │       - Deep-link: voters.eci.gov.in (advance
│               │       │         application guidance, if applicable —
│               │       │         verify whether advance application
│               │       │         ahead of the qualifying date is
│               │       │         supported before asserting)
│               │       └── 18 or older
│               │           └── [question] "What kind of place do you
│               │               currently live at?"
│               │               ├── Own home / long-term residence
│               │               │   └── TERMINAL: Form 6 (new
│               │               │       registration, ordinary residence)
│               │               ├── Rental / PG (non-student)
│               │               │   └── TERMINAL: Form 6 (new
│               │               │       registration); document category:
│               │               │       proof of ordinary residence at the
│               │               │       rented address (verify accepted
│               │               │       proof types against current Form 6
│               │               │       instructions)
│               │               ├── Hostel/mess as a student
│               │               │   └── [question] "Would you like to
│               │               │       register at your hostel/college
│               │               │       address, or your family/home
│               │               │       address?"
│               │               │       ├── Hostel/college address
│               │               │       │   └── TERMINAL: Form 6 at
│               │               │       │       hostel address
│               │               │       │       - Docs: bonafide
│               │               │       │         certificate from Head of
│               │               │       │         Institution (course
│               │               │       │         recognized by a Central/
│               │               │       │         State body, Board,
│               │               │       │         University or Deemed
│               │               │       │         University, minimum 1
│               │               │       │         year duration), plus
│               │               │       │         proof of ordinary
│               │               │       │         residence
│               │               │       │       - Citation: Registration
│               │               │       │         of Electors (Amendment)
│               │               │       │         Rules 2022; ECI SVEEP
│               │               │       │         student registration
│               │               │       │         guidance
│               │               │       │       - Deep-link:
│               │               │       │         voters.eci.gov.in
│               │               │       └── Family/home address
│               │               │           └── TERMINAL: Form 6 at home
│               │               │               address (standard path)
│               │               ├── Homeless / no fixed address
│               │               │   └── TERMINAL: Roll search + BLO/1950
│               │               │       consultation recommended
│               │               │       - Caution: proof-of-ordinary-
│               │               │         residence requirements for
│               │               │         electors without a fixed address
│               │               │         must be verified against current
│               │               │         ECI guidance; do not assert a
│               │               │         specific document list without
│               │               │         confirmation
│               │               │       - Deep-link: 1950 helpline, local
│               │               │         BLO contact via Voter Helpline
│               │               │         app "Book-a-Call with BLO"
│               │               └── Tribal/remote area with limited
│               │                   connectivity
│               │                   └── TERMINAL: Form 6, with BLO/1950/
│               │                       Voter Helpline app as the primary
│               │                       recommended channel (not
│               │                       assuming reliable self-service web
│               │                       access)
│
├── C. "I'm already registered, but I've moved"
│   └── [question] "Did you move within the same Assembly Constituency
│       (AC), or to a different AC (different city/state, or a different
│       part of the same city that falls in a different AC)?"
│       ├── Same AC
│       │   └── TERMINAL: Form 8 (shifting of residence, within AC)
│       │       - Citation: Registration of Electors (Amendment) Rules
│       │         2022 (Form 8A discontinued, merged into Form 8)
│       │       - Deep-link: voters.eci.gov.in
│       └── Different AC
│           └── TERMINAL: Form 8 (shifting of residence, cross-AC)
│               - Same citation; note that cross-AC shifts may take
│                 longer to process — verify current processing-time
│                 guidance before asserting a specific duration
│               - Deep-link: voters.eci.gov.in
│
├── D. "I need to correct something on my voter ID / roll entry"
│   └── [question] "What needs correcting?" (name / date of birth / photo
│       / relative's name / gender / other)
│       └── TERMINAL: Form 8 (correction of entries)
│           - Docs: proof of the correct value (general category; specific
│             accepted proofs vary by field — verify against current Form
│             8 instructions)
│           - Citation: Registration of Electors (Amendment) Rules 2022
│           - Deep-link: voters.eci.gov.in
│
├── E. "My voter ID is lost, damaged, or I never received it"
│   └── [question] "Would you like a digital copy now, or a replacement
│       physical card?"
│       ├── Digital copy (e-EPIC)
│       │   └── TERMINAL: e-EPIC download
│       │       - Requirement: OTP verification via registered mobile
│       │         number/email on voters.eci.gov.in
│       │       - Citation: ECI e-EPIC launch announcement (25 Jan 2021);
│       │         voters.eci.gov.in e-EPIC guidance
│       │       - Note: e-EPIC is legally equivalent to the physical card
│       │       - Deep-link: voters.eci.gov.in
│       └── Physical replacement
│           └── TERMINAL: Form 8 (replacement/duplicate EPIC — Form 001
│               discontinued, merged into Form 8)
│               - Citation: Registration of Electors (Amendment) Rules
│                 2022
│               - Deep-link: voters.eci.gov.in
│
├── F. "I want to report a name that shouldn't be on the roll" (death,
│   duplicate, disqualification, moved away)
│   └── [question] "What is the reason?" (deceased / duplicate entry /
│       moved away permanently / other disqualification)
│       └── TERMINAL: Form 7 (objection to inclusion / claim for
│           deletion)
│           - Docs: proof supporting the objection (e.g., death
│             certificate for a deceased-elector case — general category;
│             verify specifics against current Form 7 instructions)
│           - Citation: Registration of Electors (Amendment) Rules 2022
│           - Deep-link: voters.eci.gov.in
│           - Caution: this is a third-party objection process; the
│             product must not encourage frivolous or targeted misuse —
│             copy should emphasize this is for genuine roll-accuracy
│             corrections only
│
├── G. "I have a disability and want it recorded, and/or want to vote from
│   home"
│   └── [question] "Do you want to (a) mark PwD status on the roll, (b)
│       request home voting for an upcoming election, or (c) both?"
│       ├── Mark PwD status
│       │   └── TERMINAL: Form 8 (PwD marking)
│       │       - Citation: Registration of Electors (Amendment) Rules
│       │         2022
│       │       - Deep-link: voters.eci.gov.in
│       └── Request home voting
│           └── [branch] Check: PwD status already marked, or age 85+?
│               └── TERMINAL: Form 12D (postal ballot / home voting
│                   request)
│                   - Timing: must be submitted to the Returning Officer
│                     within 5 days of election notification
│                   - Citation: ECI/PIB home-voting facility
│                     announcements; RP Rules postal ballot provisions
│                   - Deep-link: CEO portal / Returning Officer contact
│                     via Voter Helpline app or 1950 helpline
│                   - Caution: exact eligibility and window should be
│                     re-verified each election cycle as procedures have
│                     evolved
│
└── H. "I don't know my constituency, polling station, or BLO"
    └── TERMINAL: Roll search / polling station locator
        - Deep-link: voters.eci.gov.in (search tools), Voter Helpline app
          (polling station lookup, BLO contact, "Book-a-Call with BLO"),
          1950 helpline
```

## 3. Terminal Outcome Contract

Every terminal node in the production knowledge base must populate the
following fields (see `09-knowledge-base-schema.md` for the exact schema):

1. `recommended_form` — one of: none, Form 6, Form 6A, Form 7, Form 8,
   Form 12D, or "consult BLO/1950" where no form applies.
2. `required_documents` — general categories only (e.g., "proof of age,"
   "proof of address"), never a fabricated exhaustive list, unless the
   specific list has been confirmed against an official source and cited.
3. `citation` — one or more entries from `citations.md`.
4. `deep_link_target` — one of: voters.eci.gov.in, ecinet.eci.gov.in,
   Voter Helpline app (store listing), a state CEO portal, or the 1950
   helpline.
5. `verification_caution` — standard boilerplate: "Rules and portal steps
   can change. Confirm current requirements on voters.eci.gov.in, ECINET,
   or by calling the National Voter Helpline 1950 before you apply."

## 4. Explicit Scope Statement

This tree is v1 coverage only. It intentionally does not yet encode:

- State-specific variations in accepted documents or procedures.
- Every service-voter subcategory (e.g., specific defence/paramilitary
  categories and their spouses) in full legal detail.
- Constituency-specific reservations or tribal-area special provisions
  beyond generic guidance to consult the local BLO/CEO.
- Any scenario not explicitly listed in branches A-H above.

Expanding this tree to genuinely cover "every legal scenario" requires
state-by-state legal review, tracked as a v1/v2 prerequisite in
`19-roadmap.md` and `06-legal-compliance-review.md`. Until that review is
complete, any gap the decision engine encounters must terminate in a
"consult your BLO or call 1950" outcome rather than a guessed answer.
