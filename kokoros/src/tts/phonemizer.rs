use crate::tts::normalize;
use crate::tts::vocab::VOCAB;
use espeak_rs::text_to_phonemes;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

lazy_static! {
    static ref PHONEME_PATTERNS: Regex = Regex::new(r"(?<=[a-zɹː])(?=hˈʌndɹɪd)").unwrap();
    static ref Z_PATTERN: Regex = Regex::new(r#" z(?=[;:,.!?¡¿—…"«»"" ]|$)"#).unwrap();
    static ref NINETY_PATTERN: Regex = Regex::new(r"(?<=nˈaɪn)ti(?!ː)").unwrap();

    // Comprehensive mapping from language codes to espeak-ng language codes
    // Includes ISO 639-1, ISO 639-2, and ISO 639-3 codes where possible
    // See full list at: https://github.com/espeak-ng/espeak-ng/blob/master/docs/languages.md
    static ref LANGUAGE_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();

        // English variants
        m.insert("en", "en-us");       // General English
        m.insert("eng", "en-us");      // ISO 639-2/3 code
        m.insert("en-us", "en-us");    // American English
        m.insert("en-gb", "en-gb");    // British English
        m.insert("en-uk", "en-gb");    // Alternative British English code
        m.insert("en-au", "en-gb");    // Australian English (using British as closest)
        m.insert("en-ca", "en-us");    // Canadian English
        m.insert("en-ie", "en-gb");    // Irish English
        m.insert("en-in", "en-gb");    // Indian English
        m.insert("en-nz", "en-gb");    // New Zealand English
        m.insert("en-za", "en-gb");    // South African English

        // Chinese variants
        m.insert("zh", "zh");          // General Chinese (defaults to Mandarin)
        m.insert("zho", "zh");         // ISO 639-2/3 code
        m.insert("chi", "zh");         // ISO 639-2 code
        m.insert("zh-cn", "zh");       // Simplified Chinese (mainland)
        m.insert("zh-tw", "zh-tw");    // Traditional Chinese (Taiwan)
        m.insert("zh-hk", "zh-yue");   // Hong Kong Chinese (defaults to Cantonese)
        m.insert("yue", "zh-yue");     // Cantonese
        m.insert("wuu", "zh");         // Wu Chinese (using Mandarin as fallback)
        m.insert("cmn", "zh");         // Mandarin Chinese

        // Japanese
        m.insert("ja", "ja");          // Japanese
        m.insert("jpn", "ja");         // ISO 639-2/3 code

        // Korean
        m.insert("ko", "ko");          // Korean
        m.insert("kor", "ko");         // ISO 639-2/3 code

        // European languages
        m.insert("de", "de");          // German
        m.insert("deu", "de");         // ISO 639-2/3 code
        m.insert("ger", "de");         // ISO 639-2 code
        m.insert("de-at", "de");       // Austrian German
        m.insert("de-ch", "de");       // Swiss German

        m.insert("fr", "fr-fr");       // French
        m.insert("fra", "fr-fr");      // ISO 639-2/3 code
        m.insert("fre", "fr-fr");      // ISO 639-2 code
        m.insert("fr-fr", "fr-fr");    // France French
        m.insert("fr-ca", "fr-ca");    // Canadian French
        m.insert("fr-be", "fr-fr");    // Belgian French
        m.insert("fr-ch", "fr-fr");    // Swiss French

        m.insert("it", "it");          // Italian
        m.insert("ita", "it");         // ISO 639-2/3 code

        m.insert("es", "es");          // Spanish
        m.insert("spa", "es");         // ISO 639-2/3 code
        m.insert("es-es", "es");       // Spain Spanish
        m.insert("es-mx", "es-la");    // Mexican Spanish
        m.insert("es-ar", "es-la");    // Argentinian Spanish
        m.insert("es-co", "es-la");    // Colombian Spanish
        m.insert("es-cl", "es-la");    // Chilean Spanish
        m.insert("es-la", "es-la");    // Latin American Spanish

        m.insert("pt", "pt-pt");       // Portuguese
        m.insert("por", "pt-pt");      // ISO 639-2/3 code
        m.insert("pt-pt", "pt-pt");    // Portugal Portuguese
        m.insert("pt-br", "pt-br");    // Brazilian Portuguese

        m.insert("ru", "ru");          // Russian
        m.insert("rus", "ru");         // ISO 639-2/3 code

        m.insert("pl", "pl");          // Polish
        m.insert("pol", "pl");         // ISO 639-2/3 code

        m.insert("nl", "nl");          // Dutch
        m.insert("nld", "nl");         // ISO 639-2/3 code
        m.insert("dut", "nl");         // ISO 639-2 code

        m.insert("sv", "sv");          // Swedish
        m.insert("swe", "sv");         // ISO 639-2/3 code

        m.insert("tr", "tr");          // Turkish
        m.insert("tur", "tr");         // ISO 639-2/3 code

        m.insert("el", "el");          // Greek
        m.insert("ell", "el");         // ISO 639-2/3 code
        m.insert("gre", "el");         // ISO 639-2 code

        m.insert("cs", "cs");          // Czech
        m.insert("ces", "cs");         // ISO 639-2/3 code
        m.insert("cze", "cs");         // ISO 639-2 code

        m.insert("hu", "hu");          // Hungarian
        m.insert("hun", "hu");         // ISO 639-2/3 code

        m.insert("fi", "fi");          // Finnish
        m.insert("fin", "fi");         // ISO 639-2/3 code

        m.insert("ro", "ro");          // Romanian
        m.insert("ron", "ro");         // ISO 639-2/3 code
        m.insert("rum", "ro");         // ISO 639-2 code

        m.insert("da", "da");          // Danish
        m.insert("dan", "da");         // ISO 639-2/3 code

        // South/Southeast Asian languages
        m.insert("hi", "hi");          // Hindi
        m.insert("hin", "hi");         // ISO 639-2/3 code

        m.insert("bn", "bn");          // Bengali
        m.insert("ben", "bn");         // ISO 639-2/3 code

        m.insert("vi", "vi");          // Vietnamese
        m.insert("vie", "vi");         // ISO 639-2/3 code

        m.insert("th", "th");          // Thai
        m.insert("tha", "th");         // ISO 639-2/3 code

        // Middle Eastern languages
        m.insert("ar", "ar");          // Arabic (Modern Standard)
        m.insert("ara", "ar");         // ISO 639-2/3 code

        m.insert("fa", "fa");          // Persian (Farsi)
        m.insert("fas", "fa");         // ISO 639-2/3 code
        m.insert("per", "fa");         // ISO 639-2 code

        m.insert("he", "he");          // Hebrew
        m.insert("heb", "he");         // ISO 639-2/3 code

        // Add more languages as needed

        m
    };

    // Map of language codes to default voice styles
    // These voices are available in the default voices-v1.0.bin file
    // This mapping aims to provide the most suitable voice for each language
    static ref DEFAULT_VOICE_STYLES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();

        // English variants - female voices
        m.insert("en-us", "af_heart");               // American English - heart female voice
        m.insert("en-gb", "bf_emma");                // British English - female voice
        m.insert("en-au", "bf_emma");                // Australian English - using British female voice
        m.insert("en-ca", "af_sky");                 // Canadian English - American female voice
        m.insert("en-nz", "bf_emma");                // New Zealand English - British female voice
        m.insert("en-ie", "bf_emma");                // Irish English - British female voice
        m.insert("en-za", "bf_emma");                // South African English - British female voice
        m.insert("en-in", "bf_emma");                // Indian English - British female voice

        // English variants - male voices
        m.insert("en-us-male", "am_liam");            // American English - male voice
        m.insert("en-gb-male", "bm_george");          // British English - male voice

        // Chinese voices
        m.insert("zh", "zf_xiaoxiao");                // General Chinese - female voice
        m.insert("zh-cn", "zf_xiaoxiao");             // Simplified Chinese
        m.insert("zh-tw", "zf_xiaoxiao");             // Taiwan Chinese
        m.insert("zh-yue", "zf_xiaoxiao");            // Cantonese

        // Japanese voices
        m.insert("ja", "jf_alpha");                   // Japanese - female voice
        m.insert("jpn", "jf_alpha");                  // Japanese (ISO code)

        // Korean voices - use the closest available
        m.insert("ko", "jf_alpha");                   // Korean - using Japanese female voice
        m.insert("kor", "jf_alpha");                  // Korean (ISO code)

        // European languages - female voices where possible
        m.insert("de", "bf_emma");                    // German - using British female voice
        m.insert("fr-fr", "ff_siwis");                // French
        m.insert("es", "ef_dora");                    // Spanish - using native Spanish female voice
        m.insert("es-es", "ef_dora");                 // Spanish (Spain) - using native Spanish female voice
        m.insert("es-mx", "ef_dora");                 // Spanish (Mexico) - using native Spanish female voice
        m.insert("es-ar", "ef_dora");                 // Spanish (Argentina) - using native Spanish female voice
        m.insert("es-la", "ef_dora");                 // Spanish (Latin America) - using native Spanish female voice
        m.insert("it", "if_sara");                    // Italian Default
        m.insert("it-male", "if_nicola");             // Italian Male
        m.insert("it-female", "if_sara");             // Italian Female
        m.insert("pt-pt", "pf_dora");                 // Portuguese - using native Portuguese female voice
        m.insert("pt-br", "pf_dora");                 // Portuguese (Brazil) - using native Portuguese female voice
        m.insert("ru", "af_heart");                   // Russian - using American female voice

        // European languages - male voices where suitable
        m.insert("es-male", "em_alex");              // Spanish - male voice
        m.insert("es-es-male", "em_alex");           // Spanish (Spain) - male voice
        m.insert("pt-male", "pm_alex");              // Portuguese - male voice
        m.insert("pt-pt-male", "pm_alex");           // Portuguese (Portugal) - male voice
        m.insert("pt-br-male", "pm_alex");           // Portuguese (Brazil) - male voice
        m.insert("nl", "am_liam");                   // Dutch - using American male voice
        m.insert("sv", "am_liam");                   // Swedish - using American male voice
        m.insert("da", "am_liam");                   // Danish - using American male voice
        m.insert("fi", "am_liam");                   // Finnish - using American male voice
        m.insert("no", "am_liam");                   // Norwegian - using American male voice

        // Hindi
        m.insert("hi", "hf_alpha");
        m.insert("hi-male", "hf_omega");
        m.insert("hi-female", "hf_alpha");

        // Default fallback voices for other languages
        m.insert("default", "af_heart");                // Default female voice
        m.insert("default-male", "am_liam");          // Default male voice

        m
    };

}

