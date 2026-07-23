export function NotOfficialBanner({ text }: { text: string }) {
  return (
    <div className="va-banner" role="note">
      {text}
    </div>
  );
}
