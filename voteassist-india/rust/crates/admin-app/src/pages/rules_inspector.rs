//! Rules Inspector — not one of PRD v2 Section 11's original 13 numbered
//! admin pages, but the page that makes `rules` (the new declarative,
//! cited, explainable eligibility/form-selection rules engine) auditable.
//! Before this page, the only way to see what the engine concludes for a
//! given hypothetical citizen situation — and, more importantly, *why*,
//! down to the citation backing each rule — was to read `rules`' own unit
//! tests. This page lets a reviewer type in a hypothetical situation and
//! read the same `EvaluationTrace` the engine would hand a citizen-facing
//! surface, rendered step by step.
//!
//! ## Why the form's fields are (almost) all optional
//!
//! `rules::EligibilityInput` and `rules::FormSelectionInput` make every
//! field except `as_of` an `Option` on purpose: a `None` means "not asked /
//! not yet known," and the crate's single most important behavioural
//! guarantee (see `rules`' crate-root doc comment) is that a missing input
//! must never render as "not eligible" — it must come back
//! `Verdict::CannotDetermine`. This form mirrors that honestly: every
//! `<select>` below defaults to an explicit "— not specified —" option
//! (mapped to `None`, never silently defaulted to "no"), so the
//! tri-state's third state is reachable — and, in fact, is exactly what
//! this page shows on first load, before a reviewer has touched anything:
//! an all-blank form evaluates to `CannotDetermine` on both traces, not
//! "not eligible," which is the single most important thing this page
//! exists to demonstrate.
//!
//! ## What this page does NOT do
//!
//! It never decides anyone's real eligibility or files anything — it is a
//! read-only "run the engine against a hypothetical" tool, gated to
//! `reviewer`/`legal_reviewer` (an audit function, not a citizen-facing
//! one). `rules::deadlines` and `rules::mcc` (the two remaining rule
//! categories, both requiring an injected elections-calendar/MCC-windows
//! input rather than a flat citizen-answerable form) are out of scope for
//! this pass — only `eligibility` and `forms` are wired up here.

use leptos::prelude::*;
use leptos_meta::Title;

use crate::server_fns::{evaluate_rules_inspector, RuleStepView, RulesInspectorResult, TraceView};

/// Maps a `TraceView::overall`/`RuleStepView::verdict` string onto this
/// app's existing `status-badge status-*` CSS classes (see
/// `style/main.scss`) rather than inventing new ones: `status-verified`
/// (green, "satisfied") and `status-needs_reverification` (amber, "not
/// satisfied") are already exactly the right colours for this purpose —
/// only `cannot_determine` needed a genuinely new, deliberately *neutral*
/// class (`status-cannot_determine`), since neither existing colour reads
/// as "we don't know" rather than "good" or "bad."
fn verdict_badge_class(verdict: &str) -> &'static str {
    match verdict {
        "satisfied" => "status-verified",
        "not_satisfied" => "status-needs_reverification",
        _ => "status-cannot_determine",
    }
}

fn verdict_label(verdict: &str) -> &'static str {
    match verdict {
        "satisfied" => "satisfied",
        "not_satisfied" => "not satisfied",
        _ => "cannot determine",
    }
}

/// `""` -> `None`, `"yes"` -> `Some(true)`, `"no"` -> `Some(false)` — the
/// tri-state select convention every `Option<bool>` field on this form
/// uses, so leaving a select at its default is how a reviewer expresses
/// "not asked," not an implicit "no."
fn parse_tri_bool(value: &str) -> Option<bool> {
    match value {
        "yes" => Some(true),
        "no" => Some(false),
        _ => None,
    }
}

