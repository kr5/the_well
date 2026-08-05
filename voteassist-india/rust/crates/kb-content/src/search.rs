//! Ranked, alias-aware in-process search over the knowledge base.
//!
//! Replaces the original `.contains()`-across-fields MVP search (see the
//! git history of `lib.rs`'s `search_entries`, which matched the
//! TypeScript prototype's naive substring search verbatim). That version
//! had no ranking (results came back in load order), no field weighting
//! (a body match and a title match counted identically), and no
//! vocabulary bridging (an Indian voter typing "voter id" got zero
//! results against a knowledge base that only ever says "EPIC").
//!
//! ## Why hand-rolled instead of a search-index crate
//! `docs/PRD-V2-RUST-PLATFORM.md` Section 6.5 names Meilisearch as the
//! intended production search backend once content volume and the
//! multilingual surface justify a separate service. That is explicitly
//! *not* this crate's job: `kb-content` is documented (`lib.rs`) as a
//! zero-I/O crate — it is linked directly into `crates/web-app`'s
//! wasm32 client bundle (see that crate's `Cargo.toml` comment) alongside
//! `crates/core-domain`, so anything requiring a network call, a
//! filesystem read at runtime, or a non-wasm32-safe dependency is
//! architecturally off the table here regardless of search-quality
//! ambitions. Everything below is pure, synchronous, allocation-only
//! Rust operating on the already-loaded `&'static [KnowledgeEntry]`
//! slice, built once behind a `OnceLock` exactly like `lib.rs`'s own
//! `all_entries()`/`by_id()` caches.
//!
//! ## Scoring scheme
//! Each query term contributes `field_weight * tf_weight * idf` to a
//! document's score, summed across every query term and every field it
//! matches in (a BM25-flavored scheme, minus BM25's document-length
//! normalization — see [`FIELD_WEIGHTS`]-adjacent constants below for
//! why that term is skipped here):
//!
//! - **Field weight** — title > related forms/entities > summary > body.
//!   A query term landing in the title is the strongest possible signal
//!   that an entry is *about* that term; a term buried once in a long
//!   body paragraph is the weakest. See the `TITLE_WEIGHT` etc. constants
//!   just below for the exact ratios and their reasoning.
//! - **`tf_weight` (term-frequency saturation)** — `1 + ln(term_freq)`,
//!   so a term appearing twice in a field counts for more than once but
//!   not twice as much, and a term appearing twenty times (unlikely in
//!   this corpus's short curated prose, but not impossible in a future
//!   longer entry) doesn't dominate the score just from repetition.
//!   Document-length normalization (BM25's `k1`/`b` machinery) is
//!   deliberately omitted: this corpus is a small set of similarly-sized,
//!   hand-curated entries (not a general web corpus with wildly varying
//!   lengths), so the extra free parameters would add tuning surface
//!   without a real problem to solve — an honest simplification, not an
//!   oversight.
//! - **`idf` (inverse document frequency)** — `ln(1 + N / df)`, where
//!   `N` is the total entry count and `df` is how many entries contain
//!   the term at all. This is the standard IDF shape but with the `+1`
//!   inside the log (rather than classic `ln(N/df)`): with classic IDF a
//!   term present in *every* document scores `ln(1) = 0` (or worse,
//!   negative, if `df > N` could ever happen), which would zero out
//!   legitimately important domain terms like "voter" or "electoral"
//!   that are common precisely because they're central to this
//!   knowledge base, not because they're noise. The `+1` guarantees
//!   `idf > 0` for every term that appears anywhere, so a term that's
//!   everywhere still counts for something, just less than a rare one.
//!
//! ## Field weighting
//! ```text
//! title              -> 6.0   (strongest: this term IS what the entry is about)
//! related_forms /
//! related_entities   -> 3.0   (curated cross-reference metadata — "EPIC",
//!                              "form-8", "BLO" — deliberately weighted
//!                              above prose, since these are exact
//!                              canonical-vocabulary tags an author chose
//!                              on purpose, not incidental word choice)
//! summary             -> 2.0   (a short, deliberately-written abstract)
//! body                -> 1.0   (weakest: long-form prose, most likely to
//!                              mention a term in passing without the
//!                              entry being centrally "about" it)
//! ```
//!
//! ## Tokenization and Unicode
//! Tokenization uses `unicode-segmentation`'s `unicode_words()`, which
//! implements the Unicode Text Segmentation algorithm (UAX #29) rather
//! than splitting on ASCII whitespace/punctuation. This matters
//! concretely for this project: Devanagari, Bengali, Tamil etc. text is
//! multi-byte per character in UTF-8, and naive byte-offset slicing
//! (`s[..n]`, manual `char_indices` arithmetic assuming 1 byte/char)
//! would panic or silently mis-tokenize on any Indic-script input. Every
//! substring operation in this file (`str::contains`, `str::starts_with`,
//! `.to_lowercase()`) is a standard-library method that already operates
//! on whole Unicode scalar values / grapheme-safe byte sequences
//! internally — nothing here does manual byte-index arithmetic.
//!
//! ## Indic-script queries — what works today and what doesn't
//! Every knowledge-base entry today is English-only (`language: "en"` in
//! every `knowledge-base/sources/*.json` file; see `types.rs`'s
//! `translation_group_id` doc comment for the planned per-language
//! variant mechanism, not yet populated). So:
//! - A query typed in Devanagari (or any other Indic script) will
//!   **never crash or panic** — tokenization, lowercasing, and substring
//!   matching are all Unicode-correct — but it will only surface results
//!   through the [`ALIASES`] table's curated Devanagari entries (a
//!   handful of high-frequency terms: मतदाता, नाम, सूची, पहचान पत्र,
//!   स्थानांतरण, मतदान — see the table below), not through free-text
//!   matching against KB prose, because there is no Devanagari prose to
//!   match against yet.
//! - Once `hi`-language entries exist (via `translation_group_id`), this
//!   same index-and-score machinery works over them unchanged — the
//!   tokenizer and scorer have no English-specific logic anywhere. The
//!   only thing that needs to grow is the `ALIASES` table and, later,
//!   language-aware filtering so a Hindi query preferentially ranks
//!   Hindi entries — genuinely out of scope until Hindi content exists
//!   to rank.
//!
//! ## Alias / synonym expansion — why a curated static table
//! Real Indian voters search with different vocabulary than ECI's
//! official terms: "voter id" for EPIC, "form 6" for new registration,
//! "shift"/"moved" for shifting of residence, and so on (the full
//! rationale and worked examples are in each [`ALIASES`] entry's
//! grouping comment below). This project deliberately uses a small,
//! human-authored, git-diffable table — not a fuzzy string-distance
//! matcher, embedding-similarity lookup, or any ML-based expansion —
//! for the same reason `docs/SECURITY-AND-SRE-OPERATIONS.md`'s
//! link-checker only ever *flags* content rather than auto-editing it:
//! this is a civic-information tool, and a wrong or over-eager synonym
//! match doesn't just return a bad search result, it can send a citizen
//! toward the wrong form for their actual situation. Every row in
//! [`ALIASES`] is something a human reviewer can read, question, and
//! diff in a pull request; nothing in this file guesses at meaning it
//! wasn't told.
//!
//! ## Minimum score threshold
//! After scoring, results are dropped if their score falls below
//! `RELATIVE_CUTOFF` (15%) of the top-scoring result for that query —
//! see that constant below for the full reasoning. In today's
//! eleven-entry corpus this rarely trims anything (most queries either
//! match a handful of clearly-relevant entries or match nothing at all),
//! but it establishes the floor now, before content volume grows to the
//! point where a long tail of one-weak-term-in-the-body matches could
//! otherwise clutter rank 15-20 with genuinely irrelevant entries.

