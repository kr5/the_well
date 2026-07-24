//! Cross-crate consistency check: every citation in every shipped tree
//! must resolve to a real, sourced kb-content entry. Rust port of
//! packages/decision-engine/test/knowledge-integration.test.ts.
//!
//! core-domain's production code has zero dependency on kb-content (see
//! Cargo.toml) — this is a dev-only, test-time cross-check, matching the
//! architecture note there.

use core_domain::{vote_assist_tree_v1, vote_assist_tree_v2, DecisionNode, DecisionTree};
use kb_content::get_entry;

fn assert_every_citation_resolves(tree: &DecisionTree, tree_name: &str) {
    let terminals: Vec<_> = tree.nodes.values().filter(|n| n.is_terminal()).collect();
    assert!(!terminals.is_empty(), "{tree_name} should have at least one terminal node");

    for node in terminals {
        let DecisionNode::Terminal(terminal) = node else { unreachable!() };
        for citation in &terminal.citations {
            let entry = get_entry(&citation.knowledge_base_id).unwrap_or_else(|| {
                panic!(
                    "{tree_name}: terminal \"{}\" cites unknown knowledge base id \"{}\"",
                    terminal.id, citation.knowledge_base_id
                )
            });
            assert!(
                !entry.sources.is_empty(),
                "{tree_name}: terminal \"{}\" cites \"{}\", which has no sources",
                terminal.id,
                citation.knowledge_base_id
            );
            for source in &entry.sources {
                url::Url::parse(&source.url).unwrap_or_else(|e| {
                    panic!("{tree_name}: source url for \"{}\" is invalid: {e}", citation.knowledge_base_id)
                });
            }
        }

        // Form ids referenced by a terminal should either have a matching
        // knowledge-base entry (form-6/6a/7/8) or be an explicitly-allowed
        // non-KB form (form-2, form-12d), documented in
        // docs/PRD-V2-RUST-PLATFORM.md Section 10 as election-time/
        // service-voter forms outside the core Form 6/6A/7/8 KB scope.
        let known_non_kb_forms = ["form-2", "form-12d"];
        for form_id in &terminal.recommended_forms {
            let has_kb_entry = get_entry(form_id).is_some();
            assert!(
                has_kb_entry || known_non_kb_forms.contains(&form_id.as_str()),
                "{tree_name}: terminal \"{}\" recommends unknown form \"{}\"",
                terminal.id,
                form_id
            );
        }
    }
}

#[test]
fn v1_tree_citations_all_resolve() {
    assert_every_citation_resolves(vote_assist_tree_v1(), "v1");
}

#[test]
fn v2_tree_citations_all_resolve() {
    assert_every_citation_resolves(vote_assist_tree_v2(), "v2");
}
