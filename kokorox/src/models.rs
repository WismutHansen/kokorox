use std::path::{Path, PathBuf};
use std::fs;
use std::error::Error;
use std::fmt;
use tokio::fs as async_fs;
use tokio::io::AsyncWriteExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest;
use serde_json;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ModelDownloadError {
    pub message: String,
}

impl fmt::Display for ModelDownloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Model download error: {}", self.message)
    }
}

impl Error for ModelDownloadError {}

// Model metadata for tracking and verification
#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub version: String,
    pub url: String,
    pub checksum: Option<String>,
    pub size: Option<u64>,
    pub language: Option<String>,
    pub model_type: ModelType,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub enum ModelType {
    Kokoro,
    DeepPhonemizer,
}

pub struct ModelManager {
    cache_dir: PathBuf,
    model_registry: HashMap<String, ModelInfo>,
}

impl ModelManager {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let cache_dir = Self::get_cache_dir()?;
        fs::create_dir_all(&cache_dir)?;
        
        let model_registry = Self::load_default_models();
        
        Ok(Self {
            cache_dir,
            model_registry,
        })
    }
    
    /// Get the appropriate cache directory for the platform
    fn get_cache_dir() -> Result<PathBuf, Box<dyn Error>> {
        let cache_dir = if let Some(cache_home) = std::env::var_os("XDG_CACHE_HOME") {
            PathBuf::from(cache_home).join("kokorox")
        } else if let Some(home) = std::env::var_os("HOME") {
            #[cfg(target_os = "macos")]
            {
                PathBuf::from(home).join("Library").join("Caches").join("kokorox")
            }
            #[cfg(not(target_os = "macos"))]
            {
                PathBuf::from(home).join(".cache").join("kokorox")
            }
        } else if let Some(appdata) = std::env::var_os("APPDATA") {
            PathBuf::from(appdata).join("kokorox").join("cache")
        } else {
            return Err("Could not determine cache directory".into());
        };
        
        Ok(cache_dir)
    }
    
    /// Load default model configurations
    fn load_default_models() -> HashMap<String, ModelInfo> {
        let mut models = HashMap::new();
        
        // Kokoro model
        models.insert("kokoro".to_string(), ModelInfo {
            name: "kokoro".to_string(),
            version: "v0.19".to_string(),
            url: "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files/kokoro-v0_19.onnx".to_string(),
            checksum: None, // TODO: Add checksums
            size: Some(98_000_000), // ~98MB
            language: None,
            model_type: ModelType::Kokoro,
        });
        
        // DeepPhonemizer models - using actual pre-trained models from Spring Media
        
        // English US models
        models.insert("deepphonemizer_en_us_ipa".to_string(), ModelInfo {
            name: "deepphonemizer_en_us_ipa".to_string(),
            version: "v0.0.10".to_string(),
            url: "https://public-asai-dl-models.s3.eu-central-1.amazonaws.com/DeepPhonemizer/en_us_cmudict_ipa_forward.pt".to_string(),
            checksum: None, // TODO: Add checksums
            size: Some(45_000_000), // ~45MB estimate
            language: Some("en_us".to_string()),
            model_type: ModelType::DeepPhonemizer,
        });
        
        models.insert("deepphonemizer_en_us_arpabet".to_string(), ModelInfo {
            name: "deepphonemizer_en_us_arpabet".to_string(),
            version: "v0.0.10".to_string(),
            url: "https://public-asai-dl-models.s3.eu-central-1.amazonaws.com/DeepPhonemizer/en_us_cmudict_forward.pt".to_string(),
            checksum: None,
            size: Some(45_000_000),
            language: Some("en_us".to_string()),
            model_type: ModelType::DeepPhonemizer,
        });
        
        // Multi-language Latin IPA model (supports en_uk, en_us, de, fr, es)
        models.insert("deepphonemizer_latin_ipa".to_string(), ModelInfo {
            name: "deepphonemizer_latin_ipa".to_string(),
            version: "v0.0.10".to_string(),
            url: "https://public-asai-dl-models.s3.eu-central-1.amazonaws.com/DeepPhonemizer/latin_ipa_forward.pt".to_string(),
            checksum: None,
            size: Some(60_000_000), // ~60MB estimate for multi-language model
            language: Some("multi".to_string()), // Special marker for multi-language
            model_type: ModelType::DeepPhonemizer,
        });
        
        models
    }
    
    /// Get the local path for a model, downloading if necessary
    pub async fn get_model_path(&self, model_name: &str) -> Result<PathBuf, Box<dyn Error>> {
        let model_path = self.cache_dir.join("models").join(model_name);
        
        // Check if model already exists
        if model_path.exists() {
            return Ok(model_path);
        }
        
        // Download the model
        self.download_model(model_name).await?;
        
        if model_path.exists() {
            Ok(model_path)
        } else {
            Err(Box::new(ModelDownloadError {
                message: format!("Model {} was downloaded but not found at expected path", model_name),
            }))
        }
    }
    
    /// Download a model if it doesn't exist locally
    async fn download_model(&self, model_name: &str) -> Result<(), Box<dyn Error>> {
        let model_info = self.model_registry.get(model_name)
            .ok_or_else(|| ModelDownloadError {
                message: format!("Unknown model: {}", model_name),
            })?;
        
        let models_dir = self.cache_dir.join("models");
        async_fs::create_dir_all(&models_dir).await?;
        
        let model_path = models_dir.join(model_name);
        
        println!("Downloading {} model from {}...", model_name, model_info.url);
        
        // Create progress bar
        let pb = ProgressBar::new(model_info.size.unwrap_or(0));
        pb.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")?
            .progress_chars("##-"));
        pb.set_message(format!("Downloading {}", model_name));
        
        // Download the file
        let response = reqwest::get(&model_info.url).await?;
        let total_size = response.content_length().unwrap_or(0);
        
        if total_size > 0 {
            pb.set_length(total_size);
        }
        
        let mut file = async_fs::File::create(&model_path).await?;
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        
        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }
        
        pb.finish_with_message(format!("Downloaded {}", model_name));
        file.flush().await?;
        
        // TODO: Verify checksum if available
        if let Some(_checksum) = &model_info.checksum {
            // Implement checksum verification
            println!("TODO: Verify checksum for {}", model_name);
        }
        
        println!("Successfully downloaded {} to {:?}", model_name, model_path);
        Ok(())
    }
    
    /// Get DeepPhonemizer model and config paths for a language
    /// Automatically selects the best available model for the language
    pub async fn get_deepphonemizer_paths(&self, language: &str) -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
        let model_name = self.select_best_deepphonemizer_model(language)?;
        let model_path = self.get_model_path(&model_name).await?;
        
        // Create appropriate config file
        let config_path = model_path.with_extension("yaml");
        if !config_path.exists() {
            self.create_deepphonemizer_config(&config_path, &model_name, language).await?;
        }
        
        Ok((model_path, config_path))
    }
    
    /// Select the best DeepPhonemizer model for a given language
    fn select_best_deepphonemizer_model(&self, language: &str) -> Result<String, Box<dyn Error>> {
        match language {
            // For English US, prefer IPA model for better compatibility with Kokoro
            "en_us" | "en-us" => Ok("deepphonemizer_en_us_ipa".to_string()),
            
            // For other supported languages, use the multi-language Latin IPA model
            "en_uk" | "en-uk" | "en_gb" | "en-gb" => Ok("deepphonemizer_latin_ipa".to_string()),
            "de" | "german" => Ok("deepphonemizer_latin_ipa".to_string()),
            "fr" | "french" => Ok("deepphonemizer_latin_ipa".to_string()),
            "es" | "spanish" => Ok("deepphonemizer_latin_ipa".to_string()),
            
            // For unsupported languages, fall back to multi-language model
            _ => {
                println!("Warning: Language '{}' not specifically supported, using multi-language model", language);
                Ok("deepphonemizer_latin_ipa".to_string())
            }
        }
    }
    
    /// Create a proper configuration file for DeepPhonemizer models
    async fn create_deepphonemizer_config(&self, config_path: &Path, model_name: &str, language: &str) -> Result<(), Box<dyn Error>> {
        let config = if model_name.contains("ipa") {
            // IPA-based models (CMUDict IPA or Latin IPA)
            serde_json::json!({
                "model_name": model_name,
                "target_language": language,
                "phoneme_format": "ipa",
                "preprocessing": {
                    "lowercase": true,
                    "punctuation_handling": "preserve",
                    "expand_abbreviations": true,
                    "char_repeats": 1
                },
                "model": {
                    "architecture": "transformer",
                    "d_model": 512,
                    "d_fft": 2048,
                    "heads": 8,
                    "layers": 6,
                    "dropout": 0.1
                },
                "inference": {
                    "batch_size": 1,
                    "beam_size": 1,
                    "max_length": 200
                }
            })
        } else if model_name.contains("arpabet") {
            // ARPABET-based models (CMUDict)
            serde_json::json!({
                "model_name": model_name,
                "target_language": language,
                "phoneme_format": "arpabet",
                "preprocessing": {
                    "lowercase": true,
                    "punctuation_handling": "preserve",
                    "expand_abbreviations": true,
                    "char_repeats": 1
                },
                "model": {
                    "architecture": "transformer",
                    "d_model": 512,
                    "d_fft": 2048,
                    "heads": 8,
                    "layers": 6,
                    "dropout": 0.1
                },
                "inference": {
                    "batch_size": 1,
                    "beam_size": 1,
                    "max_length": 200
                }
            })
        } else {
            // Generic fallback config
            serde_json::json!({
                "model_name": model_name,
                "target_language": language,
                "phoneme_format": "ipa",
                "preprocessing": {
                    "lowercase": true,
                    "punctuation_handling": "preserve",
                    "expand_abbreviations": true,
                    "char_repeats": 1
                },
                "model": {
                    "architecture": "transformer",
                    "d_model": 512,
                    "d_fft": 2048,
                    "heads": 8,
                    "layers": 6,
                    "dropout": 0.1
                },
                "inference": {
                    "batch_size": 1,
                    "beam_size": 1,
                    "max_length": 200
                }
            })
        };
        
        let yaml_content = serde_yaml::to_string(&config)?;
        async_fs::write(config_path, yaml_content).await?;
        
        println!("Created DeepPhonemizer config for {} at {:?}", model_name, config_path);
        Ok(())
    }
    
    /// Get Kokoro model path
    pub async fn get_kokoro_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        self.get_model_path("kokoro").await
    }
    
    /// List all available models
    pub fn list_models(&self) -> Vec<String> {
        self.model_registry.keys().cloned().collect()
    }
    
    /// Check if a model is available locally
    pub fn is_model_cached(&self, model_name: &str) -> bool {
        let model_path = self.cache_dir.join("models").join(model_name);
        model_path.exists()
    }
    
    /// Get cache directory path
    pub fn get_cache_directory(&self) -> &Path {
        &self.cache_dir
    }
}

// Global model manager instance
use std::sync::Arc;
use tokio::sync::Mutex;

lazy_static::lazy_static! {
    static ref GLOBAL_MODEL_MANAGER: Arc<Mutex<Option<ModelManager>>> = Arc::new(Mutex::new(None));
}

/// Get or initialize the global model manager
pub async fn get_model_manager() -> Result<Arc<Mutex<ModelManager>>, Box<dyn Error>> {
    let mut global_manager = GLOBAL_MODEL_MANAGER.lock().await;
    
    if global_manager.is_none() {
        *global_manager = Some(ModelManager::new()?);
    }
    
    // Clone the Arc to return a reference to the inner ModelManager
    Ok(Arc::new(Mutex::new(global_manager.take().unwrap())))
}