export interface ResolvedCitation {
  title: string;
  url: string;
  publisher?: string;
}

export interface DeepLinkView {
  label: string;
  url: string;
}

export interface TerminalResultProps {
  title: string;
  description: string;
  recommendedFormsLabel?: string;
  recommendedForms: string[];
  checklistLabel: string;
  checklist: string[];
  officialLinksLabel: string;
  deepLinks: DeepLinkView[];
  sourcesLabel: string;
  citations: ResolvedCitation[];
  cautionLabel: string;
  caution: string;
  restartLabel: string;
  onRestart: () => void;
  officialLinkDisclaimer: string;
}

export function TerminalResult(props: TerminalResultProps) {
  return (
    <div className="va-result" role="region" aria-label={props.title}>
      <h2 className="va-result-title">{props.title}</h2>
      <p className="va-result-description">{props.description}</p>

      {props.recommendedForms.length > 0 ? (
        <p className="va-result-forms">
          <strong>{props.recommendedFormsLabel ?? "Recommended form(s)"}:</strong>{" "}
          {props.recommendedForms.map((f) => f.toUpperCase()).join(", ")}
        </p>
      ) : null}

      <section aria-labelledby="va-checklist-heading">
        <h3 id="va-checklist-heading">{props.checklistLabel}</h3>
        <ul className="va-checklist">
          {props.checklist.map((item, i) => (
            <li key={i}>{item}</li>
          ))}
        </ul>
      </section>

      <section aria-labelledby="va-links-heading" className="va-official-links">
        <h3 id="va-links-heading">{props.officialLinksLabel}</h3>
        <p className="va-official-disclaimer">{props.officialLinkDisclaimer}</p>
        <div className="va-deep-links">
          {props.deepLinks.map((link) => (
            <a
              key={link.url}
              className="va-deep-link-button"
              href={link.url}
              target="_blank"
              rel="noopener noreferrer"
            >
              {link.label} ↗
            </a>
          ))}
        </div>
      </section>

      <section aria-labelledby="va-caution-heading" className="va-caution">
        <h3 id="va-caution-heading">{props.cautionLabel}</h3>
        <p>{props.caution}</p>
      </section>

      <section aria-labelledby="va-sources-heading" className="va-sources">
        <h3 id="va-sources-heading">{props.sourcesLabel}</h3>
        <ul>
          {props.citations.map((c) => (
            <li key={c.url}>
              <a href={c.url} target="_blank" rel="noopener noreferrer">
                {c.title}
              </a>
              {c.publisher ? ` — ${c.publisher}` : null}
            </li>
          ))}
        </ul>
      </section>

      <button type="button" className="va-restart-button" onClick={props.onRestart}>
        {props.restartLabel}
      </button>
    </div>
  );
}