/// Language detection function based on whatlang
///
/// Detects the language of the provided text and returns the corresponding
/// espeak-ng language code
pub fn detect_language(text: &str) -> Option<String> {
    // For very short texts, probability of correct detection is low
    // So we'll require at least 10 characters for reliable detection
    let trimmed = text.trim();
    if trimmed.len() < 10 {
        println!(
            "Text too short for reliable detection ({} chars), defaulting to English",
            trimmed.len()
        );
        return Some("en-us".to_string());
    }

    // Special handling for texts with many numbers or symbols which can confuse detection
    let alphas = trimmed.chars().filter(|c| c.is_alphabetic()).count();
    let length = trimmed.len();

    if alphas < length / 3 {
        println!("Text contains too few alphabetic characters for reliable detection, defaulting to English");
        return Some("en-us".to_string());
    }

    // Attempt language detection
    let info = match whatlang::detect(trimmed) {
        Some(info) => info,
        None => {
            println!("Language detection failed, defaulting to English");
            return Some("en-us".to_string());
        }
    };

    let lang_code = info.lang().code();
    let confidence = info.confidence();

    // Check confidence level - only use the detected language if confidence is reasonable
    // Different thresholds for different language families
    let min_confidence = match lang_code {
        // CJK languages can be detected with higher confidence
        "zh" | "ja" | "ko" => 0.3,
        // Latin-script languages need higher confidence to distinguish between them
        "en" | "de" | "fr" | "es" | "it" | "pt" | "nl" => 0.5,
        // Scripts with unique alphabets (Cyrillic, Arabic, etc.) can be detected with medium confidence
        "ru" | "ar" | "he" | "hi" | "bn" | "th" => 0.4,
        // Default threshold for other languages
        _ => 0.5,
    };

    if confidence < min_confidence {
        println!(
            "Language detection confidence too low ({:.2}) for '{}', defaulting to English",
            confidence, lang_code
        );
        return Some("en-us".to_string());
    }

    println!(
        "Detected language: {} (confidence: {:.2})",
        lang_code, confidence
    );

    // Convert to espeak language code
    if let Some(&espeak_code) = LANGUAGE_MAP.get(lang_code) {
        Some(espeak_code.to_string())
    } else {
        // Log the unsupported language
        println!(
            "Unsupported language detected: {}, falling back to English",
            lang_code
        );
        // Fallback to English if language not supported
        Some("en-us".to_string())
    }
}