use std::collections::HashMap;
use std::sync::OnceLock;

use unicode_segmentation::UnicodeSegmentation;

use crate::types::{KnowledgeEntry, RelatedForm};

// ---------------------------------------------------------------------
// Field weights (see module doc "Field weighting" above for reasoning).
// ---------------------------------------------------------------------

const TITLE_WEIGHT: f64 = 6.0;
const RELATED_WEIGHT: f64 = 3.0;
const SUMMARY_WEIGHT: f64 = 2.0;
const BODY_WEIGHT: f64 = 1.0;

/// A prefix-only match (query "regist" against indexed "registration") is
/// real signal — Indian users routinely truncate long official terms —
/// but weaker than an exact term match, so it's discounted to 60% of
/// whatever score the matched vocabulary term would otherwise contribute.
const PREFIX_MULTIPLIER: f64 = 0.6;

/// Terms pulled in via [`ALIASES`] expansion are discounted to 80%
/// relative to a literal query term match. Rationale: if a citizen's
/// exact words *also* appear directly in an entry, that entry is
/// probably the single best answer and should edge out an entry that
/// only matches through a synonym bridge — but the discount is mild
/// (not, say, 30%) because a correct alias match is still a *correct*
/// match, not a fuzzy guess (see the module-level "why a curated static
/// table" note): this project trusts its own alias table.
const ALIAS_MULTIPLIER: f64 = 0.8;