#[component]
pub fn RulesInspectorPage() -> impl IntoView {
    // ----- Eligibility fields -----
    let date_of_birth = RwSignal::new(String::new());
    let as_of = RwSignal::new(chrono::Utc::now().date_naive().to_string());
    let citizenship = RwSignal::new(String::new());
    let residence = RwSignal::new(String::new());
    let court_declared_unsound_mind = RwSignal::new(String::new());
    let disqualified_for_electoral_offence = RwSignal::new(String::new());

    // ----- Form-selection fields -----
    let already_registered_elector = RwSignal::new(String::new());
    let is_service_voter_category = RwSignal::new(String::new());
    let is_overseas_elector_category = RwSignal::new(String::new());
    let reason_kind = RwSignal::new(String::new());
    let postal_category_kind = RwSignal::new(String::new());
    let postal_other_description = RwSignal::new(String::new());

    let eval_action = Action::new(move |input: &(rules::EligibilityInput, rules::FormSelectionInput)| {
        let (eligibility_input, form_input) = input.clone();
        async move { evaluate_rules_inspector(eligibility_input, form_input).await }
    });

    let show_postal_category = move || reason_kind.get() == "wants_postal_ballot_for_an_election";
    let show_postal_other = move || postal_category_kind.get() == "other_notified_absentee_category";

    view! {
        <Title text="Rules inspector — VoteAssist India Admin"/>
        <h1>"Rules inspector"</h1>
        <p>
            "Runs " <code>"rules::evaluate_eligibility"</code> " and " <code>"rules::evaluate_form_selection"</code>
            " against a hypothetical citizen situation and shows the full, cited, step-by-step trace — the same "
            "explanation a citizen-facing surface would eventually render as \"you appear eligible BECAUSE x, y, z.\""
        </p>
        <p class="analytics-caveat">
            "Every field below is optional except \"Assessment date\" — leave a field at "
            "\"— not specified —\" to see the tri-state "
            <code>"Verdict::CannotDetermine"</code>
            " path this engine guarantees for missing input (never silently \"not eligible\"). "
            "Evaluating the form completely blank, as it loads, demonstrates exactly that."
        </p>

        <form
            class="rules-inspector-form"
            on:submit=move |ev| {
                ev.prevent_default();

                let dob = date_of_birth.get();
                let parsed_dob = if dob.trim().is_empty() { None } else { dob.parse().ok() };
                let parsed_as_of = as_of.get().parse().unwrap_or_else(|_| chrono::Utc::now().date_naive());

                let parsed_citizenship = match citizenship.get().as_str() {
                    "indian" => Some(rules::Citizenship::Indian),
                    "foreign_national" => Some(rules::Citizenship::ForeignNational),
                    "oci_cardholder_not_citizen" => Some(rules::Citizenship::OciCardholderNotCitizen),
                    _ => None,
                };
                let parsed_residence = match residence.get().as_str() {
                    "ordinarily_resident_at_claimed_address" => Some(rules::ResidenceSituation::OrdinarilyResidentAtClaimedAddress),
                    "student_away_for_studies" => Some(rules::ResidenceSituation::StudentAwayForStudies),
                    "temporarily_absent_intending_to_return" => Some(rules::ResidenceSituation::TemporarilyAbsentIntendingToReturn),
                    "no_longer_resident_and_not_returning" => Some(rules::ResidenceSituation::NoLongerResidentAndNotReturning),
                    "overseas_elector_under_section_20a" => Some(rules::ResidenceSituation::OverseasElectorUnderSection20a),
                    _ => None,
                };

                let eligibility_input = rules::EligibilityInput {
                    date_of_birth: parsed_dob,
                    as_of: parsed_as_of,
                    citizenship: parsed_citizenship,
                    residence: parsed_residence,
                    court_declared_unsound_mind: parse_tri_bool(&court_declared_unsound_mind.get()),
                    disqualified_for_electoral_offence: parse_tri_bool(&disqualified_for_electoral_offence.get()),
                };

                let parsed_reason = match reason_kind.get().as_str() {
                    "first_time_registration" => Some(rules::FormReason::FirstTimeRegistration),
                    "already_registered_and_shifted_residence" => Some(rules::FormReason::AlreadyRegisteredAndShiftedResidence),
                    "correction_of_particulars" => Some(rules::FormReason::CorrectionOfParticulars),
                    "lost_or_damaged_epic" => Some(rules::FormReason::LostOrDamagedEpic),
                    "mark_as_person_with_disability" => Some(rules::FormReason::MarkAsPersonWithDisability),
                    "object_to_another_persons_inclusion" => Some(rules::FormReason::ObjectToAnotherPersonsInclusion),
                    "seek_deletion_of_an_entry" => Some(rules::FormReason::SeekDeletionOfAnEntry),
                    "wants_postal_ballot_for_an_election" => {
                        let category = match postal_category_kind.get().as_str() {
                            "senior_citizen_85_or_older" => rules::PostalBallotCategory::SeniorCitizen85OrOlder,
                            "person_with_disability" => rules::PostalBallotCategory::PersonWithDisability,
                            "essential_service_employee" => rules::PostalBallotCategory::EssentialServiceEmployee,
                            _ => rules::PostalBallotCategory::OtherNotifiedAbsenteeCategory {
                                description: postal_other_description.get(),
                            },
                        };
                        Some(rules::FormReason::WantsPostalBallotForAnElection(category))
                    }
                    _ => None,
                };

                let form_input = rules::FormSelectionInput {
                    already_registered_elector: parse_tri_bool(&already_registered_elector.get()),
                    is_service_voter_category: parse_tri_bool(&is_service_voter_category.get()),
                    is_overseas_elector_category: parse_tri_bool(&is_overseas_elector_category.get()),
                    reason: parsed_reason,
                };

                eval_action.dispatch((eligibility_input, form_input));
            }
        >
            <h2>"Eligibility input"</h2>

            <div class="form-field">
                <label for="ri-dob">"Date of birth (optional)"</label>
                <input id="ri-dob" type="date"
                    prop:value=move || date_of_birth.get()
                    on:input=move |ev| date_of_birth.set(event_target_value(&ev))/>
            </div>

            <div class="form-field">
                <label for="ri-as-of">"Assessment date (required — the date this check is being made as of)"</label>
                <input id="ri-as-of" type="date" required
                    prop:value=move || as_of.get()
                    on:input=move |ev| as_of.set(event_target_value(&ev))/>
            </div>

            <div class="form-field">
                <label for="ri-citizenship">"Citizenship (optional)"</label>
                <select id="ri-citizenship" prop:value=move || citizenship.get()
                    on:change=move |ev| citizenship.set(event_target_value(&ev))>
                    <option value="">"— not specified —"</option>
                    <option value="indian">"Indian citizen"</option>
                    <option value="foreign_national">"Foreign national"</option>
                    <option value="oci_cardholder_not_citizen">"OCI cardholder (not a citizen)"</option>
                </select>
            </div>

            <div class="form-field">
                <label for="ri-residence">"Residence situation (optional)"</label>
                <select id="ri-residence" prop:value=move || residence.get()
                    on:change=move |ev| residence.set(event_target_value(&ev))>
                    <option value="">"— not specified —"</option>
                    <option value="ordinarily_resident_at_claimed_address">"Ordinarily resident at claimed address"</option>
                    <option value="student_away_for_studies">"Student away for studies"</option>
                    <option value="temporarily_absent_intending_to_return">"Temporarily absent, intending to return"</option>
                    <option value="no_longer_resident_and_not_returning">"No longer resident, not returning"</option>
                    <option value="overseas_elector_under_section_20a">"Overseas elector (s.20A / NRI)"</option>
                </select>
            </div>

            <div class="form-field">
                <label for="ri-unsound-mind">"Court-declared of unsound mind? (optional)"</label>
                <select id="ri-unsound-mind" prop:value=move || court_declared_unsound_mind.get()
                    on:change=move |ev| court_declared_unsound_mind.set(event_target_value(&ev))>
                    <option value="">"— not specified —"</option>
                    <option value="no">"No"</option>
                    <option value="yes">"Yes"</option>
                </select>
            </div>

            <div class="form-field">
                <label for="ri-disqualified">"Disqualified for electoral offence? (optional)"</label>
                <select id="ri-disqualified" prop:value=move || disqualified_for_electoral_offence.get()
                    on:change=move |ev| disqualified_for_electoral_offence.set(event_target_value(&ev))>
                    <option value="">"— not specified —"</option>
                    <option value="no">"No"</option>
                    <option value="yes">"Yes"</option>
                </select>
            </div>

            <h2>"Form-selection input"</h2>

            <div class="form-field">
                <label for="ri-already-registered">"Already a registered elector anywhere? (optional)"</label>
                <select id="ri-already-registered" prop:value=move || already_registered_elector.get()
                    on:change=move |ev| already_registered_elector.set(event_target_value(&ev))>
                    <option value="">"— not specified —"</option>
                    <option value="no">"No"</option>
                    <option value="yes">"Yes"</option>
                </select>
            </div>

            <div class="form-field">
                <label for="ri-service-voter">"Service-voter category? (optional)"</label>
                <select id="ri-service-voter" prop:value=move || is_service_voter_category.get()
                    on:change=move |ev| is_service_voter_category.set(event_target_value(&ev))>
                    <option value="">"— not specified —"</option>
                    <option value="no">"No"</option>
                    <option value="yes">"Yes"</option>
                </select>
            </div>

            <div class="form-field">
                <label for="ri-overseas">"Overseas-elector category? (optional)"</label>
                <select id="ri-overseas" prop:value=move || is_overseas_elector_category.get()
                    on:change=move |ev| is_overseas_elector_category.set(event_target_value(&ev))>
                    <option value="">"— not specified —"</option>
                    <option value="no">"No"</option>
                    <option value="yes">"Yes"</option>
                </select>
            </div>

            <div class="form-field">
                <label for="ri-reason">"Reason for the roll interaction (optional)"</label>
                <select id="ri-reason" prop:value=move || reason_kind.get()
                    on:change=move |ev| reason_kind.set(event_target_value(&ev))>
                    <option value="">"— not specified —"</option>
                    <option value="first_time_registration">"First-time registration"</option>
                    <option value="already_registered_and_shifted_residence">"Already registered, shifted residence"</option>
                    <option value="correction_of_particulars">"Correction of particulars"</option>
                    <option value="lost_or_damaged_epic">"Lost or damaged EPIC"</option>
                    <option value="mark_as_person_with_disability">"Mark as person with disability"</option>
                    <option value="object_to_another_persons_inclusion">"Object to another person's inclusion"</option>
                    <option value="seek_deletion_of_an_entry">"Seek deletion of an entry"</option>
                    <option value="wants_postal_ballot_for_an_election">"Wants postal ballot for an election"</option>
                </select>
            </div>

            <Show when=show_postal_category>
                <div class="form-field">
                    <label for="ri-postal-category">"Postal ballot category"</label>
                    <select id="ri-postal-category" prop:value=move || postal_category_kind.get()
                        on:change=move |ev| postal_category_kind.set(event_target_value(&ev))>
                        <option value="">"— select a category —"</option>
                        <option value="senior_citizen_85_or_older">"Senior citizen, 85 or older"</option>
                        <option value="person_with_disability">"Person with disability"</option>
                        <option value="essential_service_employee">"Essential-service employee"</option>
                        <option value="other_notified_absentee_category">"Other ECI-notified absentee category"</option>
                    </select>
                </div>
            </Show>

            <Show when=move || show_postal_category() && show_postal_other()>
                <div class="form-field">
                    <label for="ri-postal-other">"Describe the ECI-notified category"</label>
                    <input id="ri-postal-other" type="text"
                        prop:value=move || postal_other_description.get()
                        on:input=move |ev| postal_other_description.set(event_target_value(&ev))/>
                </div>
            </Show>

            {move || eval_action.value().get().and_then(|r| r.err()).map(|err| view! {
                <p class="form-error" role="alert">{err.to_string()}</p>
            })}

            <button type="submit">"Evaluate"</button>
        </form>

        {move || eval_action.value().get().and_then(|r| r.ok()).map(|result: RulesInspectorResult| view! {
            <TraceSection heading="Eligibility trace" trace=result.eligibility/>
            <TraceSection heading="Form-selection trace" trace=result.form_selection/>
        })}
    }
}

