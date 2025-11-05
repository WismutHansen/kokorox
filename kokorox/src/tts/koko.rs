use crate::tts::phonemizer::{detect_language, get_default_voice_for_language, normalize_language_code};
use crate::tts::phonemizer::japanese_text_to_phonemes;
use crate::tts::tokenize::tokenize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::onn::ort_koko::{self};
use crate::utils;
use ndarray::Array3;
use ndarray_npy::NpzReader;
use std::fs::File;

use espeak_rs::text_to_phonemes;

#[derive(Debug, Clone)]
pub struct TTSOpts<'a> {
    pub txt: &'a str,
    pub lan: &'a str,
    pub auto_detect_language: bool,
    pub force_style: bool,  // Whether to override auto style selection
    pub style_name: &'a str,
    pub save_path: &'a str,
    pub mono: bool,
    pub speed: f32,
    pub initial_silence: Option<usize>,
    pub phonemes: bool,  // Whether input is IPA phonemes instead of text
}

#[derive(Clone)]
pub struct TTSKoko {
    #[allow(dead_code)]
    model_path: String,
    voices_path: String,
    model: Arc<ort_koko::OrtKoko>,
    styles: HashMap<String, Vec<[[f32; 256]; 1]>>,
    init_config: InitConfig,
}

#[derive(Clone)]
pub struct InitConfig {
    pub model_url: String,
    pub voices_url: String,
    pub sample_rate: u32,
}

impl Default for InitConfig {
    fn default() -> Self {
        Self {
            model_url: "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/kokoro-v1.0.onnx".into(),
            voices_url: "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/voices-v1.0.bin".into(),
            sample_rate: 24000,
        }
    }
}

