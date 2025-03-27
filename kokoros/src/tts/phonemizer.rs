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
    
    // Map of language codes to default voice styles
    // These voices are available in the default voices-v1.0.bin file
    static ref DEFAULT_VOICE_STYLES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // English voices
        m.insert("en-us", "af_sarah.4+af_nicole.6");
        m.insert("en-gb", "bf_emma");
        
        // Chinese voices
        m.insert("zh", "zf_xiaoxiao");
        
        // Japanese voices
        m.insert("ja", "jf_alpha");
        
        // German voices
        m.insert("de", "bf_emma");
        
        // Default fallback for other languages
        m.insert("default", "af_sarah.4+af_nicole.6");
        m
    };
    
    // Map of language codes to default voice styles for the full voice set
    // These are available in the custom voices file after conversion
    static ref CUSTOM_VOICE_STYLES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // English voices
        m.insert("en-us", "en_eey");
        m.insert("en-gb", "en_bft");
        
        // Chinese voices
        m.insert("zh", "zh_awb");
        
        // Japanese voices
        m.insert("ja", "ja_fay");
        
        // German voices
        m.insert("de", "de_hft");
        
        // French voices
        m.insert("fr-fr", "fr_cft");
        
        // Spanish voices
        m.insert("es", "es_faz");
        
        // Portuguese voices
        m.insert("pt-pt", "pt_eey");
        
        // Russian voices
        m.insert("ru", "ru_erb");
        
        // Korean voices
        m.insert("ko", "ko_fay");
        
        // Default fallback for other languages
        m.insert("default", "en_eey");
        m
    };
}

// Language detection function based on whatlang
pub fn detect_language(text: &str) -> Option<String> {
    // For very short texts, probability of correct detection is low
    // So we'll require at least 5 characters
    if text.trim().len() < 5 {
        return Some("en-us".to_string());
    }

    let info = whatlang::detect(text)?;
    let lang_code = info.lang().code();
    
    // Check confidence level - only use the detected language if confidence is reasonable
    if info.confidence() < 0.5 {
        println!("Language detection confidence too low ({:.2}), defaulting to English", info.confidence());
        return Some("en-us".to_string());
    }
    
    // Convert to espeak language code
    if let Some(&espeak_code) = LANGUAGE_MAP.get(lang_code) {
        Some(espeak_code.to_string())
    } else {
        // Log the unsupported language
        println!("Unsupported language detected: {}, falling back to English", lang_code);
        // Fallback to English if language not supported
        Some("en-us".to_string())
    }
}

/// Get the default voice style for a language
/// 
/// If is_custom is true, it will return a voice from the custom voice set
/// (available after running the convert_pt_voices.py script)
pub fn get_default_voice_for_language(language: &str, is_custom: bool) -> String {
    let voice_map = if is_custom {
        &*CUSTOM_VOICE_STYLES
    } else {
        &*DEFAULT_VOICE_STYLES
    };
    
    voice_map.get(language).unwrap_or_else(|| voice_map.get("default").unwrap()).to_string()
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
