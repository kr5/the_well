export interface QuestionOption {
  value: string;
  label: string;
}

export interface QuestionCardProps {
  prompt: string;
  helpText?: string;
  options: QuestionOption[];
  onAnswer: (value: string) => void;
  backLabel?: string;
  onBack?: () => void;
}

export function QuestionCard({ prompt, helpText, options, onAnswer, backLabel, onBack }: QuestionCardProps) {
  return (
    <fieldset className="va-question-card">
      <legend className="va-question-prompt">{prompt}</legend>
      {helpText ? <p className="va-question-help">{helpText}</p> : null}
      <div className="va-question-options" role="group" aria-label={prompt}>
        {options.map((option) => (
          <button
            key={option.value}
            type="button"
            className="va-option-button"
            onClick={() => onAnswer(option.value)}
          >
            {option.label}
          </button>
        ))}
      </div>
      {onBack ? (
        <button type="button" className="va-link-button" onClick={onBack}>
          {backLabel ?? "Back"}
        </button>
      ) : null}
    </fieldset>
  );
}
