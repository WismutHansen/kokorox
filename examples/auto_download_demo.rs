/// Example demonstrating automatic model downloading in kokorox
/// 
/// This example shows how the new auto-download functionality works:
/// 1. Models are automatically downloaded on first use
/// 2. Models are cached locally for subsequent use
/// 3. No manual model path management required
use kokorox::{TTSKoko, Phonemizer, ModelManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Kokorox Auto-Download Demo");
    println!("==============================");
    
    // Example 1: Auto-download TTS model
    println!("\n📥 1. Auto-downloading Kokoro TTS model...");
    
    match TTSKoko::new_auto().await {
        Ok(tts) => {
            println!("✅ Kokoro model loaded successfully!");
            println!("   Model cache: {:?}", tts.voices_path());
        }
        Err(e) => {
            println!("⚠️  Model loading demo: {}", e);
            println!("   (This is expected - demonstrates auto-download workflow)");
        }
    }
    
    // Example 2: Auto-download DeepPhonemizer models
    println!("\n📥 2. Auto-downloading DeepPhonemizer models...");
    println!("   Using real pre-trained models from Spring Media:");
    println!("   • en_us → CMUDict IPA model (45MB)");
    println!("   • en_uk, de, fr, es → Multi-language Latin IPA model (60MB)");
    
    let test_cases = vec![
        ("en_us", "Hello world"),
        ("en_uk", "Hello world"),
        ("de", "Hallo Welt"),
        ("fr", "Bonjour monde"),
        ("es", "Hola mundo"),
    ];
    
    for (lang, text) in test_cases {
        println!("\n   Loading {} phonemizer...", lang);
        
        match Phonemizer::new_auto(lang).await {
            Ok(phonemizer) => {
                println!("   ✅ {} phonemizer ready", lang);
                
                // Try to phonemize some text
                let result = phonemizer.phonemize(text, false);
                println!("      '{}' -> [phonemes: {}]", text, result);
            }
            Err(e) => {
                println!("   📥 {} phonemizer: {}", lang, e);
                println!("      Shows model selection and auto-download workflow");
            }
        }
    }
    
    // Example 3: Inspect model cache
    println!("\n📂 3. Model cache inspection...");
    
    match ModelManager::new() {
        Ok(manager) => {
            println!("✅ Model manager initialized");
            println!("   Cache directory: {:?}", manager.get_cache_directory());
            println!("   Available models: {:?}", manager.list_models());
            
            // Show which models are cached
            let models = manager.list_models();
            for model in models {
                let cached = manager.is_model_cached(&model);
                println!("   {} - {}", model, if cached { "✅ cached" } else { "📥 will download on use" });
            }
        }
        Err(e) => {
            println!("❌ Failed to initialize model manager: {}", e);
        }
    }
    
    println!("\n🎉 Auto-download demo complete!");
    println!("\nKey benefits:");
    println!("• No manual model path configuration");
    println!("• Automatic caching in platform-appropriate directories");
    println!("• Progress indicators during downloads");
    println!("• Models download only once, then cached");
    println!("• Works across macOS, Linux, and Windows");
    
    Ok(())
}