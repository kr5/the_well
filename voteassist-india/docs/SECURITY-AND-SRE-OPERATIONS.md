# VoteAssist India — Security & Site Reliability Operations

**Status:** Draft v1.0 — concrete operational plan, complementing (not
replacing) `docs/PRD-V2-RUST-PLATFORM.md` Section 13 (Security Threat
Model) and Section 16 (CI/CD). Where PRD v2 established *what* the threat
model and CI gates are, this document specifies *how the system runs in
production*: named tools, concrete configs, response processes, and
verified-current tool choices (every tool below was checked against its
actual 2026 maintenance status before being recommended — see the note on
Grafana OnCall below for why that check matters).
**Last updated:** 2026-07-23

## Table of Contents

1. Why this document exists (and what changed since PRD v2)
2. Secrets Management
3. Container & Supply-Chain Security
4. Runtime Security Monitoring
5. Backup, Disaster Recovery & Point-in-Time Restore
6. Observability Stack (concrete configuration)
7. Uptime Monitoring & Public Status Page
8. Incident Response & On-Call
9. SLOs, SLIs, and Error Budgets
10. Load & Capacity Testing (election-day readiness)
11. Vulnerability Disclosure & Penetration Testing Cadence
12. Runbook Template & Worked Example
13. Data Classification Cross-Reference

---

## 1. Why this document exists

PRD v2 Section 13's threat model correctly names the risk categories
(spoofing, tampering, DoS, etc.) but stops short of naming the actual
tools and processes that make those mitigations real rather than aspirational.
"Self-hosted Grafana/Loki/Tempo" is an architecture decision; it is not yet
an operations plan. This document closes that gap.

It is also a worked example of a discipline this whole project claims to
value: **verify before you commit**. While researching this document, one
previously-plausible recommendation — Grafana OnCall as the on-call/
escalation tool — turned out to be wrong: **Grafana OnCall (OSS) entered
maintenance mode on 2025-03-11 and was archived on 2026-03-24**, with its
SMS/phone/push notification relay (which depended on Grafana Cloud)
discontinued at the same time. A plan written even a few months earlier
without re-checking would have shipped a dead dependency into a safety-
critical on-call path. Section 8 below names the tool actually chosen
instead (GoAlert) and why.

## 2. Secrets Management

**Tool: SOPS (Secrets OPerationS) + `age` as the encryption backend.**
SOPS is a CNCF Sandbox project and has adopted `age` as a first-class
encryption backend, making the combination the de facto standard for
file-level secret encryption in GitOps-style workflows as of 2026. Chosen
over HashiCorp Vault for this project specifically because:

- Vault is the right tool for a team that needs dynamic secret generation,
  fine-grained runtime access policies, and automatic rotation at scale —
  capabilities VoteAssist does not need at MVP-Rust-v1/v1 scale (a handful
  of services, a handful of secrets: DB credentials, WhatsApp/Telegram bot
  tokens, session-signing material if ever added).
- SOPS lets encrypted secret files sit directly in the Git repository
  (`secrets/*.enc.yaml`), decrypted only at deploy time by a CI job or an
  operator holding the `age` private key — no separate always-on service to
  operate, patch, and secure, which matters for a volunteer-operated
  project with no dedicated platform team.
- If the project's operational maturity later demands centralized dynamic
  secrets (e.g., once `crates/admin-app` has many non-founder operators
  needing scoped, auditable, rotatable database credentials), migrating to
  Vault or a managed equivalent (e.g., Infisical) is a deliberate future
  decision, not something to prematurely build now.

**Concrete workflow:**
- One `age` keypair per environment (dev/staging/prod), private keys held
  only by the deploying CI runner (as a GitHub Actions encrypted secret)
  and by a small number of named maintainers for break-glass access — never
  by every contributor.
- `.sops.yaml` at the repo root pins which `age` public keys can decrypt
  which path globs, so a contributor's PR can add a new *encrypted* secret
  file without ever having the ability to decrypt existing ones.
- CI decrypts secrets into environment variables at deploy time only,
  never writes them to a build artifact or log; `cargo` build logs and
  `docker build` output are treated as a leak surface and reviewed for
  accidental secret echo as part of the CI pipeline definition itself.
- Rotation policy: DB credentials and bot tokens rotated on any suspected
  exposure immediately, and on a routine schedule otherwise (recommend
  every 180 days, tied to the same cadence as the KB re-verification job
  from PRD v2 Section 6.8, for operational simplicity — one calendar
  reminder, not two).

