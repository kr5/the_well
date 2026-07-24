//! `/about/*` — per docs/05-information-architecture.md Section 9,
//! content grounded in docs/06-legal-compliance-review.md. That document
//! is explicitly a "draft for legal counsel review," not settled legal
//! advice — these pages carry the same posture and must not overclaim
//! finality beyond what counsel has actually signed off on.

use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn AboutWhatThisIs() -> impl IntoView {
    view! {
        <Title text="What this is — VoteAssist India"/>
        <section class="about-page">
            <h1>"What VoteAssist India is"</h1>
            <p>
                "VoteAssist India is an independent, non-official, open-source civic-technology "
                "project. It helps eligible Indian citizens understand what they need to do — "
                "register, correct their entry, move their registration, and more — and then "
                "deep-links to the real, official Election Commission of India (ECI) service to "
                "actually do it."
            </p>

            <h2>"What we are not"</h2>
            <ul>
                <li>"We are not the Election Commission of India, and we are not affiliated with it or any government body."</li>
                <li>"We never submit any registration, correction, or other application on your behalf."</li>
                <li>"We never provide legal advice, and we don't adjudicate your individual eligibility — we map what you tell us about your situation to publicly documented ECI procedures."</li>
                <li>"We take no political position, and never will — see our neutrality commitment below."</li>
            </ul>

            <h2>"Political neutrality"</h2>
            <p>
                "VoteAssist India never references or implies support or opposition for any party, "
                "candidate, or ideological position, in any content, at any time. During a Model "
                "Code of Conduct period for any election our content touches, we suspend anything "
                "that could be read as voter-mobilization messaging beyond neutral procedural facts."
            </p>

            <h2>"Who runs this"</h2>
            <p>
                "VoteAssist India is maintained as an open-source project — see "
                <a href="/about/open-source">"Open source"</a>
                " for the repository and governance model."
            </p>

            <p>
                "For anything official — registration, corrections, checking your status — always "
                "use "
                <a href="https://voters.eci.gov.in" rel="noopener noreferrer">"voters.eci.gov.in"</a>
                ", ECINET, the Voter Helpline app, or call 1800-11-1950."
            </p>
        </section>
    }
}

#[component]
pub fn AboutLegal() -> impl IntoView {
    view! {
        <Title text="Terms and legal — VoteAssist India"/>
        <section class="about-page">
            <h1>"Terms and legal disclaimers"</h1>
            <p class="draft-notice">
                "This page reflects docs/06-legal-compliance-review.md, an internal working "
                "document explicitly marked as a draft for legal counsel review. Nothing here is "
                "legal advice, and specific conclusions below should be treated as working "
                "hypotheses pending confirmation by qualified counsel."
            </p>

            <h2>"Nature of this service"</h2>
            <p>
                "VoteAssist India provides general information to help you understand electoral "
                "registration procedures. It is not legal advice, is not affiliated with the "
                "Election Commission of India or any government body, and does not guarantee "
                "eligibility or any outcome. For official action and current requirements, use "
                <a href="https://voters.eci.gov.in" rel="noopener noreferrer">"voters.eci.gov.in"</a>
                ", ECINET, the Voter Helpline app, or call the National Voter Helpline at "
                "1800-11-1950."
            </p>

            <h2>"No warranty"</h2>
            <p>
                "This service and its content are provided as-is, without warranty of accuracy, "
                "completeness, or fitness for a particular purpose, to the fullest extent "
                "permitted by law. Every piece of substantive content carries a citation to its "
                "official source and a last-verified date — check the citation for anything "
                "you rely on."
            </p>

            <h2>"No impersonation of government"</h2>
            <p>
                "VoteAssist India does not use the ECI emblem, the State Emblem of India, "
                "government tricolour/Ashoka Chakra iconography, or any CEO office logo. Our "
                "visual design, domain name, and branding are deliberately distinct from "
                "eci.gov.in and voters.eci.gov.in."
            </p>

            <h2>"Reporting an error or a takedown request"</h2>
            <p>
                "If you spot an inaccuracy, or represent the ECI or a CEO office with a "
                "correction or takedown request, please use the "
                <a href="/feedback">"feedback form"</a>
                "."
            </p>
        </section>
    }
}

