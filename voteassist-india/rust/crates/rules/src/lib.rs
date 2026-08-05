//! Declarative, cited, explainable electoral rules for VoteAssist India.
//!
//! # Why this crate exists
//!
//! Today, whether a citizen is eligible to register, which ECI form
//! applies to their situation, and what deadline governs their filing are
//! decided implicitly — as hardcoded branches inside `core-domain`'s
//! decision-tree JSON, and as prose inside the `kb-content` knowledge
//! base. There is no single place to ask "is this person eligible?" or
//! "which form do they need?" and get a machine-checkable, explainable
//! answer, and no way to audit *why* a given branch of the decision tree
//! said what it said beyond reading the tree JSON by eye.
//!
//! This crate is that place. It formalizes those rules — grounded in the
//! Representation of the People Act, 1950 and 1951, and ECI's published
//! procedure — as data: each [`trace::RuleStep`] a rule function produces
//! carries a stable id, a human-readable description of what it tests, a
//! [`citation::Citation`] for what backs it, and a [`trace::Verdict`].
//!
//! # What this crate does NOT do
//!
//! - It never submits, files, or transmits anything. It is pure
//!   computation over the input it is given.
//! - It never gives legal advice. A `Satisfied` verdict means "appears to
//!   meet this rule based on what was supplied," not a legal
//!   determination — the Electoral Registration Officer (or, for MCC, the
//!   ECI/legal-review process) remains the actual authority.
//! - It never invents a number it isn't confident of. Where the real
//!   answer depends on a specific ECI notification that varies by round,
//!   state, or election (see `deadlines`), this crate takes that number as
//!   an injected input and says so in its doc comments, rather than
//!   guessing a plausible one.
//! - It performs no I/O. No filesystem, network, or database access,
//!   matching `core-domain` and `jurisdiction`'s discipline so it can be a
//!   plain, wasm32-safe dependency of the Leptos web app.
//!
//! # The tri-state [`trace::Verdict`]
//!
//! This crate's single most important design decision: every rule
//! evaluates to `Satisfied`, `NotSatisfied`, or `CannotDetermine` — never
//! just a `bool`. A missing input (a question the citizen didn't answer,
//! a field a channel like IVR never collected) must never silently render
//! as "not eligible" — that is the worst failure mode a civic-information
//! tool like this one can have, misinforming a citizen that they cannot
//! vote when the truth is simply "we don't know yet." See
//! `trace::EvaluationTrace::overall` for exactly how the three states
//! combine across a rule set.
//!
//! # Explainable traces
//!
//! Every evaluation function in this crate returns not a bare verdict but
//! a `trace::EvaluationTrace` — the full, ordered list of every rule that
//! ran, what it concluded, and why. This is designed in from the start
//! (not bolted on) so a citizen-facing surface can say "you appear
//! eligible BECAUSE x, y, z" and an admin can audit exactly why the engine
//! said what it said, down to the citation backing each claim.
//!
//! # Modules
//!
//! - [`citation`] — what backs a rule's claim ([`citation::Citation`]).
//! - [`trace`] — the tri-state [`trace::Verdict`] and [`trace::EvaluationTrace`].
//! - [`qualifying_date`] — the four annual qualifying dates (since the 2022
//!   amendment) and the "which one applies" computation.
//! - [`eligibility`] — age, citizenship, ordinary residence, and
//!   disqualification rules ([`eligibility::evaluate_eligibility`]).
//! - [`forms`] — which ECI form applies to a citizen's situation
//!   ([`forms::evaluate_form_selection`]).
//! - [`deadlines`] — deadline computations that require an injected
//!   elections-calendar input, by design (see that module's doc comment
//!   for why this crate refuses to hardcode those numbers).
//! - [`mcc`] — Model Code of Conduct applicability, read from an injected
//!   `mcc_windows`-shaped input, consistent with `channel-core`'s
//!   broadcast gate.

pub mod citation;
pub mod deadlines;
pub mod eligibility;
pub mod forms;
pub mod mcc;
pub mod qualifying_date;
pub mod trace;

pub use citation::Citation;
pub use deadlines::{evaluate_claims_and_objections_window_open, evaluate_roll_not_yet_frozen_for_poll, ElectionsCalendarInput};
pub use eligibility::{evaluate_eligibility, Citizenship, EligibilityInput, ResidenceSituation};
pub use forms::{evaluate_form_selection, recommended_forms, FormRecommendation, FormReason, FormSelectionInput, PostalBallotCategory};
pub use mcc::{evaluate_mcc_applicability, McWindow};
pub use qualifying_date::{applicable_qualifying_date, is_18_or_older_on, next_qualifying_date_after, qualifying_dates_for_year};
pub use trace::{EvaluationTrace, RuleStep, Verdict};
