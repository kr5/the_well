# Security Threat Model (STRIDE)

## 1. Data Classification Table

| Data category | Examples | Classification | Handling rule |
|---|---|---|---|
| Public content | KB entries, form explainers, citations | Public | No restriction; cacheable, CDN-served |
| Anonymous session/analytics data | `decision_sessions`, `decision_answers`, `deep_link_clicks` | Internal, aggregate-sensitive | No PII by design; aggregated for reporting; retention-limited (see `18-analytics-plan.md`) |
| Optional user-provided contact info | Feedback `contact_email`, optional account `email` | Personal data (low-sensitivity) | DPDP-minimal collection, purpose-limited, deletable on request |
| Never collected | Aadhaar number, EPIC number, passport number, caste, religion, party affiliation, political opinion, uploaded ID documents | Prohibited | Must never be present in any schema, log, or analytics event; treat any accidental capture as a security incident requiring immediate deletion and root-cause fix |
| Infrastructure secrets | API keys, database credentials, signing keys | Confidential | Standard secrets management (vault/CI secrets), never committed to the repo, rotated on suspected exposure |

## 2. STRIDE Analysis

### 2.1 Spoofing

- **Primary risk: phishing / ECI-lookalike impersonation.** The single
  biggest brand-adjacent risk for this product is a *third party* (not
  VoteAssist itself) building a convincing VoteAssist-lookalike or
  ECI-lookalike site to phish for Aadhaar/EPIC numbers or credentials,
  potentially trading on VoteAssist's own name recognition once it exists.
  - Mitigations: a strong, persistent, non-dismissible-past-minimized
    "we are not the ECI" banner (see `06-legal-compliance-review.md` and
    `07-design-system.md`) reduces the chance a user mistakes an actual
    phishing clone of *voters.eci.gov.in* for something legitimate,
    because VoteAssist consistently models "always double check you're on
    the real official domain" as a habit. VoteAssist never asks users to
    enter Aadhaar/EPIC/passport numbers itself, so there's no VoteAssist
    login flow for a phisher to usefully imitate.
    - No login wall that mimics government authentication (no fake
    Digilocker/Aadhaar-OTP-style login screens, ever) — VoteAssist has no
    reason to ever request such credentials, and the product must treat
    "add a login step" proposals with suspicion precisely because it
    increases spoofing surface for no product benefit.
  - Domain/branding hygiene: register look-alike domain variants
    defensively where feasible (a business/ops task, tracked outside this
    engineering doc) and monitor for third-party impersonation.
- **Secondary risk: spoofed API clients.** Since core endpoints require no
  auth (`10-api-specification.md`), rate limiting and bot-detection
  heuristics (not full auth) are the main defenses against abusive
  scripted use — see denial-of-service section below.

### 2.2 Tampering

- **Decision-tree content tampering.** Because decision-tree terminal
  outcomes drive real user action, unauthorized modification of
  `packages/decision-engine` or `knowledge-base/sources/*` content is a
  high-impact risk. Mitigation: all changes go through PR review and the
  content-review process in `06-legal-compliance-review.md`; branch
  protection (`15-cicd.md`) prevents direct pushes to `main`; the
  decision-tree version is recorded per session so any bad content that
  did ship is traceable to an exact reviewed version.
- **Client-side tampering with answer submission.** A malicious client
  could submit an `answer_value` outside the valid enum for a node.
  Mitigation: server-side validation in `packages/api` against the
  decision-engine's defined node schema, never trusting client-supplied
  branching logic (see `08-database-schema.md` notes on
  `decision_answers.answer_value`).
- **Supply-chain tampering.** Compromised npm dependency. Mitigation:
  lockfile-pinned installs (`pnpm install --frozen-lockfile` in CI),
  automated dependency update review (`15-cicd.md`), and avoiding
  unnecessary third-party runtime dependencies in
  `packages/decision-engine` specifically, given its outsized importance.

### 2.3 Repudiation

- Low relevance for an anonymous-by-default product with no user
  authentication for core flows. For the content-review audit trail
  (who approved what content change), Git/PR history in the repository
  serves as the repudiation-resistant log (see
  `06-legal-compliance-review.md` section 7).
- Feedback submissions with an optional `contact_email` are not treated as
  a non-repudiation mechanism (email is easily spoofed/omitted) — no
  security decision should depend on trusting an unauthenticated
  `contact_email` value.

### 2.4 Information Disclosure

- **Core constraint: minimize what there is to disclose.** Because no
  Aadhaar/EPIC/passport numbers, no political-opinion data, and no ID
  documents are ever collected (see `01-prd.md`, `08-database-schema.md`),
  the "blast radius" of any future data breach is deliberately limited to,
  at most, anonymous session analytics and optionally-provided email
  addresses.
- **No cross-session political profiling, ever**, even in aggregate form —
  see `18-analytics-plan.md` for the specific metrics allowed and
  disallowed.
- Standard web app protections apply regardless: TLS everywhere, no
  sensitive data in URLs/query strings, no verbose error messages leaking
  internal state to clients, database credentials/secrets never logged.
- Third-party deep-links must not leak session/answer data to the
  destination site via referrer headers beyond what's unavoidable
  (consider `Referrer-Policy: strict-origin-when-cross-origin` or
  stricter) — the user's specific situation (e.g., "PwD home voting
  request") should not be inferable by voters.eci.gov.in or any other
  destination from the referring URL.

### 2.5 Denial of Service

- Public, unauthenticated API surface (`10-api-specification.md`) is
  inherently exposed to scripted abuse. Mitigations: per-IP rate limiting
  on session creation and feedback submission, tuned generously to avoid
  penalizing legitimate shared-IP usage (public libraries, cyber cafes —
  relevant given the target user base per `02-personas.md`), CDN/edge
  caching for public KB content, and standard infrastructure-level DDoS
  protection from the hosting provider.
- Because the product's value depends on availability during
  high-salience moments (e.g., approaching a registration deadline),
  availability itself is a mission-relevant property, not just a generic
  ops concern.

### 2.6 Elevation of Privilege

- MVP has no user roles/privilege tiers in the public product (no admin
  panel exposed publicly). Any future admin/content-management interface
  (e.g., a CMS for `knowledge-base/` content) must be:
  - Separately authenticated (not reachable via the same anonymous public
    API surface).
  - Scoped so that a compromised content-editor credential cannot alter
    infrastructure/deployment configuration, only content within the
    review-gated workflow.
  - Subject to the same content-review sign-off requirements regardless
    of who has technical write access (a compliance control, not just a
    technical one — see `06-legal-compliance-review.md`).

## 3. Summary Priorities

1. Keep the product's actual data footprint minimal — the strongest
   mitigation against most of the above is simply not holding sensitive
   data in the first place.
2. Treat decision-tree and knowledge-base content integrity as a
   security-relevant concern, not just an editorial one, given real-world
   impact of bad guidance.
3. Assume third-party phishing/impersonation attempts against ECI's real
   systems will occur regardless of VoteAssist's existence, and design
   VoteAssist's own messaging to reinforce (never undermine) users'
   ability to distinguish official from non-official destinations.
