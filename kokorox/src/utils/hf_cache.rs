use std::path::PathBuf;
use dirs::cache_dir;
use hf_hub::api::tokio::Api;

const HF_REPO: &str = "onnx-community/Kokoro-82M-v1.0-ONNX";
const DEFAULT_MODEL_FILE: &str = "onnx/model.onnx";

/// Get the Hugging Face cache directory for Kokoro models
pub fn get_hf_cache_dir() -> PathBuf {
    cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("huggingface")
        .join("kokoro")
}

/// Get the default model path in HF cache
pub fn get_default_model_path() -> PathBuf {
    get_hf_cache_dir().join("model.onnx")
}

/// Get the default voices path in HF cache (for combined voices file)
pub fn get_default_voices_path() -> PathBuf {
    get_hf_cache_dir().join("voices.bin")
}

/// Download model from Hugging Face hub to cache
pub async fn download_model(model_type: Option<&str>) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let api = Api::new()?;
    let repo = api.model(HF_REPO.to_string());
    
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

    println!("📦 Downloading Kokoro model from Hugging Face: {}", model_file);
    println!("   Repository: {}", HF_REPO);
    
    let model_path = repo.get(model_file).await?;
    
    // Copy to our cache directory with a consistent name
    let cache_path = get_default_model_path();
    std::fs::create_dir_all(cache_path.parent().unwrap())?;
    std::fs::copy(&model_path, &cache_path)?;
    
    println!("✅ Model cached at: {}", cache_path.display());
    Ok(cache_path)
}

/// Download a specific voice file from Hugging Face hub
pub async fn download_voice(voice_name: &str) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let api = Api::new()?;
    let repo = api.model(HF_REPO.to_string());
    
    let voice_file = format!("voices/{}.bin", voice_name);
    println!("🎤 Downloading voice: {}", voice_name);
    
    let voice_path = repo.get(&voice_file).await?;
    
    // Copy to our cache directory
    let cache_dir = get_hf_cache_dir().join("voices");
    std::fs::create_dir_all(&cache_dir)?;
    let cache_path = cache_dir.join(format!("{}.bin", voice_name));
    std::fs::copy(&voice_path, &cache_path)?;
    
    Ok(cache_path)
}

/// Download the original combined voices file as fallback
pub async fn download_original_voices_file() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let cache_path = get_default_voices_path();
    let original_url = "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/voices-v1.0.bin";
    
    println!("📦 Downloading original combined voices file...");
    println!("   URL: {}", original_url);
    
    std::fs::create_dir_all(cache_path.parent().unwrap())?;
    
    // Download the original combined voices file
    crate::utils::fileio::download_file_from_url(original_url, cache_path.to_string_lossy().as_ref())
        .await
        .map_err(|e| format!("Failed to download original voices file: {}", e))?;
    
    println!("✅ Combined voices file downloaded to: {}", cache_path.display());
    Ok(cache_path)
}

/// Create a combined voices file from individual voice files (NPZ format - not implemented yet)
pub async fn download_and_create_voices_file(_voice_names: Vec<&str>) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    // For now, fallback to downloading the original combined file
    // TODO: Implement proper NPZ creation from individual HF voice files
    println!("⚠️  NPZ creation from individual voices not yet implemented.");
    println!("   Falling back to original combined voices file...");
    
    download_original_voices_file().await
}

/// Download default voices (v1.0 voices)
pub async fn download_default_voices() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let default_voices = vec![
        // American Female
        "af_heart", "af_alloy", "af_aoede", "af_bella", "af_jessica", 
        "af_kore", "af_nicole", "af_nova", "af_river", "af_sarah", "af_sky",
        // American Male
        "am_adam", "am_echo", "am_eric", "am_fenrir", "am_liam", 
        "am_michael", "am_onyx", "am_puck", "am_santa",
        // British Female
        "bf_alice", "bf_emma", "bf_isabella", "bf_lily",
        // British Male
        "bm_daniel", "bm_fable", "bm_george", "bm_lewis"
    ];
    
    download_and_create_voices_file(default_voices).await
}

/// Ensure model and voices are available, downloading if necessary
pub async fn ensure_files_available(
    custom_model_path: Option<&str>,
    custom_voices_path: Option<&str>,
    model_type: Option<&str>
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
        let cache_path = get_default_model_path();
        if !cache_path.exists() {
            download_model(model_type).await?
        } else {
            println!("📦 Using cached model: {}", cache_path.display());
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
        let cache_path = get_default_voices_path();
        if !cache_path.exists() {
            download_default_voices().await?
        } else {
            println!("🎭 Using cached voices: {}", cache_path.display());
            cache_path
        }
    };
    
    Ok((model_path, voices_path))
}