//! Locale metadata, mirroring `packages/i18n/src/locales.ts` exactly (the
//! same 22 non-English Eighth Schedule languages tracked as "planned"
//! there, plus English/Hindi already shipped).

pub struct LocaleInfo {
    pub code: &'static str,
    pub english_name: &'static str,
}

pub const TARGET_LOCALES: &[LocaleInfo] = &[
    LocaleInfo { code: "hi", english_name: "Hindi" },
    LocaleInfo { code: "bn", english_name: "Bengali" },
    LocaleInfo { code: "ta", english_name: "Tamil" },
    LocaleInfo { code: "te", english_name: "Telugu" },
    LocaleInfo { code: "mr", english_name: "Marathi" },
    LocaleInfo { code: "gu", english_name: "Gujarati" },
    LocaleInfo { code: "kn", english_name: "Kannada" },
    LocaleInfo { code: "ml", english_name: "Malayalam" },
    LocaleInfo { code: "pa", english_name: "Punjabi" },
    LocaleInfo { code: "ur", english_name: "Urdu" },
    LocaleInfo { code: "or", english_name: "Odia" },
    LocaleInfo { code: "as", english_name: "Assamese" },
    LocaleInfo { code: "kok", english_name: "Konkani" },
    LocaleInfo { code: "mni", english_name: "Manipuri" },
    LocaleInfo { code: "doi", english_name: "Dogri" },
    LocaleInfo { code: "brx", english_name: "Bodo" },
    LocaleInfo { code: "sat", english_name: "Santali" },
    LocaleInfo { code: "mai", english_name: "Maithili" },
    LocaleInfo { code: "sd", english_name: "Sindhi" },
    LocaleInfo { code: "ne", english_name: "Nepali" },
    LocaleInfo { code: "ks", english_name: "Kashmiri" },
    LocaleInfo { code: "sa", english_name: "Sanskrit" },
];

pub fn find_locale(code: &str) -> Option<&'static LocaleInfo> {
    TARGET_LOCALES.iter().find(|locale| locale.code == code)
}