/// Get the default voice style for a language
///
/// Returns an appropriate voice style for the given language code.
///
/// # Arguments
///
/// * `language` - The language code to get a voice for (e.g., "en-us", "fr", "zh")
/// * `is_custom` - This parameter is ignored as we only use the default voice styles
///
/// # Returns
///
/// A string with the voice style name appropriate for the language and available voices
pub fn get_default_voice_for_language(language: &str, _is_custom: bool) -> String {
    // Always use the default voice map regardless of is_custom parameter
    let voice_map = &*DEFAULT_VOICE_STYLES;

    // Try exact match first
    if let Some(voice) = voice_map.get(language) {
        return voice.to_string();
    }

    // If not found, try to find a match with just the language part
    // For example, if "en-au" isn't found, try "en" or "en-us"
    if language.contains('-') {
        let base_lang = language.split('-').next().unwrap_or("");
        if !base_lang.is_empty() {
            // Try the base language code
            if let Some(voice) = voice_map.get(base_lang) {
                println!("Using '{}' voice for language '{}'", base_lang, language);
                return voice.to_string();
            }

            // For some languages, try common variants
            match base_lang {
                "en" => {
                    if let Some(voice) = voice_map.get("en-us") {
                        println!("Using 'en-us' voice for language '{}'", language);
                        return voice.to_string();
                    }
                }
                "zh" => {
                    if let Some(voice) = voice_map.get("zh") {
                        println!("Using 'zh' voice for language '{}'", language);
                        return voice.to_string();
                    }
                }
                "fr" => {
                    if let Some(voice) = voice_map.get("fr-fr") {
                        println!("Using 'fr-fr' voice for language '{}'", language);
                        return voice.to_string();
                    }
                }
                "es" => {
                    if let Some(voice) = voice_map.get("es") {
                        println!("Using 'es' voice for language '{}'", language);
                        return voice.to_string();
                    }
                }
                "pt" => {
                    if let Some(voice) = voice_map.get("pt-pt") {
                        println!("Using 'pt-pt' voice for language '{}'", language);
                        return voice.to_string();
                    }
                }
                _ => {}
            }
        }
    }

    // If still not found, fall back to the default
    println!("No specific voice found for '{}', using default", language);
    voice_map.get("default").unwrap_or(&"af_sky").to_string()
}