#[component]
pub fn AboutPrivacy() -> impl IntoView {
    view! {
        <Title text="Privacy — VoteAssist India"/>
        <section class="about-page">
            <h1>"Privacy policy"</h1>
            <p class="draft-notice">
                "This reflects our working data-protection principles per "
                "docs/06-legal-compliance-review.md Section 3, informed by the Digital Personal "
                "Data Protection Act 2023 — final compliance posture to be confirmed with counsel."
            </p>

            <h2>"What we collect"</h2>
            <p>
                "Answering the decision-engine questions at "
                <a href="/start">"/start"</a>
                " never sends your answers to any database. Your in-progress answers live only in "
                "your browser; each question is evaluated by our server statelessly (given this "
                "state and this answer, what's next) and nothing about your specific walkthrough "
                "is stored. We never collect your name, EPIC number, Aadhaar number, or other "
                "direct identifiers through the decision engine."
            </p>

            <h2>"Cookies and similar technologies"</h2>
            <p>
                "This site sets exactly two kinds of browser storage, and asks before setting one "
                "of them:"
            </p>
            <ul>
                <li>
                    <strong>"Strictly necessary (no consent required, set automatically): "</strong>
                    "your accessibility preferences (text size, font, contrast, motion), and, only "
                    "if you create an optional account, a session cookie that keeps you signed in. "
                    "Neither is used to track you across other sites, and neither is shared with "
                    "any third party."
                </li>
                <li>
                    <strong>"Analytics (only with your consent): "</strong>
                    "anonymous, aggregate usage events. You'll see a banner asking before this is "
                    "ever turned on for you; choosing \"Necessary only\" is always respected, and "
                    "you can change your choice at any time by clearing this site's browser storage."
                </li>
            </ul>
            <p>
                "When analytics is consented to, we record de-identified, aggregate usage events "
                "(which question node was viewed, which option was chosen — never free-text input) "
                "to understand where the product is confusing or incomplete. Timestamps are "
                "bucketed to the hour; session identifiers are random and rotate; we never store "
                "your IP address linked to an identity, and raw events are purged after 30 days "
                "(aggregated, anonymized rollups are kept longer for product planning)."
            </p>

            <h2>"Optional accounts and saved checklists"</h2>
            <p>
                "Creating an account (to save a checklist across visits) is entirely optional — "
                "nothing else on this site requires one. If you do create one: we verify your "
                "email with a one-time code, then store only a one-way cryptographic hash of your "
                "email address, never the address itself in a readable form. A saved checklist "
                "records your answers and, if you reached a result, a frozen copy of that result — "
                "we deliberately keep this separate from any identifying information, so a data "
                "breach could reveal \"someone chose the student-hostel path,\" never \"citizen X, "
                "EPIC number Y, is disabled.\" You can delete your account and every saved "
                "checklist yourself, instantly, from "
                <a href="/account">"My account"</a>
                " — no waiting period, no need to contact us first."
            </p>

            <h2>"No special-category inference"</h2>
            <p>
                "We never collect, infer, or store caste, religion, political affiliation, or "
                "similar sensitive categories, under any feature, ever."
            </p>

            <h2>"International visitors"</h2>
            <p>
                "VoteAssist India is built for Indian electors and designed around the Digital "
                "Personal Data Protection Act 2023. If you reach this site from outside India — "
                "including if it's embedded or linked to from another platform — the same "
                "minimal-data-by-design principles above apply regardless of which country's law "
                "specifically governs your visit: no non-essential cookie is set without asking "
                "first, no personal data is sold or shared with third parties, and every right "
                "described on this page (access, deletion, withdrawing consent) is available to "
                "you the same way."
            </p>

            <h2>"Your rights"</h2>
            <p>
                "Any personal data we do hold is accessible and deletable on request. For an "
                "account and its saved checklists, do this yourself instantly at "
                <a href="/account">"My account"</a>
                ". For anything else (a feedback submission with contact info, a question about "
                "this policy), use the "
                <a href="/feedback">"feedback form"</a>
                " to reach us."
            </p>
        </section>
    }
}

#[component]
pub fn AboutOpenSource() -> impl IntoView {
    view! {
        <Title text="Open source — VoteAssist India"/>
        <section class="about-page">
            <h1>"Open source"</h1>
            <p>
                "VoteAssist India is built and published in the open. We believe a project that "
                "helps citizens exercise their voting rights should be independently auditable — "
                "the same discipline civic-tech/governance projects like the Association for "
                "Democratic Reforms and MyNeta follow."
            </p>

            <h2>"What's public"</h2>
            <ul>
                <li>"The full source code — every crate in this platform, including the decision engine, knowledge base, and every channel adapter."</li>
                <li>"The curated knowledge base itself, with every citation to its official source."</li>
                <li>"Our product requirements documents, security posture, and roadmap."</li>
            </ul>

            <h2>"Contributing"</h2>
            <p>
                "Content changes (a new knowledge-base entry, a correction, a new decision-tree "
                "branch) go through a review process before they ship: a citation check against "
                "the live official source, and a compliance-owner sign-off for neutrality and "
                "non-impersonation, per docs/06-legal-compliance-review.md Section 7."
            </p>

            <h2>"Governance"</h2>
            <p>
                "A named Compliance Owner role — distinct from a code Maintainer — is accountable "
                "for legal/procedural content accuracy, takedown requests, and monitoring Model "
                "Code of Conduct periods. See the repository's GOVERNANCE.md for the current "
                "holder of that role."
            </p>
        </section>
    }
}
