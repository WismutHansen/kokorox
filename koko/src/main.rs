use kokorox::tts::koko::TTSKoko;
use std::env;
use std::io::{self, Read};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse basic arguments
    let mut style = "af_bella";
    let mut language = "en-us";
    let mut mode = "text";
    let mut text = "Hello from Kokoro!";
    let mut output_wav = false;

    // Simple argument parsing
    for i in 0..args.len() {
        match args.get(i).map(|s| s.as_str()) {
            Some("-s") => {
                if let Some(s) = args.get(i + 1) {
                    style = s;
                }
            }
            Some("-l") => {
                if let Some(l) = args.get(i + 1) {
                    language = l;
                }
            }
            Some("--output-wav") => {
                output_wav = true;
            }
            Some("pipe") => {
                mode = "pipe";
            }
            Some("text") => {
                if let Some(t) = args.get(i + 1) {
                    text = t;
                    mode = "text";
                }
            }
            _ => {}
        }
    }

    let tts = TTSKoko::new("checkpoints/kokoro-v1.0.onnx", "data/voices-v1.0.bin").await;

    match mode {
        "pipe" => {
            // Read from stdin
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .expect("Failed to read from stdin");
            let text = buffer.trim();
            if !text.is_empty() {
                if output_wav {
                    // Output WAV to stdout
                    if let Err(e) = tts.tts_pipe_to_stdout(text, language, style) {
                        eprintln!("TTS pipe error: {e}");
                        std::process::exit(1);
                    }
                } else {
                    // Play audio directly (default behavior)
                    if let Err(e) = tts.tts_pipe_play(text, language, style) {
                        eprintln!("TTS pipe error: {e}");
                        std::process::exit(1);
                    }
                }
            }
        }
        _ => {
            tts.tts(text, language, style);
        }
    }
}