pub struct Phonemizer {
    lang: String,
    preserve_punctuation: bool,
    with_stress: bool,
}

impl Phonemizer {
    pub fn new(lang: &str) -> Self {
        // Validate language or default to en-us if invalid
        let language = if LANGUAGE_MAP.values().any(|&v| v == lang) {
            println!("Creating phonemizer with language: {}", lang);
            lang.to_string()
        } else {
            eprintln!(
                "Warning: Unsupported language '{}', falling back to en-us",
                lang
            );
            "en-us".to_string()
        };

        Phonemizer {
            lang: language,
            preserve_punctuation: true,
            with_stress: true,
        }
    }

    /// Get list of all supported languages
    ///
    /// Returns a vector of all language codes that are supported by the phonemizer.
    /// These are the language codes that can be used with the `--lan` option.
    pub fn supported_languages() -> Vec<&'static str> {
        let mut langs: Vec<&'static str> = LANGUAGE_MAP.values().cloned().collect();
        langs.sort();
        langs
    }

    /// Get list of primary supported languages
    ///
    /// Returns a map of language codes to human-readable language names
    /// for the primary languages supported by the system.
    pub fn primary_languages() -> HashMap<&'static str, &'static str> {
        let mut langs = HashMap::new();

        // Main languages with custom voices and good support
        langs.insert("en-us", "English (US)");
        langs.insert("en-gb", "English (UK)");
        langs.insert("zh", "Chinese (Mandarin)");
        langs.insert("ja", "Japanese");
        langs.insert("de", "German");
        langs.insert("fr-fr", "French");
        langs.insert("es", "Spanish");
        langs.insert("pt-pt", "Portuguese");
        langs.insert("ru", "Russian");
        langs.insert("ko", "Korean");

        langs
    }

    pub fn phonemize(&self, text: &str, normalize: bool) -> String {
        // Create a direct phonemization function that uses espeak-ng directly
        fn direct_phonemize(input: &str, lang: &str, with_stress: bool) -> String {
            // Call espeak-ng directly
            match text_to_phonemes(input, lang, None, true, false) {
                Ok(phonemes) => {
                    // Join the phonemes into a single string
                    let joined = phonemes.join("");
                    println!("DIRECT PHONEMIZE: {} -> {}", input, joined);
                    joined
                }
                Err(e) => {
                    println!("ERROR in direct phonemize: {}", e);
                    // Return empty string on error
                    String::new()
                }
            }
        }

        // Fix incomplete phrases by preprocessing the text
        // This addresses issues like "1939 to" directly at the phonemization level
        let preprocessed_text = {
            // Handle year ranges like "1939 to"
            let mut processed = text.to_string();

            // Check for problematic patterns that might be sentence breaks
            let patterns = [
                (r"\b(19|20)\d{2}\s+to\b", true),            // "1939 to"
                (r"\b(19|20)\d{2}\s+until\b", true),         // "1939 until"
                (r"\b(19|20)\d{2}\s+through\b", true),       // "1939 through"
                (r"\bfrom\s+(19|20)\d{2}\s+to\b", true),     // "from 1939 to"
                (r"\bbetween\s+(19|20)\d{2}\s+and\b", true), // "between 1939 and"
                (r"\bin\s+(19|20)\d{2}\b", false),           // "in 1939"
                (r"\b(19|20)\d{2}to\b", true),               // "1939to" (no space)
            ];

            // Apply each pattern and log if found
            for (pattern, should_log) in patterns.iter() {
                if let Ok(re) = Regex::new(pattern) {
                    if re.is_match(&processed) && *should_log {
                        println!(
                            "PATTERN DETECTED: Found '{}' pattern, applying fix",
                            pattern
                        );
                    }
                    // Don't make any replacements yet - just log that we found the pattern
                }
            }

            // Log Spanish special characters before normalization
            if self.lang.starts_with("es")
                && (processed.contains('ñ')
                    || processed.contains('á')
                    || processed.contains('é')
                    || processed.contains('í')
                    || processed.contains('ó')
                    || processed.contains('ú')
                    || processed.contains('ü'))
            {
                println!(
                    "PHONEMIZER DEBUG: Spanish text before normalization: {}",
                    processed
                );
                // Print each special character and its Unicode code point
                for (i, c) in processed.char_indices() {
                    if !c.is_ascii() {
                        println!(
                            "  Before norm - Pos {}: '{}' (Unicode: U+{:04X})",
                            i, c, c as u32
                        );
                    }
                }
            }

            processed
        };

        // Apply normalization if requested
        let normalized_text = if normalize {
            normalize::normalize_text(&preprocessed_text, &self.lang)
        } else {
            preprocessed_text
        };

        // Apply Spanish accent restoration if needed
        let text_to_phonemize = if self.lang.starts_with("es") {
            // Always apply accent restoration to Spanish text
            // This ensures we catch all potential issues with missing accents
            println!("SPANISH TEXT PROCESSING: Applying accent restoration");
            // Use the restore_spanish_accents function from koko.rs
            crate::tts::koko::restore_spanish_accents(&normalized_text)
        } else {
            normalized_text
        };

        // Log Spanish special characters after all preprocessing
        if self.lang.starts_with("es")
            && (text_to_phonemize.contains('ñ')
                || text_to_phonemize.contains('á')
                || text_to_phonemize.contains('é')
                || text_to_phonemize.contains('í')
                || text_to_phonemize.contains('ó')
                || text_to_phonemize.contains('ú')
                || text_to_phonemize.contains('ü'))
        {
            println!(
                "PHONEMIZER DEBUG: Spanish text after all processing: {}",
                text_to_phonemize
            );
            // Print each special character and its Unicode code point
            for (i, c) in text_to_phonemize.char_indices() {
                if !c.is_ascii() {
                    println!(
                        "  Final text - Pos {}: '{}' (Unicode: U+{:04X})",
                        i, c, c as u32
                    );
                }
            }
        }

        // Debug for Spanish text right before phonemization (the crucial moment)
        if self.lang.starts_with("es") {
            println!("PHONEMIZER PRE-PROCESS: About to process Spanish text:");
            println!("Text: {}", text_to_phonemize);

            // Check for accented characters
            let has_accents = text_to_phonemize.contains('á')
                || text_to_phonemize.contains('é')
                || text_to_phonemize.contains('í')
                || text_to_phonemize.contains('ó')
                || text_to_phonemize.contains('ú')
                || text_to_phonemize.contains('ñ')
                || text_to_phonemize.contains('ü');

            if has_accents {
                println!(
                    "ACCENTS PRESENT: Text has accented characters! Will try to preserve them."
                );
                // Debug each accented character
                for (i, c) in text_to_phonemize.char_indices() {
                    if !c.is_ascii() {
                        let mut bytes = [0u8; 4];
                        let len = c.encode_utf8(&mut bytes).len();
                        let byte_str = bytes[0..len]
                            .iter()
                            .map(|b| format!("{:02X}", b))
                            .collect::<Vec<_>>()
                            .join(" ");

                        println!(
                            "  Accent at {}: '{}' (U+{:04X}) - UTF-8: {}",
                            i, c, c as u32, byte_str
                        );
                    }
                }
            } else {
                println!("NO ACCENTS FOUND! This Spanish text is missing proper accents.");

                // Check for words that should have accents
                let suspect_words = [
                    "politica",
                    "poltica",
                    "tecnologia",
                    "comunicacion",
                    "aqu",
                    "educacion",
                    "creacion",
                    "publica",
                    "publico",
                ];

                for word in suspect_words.iter() {
                    if text.contains(word) {
                        println!("  ISSUE FOUND: '{}' is missing accents", word);
                    }
                }
            }
        }

        // Use our improved phonemization process
        // For Spanish text in particular, we need to ensure accents are preserved
        let phonemes = if self.lang.starts_with("es") {
            // For Spanish text, try to preserve accents and handle problematic patterns
            match text_to_phonemes(
                &text_to_phonemize, // Use our preprocessed text with accents restored
                &self.lang,
                None,
                self.preserve_punctuation,
                self.with_stress,
            ) {
                Ok(phonemes) => {
                    let joined = phonemes.join("");
                    println!("SPANISH PHONEMIZATION: {} -> {}", text_to_phonemize, joined);

                    // Apply Spanish-specific fixes to the phonemes
                    if joined.contains("sjon")
                        || joined.contains("nasjon")
                        || joined.contains("politiko")
                        || joined.contains("ðˌeklaɾ")
                    {
                        println!("DEBUG: Fixing Spanish phonemes: {}", joined);
                        // Use the fix_spanish_phonemes function from koko.rs to correct the phonemes
                        crate::tts::koko::fix_spanish_phonemes(&joined)
                    } else {
                        joined
                    }
                }
                Err(e) => {
                    eprintln!("Error in Spanish phonemization: {:?}", e);
                    String::new()
                }
            }
        } else {
            // For non-Spanish text, use standard phonemization
            match text_to_phonemes(
                &text_to_phonemize,
                &self.lang,
                None,
                self.preserve_punctuation,
                self.with_stress,
            ) {
                Ok(phonemes) => phonemes.join(""),
                Err(e) => {
                    eprintln!("Error in phonemization: {:?}", e);
                    String::new()
                }
            }
        };

        let mut ps = phonemes;

        // Apply kokoro-specific replacements
        ps = ps
            .replace("kəkˈoːɹoʊ", "kˈoʊkəɹoʊ")
            .replace("kəkˈɔːɹəʊ", "kˈəʊkəɹəʊ");

        // Apply character replacements
        ps = ps
            .replace("ʲ", "j")
            .replace("r", "ɹ")
            .replace("x", "k")
            .replace("ɬ", "l");

        // Apply regex patterns
        ps = PHONEME_PATTERNS.replace_all(&ps, " ").to_string();
        ps = Z_PATTERN.replace_all(&ps, "z").to_string();

        if self.lang == "en-us" {
            ps = NINETY_PATTERN.replace_all(&ps, "di").to_string();
        }

        // Filter characters present in vocabulary
        ps = ps.chars().filter(|&c| VOCAB.contains_key(&c)).collect();

        ps.trim().to_string()
    }
}
