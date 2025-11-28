use std::path::PathBuf;
use dirs::cache_dir;
use hf_hub::api::tokio::Api;
use ndarray::Array3;
use std::io::Write;
use zip::write::FileOptions;

const HF_REPO_V1: &str = "onnx-community/Kokoro-82M-v1.0-ONNX";
const HF_REPO_ZH: &str = "onnx-community/Kokoro-82M-v1.1-zh-ONNX";
const DEFAULT_MODEL_FILE: &str = "onnx/model.onnx";

/// Query HuggingFace API to get list of available voice files for a repository
pub async fn get_available_voices_from_hf(variant: ModelVariant) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let repo = variant.repo();
    let url = format!("https://huggingface.co/api/models/{}/tree/main/voices", repo);
    
    println!("Querying available voices from HuggingFace: {}", repo);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "kokorox")
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to query HuggingFace API: {}", response.status()).into());
    }
    
    let body = response.text().await?;
    let json: serde_json::Value = serde_json::from_str(&body)?;
    
    // Parse the JSON array and extract voice names
    let voices: Vec<String> = json
        .as_array()
        .ok_or("Expected JSON array from HuggingFace API")?
        .iter()
        .filter_map(|item| {
            let path = item.get("path")?.as_str()?;
            if path.ends_with(".bin") {
                // Extract filename without extension (path is like "voices/zf_001.bin")
                let filename = path.strip_prefix("voices/").unwrap_or(path);
                filename.strip_suffix(".bin").map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();
    
    println!("Found {} voice files", voices.len());
    Ok(voices)
}

/// Model variant for downloads
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModelVariant {
    /// English/multilingual model (v1.0)
    #[default]
    V1English,
    /// Chinese model (v1.1-zh)
    V1Chinese,
}

impl ModelVariant {
    /// Get the HuggingFace repository for this variant
    pub fn repo(&self) -> &'static str {
        match self {
            ModelVariant::V1English => HF_REPO_V1,
            ModelVariant::V1Chinese => HF_REPO_ZH,
        }
    }
    
    /// Get the subdirectory name for caching
    pub fn cache_subdir(&self) -> &'static str {
        match self {
            ModelVariant::V1English => "v1.0",
            ModelVariant::V1Chinese => "v1.1-zh",
        }
    }
}

/// Get the Hugging Face cache directory for Kokoro models
pub fn get_hf_cache_dir() -> PathBuf {
    cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("huggingface")
        .join("kokoro")
}

/// Get the cache directory for a specific model variant
pub fn get_variant_cache_dir(variant: ModelVariant) -> PathBuf {
    get_hf_cache_dir().join(variant.cache_subdir())
}

/// Get the default model path in HF cache
pub fn get_default_model_path() -> PathBuf {
    get_hf_cache_dir().join("model.onnx")
}

/// Get the model path for a specific variant
pub fn get_model_path_for_variant(variant: ModelVariant) -> PathBuf {
    get_variant_cache_dir(variant).join("model.onnx")
}

/// Get the default voices path in HF cache (for combined voices file)
pub fn get_default_voices_path() -> PathBuf {
    get_hf_cache_dir().join("voices.bin")
}

/// Get the voices path for a specific variant
pub fn get_voices_path_for_variant(variant: ModelVariant) -> PathBuf {
    get_variant_cache_dir(variant).join("voices.bin")
}

/// Download model from Hugging Face hub to cache
pub async fn download_model(model_type: Option<&str>) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    download_model_variant(model_type, ModelVariant::V1English).await
}

/// Download model from Hugging Face hub to cache for a specific variant
pub async fn download_model_variant(model_type: Option<&str>, variant: ModelVariant) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let api = Api::new()?;
    let repo = api.model(variant.repo().to_string());
    
    let model_file = match model_type {
        Some("fp16") => "onnx/model_fp16.onnx",
        Some("q4") => "onnx/model_q4.onnx", 
        Some("q4f16") => "onnx/model_q4f16.onnx",
        Some("q8f16") => "onnx/model_q8f16.onnx",
        Some("quantized") => "onnx/model_quantized.onnx",
        Some("uint8") => "onnx/model_uint8.onnx",
        Some("uint8f16") => "onnx/model_uint8f16.onnx",
        _ => DEFAULT_MODEL_FILE, // Default to full precision model
    };

    println!("Downloading Kokoro model from Hugging Face: {}", model_file);
    println!("   Repository: {}", variant.repo());
    
    let model_path = repo.get(model_file).await?;
    
    // Copy to our cache directory with a consistent name
    let cache_path = get_model_path_for_variant(variant);
    std::fs::create_dir_all(cache_path.parent().unwrap())?;
    std::fs::copy(&model_path, &cache_path)?;
    
    println!("Model cached at: {}", cache_path.display());
    Ok(cache_path)
}