// Function to restore accents in Spanish text
pub fn restore_spanish_accents(text: &str) -> String {
    let mut fixed = text.to_string();
    
    // Common Spanish words with accents that might be lost
    let replacements = [
        // Spanish verbs with missing accents on the final "o" 
        // Very common issue in the past tense (preterite)
        ("dur", "duró"),
        ("Dur", "Duró"),
        ("comenz", "comenzó"),
        ("termin", "terminó"),
        ("llam", "llamó"),
        ("habl", "habló"),
        ("escrib", "escribió"),
        ("recib", "recibió"),
        ("declar", "declaró"),
        ("viaj", "viajó"),
        ("entr", "entró"),
        ("pregunt", "preguntó"),
        ("cambi", "cambió"),
        ("dej", "dejó"),
        ("explic", "explicó"),
        ("pens", "pensó"),
        
        // Common politica/política variants
        ("politica", "política"),
        ("poltica", "política"),
        ("Politica", "Política"),
        ("Poltica", "Política"),
        
        // Words ending in -ía that are frequently missing the accent
        ("economia", "economía"),
        ("Economia", "Economía"),
        ("categoria", "categoría"),
        ("tecnologia", "tecnología"),
        ("fotografia", "fotografía"),
        ("geografia", "geografía"),
        ("filosofia", "filosofía"),
        ("psicologia", "psicología"),
        ("sociologia", "sociología"),
        ("biologia", "biología"),
        ("energia", "energía"),
        ("garantia", "garantía"),
        ("compania", "compañía"),
        ("teoria", "teoría"),
        ("melodia", "melodía"),
        ("autonomia", "autonomía"),
        ("ideologia", "ideología"),
        
        // Words with accent on í
        ("aqu", "aquí"),
        ("Aqu", "Aquí"),
        ("pais", "país"),
        ("calificacion", "calificación"),
        
        // Words with accent on é
        ("epoca", "época"),
        ("telefono", "teléfono"),
        ("telefonos", "teléfonos"),
        ("metodo", "método"),
        ("metodos", "métodos"),
        ("medico", "médico"),
        ("medica", "médica"),
        ("electrico", "eléctrico"),
        ("electrica", "eléctrica"),
        ("electronico", "electrónico"),
        ("electronica", "electrónica"),
        
        // Words with accent on á
        ("practico", "práctico"),
        ("practica", "práctica"),
        ("grafico", "gráfico"),
        ("grafica", "gráfica"),
        ("matematica", "matemática"),
        ("matematicas", "matemáticas"),
        ("trafico", "tráfico"),
        
        // Words with accent on ó
        ("perodo", "período"),
        ("periodico", "periódico"),
        ("proposito", "propósito"),
        ("propositos", "propósitos"),
        ("economico", "económico"),
        ("economica", "económica"),
        ("historico", "histórico"),
        ("historica", "histórica"),
        ("ultimos", "últimos"),
        ("ultimo", "último"),
        
        // Common pronouns and short words with accents
        ("el.", "él."),
        ("el,", "él,"),
        ("el?", "él?"),
        ("el!", "él!"),
        (" el ", " él "),
        
        ("mas ", "más "),
        ("esta ", "está "),
        ("este ", "esté "),
        ("si ", "sí "),
        ("tu ", "tú "),
        ("mi ", "mí "),
        
        // Words ending in -ión (extremely common in Spanish)
        ("innovacion", "innovación"),
        ("informacion", "información"),
        ("comunicacion", "comunicación"),
        ("educacion", "educación"),
        ("investigacion", "investigación"),
        ("explicacion", "explicación"),
        ("presentacion", "presentación"),
        ("decoracion", "decoración"),
        ("situacion", "situación"),
        ("generacion", "generación"),
        ("participacion", "participación"),
        ("poblacion", "población"),
        ("aplicacion", "aplicación"),
        ("relacion", "relación"),
        ("organizacion", "organización"),
        ("celebracion", "celebración"),
        ("comunicacin", "comunicación"),
        ("construccion", "construcción"),
        ("evolucion", "evolución"),
        ("direccion", "dirección"),
        ("coleccion", "colección"),
        ("identificacion", "identificación"),
        ("revolucion", "revolución"),
        ("administracion", "administración"),
        ("civilizacion", "civilización"),
        ("seccion", "sección"),
        ("proteccion", "protección"),
        ("declaracion", "declaración"),
        ("rebelion", "rebelión"),
        ("creacion", "creación"),
        ("creacin", "creación"),
        
        // Words with ñ
        ("companía", "compañía"),
        ("compani", "compañí"),
        ("Compania", "Compañía"),
        ("Compaa", "Compañía"),
        ("Espana", "España"),
        ("espanol", "español"),
        ("montana", "montaña"),
        ("manana", "mañana"),
        ("nino", "niño"),
        ("nina", "niña"),
        ("senor", "señor"),
        ("senora", "señora"),
        ("sueno", "sueño"),
        ("pequeno", "pequeño"),
        ("pequena", "pequeña"),
        
        // Words with ü
        ("linguistica", "lingüística"),
        ("bilinguismo", "bilingüismo"),
        ("pinguino", "pingüino"),
        ("verguenza", "vergüenza"),
        ("antiguedad", "antigüedad"),
        ("bilingue", "bilingüe"),
    ];
    
    // First log the text for debugging
    println!("ACCENT RESTORATION: Processing text: {}", fixed);
    
    // First, apply specific word replacements
    for (wrong, correct) in replacements.iter() {
        if fixed.contains(wrong) {
            // Only replace if this is a whole word match (to avoid partial matches)
            // e.g., replace "dur" with "duró" only when it's the whole word, not part of "durante"
            
            // For words with spaces, we can't use the word boundary technique, so apply directly
            if wrong.contains(" ") {
                fixed = fixed.replace(wrong, correct);
                continue;
            }
            
            // For single words, try to use word boundaries for more precise replacement
            let word_pattern = format!(r"\b{}\b", regex::escape(wrong));
            if let Ok(re) = regex::Regex::new(&word_pattern) {
                if re.is_match(&fixed) {
                    println!("ACCENT FIX: Found '{}', replacing with '{}'", wrong, correct);
                    fixed = re.replace_all(&fixed, *correct).to_string();
                }
            } else {
                // Fallback if regex fails
                fixed = fixed.replace(wrong, correct);
            }
        }
    }
    
    // Apply more aggressive Spanish verb accent restoration for past tense verbs
    // This covers very common cases like "dur" -> "duró"
    let past_tense_verbs_re = regex::Regex::new(r"\b([a-z]+)(r)\b").unwrap_or_else(|_| {
        println!("WARNING: Failed to create past tense verb regex");
        regex::Regex::new(r"unmatchable").unwrap()
    });
    
    if past_tense_verbs_re.is_match(&fixed) {
        println!("VERB CHECK: Found potential past tense verb(s) missing accents");
        fixed = past_tense_verbs_re.replace_all(&fixed, |caps: &regex::Captures| {
            // Only apply to short verbs (3-5 characters) to avoid false positives
            let stem = &caps[1];
            if stem.len() >= 2 && stem.len() <= 4 {
                // Common past tense verb pattern in Spanish
                format!("{}ó", stem)
            } else {
                // Return unchanged if not a likely candidate
                caps[0].to_string()
            }
        }).to_string();
    }
    
    // Give a summary of changes
    if fixed != text {
        println!("ACCENT RESTORATION: Fixed text: {}", fixed);
    }
    
    fixed
}

