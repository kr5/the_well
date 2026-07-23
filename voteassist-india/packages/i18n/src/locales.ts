/**
 * MVP language coverage: English (en) and Hindi (hi), fully wired end to end.
 * All other Eighth Schedule languages are represented as placeholders in
 * `supportedLocales` so the language switcher and content pipeline are built
 * to scale, but they are NOT yet translated or legally reviewed — see
 * docs/11-multilingual-strategy.md. Shipping a language means both the UI
 * chrome strings below AND every knowledge-base entry / decision-tree node
 * have been translated and review-gated; do not flip `status` to "shipped"
 * before both are true.
 */
export interface LocaleMeta {
  code: string;
  englishName: string;
  nativeName: string;
  status: "shipped" | "planned";
}

export const supportedLocales: LocaleMeta[] = [
  { code: "en", englishName: "English", nativeName: "English", status: "shipped" },
  { code: "hi", englishName: "Hindi", nativeName: "हिन्दी", status: "shipped" },
  { code: "bn", englishName: "Bengali", nativeName: "বাংলা", status: "planned" },
  { code: "ta", englishName: "Tamil", nativeName: "தமிழ்", status: "planned" },
  { code: "te", englishName: "Telugu", nativeName: "తెలుగు", status: "planned" },
  { code: "mr", englishName: "Marathi", nativeName: "मराठी", status: "planned" },
  { code: "gu", englishName: "Gujarati", nativeName: "ગુજરાતી", status: "planned" },
  { code: "kn", englishName: "Kannada", nativeName: "ಕನ್ನಡ", status: "planned" },
  { code: "ml", englishName: "Malayalam", nativeName: "മലയാളം", status: "planned" },
  { code: "pa", englishName: "Punjabi", nativeName: "ਪੰਜਾਬੀ", status: "planned" },
  { code: "ur", englishName: "Urdu", nativeName: "اردو", status: "planned" },
  { code: "or", englishName: "Odia", nativeName: "ଓଡ଼ିଆ", status: "planned" },
  { code: "as", englishName: "Assamese", nativeName: "অসমীয়া", status: "planned" },
  { code: "kok", englishName: "Konkani", nativeName: "कोंकणी", status: "planned" },
  { code: "mni", englishName: "Manipuri", nativeName: "মৈতৈলোন্", status: "planned" },
  { code: "doi", englishName: "Dogri", nativeName: "डोगरी", status: "planned" },
  { code: "brx", englishName: "Bodo", nativeName: "बड़ो", status: "planned" },
  { code: "sat", englishName: "Santali", nativeName: "ᱥᱟᱱᱛᱟᱲᱤ", status: "planned" },
  { code: "mai", englishName: "Maithili", nativeName: "मैथिली", status: "planned" },
  { code: "sd", englishName: "Sindhi", nativeName: "سنڌي", status: "planned" },
  { code: "ne", englishName: "Nepali", nativeName: "नेपाली", status: "planned" },
  { code: "ks", englishName: "Kashmiri", nativeName: "कॲशुर / کٲشُر", status: "planned" },
  { code: "sa", englishName: "Sanskrit", nativeName: "संस्कृतम्", status: "planned" },
];

export const shippedLocales = supportedLocales.filter((l) => l.status === "shipped");

export const uiStrings = {
  en: {
    appName: "VoteAssist India",
    tagline: "Understand what you need to do. We'll point you to the official ECI service to do it.",
    notOfficialBanner:
      "VoteAssist India is an independent, non-official guidance tool. It is not the Election Commission of India and cannot register you to vote or submit any application on your behalf.",
    startButton: "Find out what I need to do",
    startOver: "Start over",
    backButton: "Back",
    yourAnswers: "Your answers so far",
    recommendedForms: "Recommended form(s)",
    checklist: "Checklist",
    officialLinks: "Official next steps",
    sources: "Sources",
    caution: "Please note",
    browseKnowledgeBase: "Browse the knowledge base",
    language: "Language",
    footerDisclaimer:
      "This platform provides general procedural guidance only. It is not legal advice, does not take political positions, and always defers to the Election Commission of India for anything official.",
  },
  hi: {
    appName: "VoteAssist India",
    tagline: "जानें आपको क्या करना है। हम आपको आधिकारिक ECI सेवा तक पहुंचाएंगे।",
    notOfficialBanner:
      "VoteAssist India एक स्वतंत्र, गैर-आधिकारिक मार्गदर्शन उपकरण है। यह भारत निर्वाचन आयोग नहीं है और आपकी ओर से मतदाता पंजीकरण या कोई आवेदन जमा नहीं कर सकता।",
    startButton: "पता करें मुझे क्या करना है",
    startOver: "फिर से शुरू करें",
    backButton: "पीछे",
    yourAnswers: "आपके अब तक के उत्तर",
    recommendedForms: "अनुशंसित फॉर्म",
    checklist: "चेकलिस्ट",
    officialLinks: "आधिकारिक अगला कदम",
    sources: "स्रोत",
    caution: "कृपया ध्यान दें",
    browseKnowledgeBase: "ज्ञान आधार देखें",
    language: "भाषा",
    footerDisclaimer:
      "यह मंच केवल सामान्य प्रक्रियात्मक मार्गदर्शन प्रदान करता है। यह कानूनी सलाह नहीं है, कोई राजनीतिक रुख नहीं लेता, और किसी भी आधिकारिक कार्य के लिए हमेशा भारत निर्वाचन आयोग को प्राथमिकता देता है।",
  },
} as const;

export type UiLocale = keyof typeof uiStrings;
export type UiStringKey = keyof (typeof uiStrings)["en"];
