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
    
    // Map of ISO 639-1 language codes to espeak language codes
    static ref LANGUAGE_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("en", "en-us");
        m.insert("zh", "zh");
        m.insert("ja", "ja");
        m.insert("de", "de");
        m.insert("fr", "fr-fr");
        m.insert("it", "it");
        m.insert("es", "es");
        m.insert("pt", "pt-pt");
        m.insert("ru", "ru");
        m.insert("ko", "ko");
        m.insert("ar", "ar");
        m.insert("hi", "hi");
        m
    };
}

// Language detection function based on whatlang
pub fn detect_language(text: &str) -> Option<String> {
    let info = whatlang::detect(text)?;
    let lang_code = info.lang().code();
    
    // Convert to espeak language code
    if let Some(&espeak_code) = LANGUAGE_MAP.get(lang_code) {
        Some(espeak_code.to_string())
    } else {
        // Fallback to English if language not supported
        Some("en-us".to_string())
    }
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
            lang.to_string()
        } else {
            eprintln!("Warning: Unsupported language '{}', falling back to en-us", lang);
            "en-us".to_string()
        };

        Phonemizer {
            lang: language,
            preserve_punctuation: true,
            with_stress: true,
        }
    }
    
    // Get list of supported languages
    pub fn supported_languages() -> Vec<&'static str> {
        LANGUAGE_MAP.values().cloned().collect()
    }

    pub fn phonemize(&self, text: &str, normalize: bool) -> String {
        let text = if normalize {
            normalize::normalize_text(text)
        } else {
            text.to_string()
        };

        // Use espeak-rs directly for phonemization
        let phonemes = match text_to_phonemes(
            &text,
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