/// Below this length, prefix matching is disabled for that query term.
/// Without a floor, a one- or two-character query term (e.g. the "s" a
/// user might type mid-word before finishing "shifting") would prefix-
/// match a huge fraction of the vocabulary and return effectively
/// unranked noise — the opposite of what ranked search is for.
const MIN_PREFIX_LEN: usize = 3;

/// Keep only results scoring at least this fraction of the top result's
/// score for the same query. A relative (rather than a fixed absolute)
/// cutoff is used deliberately: it scales automatically as the corpus
/// grows and score magnitudes shift with it, instead of requiring this
/// constant to be re-tuned every time content is added. 15% was chosen
/// as a middle ground — tight enough to cut a long tail of
/// single-weak-body-term matches once the corpus is large, loose enough
/// that today's short, closely-related entries (many of which
/// legitimately share vocabulary — "EPIC", "BLO", "electoral roll" —
/// across several forms) aren't over-pruned.
const RELATIVE_CUTOFF: f64 = 0.15;

/// Common English function words, excluded from indexing and from query
/// tokenization so they don't dilute scores or trigger noisy prefix
/// matches. Deliberately short and conservative — this is a relevance
/// tweak, not a linguistic exercise, so it only lists words that would
/// otherwise appear in nearly every entry (e.g. "of", "the", "a") and
/// therefore carry almost no discriminating power once IDF is applied
/// anyway. Devanagari and other Indic-script tokens never collide with
/// this list (disjoint scripts), so it has no effect on non-English
/// query terms one way or the other.
const STOPWORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "by", "do", "does", "did", "for", "from", "has",
    "have", "he", "i", "in", "is", "it", "its", "my", "not", "of", "on", "or", "our", "she",
    "that", "the", "their", "them", "there", "these", "they", "this", "those", "to", "us", "was",
    "we", "were", "will", "with", "would", "you", "your",
];

/// Which structural field a matched term came from — see module doc
/// "Field weighting" for the ratios and rationale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Title,
    /// `related_forms` (e.g. `form-8`) and `related_entities` (e.g.
    /// `EPIC`, `BLO`) are indexed together at the same weight tier: both
    /// are short, hand-curated cross-reference tags an author attached
    /// on purpose, as opposed to incidental word choice in prose, so
    /// they deserve the same trust level regardless of which of the two
    /// schema fields they happen to live in.
    Related,
    Summary,
    Body,
}

impl Field {
    fn weight(self) -> f64 {
        match self {
            Field::Title => TITLE_WEIGHT,
            Field::Related => RELATED_WEIGHT,
            Field::Summary => SUMMARY_WEIGHT,
            Field::Body => BODY_WEIGHT,
        }
    }
}

/// One occurrence record: entry `doc_idx` contains the indexed term
/// `term_freq` times in `field`. `doc_idx` indexes into
/// `crate::knowledge_entries()`'s slice, which is stable for the process
/// lifetime (it's itself backed by a `OnceLock` in `lib.rs`).
struct Posting {
    doc_idx: usize,
    field: Field,
    term_freq: u32,
}

