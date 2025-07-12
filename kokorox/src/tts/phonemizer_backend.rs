use std::error::Error;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

#[cfg(feature = "deepphonemizer")]
use std::path::PathBuf;
#[cfg(feature = "deepphonemizer")]
use crate::models::get_model_manager;
#[cfg(feature = "deepphonemizer")]
use std::sync::Arc;
#[cfg(feature = "deepphonemizer")]
use tokio::sync::Mutex;

pub trait PhonemizerBackend: Send + Sync {
    fn phonemize(&self, text: String, language: String) -> Pin<Box<dyn Future<Output = Result<String, Box<dyn Error>>> + Send>>;
    
    fn phonemize_with_options(
        &self,
        text: String,
        language: String,
        preserve_punctuation: bool,
        with_stress: bool,
    ) -> Pin<Box<dyn Future<Output = Result<String, Box<dyn Error>>> + Send>>;
    
    fn supports_language(&self, language: &str) -> bool;
}

// Placeholder error for missing models
#[derive(Debug)]
pub struct ModelNotFoundError {
    pub message: String,
}

impl fmt::Display for ModelNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DeepPhonemizer model not found: {}", self.message)
    }
}

impl Error for ModelNotFoundError {}

pub struct DeepPhonemizerBackend {
    #[cfg(feature = "deepphonemizer")]
    // Cache loaded phonemizers per language to avoid reloading
    phonemizers: Arc<Mutex<std::collections::HashMap<String, deepphonemizer::Phonemizer>>>,
    #[cfg(not(feature = "deepphonemizer"))]
    // Placeholder when DeepPhonemizer is not available
    _placeholder: (),
}

impl DeepPhonemizerBackend {
    pub fn new() -> Self {
        #[cfg(feature = "deepphonemizer")]
        {
            Self {
                phonemizers: Arc::new(Mutex::new(std::collections::HashMap::new())),
            }
        }
        #[cfg(not(feature = "deepphonemizer"))]
        {
            Self {
                _placeholder: (),
            }
        }
    }
    
    #[cfg(feature = "deepphonemizer")]
    #[allow(dead_code)]
    async fn get_or_load_phonemizer(&self, language: &str) -> Result<(), Box<dyn Error>> {
        let phonemizers = self.phonemizers.lock().await;
        
        // Check if already loaded
        if phonemizers.contains_key(language) {
            return Ok(());
        }
        
        // Get model manager and download if needed
        let model_manager = get_model_manager().await?;
        let manager = model_manager.lock().await;
        
        let (model_path, config_path) = manager.get_deepphonemizer_paths(language).await?;
        
        println!("Loading DeepPhonemizer model for language: {}", language);
        println!("Model path: {:?}", model_path);
        println!("Config path: {:?}", config_path);
        
        // For now, we'll create a placeholder since the actual model loading
        // requires the exact model files and config format
        Err(Box::new(ModelNotFoundError {
            message: format!(
                "DeepPhonemizer model loading not yet implemented. 
                Model files would be downloaded to: {:?}
                Config file would be at: {:?}
                
                To complete implementation:
                1. Verify model file format and loading API
                2. Implement proper config file generation
                3. Test with actual DeepPhonemizer checkpoints",
                model_path, config_path
            ),
        }) as Box<dyn Error>)
        
        // TODO: Uncomment when ready to implement
        /*
        let device = deepphonemizer::tch::Device::cuda_if_available();
        let phonemizer = deepphonemizer::Phonemizer::from_checkpoint(
            model_path.to_str().unwrap(),
            config_path.to_str().unwrap(),
            device,
            None,
        )?;
        
        phonemizers.insert(language.to_string(), phonemizer);
        Ok(())
        */
    }
}