// Function to fix common Spanish phoneme issues
pub fn fix_spanish_phonemes(phonemes: &str) -> String {
    println!("DEBUG: Fixing Spanish phonemes: {}", phonemes);
    let mut fixed = phonemes.to_string();
    
    // Fix for words ending in "ción" (often mispronounced)
    // The correct phonemes should emphasize the "ón" sound and place stress on it
    if fixed.contains("sjon") {
        fixed = fixed.replace("sjon", "sjˈon");
    }
    
    // Fix for words ending in "ciones" (plural form)
    if fixed.contains("sjones") {
        fixed = fixed.replace("sjones", "sjˈones");
    }
    
    // Fix for "político" and similar words with accented i
    if fixed.contains("politiko") {
        fixed = fixed.replace("politiko", "polˈitiko");
    }
    
    // Common Spanish word corrections
    let corrections = [
        // Add stress markers for common words
        ("nasjon", "nasjˈon"),         // nación
        ("edukasjon", "edukasjˈon"),   // educación
        ("komunikasjon", "komunikasjˈon"), // comunicación
        ("oɾɣanisasjon", "oɾɣanisasjˈon"), // organización
        ("kondisjon", "kondisjˈon"),   // condición
        
        // Spanish stress patterns on penultimate syllable for words 
        // ending in 'n', 's', or vowel (without written accent)
        ("tɾabaxa", "tɾabˈaxa"),      // trabaja
        ("komida", "komˈida"),        // comida
        ("espeɾansa", "espeɾˈansa"),  // esperanza
        
        // Words with stress on final syllable (ending in consonants other than n, s)
        ("papeɫ", "papˈeɫ"),         // papel
        ("maðɾið", "maðɾˈið"),       // Madrid
        
        // Words with explicit accents
        ("politika", "polˈitika"),    // política
        ("ekonomia", "ekonomˈia"),    // economía
    ];
    
    for (pattern, replacement) in corrections.iter() {
        if fixed.contains(pattern) {
            fixed = fixed.replace(pattern, replacement);
        }
    }
    
    // Add more fixes here based on observations
    
    fixed
}

impl TTSKoko {
    pub fn sample_rate(&self) -> u32 {
        self.init_config.sample_rate
    }
    
    pub fn voices_path(&self) -> &str {
        &self.voices_path
    }
    
    /// Get a list of all available voice IDs
    pub fn get_available_voices(&self) -> Vec<String> {
        let mut voices: Vec<String> = self.styles.keys().cloned().collect();
        voices.sort();
        voices
    }
    
    /// Create a new TTSKoko instance with automatic HF cache downloads
    pub async fn new(model_path: Option<&str>, voices_path: Option<&str>) -> Self {
        Self::new_with_model_type(model_path, voices_path, None).await
    }

    /// Create a new TTSKoko instance with specific model type
    pub async fn new_with_model_type(model_path: Option<&str>, voices_path: Option<&str>, model_type: Option<&str>) -> Self {
        // Use HF cache logic to ensure files are available
        let (resolved_model_path, resolved_voices_path) = crate::utils::hf_cache::ensure_files_available(
            model_path,
            voices_path, 
            model_type
        ).await.expect("Failed to ensure model and voices files are available");
        
        Self::from_paths(resolved_model_path.to_string_lossy().as_ref(), resolved_voices_path.to_string_lossy().as_ref()).await
    }

    /// Create TTSKoko from explicit file paths (legacy method)
    pub async fn from_paths(model_path: &str, voices_path: &str) -> Self {
        Self::from_config(model_path, voices_path, InitConfig::default()).await
    }

    pub async fn from_config(model_path: &str, voices_path: &str, cfg: InitConfig) -> Self {
        if !Path::new(model_path).exists() {
            utils::fileio::download_file_from_url(cfg.model_url.as_str(), model_path)
                .await
                .expect("download model failed.");
        }

        if !Path::new(voices_path).exists() {
            utils::fileio::download_file_from_url(cfg.voices_url.as_str(), voices_path)
                .await
                .expect("download voices data file failed.");
        }

        let model = Arc::new(
            ort_koko::OrtKoko::new(model_path.to_string())
                .expect("Failed to create Kokoro TTS model"),
        );

        // TODO: if(not streaming) { model.print_info(); }
        // model.print_info();

        let styles = Self::load_voices(voices_path);

        TTSKoko {
            model_path: model_path.to_string(),
            voices_path: voices_path.to_string(),
            model,
            styles,
            init_config: cfg,
        }
    }
    