/// The in-memory inverted index, built once and cached — see
/// [`search_index`]. Kept deliberately simple (a couple of `HashMap`s and
/// a sorted `Vec` for prefix scans) rather than a trie or other more
/// elaborate structure: at today's and any foreseeable near-term corpus
/// size (tens, not thousands, of entries — this is a hand-curated,
/// human-reviewed knowledge base, not a crawled corpus), a linear scan
/// over the vocabulary for prefix matching costs microseconds, and the
/// simpler structure is easier for the next person to audit against the
/// scoring formula documented above.
struct SearchIndex {
    /// term -> every (doc, field, term_freq) occurrence of that term.
    postings: HashMap<String, Vec<Posting>>,
    /// term -> number of distinct entries containing it at least once,
    /// in any field. Used for IDF.
    doc_freq: HashMap<String, usize>,
    /// Every indexed term, sorted, for prefix-match scanning.
    vocabulary: Vec<String>,
    doc_count: usize,
}

impl SearchIndex {
    fn idf(&self, term: &str) -> f64 {
        let df = self.doc_freq.get(term).copied().unwrap_or(0);
        if df == 0 {
            return 0.0;
        }
        (1.0 + self.doc_count as f64 / df as f64).ln()
    }
}

fn search_index() -> &'static SearchIndex {
    static INDEX: OnceLock<SearchIndex> = OnceLock::new();
    INDEX.get_or_init(build_index)
}

/// Splits `text` into lowercased word tokens using Unicode word-boundary
/// segmentation (not ASCII whitespace/punctuation splitting), then drops
/// [`STOPWORDS`]. Safe and correct on any script, including multi-byte
/// Indic scripts — see the module doc's "Tokenization and Unicode"
/// section.
fn tokenize(text: &str) -> Vec<String> {
    text.unicode_words()
        .map(|w| w.to_lowercase())
        .filter(|w| !STOPWORDS.contains(&w.as_str()))
        .collect()
}

/// Maps a `RelatedForm` enum variant back to the canonical string form
/// used in `knowledge-base/sources/*.json` and in query-facing aliases
/// (e.g. "form 6" -> `Form6` -> `"form-6"`), so it can be tokenized and
/// indexed alongside `related_entities` in the [`Field::Related`] tier.
/// Hand-matched against `types.rs`'s `#[serde(rename = ...)]` values
/// rather than round-tripped through `serde_json`, since this is a
/// small, fixed, non-growing enum and a direct match is one line of code
/// simpler to audit than a serialize-then-strip-quotes dance.
fn related_form_str(form: RelatedForm) -> &'static str {
    match form {
        RelatedForm::Form2 => "form-2",
        RelatedForm::Form6 => "form-6",
        RelatedForm::Form6a => "form-6a",
        RelatedForm::Form7 => "form-7",
        RelatedForm::Form8 => "form-8",
        RelatedForm::Form12d => "form-12d",
    }
}

fn index_field(index: &mut HashMap<String, Vec<Posting>>, doc_idx: usize, field: Field, text: &str) {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for tok in tokenize(text) {
        *counts.entry(tok).or_insert(0) += 1;
    }
    for (term, term_freq) in counts {
        index.entry(term).or_default().push(Posting { doc_idx, field, term_freq });
    }
}

fn build_index() -> SearchIndex {
    let entries = crate::knowledge_entries();
    let mut postings: HashMap<String, Vec<Posting>> = HashMap::new();

    for (doc_idx, entry) in entries.iter().enumerate() {
        index_field(&mut postings, doc_idx, Field::Title, &entry.title);
        index_field(&mut postings, doc_idx, Field::Summary, &entry.summary);
        index_field(&mut postings, doc_idx, Field::Body, &entry.body);

        let mut related_text = entry.related_entities.join(" ");
        for form in &entry.related_forms {
            related_text.push(' ');
            related_text.push_str(related_form_str(*form));
        }
        index_field(&mut postings, doc_idx, Field::Related, &related_text);
    }

    let mut doc_freq: HashMap<String, usize> = HashMap::new();
    for (term, plist) in &postings {
        let distinct_docs: std::collections::HashSet<usize> =
            plist.iter().map(|p| p.doc_idx).collect();
        doc_freq.insert(term.clone(), distinct_docs.len());
    }

    let mut vocabulary: Vec<String> = postings.keys().cloned().collect();
    vocabulary.sort();

    SearchIndex { postings, doc_freq, vocabulary, doc_count: entries.len() }
}

