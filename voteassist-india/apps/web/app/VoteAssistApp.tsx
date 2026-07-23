"use client";

import { useMemo, useState } from "react";
import {
  answer,
  createSession,
  estimateProgress,
  getCurrentNode,
  isSessionComplete,
  voteAssistTreeV1,
} from "@voteassist/decision-engine";
import type { EngineState } from "@voteassist/decision-engine";
import { getEntry } from "@voteassist/knowledge";
import { pick, shippedLocales, uiStrings, type UiLocale } from "@voteassist/i18n";
import { LanguageSwitcher, NotOfficialBanner, ProgressBar, QuestionCard, TerminalResult } from "@voteassist/ui";

const tree = voteAssistTreeV1;

export function VoteAssistApp() {
  const [locale, setLocale] = useState<UiLocale>("en");
  const [started, setStarted] = useState(false);
  const [state, setState] = useState<EngineState>(() => createSession(tree));

  const t = uiStrings[locale];
  const node = getCurrentNode(tree, state);
  const complete = isSessionComplete(tree, state);
  const progress = estimateProgress(tree, state);

  const canGoBack = state.history.length > 0;

  function handleAnswer(value: string) {
    setState((prev) => answer(tree, prev, value));
  }

  function handleBack() {
    setState((prev) => {
      if (prev.history.length === 0) return prev;
      const trimmedHistory = prev.history.slice(0, -1);
      let replay = createSession(tree);
      for (const step of trimmedHistory) {
        replay = answer(tree, replay, step.value);
      }
      return replay;
    });
  }

  function handleRestart() {
    setState(createSession(tree));
    setStarted(false);
  }

  const resolvedCitations = useMemo(() => {
    if (!complete || node.type !== "terminal") return [];
    return node.citations
      .map((c) => getEntry(c.knowledgeBaseId))
      .filter((entry): entry is NonNullable<typeof entry> => Boolean(entry))
      .flatMap((entry) => entry.sources.map((s) => ({ title: s.title, url: s.url, publisher: s.publisher })));
  }, [complete, node]);

  return (
    <div className="va-app-shell">
      <header className="va-header">
        <div>
          <p className="va-app-name">{t.appName}</p>
          <p className="va-tagline">{t.tagline}</p>
        </div>
        <LanguageSwitcher
          label={t.language}
          locales={shippedLocales}
          current={locale}
          onChange={(code: string) => setLocale(code as UiLocale)}
        />
      </header>

      <NotOfficialBanner text={t.notOfficialBanner} />

      {!started ? (
        <div className="va-start-screen">
          <button type="button" className="va-start-button" onClick={() => setStarted(true)}>
            {t.startButton}
          </button>
          <div>
            <a className="va-kb-link" href="/knowledge">
              {t.browseKnowledgeBase}
            </a>
          </div>
        </div>
      ) : (
        <>
          <ProgressBar progress={progress} />

          {node.type === "question" ? (
            <QuestionCard
              prompt={pick(node.prompt, locale)}
              helpText={node.helpText ? pick(node.helpText, locale) : undefined}
              options={node.options.map((o) => ({ value: o.value, label: pick(o.label, locale) }))}
              onAnswer={handleAnswer}
              backLabel={t.backButton}
              onBack={canGoBack ? handleBack : undefined}
            />
          ) : (
            <TerminalResult
              title={pick(node.outcomeTitle, locale)}
              description={pick(node.outcomeDescription, locale)}
              recommendedFormsLabel={t.recommendedForms}
              recommendedForms={node.recommendedForms}
              checklistLabel={t.checklist}
              checklist={node.checklist.map((c) => pick(c, locale))}
              officialLinksLabel={t.officialLinks}
              officialLinkDisclaimer={t.notOfficialBanner}
              deepLinks={node.deepLinks.map((l) => ({ label: pick(l.label, locale), url: l.url }))}
              sourcesLabel={t.sources}
              citations={resolvedCitations}
              cautionLabel={t.caution}
              caution={pick(node.caution, locale)}
              restartLabel={t.startOver}
              onRestart={handleRestart}
            />
          )}
        </>
      )}

      <footer className="va-footer">
        <p>{t.footerDisclaimer}</p>
      </footer>
    </div>
  );
}
