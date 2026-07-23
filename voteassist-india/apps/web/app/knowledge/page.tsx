import Link from "next/link";
import { knowledgeEntries } from "@voteassist/knowledge";

export const metadata = {
  title: "Knowledge base — VoteAssist India",
};

export default function KnowledgePage() {
  return (
    <div className="va-app-shell">
      <p>
        <Link href="/">← Back</Link>
      </p>
      <h1>Curated, cited voter knowledge base</h1>
      <p>
        Every entry below links to an official source (Election Commission of India, SVEEP, or a Press
        Information Bureau release). Entries marked "in review" are awaiting a second-pass legal/editorial
        review before being marked verified — see docs/06-legal-compliance-review.md.
      </p>
      <ul>
        {knowledgeEntries.map((entry) => (
          <li key={entry.id} style={{ marginBottom: "1.5rem" }}>
            <h2 style={{ marginBottom: "0.25rem" }}>{entry.title}</h2>
            <p style={{ margin: 0, color: "#4a5568" }}>{entry.summary}</p>
            <p style={{ fontSize: "0.85rem" }}>
              Status: {entry.reviewStatus} · Last verified: {entry.lastVerifiedDate}
            </p>
            <ul style={{ fontSize: "0.85rem" }}>
              {entry.sources.map((s) => (
                <li key={s.url}>
                  <a href={s.url} target="_blank" rel="noopener noreferrer">
                    {s.title}
                  </a>
                  {s.publisher ? ` — ${s.publisher}` : null}
                </li>
              ))}
            </ul>
          </li>
        ))}
      </ul>
    </div>
  );
}