/// Curated colloquial/alternate-term -> canonical-KB-vocabulary alias
/// table. See the module doc's "Alias / synonym expansion" section for
/// why this is a hand-authored static table rather than any fuzzy or
/// ML-based matching. Keys are matched as a case-insensitive substring
/// against the whole (whitespace-trimmed, lowercased) query string —
/// deliberately simple substring matching rather than exact-token-
/// sequence matching, so a key like "form 6" matches a query of "form 6
/// application" too, not only an exact "form 6" query. Values are
/// canonical phrases; each is tokenized the same way as indexed content
/// before being added to the query's term set (see
/// [`expand_query_terms`]), at [`ALIAS_MULTIPLIER`] weight.
///
/// Grounded against the real KB entries in
/// `knowledge-base/sources/*.json` (not invented in the abstract) —
/// every canonical phrase on the right either matches a literal word in
/// some entry's title/summary/body, or matches a `related_entities` /
/// `related_forms` tag, at the time this table was written. If KB
/// content is later reworded, some rows here may stop matching anything
/// — that's a content-drift risk to watch for, not a crash risk: an
/// alias expanding to zero matches just contributes nothing to any
/// score, exactly like any other term nobody's content happens to use.
const ALIASES: &[(&str, &[&str])] = &[
    // --- EPIC (voter ID card) -------------------------------------------------
    ("voter id", &["epic"]),
    ("voter card", &["epic"]),
    ("voter id card", &["epic"]),
    ("epic card", &["epic"]),
    ("election card", &["epic"]),
    ("election id card", &["epic"]),
    ("identity card", &["epic"]),
    // --- Form 6: new / first-time registration --------------------------------
    ("form 6", &["new registration", "first time voter"]),
    ("new registration", &["form 6"]),
    ("first time voter", &["form 6"]),
    ("new voter", &["form 6", "new registration"]),
    ("turning 18", &["form 6", "qualifying date"]),
    ("register to vote", &["form 6", "new registration"]),
    ("enroll as voter", &["form 6", "new registration"]),
    // --- Form 8: shifting of residence -----------------------------------------
    ("shift", &["shifting of residence"]),
    ("shifted", &["shifting of residence"]),
    ("moved", &["shifting of residence"]),
    ("moving", &["shifting of residence"]),
    ("change address", &["shifting of residence"]),
    ("change my address", &["shifting of residence"]),
    ("changed address", &["shifting of residence"]),
    ("address change", &["shifting of residence"]),
    ("transfer", &["shifting of residence"]),
    ("transferred", &["shifting of residence"]),
    ("relocate", &["shifting of residence"]),
    ("relocated", &["shifting of residence"]),
    ("new address", &["shifting of residence", "form 8"]),
    // --- Form 8: correction of entries -----------------------------------------
    ("correction", &["form 8"]),
    ("mistake", &["form 8", "correction of entries"]),
    ("wrong name", &["form 8", "correction of entries"]),
    ("wrong spelling", &["form 8"]),
    ("spelling mistake", &["form 8"]),
    ("wrong dob", &["form 8"]),
    ("wrong date of birth", &["form 8"]),
    ("wrong photo", &["form 8"]),
    ("update details", &["form 8", "correction of entries"]),
    // --- Form 7: deletion / objection -------------------------------------------
    ("delete", &["form 7"]),
    ("deletion", &["form 7"]),
    ("remove name", &["form 7"]),
    ("removal", &["form 7"]),
    ("objection", &["form 7"]),
    ("duplicate entry", &["form 7"]),
    ("duplicate registration", &["form 7"]),
    ("deceased", &["form 7"]),
    ("death", &["form 7"]),
    // --- Form 6A: NRI / overseas elector -----------------------------------------
    ("nri", &["form 6a", "overseas elector"]),
    ("nri voter", &["form 6a"]),
    ("abroad", &["form 6a", "overseas elector"]),
    ("overseas", &["form 6a"]),
    ("living abroad", &["form 6a"]),
    ("foreign country", &["form 6a"]),
    // --- Service voters (armed forces) -------------------------------------------
    ("army", &["service voter"]),
    ("military", &["service voter"]),
    ("defence", &["service voter"]),
    ("defense", &["service voter"]),
    ("armed forces", &["service voter"]),
    ("soldier", &["service voter"]),
    ("jawan", &["service voter"]),
    // --- PwD -----------------------------------------------------------------------
    ("disabled", &["pwd"]),
    ("handicapped", &["pwd"]),
    ("pwd", &["persons with disabilities"]),
    ("wheelchair", &["pwd"]),
    ("disability", &["pwd"]),
    // --- Form 12D: postal ballot / home voting --------------------------------------
    ("postal ballot", &["form 12d"]),
    ("vote from home", &["form 12d", "home voting"]),
    ("home voting", &["form 12d"]),
    ("senior citizen", &["form 12d", "postal ballot"]),
    ("above 85", &["form 12d"]),
    ("elderly voter", &["form 12d"]),
    // --- Polling station / BLO / ERO / DEO / CEO ------------------------------------
    ("booth", &["polling station"]),
    ("polling booth", &["polling station"]),
    ("where do i vote", &["polling station"]),
    ("find my booth", &["polling station"]),
    ("which booth", &["polling station"]),
    ("booth level officer", &["blo"]),
    ("electoral registration officer", &["ero"]),
    ("district election officer", &["deo"]),
    ("chief electoral officer", &["ceo"]),
    // --- Helpline / grievance ---------------------------------------------------------
    ("1950", &["voter helpline"]),
    ("helpline number", &["voter helpline"]),
    ("toll free", &["voter helpline"]),
    ("complain", &["grievance"]),
    ("complaint", &["grievance"]),
    ("track application", &["grievance", "voter helpline"]),
    // --- Roll search --------------------------------------------------------------------
    ("find my name", &["electoral roll"]),
    ("check my name", &["electoral roll"]),
    ("am i registered", &["electoral roll"]),
    ("search voter list", &["electoral roll"]),
    // --- Ordinary residence (students) ---------------------------------------------------
    ("student hostel", &["ordinary residence"]),
    ("hostel address", &["ordinary residence"]),
    // --- Devanagari (Hindi script) — see module doc "Indic-script queries" -----------------
    ("मतदाता", &["voter"]),
    ("नाम", &["name"]),
    ("सूची", &["electoral roll"]),
    ("सूची में नाम", &["electoral roll", "name"]),
    ("पहचान पत्र", &["epic", "identity card"]),
    ("स्थानांतरण", &["shifting of residence", "transfer"]),
    ("मतदान", &["voting"]),
    ("मतदान केंद्र", &["polling station"]),
    ("केंद्र", &["centre", "polling station"]),
    // --- Transliterated (Romanized) Hindi -------------------------------------------------
    ("matdata", &["voter"]),
    ("pehchan patra", &["epic", "identity card"]),
    ("naam", &["name"]),
    ("suchi mein naam", &["electoral roll", "name"]),
    ("suchi", &["electoral roll"]),
    ("sthanantaran", &["shifting of residence", "transfer"]),
    ("matdan", &["voting"]),
    ("kendra", &["centre", "polling station"]),
];