## 3. Container & Supply-Chain Security

Layered, each layer catching what the previous one can't:

| Layer | Tool | What it catches |
|---|---|---|
| Rust dependency vulnerabilities | `cargo-audit` (RustSec advisory database) | Known CVEs in any crate in `Cargo.lock` |
| Rust dependency policy | `cargo-deny` | License violations, duplicate/banned crate versions, unmaintained crates flagged by RustSec |
| Container image vulnerabilities | **Trivy** (Aqua Security) | OS-package and language-dependency CVEs inside the built distroless images; also generates an SBOM per image | 
| Container image provenance | Cosign (Sigstore) — v1-roadmap item | Signs each built image so a deploy step can verify it wasn't tampered with between CI and production |
| Application-layer dynamic scanning | OWASP ZAP, run against a staging deployment | Runtime web vulnerabilities (injection, misconfigured headers, etc.) that static analysis can't see |

Trivy runs as a required CI job on every image build (blocking merge on
HIGH/CRITICAL findings with no available fix; findings with an available
fix become a required-before-merge dependency bump). This is in addition
to, not instead of, the `cargo-audit`/`cargo-deny` gates already specified
in PRD v2 Section 16 — Trivy catches OS-level and transitively-vendored
issues inside the final container image that a pure Rust-dependency scan
cannot see (e.g., a vulnerable version of a system library baked into the
distroless base image).

OWASP ZAP runs on a schedule (weekly) against the staging deployment, not
on every PR — a full DAST pass is too slow for per-PR CI, so it's a
recurring job whose findings feed the same triage process as any other
security finding (Section 11).

## 4. Runtime Security Monitoring

**Tool: Falco** (CNCF graduated project — the highest CNCF maturity tier,
meaning production-hardened and broadly adopted; not a fringe or
experimental tool). Falco does syscall-level runtime threat detection:
it can flag things static/pre-deploy scanning cannot, such as an unexpected
process spawning inside a container, an unexpected outbound network
connection from the `api` service, or a write to a path that container
should never write to.

Why this matters specifically for VoteAssist: the `api`/`jobs`/bot-adapter
containers have a narrow, predictable behavioral envelope (they talk to
Postgres, Meilisearch, and a small allowlist of external APIs — WhatsApp
Cloud API, Exotel, Bhashini per PRD v3 Section V2 — and nothing else).
Falco rules are written to that narrow envelope, so almost any deviation is
a genuine signal, not noise — a favorable ratio that's harder to achieve
for a general-purpose application with a wide legitimate behavioral range.

Concrete deployment: Falco runs as a sidecar/daemonset (or, pre-Kubernetes,
as a host-level agent) watching the containers described above, exporting
alerts via its Prometheus/webhook output straight into the Grafana Alerting
pipeline (Section 6), not as a separate, unmonitored tool on the side.

## 5. Backup, Disaster Recovery & Point-in-Time Restore

**Tool: pgBackRest** for Postgres physical backups + continuous WAL
archiving, enabling point-in-time recovery (PITR) and parallel restore.
Chosen over WAL-G specifically because pgBackRest's block-level (not
file-level) incremental backups and multi-repository support fit a
single-Postgres-instance-per-environment deployment better; WAL-G remains
the noted lighter-weight alternative if the project ever moves to a
cloud-object-storage-first, multi-database-engine deployment model where
its simplicity is the better trade.

**Transparency note, in the same spirit as Section 1:** pgBackRest went
through a real maintenance funding crisis in April-May 2026 before being
rescued by a sponsor coalition (AWS, Supabase, Percona, pgEdge, Tiger Data,
Eon.io) that diversified its funding model. This is disclosed here
deliberately rather than glossed over: it is evidence the tool is
currently healthy, but it is also a reminder that even foundational
infrastructure tooling can hit funding/maintenance risk, which is exactly
why Section 9's SLOs and this section's restore-drill discipline (below)
must never assume any single tool's continuity without a periodic
re-check — the same discipline applied to catch the Grafana OnCall
archival in Section 1.

