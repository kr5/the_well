# rules

Declarative, cited, explainable electoral eligibility/form-selection/deadline/MCC
rules for VoteAssist India.

Today, whether a citizen is eligible to register, which ECI form applies to
their situation, and what deadline governs their filing are decided
implicitly — as hardcoded branches inside `core-domain`'s decision-tree JSON
and as prose inside the `kb-content` knowledge base. There was no single
place to ask "is this person eligible?" or "which form do they need?" and get
a machine-checkable, auditable answer. This crate is that place.

It is pure, zero-I/O, wasm32-safe logic — no `tokio`, `sqlx`, filesystem, or
network access — matching the discipline of the sibling `core-domain` and
`jurisdiction` crates, so it can be a plain dependency of the Leptos web app
(`crates/web-app`) and admin app (`crates/admin-app`) alike.

## What this crate models

- **`eligibility`** — age (18+ as on the applicable qualifying date),
  citizenship, ordinary residence (including the student/temporary-absence/
  overseas-elector-under-s.20A nuances), and the two statutory
  disqualifications (unsound mind declared by a competent court; disqualified
  for a corrupt practice/electoral offence). Grounded in Representation of
  the People Act, 1950 ss. 14(b), 16, 19, 20, 20A.
- **`qualifying_date`** — the four annual qualifying dates (1 January, 1
  April, 1 July, 1 October — since the Election Laws (Amendment) Act, 2021 /
  Registration of Electors (Amendment) Rules, 2022 replaced the old
  single-1-January rule) and the computation of which one applies to an
  assessment made on a given date.
- **`forms`** — which ECI form a citizen's situation calls for: Form 6 (new
  registration), Form 6A (overseas/NRI elector), Form 7 (objection to
  inclusion / deletion), Form 8 (shifting of residence, correction, EPIC
  replacement, PwD marking — the post-2022-consolidated form that absorbed
  the old Form 8A), Form 12D (postal ballot for 85+/PwD/essential-service/
  other ECI-notified absentee categories), and Form 2/2A (service voters).
- **`deadlines`** — the *shape* of deadline reasoning (is a claims-and-
  objections window still open; is a roll frozen ahead of a specific poll),
  built around an injected elections-calendar input rather than any
  hardcoded date (see "What this crate deliberately does NOT decide" below).
- **`mcc`** — whether the Model Code of Conduct is currently in force for a
  state, read from an injected `mcc_windows`-shaped input, kept consistent
  with `channel-core`'s broadcast gate (`channel_core::mcc_gate`) and the
  admin MCC panel (`admin_app::pages::mcc_panel`).
- **`citation`** and **`trace`** — the cross-cutting machinery every rule
  category above is built on: every rule's claim carries a `Citation` (a
  knowledge-base entry id, a statute/section, or an honestly-flagged
  "missing"), and every evaluation returns a tri-state `Verdict`
  (`Satisfied` / `NotSatisfied` / `CannotDetermine`) plus a full
  `EvaluationTrace` of every rule that ran.

## What this crate deliberately does NOT decide

- **It never submits, files, or transmits anything.** It is pure
  computation over the input it's given — no form gets sent, no
  registration gets created.
- **It never gives legal advice.** A `Satisfied` overall verdict means
  "appears to meet this rule based on what was supplied," not a legal
  determination. The Electoral Registration Officer remains the actual
  decision-maker for registration; the ECI remains the actual authority on
  everything this crate reasons about. Any citizen-facing surface built on
  this crate should say so.
- **It never invents a statutory number it isn't confident of.** The exact
  "last date for filing claims and objections" in a roll-revision round, and
  the exact date a roll is frozen ahead of a specific poll, are fixed by ECI/
  ERO notification per round/per election and genuinely vary — this crate
  does not guess a plausible one. `deadlines::ElectionsCalendarInput` takes
  those dates as input; if they're not supplied, the relevant rule reports
  `Verdict::CannotDetermine`, never a fabricated deadline.
- **It never recomputes MCC status from dates.** `mcc::evaluate_mcc_applicability`
  trusts the same `is_active` flag `channel-core`'s broadcast gate already
  trusts (set by a legal reviewer via the admin MCC panel) rather than
  independently reinterpreting `window_start`/`window_end` — two disagreeing
  computations of the same fact would be worse than one shared source of
  truth being wrong.

## The tri-state `Verdict`

Every rule in this crate evaluates to one of three states, never a bare
`bool`:

- `Verdict::Satisfied`
- `Verdict::NotSatisfied`
- `Verdict::CannotDetermine` — insufficient input to evaluate this rule at
  all.

A missing input must never silently collapse into "not eligible." A citizen
who didn't answer a question (or whose channel — IVR, WhatsApp — never
collected a field) must be told "we don't have enough information," never
"you appear ineligible." Every rule function in this crate returns
`CannotDetermine` when a required input is `None`, as a matter of type-level
construction, not convention — see `EvaluationTrace::overall`'s doc comment
for exactly how the three states combine across a rule set (a confirmed
`NotSatisfied` always wins over an unrelated unknown; otherwise any unknown
wins over a bare `Satisfied`).

## Explainable traces

Every evaluation function returns an `EvaluationTrace`: the full, ordered
list of every rule that ran, each with its own `rule_id`, `description`,
`Citation`, `Verdict`, and a plain-language `detail` filled in for that
specific evaluation (the actual computed qualifying date, the actual age,
etc.). This is what lets a citizen-facing surface say "you appear eligible
BECAUSE x, y, z" and lets an admin audit exactly why the engine said what it
said — down to the citation backing each claim. This is designed in from the
start, not bolted on: there is no code path in this crate that produces a
bare verdict without the trace that explains it.

## How to add a rule

1. Pick the right module (`eligibility`, `forms`, `deadlines`, or `mcc`) — or
   propose a new one if the rule genuinely doesn't fit any existing
   category.
2. Add a `pub const RULE_...: &str = "..."` id, following the existing
   `category.snake_case_name` convention.
3. Write a function `fn evaluate_your_rule(input: &...) -> RuleStep` that:
   - returns `Verdict::CannotDetermine` for every input combination where a
     required field is `None` — never guess;
   - attaches a real `Citation` (`Citation::kb("...")` for a knowledge-base
     entry id that actually exists in `kb-content`, or `Citation::statute(...)`
     for an Act/Rule section) — or, if you genuinely don't have one yet,
     `Citation::missing("...")` with an honest reason, never a fabricated
     citation;
   - fills in `detail` with the specific reasoning for this evaluation, not
     just a repeat of `description`.
4. Wire it into that module's `evaluate_...` driver function (e.g.
   `eligibility::evaluate_eligibility`) so it always runs alongside the
   others.
5. Add tests: at minimum, the `Satisfied` case, the `NotSatisfied` case, and
   the `CannotDetermine` case for every field your rule reads. If the rule
   involves a date boundary, test the boundary explicitly (see
   `qualifying_date`'s per-qualifying-date boundary tests for the pattern).
6. If your rule cites a knowledge-base entry, add it to
   `tests/citations_resolve.rs`'s exercised set so the cross-crate check
   catches it if that entry is ever renamed or removed.
