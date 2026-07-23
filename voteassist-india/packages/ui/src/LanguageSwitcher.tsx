export interface LanguageOption {
  code: string;
  nativeName: string;
  status: "shipped" | "planned";
}

export interface LanguageSwitcherProps {
  label: string;
  locales: LanguageOption[];
  current: string;
  onChange: (code: string) => void;
}

export function LanguageSwitcher({ label, locales, current, onChange }: LanguageSwitcherProps) {
  const shipped = locales.filter((l) => l.status === "shipped");
  return (
    <label className="va-language-switcher">
      <span className="va-visually-hidden">{label}</span>
      <select value={current} onChange={(e) => onChange(e.target.value)} aria-label={label}>
        {shipped.map((l) => (
          <option key={l.code} value={l.code}>
            {l.nativeName}
          </option>
        ))}
      </select>
    </label>
  );
}