    // Check if the voices file is a custom voices file
    pub fn is_using_custom_voices(&self, data_path: &str) -> bool {
        // Check if the file path contains "custom"
        if data_path.contains("custom") {
            println!("Using custom voices file: {}", data_path);
            return true;
        }
        
        // Also check for specific known custom voice styles in the loaded styles
        let has_custom_styles = self.styles.keys().any(|k| 
            k.starts_with("en_") || 
            k.starts_with("zh_") || 
            k.starts_with("ja_") ||
            k.starts_with("fr_") ||
            k.starts_with("de_") || 
            k.starts_with("es_") || 
            k.starts_with("pt_") || 
            k.starts_with("ru_") || 
            k.starts_with("ko_")
        );
        
        if has_custom_styles {
            println!("Custom voice styles detected in: {}", data_path);
            return true;
        }
        
        println!("Using standard voices file: {}", data_path);
        false
    }

    fn split_text_into_chunks(&self, text: &str, max_tokens: usize) -> Vec<String> {
        let mut chunks = Vec::new();

        // First split by sentences - using common sentence ending punctuation
        let sentences: Vec<&str> = text
            .split(['.', '?', '!', ';'])
            .filter(|s| !s.trim().is_empty())
            .collect();

        let mut current_chunk = String::new();

        // Note: We don't use auto-detection in this function anymore
        // The language to use will be properly determined in tts_raw_audio
        // and phonemization will happen with the correct language there
        
        // For now we use detect_language as fallback for sentence chunking only
        let lang = detect_language(text).unwrap_or_else(|| "en-us".to_string());
        
        for sentence in sentences {
            // Clean up the sentence and add back punctuation
            let sentence = format!("{}.", sentence.trim());

            // Convert to phonemes to check token count
            let sentence_phonemes = text_to_phonemes(&sentence, &lang, None, true, false)
                .unwrap_or_default()
                .join("");
            let token_count = tokenize(&sentence_phonemes).len();

            if token_count > max_tokens {
                // If single sentence is too long, split by words
                let words: Vec<&str> = sentence.split_whitespace().collect();
                let mut word_chunk = String::new();

                for word in words {
                    let test_chunk = if word_chunk.is_empty() {
                        word.to_string()
                    } else {
                        format!("{} {}", word_chunk, word)
                    };

                    let test_phonemes = text_to_phonemes(&test_chunk, &lang, None, true, false)
                        .unwrap_or_default()
                        .join("");
                    let test_tokens = tokenize(&test_phonemes).len();

                    if test_tokens > max_tokens {
                        if !word_chunk.is_empty() {
                            chunks.push(word_chunk);
                        }
                        word_chunk = word.to_string();
                    } else {
                        word_chunk = test_chunk;
                    }
                }

                if !word_chunk.is_empty() {
                    chunks.push(word_chunk);
                }
            } else if !current_chunk.is_empty() {
                // Try to append to current chunk
                let test_text = format!("{} {}", current_chunk, sentence);
                let test_phonemes = text_to_phonemes(&test_text, &lang, None, true, false)
                    .unwrap_or_default()
                    .join("");
                let test_tokens = tokenize(&test_phonemes).len();

                if test_tokens > max_tokens {
                    // If combining would exceed limit, start new chunk
                    chunks.push(current_chunk);
                    current_chunk = sentence;
                } else {
                    current_chunk = test_text;
                }
            } else {
                current_chunk = sentence;
            }
        }

        // Add the last chunk if not empty
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }

    /// Split an IPA phoneme string into chunks without disturbing order or
    /// breaking inside words/syllables. This avoids splitting on '.' which is
    /// commonly used as a syllable separator in IPA and caused mid-utterance
    /// cuts and reordering when reassembled.
    fn split_phonemes_into_chunks(&self, ipa: &str, max_tokens: usize) -> Vec<String> {
        // Strategy: accumulate by whitespace-delimited words, measuring tokens
        // with `tokenize` directly on the phoneme string. This preserves
        // sequence order and does not split inside a word like "ˈæk.tʃu.əli".
        let mut chunks = Vec::new();
        let mut current = String::new();

        // Pre-trim to avoid leading spaces in first chunk
        let words: Vec<&str> = ipa.split_whitespace().collect();

        for word in words {
            let test = if current.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current, word)
            };

            let t_len = tokenize(&test).len();

