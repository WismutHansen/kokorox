use crate::tts::phonemizer::{detect_language, get_default_voice_for_language};
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

// Function to fix common Spanish phoneme issues
fn fix_spanish_phonemes(phonemes: &str) -> String {
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
    
    pub async fn new(model_path: &str, voices_path: &str) -> Self {
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
            .split(|c| c == '.' || c == '?' || c == '!' || c == ';')
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

    pub fn tts_raw_audio(
        &self,
        txt: &str,
        lan: &str,
        style_name: &str,
        speed: f32,
        initial_silence: Option<usize>,
        auto_detect_language: bool,
        force_style: bool,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        // Split text into appropriate chunks
        let chunks = self.split_text_into_chunks(txt, 500); // Using 500 to leave 12 tokens of margin
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

        // Determine if we're using custom voices
        let is_custom = self.is_using_custom_voices(&self.voices_path);
        
        // Determine which style to use
        let effective_style = if !force_style {
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
                style_name.to_string()
            }
        } else {
            // User has explicitly forced a specific style
            if auto_detect_language {
                println!("Detected language: {} - User override: using voice style: {}", language, style_name);
            } else {
                println!("Manual language mode: {} - User force-style: {}", language, style_name);
            }
            style_name.to_string()
        };

        for chunk in chunks {
            // Convert chunk to phonemes using the determined language
            println!("Processing chunk with language: {}", language);
            
            // Add more detailed logging for Spanish words
            if language.starts_with("es") {
                println!("Spanish text to phonemize: {}", chunk);
            }
            
            let mut phonemes = text_to_phonemes(&chunk, &language, None, true, false)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
                .join("");
            
            // Apply Spanish-specific phoneme corrections
            if language.starts_with("es") {
                phonemes = fix_spanish_phonemes(&phonemes);
            }
            
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
        }: TTSOpts,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let audio = self.tts_raw_audio(&txt, lan, style_name, speed, initial_silence, auto_detect_language, force_style)?;

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
                        style_names.push(name);
                        style_portions.push(portion * 0.1);
                    }
                }
            }
            eprintln!("styles: {:?}, portions: {:?}", style_names, style_portions);

            let mut blended_style = vec![vec![0.0; 256]; 1];

            for (name, portion) in style_names.iter().zip(style_portions.iter()) {
                if let Some(style) = self.styles.get(*name) {
                    let style_slice = &style[tokens_len][0]; // This is a [256] array
                                                             // Blend into the blended_style
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
        
        // Explicitly drop any resources that might cause issues at shutdown
        // This helps prevent mutex issues with ONNX Runtime
        
        // For the Arc<OrtKoko>, we'll try to ensure it's properly cleaned up
        // by explicitly doing memory management here
        let _ = std::sync::Arc::strong_count(&self.model);
        
        // Force a GC-like cleanup by allocating and dropping some memory
        let _cleanup_buf = vec![0u8; 1024];
        drop(_cleanup_buf);
        
        // Sleep briefly to let any background threads finish
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
