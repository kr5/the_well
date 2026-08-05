//! What backs a rule's claim.
//!
//! This project's absolute rule (see crate root docs) is that every
//! procedural claim traces to a source. A [`Citation`] is how a [`crate::Rule`]
//! makes that traceable and machine-checkable rather than a comment only a
//! human reviewer might read.

use serde::{Deserialize, Serialize};

/// The source backing a single rule's claim.
///
/// Two real, checkable sources are modeled, plus one deliberately dishonest-
/// looking-if-hidden third option:
///
/// - [`Citation::KnowledgeBase`] — the common case. Points at an entry id in
///   the curated, human-reviewed `kb-content` crate (mirroring how
///   `core_domain::Citation` cites `kb-content` entries from decision-tree
///   terminal nodes — this crate's citations are the same currency, just
///   attached to a formal rule instead of a tree node).
/// - [`Citation::Statute`] — for claims that rest directly on an Act/Rule
///   section, used where either there is no knowledge-base article yet or
///   the claim is foundational enough (e.g. the RPA 1950 s.19 age/
///   citizenship/residence test) that citing the statute itself is more
///   precise than citing prose about it.
/// - [`Citation::Missing`] — the honest escape hatch. A [`crate::Rule`]
///   MUST be representable even when nobody has attached a source yet
///   (blocking a rule's *existence* on having a citation would just
///   pressure someone into inventing a fake one). What this crate refuses
///   to do is let that be invisible: [`Citation::is_missing`] lets any
///   caller — a test, an admin audit view, a citizen-facing renderer —
///   detect and flag it. See `EvaluationTrace::uncited_steps` in
///   `crate::trace`.
///
/// There is deliberately no variant for "ECI notification/circular" as a
/// distinct citation kind. Where a rule's substance is genuinely tied to a
/// per-notification detail rather than a standing legal text (the exact
/// last-date-for-claims-and-objections number, for instance), this crate
/// does not cite a notification it can't verify — it takes that detail as
/// an *input* instead (see `crate::deadlines`) and cites the standing rule
/// that empowers the notification, which is a real, checkable citation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Citation {
    /// `entry_id` must match a `kb_content::KnowledgeEntry::id`. This crate
    /// does not depend on `kb-content` in production code (see this
    /// crate's `Cargo.toml`) to avoid a needless coupling for a pure rules
    /// engine, so nothing here enforces that match at compile time — the
    /// dev-dependency-gated test in `tests/citations_resolve.rs` is what
    /// actually checks every id this crate hands out against the live
    /// knowledge base.
    KnowledgeBase { entry_id: String },
    /// `act` is the Act or Rules' full name (e.g. "Representation of the
    /// People Act, 1950"); `section` is the specific section/rule (e.g.
    /// "s. 19(a)"). Kept as free text rather than a structured citation
    /// grammar deliberately — Indian election law citations don't follow
    /// one consistent numbering convention across the RPA 1950, RPA 1951,
    /// Registration of Electors Rules 1960, and the Constitution, and a
    /// clean human-readable string is more useful to a reviewer than a
    /// forced structured format that would misrepresent that variety.
    Statute { act: String, section: String },
    /// An honest placeholder. `reason` should explain *why* it's missing
    /// (e.g. "drafted before a knowledge-base entry existed for this — see
    /// TODO tracking issue") so a reviewer doesn't have to guess whether it
    /// was an oversight or a deliberate stopgap.
    Missing { reason: String },
}

impl Citation {
    pub fn kb(entry_id: impl Into<String>) -> Self {
        Citation::KnowledgeBase {
            entry_id: entry_id.into(),
        }
    }

    pub fn statute(act: impl Into<String>, section: impl Into<String>) -> Self {
        Citation::Statute {
            act: act.into(),
            section: section.into(),
        }
    }

    pub fn missing(reason: impl Into<String>) -> Self {
        Citation::Missing {
            reason: reason.into(),
        }
    }

    /// True if this citation is the honest "nobody has sourced this yet"
    /// placeholder. Every renderer of a [`crate::trace::RuleStep`] — the
    /// citizen-facing web app, the admin audit view — MUST check this and
    /// visibly flag it (e.g. a warning badge), never render it as if it
    /// were an equally solid source.
    pub fn is_missing(&self) -> bool {
        matches!(self, Citation::Missing { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kb_and_statute_are_not_missing() {
        assert!(!Citation::kb("qualifying-dates").is_missing());
        assert!(!Citation::statute("Representation of the People Act, 1950", "s. 19(a)").is_missing());
    }

    #[test]
    fn missing_is_flagged() {
        assert!(Citation::missing("no source attached yet").is_missing());
    }

    #[test]
    fn serializes_with_kind_tag() {
        let json = serde_json::to_string(&Citation::kb("qualifying-dates")).unwrap();
        assert!(json.contains("\"kind\":\"knowledge_base\""));
        assert!(json.contains("\"entry_id\":\"qualifying-dates\""));
    }
}