/// Download a specific voice file from Hugging Face hub
pub async fn download_voice(voice_name: &str) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    download_voice_variant(voice_name, ModelVariant::V1English).await
}

/// Download a specific voice file from Hugging Face hub for a specific variant
pub async fn download_voice_variant(voice_name: &str, variant: ModelVariant) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let api = Api::new()?;
    let repo = api.model(variant.repo().to_string());
    
    let voice_file = format!("voices/{}.bin", voice_name);
    println!("Downloading voice: {}", voice_name);
    
    let voice_path = repo.get(&voice_file).await?;
    
    // Copy to our cache directory
    let cache_dir = get_variant_cache_dir(variant).join("voices");
    std::fs::create_dir_all(&cache_dir)?;
    let cache_path = cache_dir.join(format!("{}.bin", voice_name));
    std::fs::copy(&voice_path, &cache_path)?;
    
    Ok(cache_path)
}



/// Create a proper NPZ file from individual voice files
pub async fn download_and_create_voices_file(voice_names: Vec<&str>) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    download_and_create_voices_file_variant(voice_names, ModelVariant::V1English).await
}

/// Download a single voice and return its data along with the voice name
async fn download_single_voice(voice_name: String, variant: ModelVariant) -> Result<(String, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
    let voice_path = download_voice_variant(&voice_name, variant).await?;
    let voice_data = std::fs::read(&voice_path)?;
    Ok((voice_name, voice_data))
}

/// Create a proper NPZ file from individual voice files for a specific variant
pub async fn download_and_create_voices_file_variant(voice_names: Vec<&str>, variant: ModelVariant) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let cache_path = get_voices_path_for_variant(variant);
    
    // If combined file already exists, return it
    if cache_path.exists() {
        return Ok(cache_path);
    }
    
    let total_voices = voice_names.len();
    println!("Downloading {} voice files in parallel...", total_voices);
    println!("   Repository: {}", variant.repo());
    std::fs::create_dir_all(cache_path.parent().unwrap())?;
    
    // Download all voices in parallel
    let download_futures: Vec<_> = voice_names
        .iter()
        .map(|name| download_single_voice(name.to_string(), variant))
        .collect();
    
    let results = futures::future::join_all(download_futures).await;
    
    // Collect successful downloads and report errors
    let mut voice_data_map: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();
    let mut errors = Vec::new();
    
    for result in results {
        match result {
            Ok((name, data)) => {
                voice_data_map.insert(name, data);
            }
            Err(e) => {
                errors.push(e.to_string());
            }
        }
    }
    
    if !errors.is_empty() {
        eprintln!("Warning: {} voice downloads failed:", errors.len());
        for err in &errors {
            eprintln!("  - {}", err);
        }
    }
    
    if voice_data_map.is_empty() {
        return Err("All voice downloads failed".into());
    }
    
    println!("Successfully downloaded {} voices, creating NPZ file...", voice_data_map.len());
    
    // Create NPZ file
    let mut npz_data = Vec::new();
    
    // NPZ is a ZIP file with .npy files inside
    // We'll create it in memory first, then write to disk
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut npz_data));
    
    // Sort voice names for consistent ordering
    let mut sorted_names: Vec<_> = voice_data_map.keys().cloned().collect();
    sorted_names.sort();
    
    for (i, voice_name) in sorted_names.iter().enumerate() {
        println!("   [{}/{}] Processing voice: {}", i + 1, sorted_names.len(), voice_name);
        
        let voice_data = voice_data_map.get(voice_name).unwrap();
        
        // Individual voice files from HF have shape [510, 1, 256]
        let expected_size = 510 * 1 * 256 * 4; // 4 bytes per f32
        if voice_data.len() != expected_size {
            eprintln!("Warning: Voice file {} has incorrect size: {} bytes (expected {}), skipping",
                voice_name, voice_data.len(), expected_size);
            continue;
        }
        
        // Convert raw bytes to f32 array
        let float_data: Vec<f32> = voice_data
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();
        
        // Create ndarray with shape [510, 1, 256] but pad to [511, 1, 256] for compatibility
        let mut padded_data = float_data;
        // Add one more row of 256 zeros to make it 511 rows
        padded_data.extend(vec![0.0; 256]);
        
        // Create ndarray with shape [511, 1, 256] 
        let array = Array3::from_shape_vec((511, 1, 256), padded_data)
            .map_err(|e| format!("Failed to reshape voice data for {}: {}", voice_name, e))?;
        
        // Create temporary .npy file
        let temp_dir = std::env::temp_dir();
        let temp_npy_path = temp_dir.join(format!("{}.npy", voice_name));
        ndarray_npy::write_npy(&temp_npy_path, &array)?;
        
        // Read the .npy file content
        let npy_data = std::fs::read(&temp_npy_path)?;
        std::fs::remove_file(&temp_npy_path)?; // Clean up temp file
        
        // Add to ZIP file
        let zip_filename = format!("{}.npy", voice_name);
        zip.start_file(&zip_filename, FileOptions::<()>::default())?;
        zip.write_all(&npy_data)?;
    }
    
    // Finalize the ZIP file
    zip.finish()?;
    
    // Write the NPZ file to disk
    std::fs::write(&cache_path, npz_data)?;
    
    println!("NPZ voices file created at: {}", cache_path.display());
    Ok(cache_path)
}