**Concrete backup policy:**
- Full base backup: daily.
- Continuous WAL archiving: real-time, to a separate storage target from
  the primary database volume (never co-located, so a single storage
  failure can't take out both the live database and its backups).
- Retention: 30 days of PITR-capable history, consistent with PRD v3's
  `analytics_events` retention discipline for the same "don't keep more
  than the stated policy" reason — but note backups are an operational
  necessity distinct from the product's own data-minimization commitments;
  a backup retention window is not a data-retention-policy violation as
  long as it's disclosed (mirrors the reasoning in PRD v3 Section V6.4's
  backup-retention exception for user account deletion).
- **Restore drills**: a quarterly, calendared, non-optional exercise —
  restore the latest backup into a scratch environment and verify the
  application boots against it and basic queries return sane data. A
  backup that has never been restored is not a verified backup; this is
  the single most commonly-skipped SRE practice and is called out
  explicitly here so it isn't skipped by omission.
- RTO (Recovery Time Objective) target: 4 hours for a full-outage restore.
  RPO (Recovery Point Objective) target: 5 minutes (bounded by WAL
  archiving frequency), consistent with the SLOs in Section 9.

## 6. Observability Stack (concrete configuration)

Builds directly on PRD v2 Section 6.12's architecture decision
(`tracing` + OpenTelemetry OTLP → self-hosted Grafana/Loki/Tempo). Concrete
pieces:

- **Metrics**: Prometheus scrapes each service's `/metrics` endpoint
  (exposed via the `axum-prometheus` or equivalent middleware crate);
  Grafana visualizes.
- **Logs**: structured JSON logs (via `tracing-subscriber`'s JSON
  formatter) shipped to Loki via Promtail or the OTLP log exporter.
- **Traces**: OTLP trace export to Tempo, correlated with logs via
  trace-id injection into the structured log format.
- **Alerting**: Grafana's built-in unified alerting (Grafana Alerting), NOT
  a separately-run Alertmanager instance — Grafana Alerting has absorbed
  Alertmanager's functionality and keeps the operational surface to one
  fewer service for a small team to run.

Example alert rule (as Grafana provisioning YAML, illustrative):

```yaml
apiVersion: 1
groups:
  - orgId: 1
    name: voteassist-api-slos
    folder: VoteAssist
    interval: 1m
    rules:
      - uid: api-p99-latency
        title: "API p99 latency above SLO"
        condition: C
        data:
          - refId: A
            datasourceUid: prometheus
            model:
              expr: histogram_quantile(0.99, rate(http_request_duration_seconds_bucket{job="voteassist-api"}[5m]))
          - refId: C
            datasourceUid: __expr__
            model:
              type: threshold
              conditions:
                - evaluator:
                    type: gt
                    params: [0.3]
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "voteassist-api p99 latency exceeded 300ms for 5 minutes"
      - uid: api-error-rate
        title: "API 5xx error rate above 1%"
        condition: C
        data:
          - refId: A
            datasourceUid: prometheus
            model:
              expr: >
                sum(rate(http_requests_total{job="voteassist-api", status=~"5.."}[5m]))
                / sum(rate(http_requests_total{job="voteassist-api"}[5m]))
          - refId: C
            datasourceUid: __expr__
            model:
              type: threshold
              conditions:
                - evaluator: { type: gt, params: [0.01] }
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "voteassist-api 5xx rate above 1% for 5 minutes"
```

## 7. Uptime Monitoring & Public Status Page

**Tool: Uptime Kuma** (self-hosted, actively maintained — version 2.1
shipped February 2026 with Globalping worldwide-probe support and domain-
expiry monitoring; 2.0 in October 2025 added MariaDB support and rootless
Docker images, evidence of ongoing, healthy maintenance rather than a
stalled project). Chosen over building a custom status-check service
because this is a genuinely solved, low-differentiation problem — writing
a bespoke uptime monitor would be effort spent on something that doesn't
advance VoteAssist's actual mission.

Concrete use: monitors public-facing endpoints (`/healthz` on `api`, the
public web app's homepage, the Meilisearch health endpoint from inside the
private network) and publishes an external-facing status page — itself a
small trust-building feature (a civic tool that's transparent about its own
uptime, including during incidents, is more credible than one that goes
silent). Notification channel: routed into the same on-call escalation
path as Section 8, not a separate silo.

## 8. Incident Response & On-Call

