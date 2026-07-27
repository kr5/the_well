-- Two small, additive knowledge_entries columns closing disclosed gaps
-- from earlier passes: grouping language variants of the same content
-- (crates/jobs::translation_completeness's own module doc names this as
-- a real limitation) and a dedicated "is this an FAQ" flag
-- (crates/web-app::pages::learn's LearnFaqPage module doc names the
-- same gap). Both are nullable/defaulted, so every existing row and
-- every existing knowledge-base/sources/*.json file (none of which sets
-- either field) remains valid without any data migration.

ALTER TABLE knowledge_entries
    ADD COLUMN translation_group_id TEXT,
    ADD COLUMN is_faq BOOLEAN NOT NULL DEFAULT false;

CREATE INDEX idx_knowledge_entries_translation_group_id ON knowledge_entries (translation_group_id);
CREATE INDEX idx_knowledge_entries_is_faq ON knowledge_entries (is_faq) WHERE is_faq;

COMMENT ON COLUMN knowledge_entries.translation_group_id IS
    'Per docs/09-knowledge-base-schema.md''s original design: every '
    'language variant of the same piece of content (including the '
    'English original) shares one identifier, so '
    'crates/jobs::translation_completeness can count "how many variants '
    'of this group exist per locale" instead of a same-language '
    'row-count approximation. docs/09 specifies a UUID; this project '
    'instead reuses the base (English) entry''s own plain-string id '
    '(e.g. "form-6") by convention, since content is hand-authored in '
    'JSON files by curators with no tool generating UUIDs for them — a '
    'translated variant (e.g. "form-6-hi") sets translation_group_id to '
    'that same value. NULL means "not yet grouped" (every existing row '
    'today), NOT "has no possible translations" — completeness queries '
    'must treat NULL as "this entry is its own group of one" via '
    'COALESCE(translation_group_id, id), never filter it out.';
COMMENT ON COLUMN knowledge_entries.is_faq IS
    'Tags an entry for the /learn/faq page (crates/web-app). A content-'
    'curation decision, not a code one — no existing entry is retroactively '
    'tagged by this migration; LearnFaqPage falls back to its prior '
    'full-list behavior until a reviewer tags real entries.';
