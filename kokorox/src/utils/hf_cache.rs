use std::path::PathBuf;
use dirs::cache_dir;
use hf_hub::api::tokio::Api;
use ndarray::Array3;
use std::io::Write;
use zip::write::FileOptions;

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



/// Create a proper NPZ file from individual voice files
pub async fn download_and_create_voices_file(voice_names: Vec<&str>) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let cache_path = get_default_voices_path();
    
    // If combined file already exists, return it
    if cache_path.exists() {
        return Ok(cache_path);
    }
    
    println!("🎭 Creating NPZ file from {} individual voice files...", voice_names.len());
    println!("   Repository: {}", HF_REPO);
    std::fs::create_dir_all(cache_path.parent().unwrap())?;
    
    // Create NPZ file
    let mut npz_data = Vec::new();
    
    // NPZ is a ZIP file with .npy files inside
    // We'll create it in memory first, then write to disk
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut npz_data));
    
    for (i, voice_name) in voice_names.iter().enumerate() {
        println!("   [{}/{}] Processing voice: {}", i + 1, voice_names.len(), voice_name);
        
        // Download the individual voice file
        let voice_path = download_voice(voice_name).await?;
        let voice_data = std::fs::read(voice_path)?;
        
        // Individual voice files from HF have shape [510, 1, 256]
        let expected_size = 510 * 1 * 256 * 4; // 4 bytes per f32
        if voice_data.len() != expected_size {
            return Err(format!(
                "Voice file {} has incorrect size: {} bytes (expected {})",
                voice_name, voice_data.len(), expected_size
            ).into());
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
    
    println!("✅ NPZ voices file created at: {}", cache_path.display());
    Ok(cache_path)
}

/// Download default voices (comprehensive v1.0 voice collection)
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
        "bm_daniel", "bm_fable", "bm_george", "bm_lewis",
        // Spanish
        "ef_dora", "em_alex", "em_santa",
        // Portuguese  
        "pf_dora", "pm_alex", "pm_santa",
        // French
        "ff_siwis",
        // Italian
        "if_sara", "im_nicola", 
        // Japanese
        "jf_alpha", "jf_gongitsune", "jf_nezumi", "jf_tebukuro", "jm_kumo",
        // Chinese
        "zf_xiaobei", "zf_xiaoni", "zf_xiaoxiao", "zf_xiaoyi", "zm_yunjian", "zm_yunxi", "zm_yunxia", "zm_yunyang",
        // Hindi
        "hf_alpha", "hf_beta", "hm_omega", "hm_psi"
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