#[component]
fn TraceSection(heading: &'static str, trace: TraceView) -> impl IntoView {
    let overall = trace.overall.clone();
    view! {
        <section class="rule-trace-section">
            <div class="page-header">
                <h2>{heading}</h2>
                <span class=format!("status-badge {}", verdict_badge_class(&overall))>
                    "Overall: " {verdict_label(&overall)}
                </span>
            </div>
            <ol class="rule-trace">
                {trace.steps.into_iter().map(|step| view! { <TraceStep step=step/> }).collect_view()}
            </ol>
        </section>
    }
}

#[component]
fn TraceStep(step: RuleStepView) -> impl IntoView {
    view! {
        <li class="rule-trace-step">
            <div class="rule-trace-head">
                <code>{step.rule_id.clone()}</code>
                <span class=format!("status-badge {}", verdict_badge_class(&step.verdict))>
                    {verdict_label(&step.verdict).to_string()}
                </span>
            </div>
            <p class="rule-trace-desc">{step.description.clone()}</p>
            <p class="rule-trace-detail">{step.detail.clone()}</p>
            {if step.citation_kind == "missing" {
                view! {
                    <p class="citation-missing" role="alert">
                        "No citation attached: " {step.citation_summary.clone()}
                    </p>
                }.into_any()
            } else {
                view! {
                    <p class="citation-line">"Source: " {step.citation_summary.clone()}</p>
                }.into_any()
            }}
        </li>
    }
}