**Tool: GoAlert** (originally built by Target's engineering team, Go-based,
Postgres-backed, single-binary deployment) for on-call scheduling and
escalation — the direct, currently-maintained replacement for the
now-archived Grafana OnCall (Section 1). GoAlert's Postgres-backed,
single-binary deployment model is also a natural operational fit alongside
this project's own Rust services, which share the same "one binary + one
Postgres instance" deployment philosophy (PRD v2 Section 6.16).

**Severity levels** (adapted for a volunteer-operated civic project, not a
24/7 commercial SaaS):

| Severity | Definition | Response expectation |
|---|---|---|
| SEV1 | Public site fully down, or the decision engine gives incorrect guidance affecting live users | Immediate — page the on-call rotation regardless of time |
| SEV2 | Significant degradation (e.g., one channel down, elevated error rate, KB search unavailable) but core guidance flow still works | Respond within business hours same day; page after hours only if it's actively worsening |
| SEV3 | Isolated, non-critical issue (e.g., one admin page broken, a single dead citation link) | Next business day, tracked as a normal issue |
| SEV4 | Cosmetic / no functional impact | Backlog |

**On-call rotation for a volunteer OSS project**: this is explicitly
flagged as an organizational, not just technical, open question (see PRD
v2 Section 22 / PRD v3 Section V9's pattern of naming real risks rather
than assuming them away) — a project without paid, dedicated staff cannot
assume 24/7 on-call coverage the way a commercial product might. The
realistic model: a small rotation of maintainer-volunteers covers SEV1/SEV2
during announced availability windows (not claimed as 24/7), with GoAlert
configured to escalate through the rotation and, if unacknowledged, fall
back to a shared notification channel (e.g., a project Slack/Discord) so
an incident is never silently missed even if the formal on-call chain
doesn't answer.

**Postmortem process**: every SEV1/SEV2 incident gets a blameless written
postmortem (what happened, timeline, root cause, remediation items with
owners) within 5 business days, filed in the repository (not lost in chat
history) so the "self-improving system" ethos from PRD v3 Section V5
applies to operations, not just content.

## 9. SLOs, SLIs, and Error Budgets