/// Download default voices (comprehensive v1.0 voice collection)
pub async fn download_default_voices() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    download_default_voices_variant(ModelVariant::V1English).await
}

/// Download default voices for a specific variant by querying HuggingFace for available files
pub async fn download_default_voices_variant(variant: ModelVariant) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    // Query HuggingFace API to get the actual list of available voices
    let voices = get_available_voices_from_hf(variant).await?;
    
    if voices.is_empty() {
        return Err("No voice files found in the repository".into());
    }
    
    // Convert Vec<String> to Vec<&str> for the download function
    let voice_refs: Vec<&str> = voices.iter().map(|s| s.as_str()).collect();
    
    download_and_create_voices_file_variant(voice_refs, variant).await
}

/// Ensure model and voices are available, downloading if necessary
pub async fn ensure_files_available(
    custom_model_path: Option<&str>,
    custom_voices_path: Option<&str>,
    model_type: Option<&str>
) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error + Send + Sync>> {
    ensure_files_available_variant(custom_model_path, custom_voices_path, model_type, ModelVariant::V1English).await
}

/// Ensure model and voices are available for a specific variant
pub async fn ensure_files_available_variant(
    custom_model_path: Option<&str>,
    custom_voices_path: Option<&str>,
    model_type: Option<&str>,
    variant: ModelVariant,
) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error + Send + Sync>> {
    
    let model_path = if let Some(path) = custom_model_path {
        // User provided custom path
        let path = PathBuf::from(path);
        if !path.exists() {
            return Err(format!("Custom model path does not exist: {}", path.display()).into());
        }
        path
    } else {
        // Use HF cache
        let cache_path = get_model_path_for_variant(variant);
        if !cache_path.exists() {
            download_model_variant(model_type, variant).await?
        } else {
            println!("Using cached model: {}", cache_path.display());
            cache_path
        }
    };
    
    let voices_path = if let Some(path) = custom_voices_path {
        // User provided custom path  
        let path = PathBuf::from(path);
        if !path.exists() {
            return Err(format!("Custom voices path does not exist: {}", path.display()).into());
        }
        path
    } else {
        // Use HF cache
        let cache_path = get_voices_path_for_variant(variant);
        if !cache_path.exists() {
            download_default_voices_variant(variant).await?
        } else {
            println!("Using cached voices: {}", cache_path.display());
            cache_path
        }
    };
    
    Ok((model_path, voices_path))
}