/// Expands a raw query into `(term, weight_multiplier)` pairs: every
/// literal token from the query itself at multiplier `1.0`, plus every
/// token from any [`ALIASES`] canonical phrase whose key appears
/// (as a substring) in the normalized query, at [`ALIAS_MULTIPLIER`].
/// Returns an empty vec for an empty/whitespace-only query.
fn expand_query_terms(query: &str) -> Vec<(String, f64)> {
    let normalized = query.trim().to_lowercase();
    if normalized.is_empty() {
        return Vec::new();
    }

    let mut terms: Vec<(String, f64)> = tokenize(&normalized).into_iter().map(|t| (t, 1.0)).collect();

    for (alias, canonicals) in ALIASES {
        if normalized.contains(alias) {
            for canonical in *canonicals {
                terms.extend(tokenize(canonical).into_iter().map(|t| (t, ALIAS_MULTIPLIER)));
            }
        }
    }

    terms
}

/// A search result paired with the score it earned for a specific query
/// — the richer API mentioned in the task: useful for a future admin
/// relevance-debugging view (see [`search_entries`] for the thin,
/// score-stripping wrapper that most call sites should keep using).
#[derive(Debug, Clone, Copy)]
pub struct ScoredEntry {
    pub entry: &'static KnowledgeEntry,
    pub score: f64,
}