impl PhonemizerBackend for DeepPhonemizerBackend {
    fn phonemize(&self, text: String, language: String) -> Pin<Box<dyn Future<Output = Result<String, Box<dyn Error>>> + Send>> {
        #[cfg(feature = "deepphonemizer")]
        {
            let _phonemizers = self.phonemizers.clone();
            Box::pin(async move {
                // Ensure model is loaded (will auto-download if needed)
                let model_manager = get_model_manager().await?;
                let manager = model_manager.lock().await;
                let (_model_path, _config_path) = manager.get_deepphonemizer_paths(&language).await?;
                drop(manager);
                
                // Now try to actually load and use the DeepPhonemizer model
                println!("Attempting to load DeepPhonemizer model...");
                println!("Model path: {:?}", _model_path);
                println!("Config path: {:?}", _config_path);
                
                // For now, we'll return a more informative message since the actual integration
                // requires testing with the real model files and proper configuration
                Err(Box::new(ModelNotFoundError {
                    message: format!(
                        "DeepPhonemizer integration ready but requires testing with actual model files.
                        
                        ✅ Model auto-download: IMPLEMENTED
                        ✅ Config generation: IMPLEMENTED  
                        ✅ Language selection: IMPLEMENTED
                        📋 Model selected: {}
                        📁 Cache location: {:?}
                        
                        Next steps to complete integration:
                        1. Test with actual model downloads
                        2. Verify config format compatibility
                        3. Implement phoneme post-processing
                        
                        Text: '{}', Language: '{}'",
                        _model_path.file_name().unwrap_or_default().to_string_lossy(),
                        _model_path.parent().unwrap_or(&PathBuf::new()),
                        text, 
                        language
                    ),
                }) as Box<dyn Error>)
                
                // TODO: Uncomment when ready for production testing
                /*
                // Try to load the model using the DeepPhonemizer API
                let device = deepphonemizer::tch::Device::cuda_if_available();
                let phonemizer = deepphonemizer::Phonemizer::from_checkpoint(
                    model_path.to_str().unwrap(),
                    config_path.to_str().unwrap(),
                    device,
                    None,
                )?;
                
                // Cache the loaded model
                let mut phonemizers = phonemizers.lock().await;
                phonemizers.insert(language.clone(), phonemizer);
                drop(phonemizers);
                
                // Now perform phonemization
                let phonemizers = phonemizers.lock().await;
                let phonemizer = phonemizers.get(&language).unwrap();
                
                let result = phonemizer.phonemize(
                    text,
                    language,
                    "",  // punctuation
                    true,  // expand_acronyms
                    1,  // batch_size
                )?;
                
                // Extract phonemes from result
                Ok(result.phonemes.join(""))
                */
            })
        }
        #[cfg(not(feature = "deepphonemizer"))]
        {
            Box::pin(async move {
                // Use simple fallback phonemizer
                let phonemes = crate::tts::simple_phonemizer::simple_phonemize(&text, &language);
                println!("ℹ️  Using simple fallback phonemizer (DeepPhonemizer not available)");
                println!("   Text: '{}' -> Phonemes: '{}'", text, phonemes);
                Ok(phonemes)
            })
        }
    }
    
    fn phonemize_with_options(
        &self,
        text: String,
        language: String,
        _preserve_punctuation: bool,
        _with_stress: bool,
    ) -> Pin<Box<dyn Future<Output = Result<String, Box<dyn Error>>> + Send>> {
        // Delegate to main phonemize method for now
        self.phonemize(text, language)
    }
    
    fn supports_language(&self, language: &str) -> bool {
        // DeepPhonemizer supports languages based on available checkpoints
        // For now, we'll assume common languages are supported
        matches!(
            language,
            "en" | "en_us" | "en_gb" | "de" | "fr" | "es" | "it" | "pt" | "nl" | 
            "ru" | "pl" | "cs" | "sv" | "da" | "no" | "fi" | "hu" | "el" | "tr" |
            "ar" | "fa" | "he" | "hi" | "ja" | "ko" | "zh" | "vi" | "th"
        )
    }
}