| Service Level Indicator | Objective | Rationale |
|---|---|---|
| Public site availability | 99.5% monthly | Realistic for a volunteer-operated, self-hosted deployment; not overpromising commercial-grade 99.9%+ the project can't credibly staff for |
| `api` p99 latency (decision-engine endpoints) | < 300ms | Generous enough to accommodate modest hosting hardware (PRD v2 Section 6.16's low-cost-binary argument), tight enough to feel instant on a 2G/3G connection where the network hop dominates anyway |
| Election-day traffic-spike headroom | Sustain 10x baseline peak concurrent sessions without breaching the above latency SLO | Directly ties to PRD v2 Section 1.2's stated rationale for choosing Rust in the first place — this SLO is the concrete test of whether that architectural bet paid off |
| KB search p99 latency | < 500ms | Meilisearch-backed, generous headroom for typo-tolerant fuzzy matching |
| Backup restore drill success rate | 100% (every quarterly drill succeeds) | A failed drill is itself the finding, tracked as a SEV2 incident, not silently retried and forgotten |

**Error budget policy**: if the public-site availability SLO is breached in
a given month, new feature work on `crates/web-app`/`crates/api` pauses in
favor of reliability work until the trend recovers — a standard SRE
error-budget practice, scaled to this project's volunteer capacity (i.e.,
"pause" means "the next volunteer PR review cycle prioritizes reliability
fixes," not a formal freeze enforced by tooling).

## 10. Load & Capacity Testing (election-day readiness)

**Tool: k6** (Grafana Labs) for scripted load testing. Distinct from
Grafana OnCall's fate (Section 1) — k6 is a separately-maintained, widely-
adopted core product in active use, but the same discipline applies: its
maintenance status should be re-checked at the time this plan is actually
executed, not assumed indefinitely from this document.

**Concrete test scenario**, tied directly to the traffic-spike rationale in
PRD v2 Section 1.2: simulate the announcement-day surge pattern — a sharp
ramp from baseline to 10x concurrent sessions within a short window (e.g.,
15 minutes), sustained for several hours, predominantly simple decision-
engine session start/answer calls (the actual real-world traffic shape),
with a smaller proportion of KB search calls. Success criteria: the p99
latency SLO (Section 9) holds throughout, and no 5xx-rate SLO breach
(Section 6's alert rule) fires.

**Cadence**: before any election-adjacent period where a meaningful
traffic spike is anticipated (tied to PRD v3 Section V3.6's tracked-
elections calendar — a scheduled election is the trigger to re-run this
test, not an arbitrary calendar cadence), and after any significant
architecture change to `crates/api`.

## 11. Vulnerability Disclosure & Penetration Testing Cadence

- **Vulnerability disclosure**: a `SECURITY.md` in the repository root with
  a named contact/reporting channel, a commitment to acknowledge reports
  within 5 business days (faster than GitHub's default expectations,
  matching this project's pattern elsewhere of voluntarily exceeding
  minimums — see PRD v3 Section V6.4's account-deletion commitment for the
  same pattern), and a coordinated-disclosure norm (no public disclosure
  before a fix ships or 90 days pass, whichever comes first — the same
  90-day figure DPDP uses for data-principal requests, chosen here for
  consistency of mental model, not because it's legally required for
  vulnerability disclosure specifically).
- **Penetration testing cadence**: an external security review before any
  v1 public launch beyond the MVP-Rust-v1 parity milestone (this is
  already listed as PRD v2 EPIC 13, task E13.F1.T6) — this document adds
  the concrete cadence going forward: annually thereafter, or triggered by
  any architecture change that adds a new trust boundary (e.g., shipping
  the optional-accounts feature from PRD v3 Section V6 is exactly the kind
  of change that should trigger an out-of-cycle review, not wait for the
  annual date).

## 12. Runbook Template & Worked Example

Every SEV1/SEV2-capable failure mode should have a runbook in the
repository under `runbooks/`, following this template:

```markdown
# Runbook: <failure mode name>

## Symptoms
What an on-call responder actually sees (alert name, dashboard panel,
user reports).

## Immediate mitigation
The fastest safe action to reduce user impact, even if not a full fix
(e.g., "roll back the last deploy," "fail over to the read replica").

## Diagnosis steps
Ordered list of what to check, with links to the specific Grafana
dashboards/Loki queries to run.

## Resolution
How to actually fix the root cause once identified.

## Escalation
Who to page if the above doesn't resolve it within <time>.

## Postmortem trigger
Confirmation this is a SEV1/SEV2 requiring the Section 8 postmortem process.
```

**Worked example — "decision-engine sessions returning 500s":**

- **Symptoms**: the `api-error-rate` alert (Section 6) fires; Uptime Kuma
  (Section 7) shows `/healthz` still green (so the process is alive, the
  issue is request-path-specific).
- **Immediate mitigation**: check whether a decision-tree publish (PRD v2
  Section 11 admin page 4) happened in the last hour — a newly published
  tree that failed `validate_tree()` should never reach production per
  PRD v2's publish-gate design, but if it somehow did, rolling back to the
  prior published tree version is the fastest mitigation.
- **Diagnosis steps**: check `tracing` spans in Tempo for the failing
  request path; check whether the failure correlates with a specific
  `node_id` (would point to bad tree content) versus being uniform across
  all requests (would point to an infrastructure issue, e.g., the API
  service losing its in-memory tree cache and failing to reload it).
- **Resolution**: if tree-content-related, fix via the admin content
  pipeline (not a hotfix deploy — content bugs get fixed as content, code
  bugs get fixed as code, keeping the two paths distinct per PRD v2's
  whole content-vs-code separation philosophy). If infrastructure-related,
  standard service restart/redeploy.
- **Escalation**: page the on-call rotation (Section 8) if unresolved
  within 30 minutes, given this is a SEV1 (core guidance flow broken).
- **Postmortem trigger**: yes, SEV1.

## 13. Data Classification Cross-Reference

This document's tools handle data whose classification is already defined
in PRD v2 Section 13's Data Classification table and PRD v3 Section V6.6 —
this section only cross-references, not redefines: backups (Section 5)
inherit the retention/sensitivity of whatever they contain (so a
`saved_drafts`-inclusive backup is exactly as sensitive as PRD v3 Section
V6.6 says that table is, and must be encrypted at rest with the same rigor);
logs/traces (Section 6) must never include full `answer_history` payloads
or any account-identifying data in a log line — structured logging fields
are allow-listed, not free-form, specifically to prevent a well-intentioned
`tracing::info!("{:?}", state)` from leaking a user's decision-tree answers
into Loki, where the tighter retention/encryption discipline of the primary
database doesn't automatically apply. This is called out explicitly because
it is a realistic, easy-to-introduce mistake, not a hypothetical one.