/// Ranked search: tokenizes and alias-expands `query`, scores every
/// entry against the resulting term set (see the module doc's "Scoring
/// scheme" section for the formula), drops anything below
/// [`RELATIVE_CUTOFF`] of the top score, and returns the rest sorted by
/// descending score (ties broken by entry id, for deterministic output).
pub fn search_entries_ranked(query: &str) -> Vec<ScoredEntry> {
    let terms = expand_query_terms(query);
    if terms.is_empty() {
        return Vec::new();
    }

    let index = search_index();
    let entries = crate::knowledge_entries();
    let mut scores: HashMap<usize, f64> = HashMap::new();

    for (term, multiplier) in &terms {
        // Exact term match.
        if let Some(postings) = index.postings.get(term) {
            let idf = index.idf(term);
            for p in postings {
                let tf_weight = 1.0 + (p.term_freq as f64).ln();
                *scores.entry(p.doc_idx).or_insert(0.0) += p.field.weight() * tf_weight * idf * multiplier;
            }
        }

        // Prefix match: other, longer vocabulary terms sharing this
        // prefix, at a discount (see PREFIX_MULTIPLIER). Excludes the
        // term itself so an exact match is never double-counted here.
        // `term.chars().count()` (not `.len()`) measures Unicode scalar
        // values rather than bytes, so a short multi-byte token (e.g. a
        // 2-character Devanagari word) isn't mistakenly treated as long
        // enough to prefix-match just because its byte length is >= 3.
        if term.chars().count() >= MIN_PREFIX_LEN {
            for vocab_term in &index.vocabulary {
                if vocab_term != term && vocab_term.starts_with(term.as_str()) {
                    if let Some(postings) = index.postings.get(vocab_term) {
                        let idf = index.idf(vocab_term);
                        for p in postings {
                            let tf_weight = 1.0 + (p.term_freq as f64).ln();
                            *scores.entry(p.doc_idx).or_insert(0.0) +=
                                p.field.weight() * tf_weight * idf * multiplier * PREFIX_MULTIPLIER;
                        }
                    }
                }
            }
        }
    }

    if scores.is_empty() {
        return Vec::new();
    }

    let max_score = scores.values().cloned().fold(0.0_f64, f64::max);
    let cutoff = max_score * RELATIVE_CUTOFF;

    let mut ranked: Vec<ScoredEntry> = scores
        .into_iter()
        .filter(|(_, score)| *score >= cutoff)
        .map(|(doc_idx, score)| ScoredEntry { entry: &entries[doc_idx], score })
        .collect();

    ranked.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.entry.id.cmp(&b.entry.id))
    });

    ranked
}