            if t_len > max_tokens {
                if !current.is_empty() {
                    chunks.push(current);
                }
                // If a single word exceeds max_tokens, push it as its own chunk
                // to avoid infinite loop, though this should be rare with IPA.
                if tokenize(word).len() > max_tokens {
                    chunks.push(word.to_string());
                    current = String::new();
                } else {
                    current = word.to_string();
                }
            } else {
                current = test;
            }
        }

        if !current.is_empty() {
            chunks.push(current);
        }

        chunks
    }

    pub fn tts_raw_audio(
        &self,
        txt: &str,
        lan: &str,
        style_name: &str,
        speed: f32,
        initial_silence: Option<usize>,
        auto_detect_language: bool,
        force_style: bool,
        phonemes: bool,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // Split input into appropriate chunks
        // In phonemes mode, avoid text-based sentence splitting to preserve
        // phoneme order and prevent cutting inside syllables.
        let chunks = if phonemes {
            println!("PHONEMES MODE: Chunking by words with token budget");
            self.split_phonemes_into_chunks(txt, 500) // leave ~12 tokens margin
        } else {
            self.split_text_into_chunks(txt, 500) // leave ~12 tokens margin
        };
        let mut final_audio = Vec::new();

        // Determine language to use
        let language = if auto_detect_language {
            // Only detect language when auto-detect flag is enabled
            println!("Attempting language detection for input text...");
            if let Some(detected) = detect_language(txt) {
                println!("Detected language: {} (confidence is good)", detected);
                detected
            } else {
                println!("Language detection failed, falling back to specified language: {}", lan);
                lan.to_string()
            }
        } else {
            // Skip detection entirely when auto-detect is disabled
            // Just use the language specified with -l flag
            println!("Using manually specified language: {}", lan);
            lan.to_string()
        };

        // Normalize the language code to espeak-ng format
        let language = normalize_language_code(&language).unwrap_or_else(|| {
            eprintln!("Warning: Unsupported language '{}', falling back to en-us", language);
            "en-us".to_string()
        });

        // Determine if we're using custom voices
        let is_custom = self.is_using_custom_voices(&self.voices_path);
        
        // Determine which style to use
        // Special case: if force_style is true but the style is the default
        // English voice (af_heart) while the language is non-English, do not
        // force the style. This avoids accidentally overriding language-
        // appropriate voices when users pass --force-style without changing
        // the default style.
        let force_style_effective = if force_style && style_name == "af_heart" && !language.starts_with("en") {
            println!(
                "NOTE: Ignoring forced style 'af_heart' for non-English language '{}'; using language-appropriate voice.",
                language
            );
            false
        } else {
            force_style
        };

        let effective_style = if !force_style_effective {
            // Try to automatically select a voice appropriate for the language
            // This applies to both auto-detect and manual language selection modes
            let default_style = get_default_voice_for_language(&language, is_custom);
            
            // Check if the default style exists in our voices
            if self.styles.contains_key(&default_style) {
                if auto_detect_language {
                    println!("Detected language: {} - Using voice style: {}", language, default_style);
                } else {
                    println!("Manual language: {} - Using appropriate voice style: {}", language, default_style);
                }
                default_style
            } else {
                // Fall back to user-provided style if default not available
                if auto_detect_language {
                    println!("Detected language: {} - Default voice unavailable, using: {}", language, style_name);
                } else {
                    println!("Manual language: {} - No specific voice available, using: {}", language, style_name);
                }
                // Check if the user's style is available
                if !self.styles.contains_key(style_name) {
                    println!("WARNING: Specified style '{}' not found in available voices", style_name);
                    println!("Available voices: {:?}", self.styles.keys().collect::<Vec<_>>());
                    // Fall back to a default voice we know exists - first voice in the list
                    let fallback_style = self.styles.keys().next().unwrap().to_string();
                    println!("Falling back to first available voice: {}", fallback_style);
                    fallback_style
                } else {
                    style_name.to_string()
                }
            }
        } else {
            // User has explicitly forced a specific style
            if auto_detect_language {
                println!("Detected language: {} - User override: using voice style: {}", language, style_name);
            } else {
                println!("Manual language mode: {} - User force-style: {}", language, style_name);
            }
            
            // Check if the forced style exists (or if it's a valid mix)
            if style_name.contains("+") {
                // Voice mixing - validate each voice exists
                let mut all_valid = true;
                for style_part in style_name.split('+') {
                    if let Some((name, _)) = style_part.split_once('.') {
                        if !self.styles.contains_key(name) {
                            println!("WARNING: Voice '{}' in mix not found in available voices", name);
                            all_valid = false;
                        }
                    }
                }
                
                if !all_valid {
                    println!("Available voices: {:?}", self.styles.keys().collect::<Vec<_>>());
                    let fallback_style = self.styles.keys().next().unwrap().to_string();
                    println!("Falling back to first available voice: {}", fallback_style);
                    fallback_style
                } else {
                    style_name.to_string()
                }
            } else if !self.styles.contains_key(style_name) {
                println!("WARNING: Forced style '{}' not found in available voices", style_name);
                println!("Available voices: {:?}", self.styles.keys().collect::<Vec<_>>());
                let fallback_style = self.styles.keys().next().unwrap().to_string();
                println!("Falling back to first available voice: {}", fallback_style);
                fallback_style
            } else {
                style_name.to_string()
            }
        };

        for chunk in chunks {
            // Convert chunk to phonemes using the determined language
            println!("Processing chunk with language: {}", language);
            
            // Process the chunk to handle numbers and accents appropriately
            let processed_chunk = {
                // First, process numbers in the appropriate language
                // Explicitly pre-normalize the text for numbers before phonemization
                let mut processed = chunk.to_string();
                
                // Handle digits, numerals, etc.
                if chunk.chars().any(|c| c.is_ascii_digit()) {
                    // Apply number expansion functions from normalize.rs
                    // We need to do some targeted number extraction and expansion
                    
                    // First, extract standalone number sequences
                    let numbers_re = regex::Regex::new(r"\b\d+\b").unwrap();
                    for number_match in numbers_re.find_iter(&chunk) {
                        let number_str = number_match.as_str();
                        let number_expansion = crate::tts::normalize::expand_number_for_tts(number_str, &language);
                        
                        // Replace directly in our processed string
                        processed = processed.replace(number_str, &number_expansion);
                    }
                    
                    // Handle decimals
                    let decimal_re = regex::Regex::new(r"\d*\.\d+").unwrap();
                    for decimal_match in decimal_re.find_iter(&chunk) {
                        let decimal_str = decimal_match.as_str();
                        let decimal_expansion = crate::tts::normalize::expand_decimal_for_tts(decimal_str, &language);
                        
                        // Replace directly in our processed string
                        processed = processed.replace(decimal_str, &decimal_expansion);
                    }
                    
                    // Convert "2023" to "two thousand twenty-three", etc.
                    // We need to be careful about years that might be followed by words like "to"
                    // First, check for years followed directly by "to" (without space)
                    // This can happen due to preprocessing or text input issues
                    let year_pattern = r"\b(19|20)\d{2}to\b";
                    if let Ok(year_connected_re) = regex::Regex::new(year_pattern) {
                        for connected_match in year_connected_re.find_iter(&chunk) {
                            let connected_str = connected_match.as_str();
                            // Split this into year and "to" to prevent it from being processed as one token
                            if let Some(year_str) = connected_str.strip_suffix("to") {
                                let year_expansion = crate::tts::normalize::expand_number_for_tts(year_str, &language);
                                // Replace with proper spacing
                                let replacement = format!("{} to", year_expansion);
                                processed = processed.replace(connected_str, &replacement);
                                println!("YEAR PATTERN: Fixed connected year-to pattern: '{}' -> '{}'", 
                                         connected_str, replacement);
                            }
                        }
                    } else {
                        println!("WARNING: Error creating regex for year_pattern");
                    }
                    
                    // Additionally, check for specific problematic patterns that commonly appear
                    // Even with all our fixes, they might still show up
                    for year in ["1939", "1940", "1941", "1942", "1945"] {
                        // Fix "1939 to It" pattern where the capitalized "It" might still cause issues
                        let error_pattern = format!("{} to It", year);
                        if chunk.contains(&error_pattern) {
                            // Replace with lowercase to prevent issues in TTS processing
                            let fixed_pattern = format!("{} to it", year);
                            processed = processed.replace(&error_pattern, &fixed_pattern);
                            println!("YEAR PATTERN: Fixed capitalization issue: '{}' -> '{}'", 
                                     error_pattern, fixed_pattern);
                        }
                    }
                    
                    // Now handle normal years
                    let year_re = regex::Regex::new(r"\b(19|20)\d{2}\b").unwrap();
                    for year_match in year_re.find_iter(&chunk) {
                        let year_str = year_match.as_str();
                        let year_expansion = crate::tts::normalize::expand_number_for_tts(year_str, &language);
                        
                        // Replace directly in our processed string
                        processed = processed.replace(year_str, &year_expansion);
                    }
                    
                    println!("NUMBER EXPANDED: '{}' -> '{}'", chunk, processed);
                }
                
                // Then, for Spanish text, check if accents need to be fixed
                if language.starts_with("es") {
                    // Check if we have encoding issues with Spanish characters
                    let has_missing_accents = processed.contains("politica") || processed.contains("poltica") || 
                                            processed.contains("Aqu") || processed.contains("tecnologicas") ||
                                            processed.contains("publicas") || processed.contains("creacin");
                    
                    if has_missing_accents {
                        println!("FIXING ACCENTS: Detected possible missing accents in Spanish text");
                        let fixed = restore_spanish_accents(&processed);
                        println!("Original: {}", processed);
                        println!("After accent fix: {}", fixed);
                        fixed
                    } else {
                        processed
                    }
                } else {
                    processed
                }
            };
            
            // Add more detailed logging for Spanish words
            if language.starts_with("es") {
                println!("Spanish text to phonemize: {}", processed_chunk);
            }
            
            // Check if processed chunk has accented characters before phonemization
            if processed_chunk.contains('á') || processed_chunk.contains('é') || 
               processed_chunk.contains('í') || processed_chunk.contains('ó') || 
               processed_chunk.contains('ú') || processed_chunk.contains('ñ') {
                println!("PRE-PHONEMIZE: Text has accented characters");
                // Show each accented character
                for (i, c) in processed_chunk.char_indices() {
                    if !c.is_ascii() {
                        println!("  Accent at {}: '{}' (U+{:04X})", i, c, c as u32);
                    }
                }
            }
            
            let phonemes = if phonemes {
                // When --phonemes flag is used, treat input as IPA phonemes directly
                println!("PHONEMES MODE: Using input as IPA phonemes directly: {}", chunk);
                chunk.to_string()
            } else if language == "ja" {
                // Use jpreprocess for Japanese to properly handle kanji and particles
                println!("CALLING JPREPROCESS ON: {}", processed_chunk);
                let phonemes = japanese_text_to_phonemes(&processed_chunk)
                    .map_err(|e| {
                        println!("JPREPROCESS ERROR: {}", e);
                        // Fall back to espeak on error
                        eprintln!("Falling back to eSpeak-ng for Japanese");
                        let fallback = text_to_phonemes(&processed_chunk, &language, None, true, false)
                            .map(|p| p.join(""))
                            .unwrap_or_default();
                        fallback
                    })
                    .unwrap_or_else(|_| String::new());
                    
                println!("JPREPROCESS RESULT: {}", phonemes);
                phonemes
            } else {
                // This is where the phonemization happens for non-Japanese languages
                println!("CALLING PHONEMIZE ON: {}", processed_chunk);
                let mut phonemes = text_to_phonemes(&processed_chunk, &language, None, true, false)
                    .map_err(|e| {
                        println!("PHONEMIZE ERROR: {}", e);
                        Box::new(e) as Box<dyn std::error::Error>
                    })?
                    .join("");
                    
                // Check what happened to the accented characters
                println!("PHONEMIZE RESULT: {}", phonemes);
                
                // Apply Spanish-specific phoneme corrections
                if language.starts_with("es") {
                    phonemes = fix_spanish_phonemes(&phonemes);
                }
                
                phonemes
            };
            
            println!("phonemes: {}", phonemes);
            
            // Add special debug for Spanish problematic words
            if language.starts_with("es") && (chunk.contains("ción") || chunk.contains("politic")) {
                println!("DEBUG - Spanish special case detected:");
                println!("Original: {}", chunk);
                println!("Phonemes after fix: {}", phonemes);
            }
            let mut tokens = tokenize(&phonemes);

            for _ in 0..initial_silence.unwrap_or(0) {
                tokens.insert(0, 30);
            }

            // Get style vectors once - using the effective style determined above
            let styles = self.mix_styles(&effective_style, tokens.len())?;

            // pad a 0 to start and end of tokens
            let mut padded_tokens = vec![0];
            for &token in &tokens {
                padded_tokens.push(token);
            }
            padded_tokens.push(0);

            let tokens = vec![padded_tokens];

            match self.model.infer(tokens, styles.clone(), speed) {
                Ok(chunk_audio) => {
                    let chunk_audio: Vec<f32> = chunk_audio.iter().cloned().collect();
                    final_audio.extend_from_slice(&chunk_audio);
                }
                Err(e) => {
                    eprintln!("Error processing chunk: {:?}", e);
                    eprintln!("Chunk text was: {:?}", chunk);
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Chunk processing failed: {:?}", e),
                    )));
                }
            }
        }

        Ok(final_audio)
    }

    pub fn tts(
        &self,
        TTSOpts {
            txt,
            lan,
            auto_detect_language,
            force_style,
            style_name,
            save_path,
            mono,
            speed,
            initial_silence,
            phonemes,
        }: TTSOpts,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let audio = self.tts_raw_audio(txt, lan, style_name, speed, initial_silence, auto_detect_language, force_style, phonemes)?;

        // Save to file
        if mono {
            let spec = hound::WavSpec {
                channels: 1,
                sample_rate: self.init_config.sample_rate,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            };

            let mut writer = hound::WavWriter::create(save_path, spec)?;
            for &sample in &audio {
                writer.write_sample(sample)?;
            }
            writer.finalize()?;
        } else {
            let spec = hound::WavSpec {
                channels: 2,
                sample_rate: self.init_config.sample_rate,
                bits_per_sample: 32,
                sample_format: hound::SampleFormat::Float,
            };

            let mut writer = hound::WavWriter::create(save_path, spec)?;
            for &sample in &audio {
                writer.write_sample(sample)?;
                writer.write_sample(sample)?;
            }
            writer.finalize()?;
        }
        eprintln!("Audio saved to {}", save_path);
        Ok(())
    }

    pub fn mix_styles(
        &self,
        style_name: &str,
        tokens_len: usize,
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        if !style_name.contains("+") {
            if let Some(style) = self.styles.get(style_name) {
                let styles = vec![style[tokens_len][0].to_vec()];
                Ok(styles)
            } else {
                Err(format!("can not found from styles_map: {}", style_name).into())
            }
        } else {
            eprintln!("parsing style mix");
            let styles: Vec<&str> = style_name.split('+').collect();

            let mut style_names = Vec::new();
            let mut style_portions = Vec::new();

            for style in styles {
                if let Some((name, portion)) = style.split_once('.') {
                    if let Ok(portion) = portion.parse::<f32>() {
                        if !self.styles.contains_key(name) {
                            return Err(format!("Voice '{}' not found in available voices", name).into());
                        }
                        style_names.push(name);
                        style_portions.push(portion * 0.1);
                    }
                }
            }
            
            if style_names.is_empty() {
                return Err(format!("Invalid voice mix format '{}'. Use format: voice1.weight+voice2.weight (e.g., jf_alpha.4+am_echo.6)", style_name).into());
            }
            
            eprintln!("styles: {:?}, portions: {:?}", style_names, style_portions);

            let mut blended_style = vec![vec![0.0; 256]; 1];

            for (name, portion) in style_names.iter().zip(style_portions.iter()) {
                if let Some(style) = self.styles.get(*name) {
                    let style_slice = &style[tokens_len][0];
                    for j in 0..256 {
                        blended_style[0][j] += style_slice[j] * portion;
                    }
                }
            }
            Ok(blended_style)
        }
    }

    fn load_voices(voices_path: &str) -> HashMap<String, Vec<[[f32; 256]; 1]>> {
        let mut npz = NpzReader::new(File::open(voices_path).unwrap()).unwrap();
        let mut map = HashMap::new();

        for voice in npz.names().unwrap() {
            let voice_data: Result<Array3<f32>, _> = npz.by_name(&voice);
            let voice_data = voice_data.unwrap();
            let mut tensor = vec![[[0.0; 256]; 1]; 511];
            for (i, inner_value) in voice_data.outer_iter().enumerate() {
                for (j, inner_inner_value) in inner_value.outer_iter().enumerate() {
                    for (k, number) in inner_inner_value.iter().enumerate() {
                        tensor[i][j][k] = *number;
                    }
                }
            }
            map.insert(voice, tensor);
        }

        let sorted_voices = {
            let mut voices = map.keys().collect::<Vec<_>>();
            voices.sort();
            voices
        };

        println!("voice styles loaded: {:?}", sorted_voices);
        map
    }
    
    // Method to properly clean up resources before application exit
    // Call this explicitly when done with the TTS engine to avoid segfault
    pub fn cleanup(&self) {
        // This method exists to provide a hook for proper cleanup
        println!("Cleaning up TTS engine resources...");
        
        // Give ONNX Runtime background threads time to complete any pending work
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // For the Arc<OrtKoko>, check if we're the only holder of the reference
        let count = std::sync::Arc::strong_count(&self.model);
        println!("Current reference count to ONNX model: {}", count);
        
        // If we're the only reference holder, we can try to explicitly drop it
        if count == 1 {
            // This is a hacky way to avoid the mutex error - create a block so the
            // temporary cloned Arc will be dropped at the end of this scope
            {
                // Create a new reference then drop it to trigger proper cleanup
                let _temp_clone = self.model.clone();
                // Let it drop here
            }
            
            // Give time for any mutex operations to complete
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        
        // Force a GC-like cleanup by allocating and dropping some memory
        {
            let _cleanup_buf = vec![0u8; 4096];
            // Drop it here
        }
        
        // Sleep to let any background threads finish
        std::thread::sleep(std::time::Duration::from_millis(20));
        
        // Note: Unfortunately Rust doesn't give us explicit control over thread synchronization
        // for the ONNX runtime internals. The best we can do is introduce these delays to
        // reduce the likelihood of the mutex error.
    }
}