/// Thin, score-stripping wrapper over [`search_entries_ranked`] — kept
/// as the crate's stable public search entry point (same signature the
/// original naive `.contains()` search exposed) so existing call sites
/// (`crates/api`, bots, admin app) don't need to change to benefit from
/// ranking, field weighting, prefix matching, and alias expansion.
pub fn search_entries(query: &str) -> Vec<&'static KnowledgeEntry> {
    search_entries_ranked(query).into_iter().map(|scored| scored.entry).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_whitespace_queries_return_nothing() {
        assert!(search_entries("").is_empty());
        assert!(search_entries("   ").is_empty());
        assert!(search_entries("\t\n").is_empty());
    }

    #[test]
    fn nonsense_query_returns_nothing_rather_than_everything() {
        // Gibberish that shares no prefix or alias with any real term.
        let results = search_entries("zzqxvbnmasdkjfhqwzz");
        assert!(results.is_empty(), "expected no matches, got {:?}", results.iter().map(|e| &e.id).collect::<Vec<_>>());
    }

    #[test]
    fn title_match_outranks_body_only_match() {
        // "helpline" is in helpline-grievance.json's *title* ("Voter
        // Helpline (1950)...") and only in the *body* prose of form-6
        // and roll-search-polling-station ("... via the Voter Helpline
        // App ..."). The title hit must win first place.
        let results = search_entries_ranked("helpline");
        assert!(!results.is_empty());
        assert_eq!(results[0].entry.id, "helpline-grievance");

        let helpline_grievance_rank =
            results.iter().position(|r| r.entry.id == "helpline-grievance").unwrap();
        for (i, r) in results.iter().enumerate() {
            if r.entry.id != "helpline-grievance" {
                assert!(
                    i > helpline_grievance_rank,
                    "title match for 'helpline' should outrank body-only match {}",
                    r.entry.id
                );
            }
        }
    }

    #[test]
    fn multi_word_query_matches_via_tokenized_terms() {
        let results = search_entries("shifting of residence");
        assert!(results.iter().any(|e| e.id == "form-8"));
    }

    #[test]
    fn multi_word_colloquial_query_matches_via_alias_expansion() {
        // "change my address" isn't in any entry verbatim, but the
        // alias table bridges it to "shifting of residence" (Form 8).
        let results = search_entries("change my address");
        assert!(
            results.iter().any(|e| e.id == "form-8"),
            "expected form-8 among results, got {:?}",
            results.iter().map(|e| &e.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn prefix_match_finds_registration_from_truncated_query() {
        // "regist" should prefix-match "registration" in form-6's title.
        let results = search_entries("regist");
        assert!(
            results.iter().any(|e| e.id == "form-6"),
            "expected form-6 among results, got {:?}",
            results.iter().map(|e| &e.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn english_colloquial_alias_voter_id_finds_epic() {
        let results = search_entries("voter id");
        assert!(
            results.iter().any(|e| e.id == "e-epic"),
            "expected e-epic among results, got {:?}",
            results.iter().map(|e| &e.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn english_colloquial_alias_wrong_name_finds_form_8() {
        let results = search_entries("wrong name on my voter card");
        assert!(
            results.iter().any(|e| e.id == "form-8"),
            "expected form-8 among results, got {:?}",
            results.iter().map(|e| &e.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn devanagari_query_matdata_finds_voter_content() {
        // मतदाता ("matdata" = voter) should not crash and should bridge
        // to the English word "voter", which appears widely (e.g. in
        // helpline-grievance's title).
        let results = search_entries("मतदाता");
        assert!(
            !results.is_empty(),
            "expected Devanagari query 'मतदाता' to match via alias expansion"
        );
    }

    #[test]
    fn devanagari_query_pehchan_patra_finds_epic() {
        // पहचान पत्र ("identity card") should bridge to EPIC.
        let results = search_entries("पहचान पत्र");
        assert!(
            results.iter().any(|e| e.id == "e-epic"),
            "expected e-epic among results, got {:?}",
            results.iter().map(|e| &e.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn devanagari_query_does_not_panic_on_pure_gibberish_script_mix() {
        // Defensive: Indic-script input with no alias-table entry must
        // return an empty result, never panic, on this English-only corpus.
        let results = search_entries("अज्ञातशब्दसमूह");
        assert!(results.is_empty());
    }

    #[test]
    fn transliterated_hindi_alias_sthanantaran_finds_shifting_of_residence() {
        let results = search_entries("sthanantaran kaise kare");
        assert!(
            results.iter().any(|e| e.id == "form-8"),
            "expected form-8 among results, got {:?}",
            results.iter().map(|e| &e.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn nri_alias_finds_form_6a() {
        let results = search_entries("i live abroad, how do i vote as an nri");
        assert!(
            results.iter().any(|e| e.id == "form-6a"),
            "expected form-6a among results, got {:?}",
            results.iter().map(|e| &e.id).collect::<Vec<_>>()
        );
    }

    #[test]
    fn scores_are_sorted_descending() {
        let ranked = search_entries_ranked("epic replacement correction");
        for pair in ranked.windows(2) {
            assert!(pair[0].score >= pair[1].score);
        }
    }
}
