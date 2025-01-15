mod onn;
mod tts;
mod utils;

use clap::Parser;
use tts::koko::TTSKoko;

#[derive(Parser, Debug)]
#[command(name = "kokoros")]
#[command(version = "0.1")]
#[command(author = "Lucas Jin")]
struct Cli {
    #[arg(short = 't', long = "text", value_name = "TEXT")]
    text: Option<String>,
}

fn main() {
    let args = Cli::parse();

    let txt = args.text.unwrap_or_else(|| {
        r#"
        Hello, This is Kokoro. Your amazing AI TTS! A TTS model with only 82 million parameters that achieve incredible audio quality. 
        This is the one of the best Rust inference, I help you will like it. 
        Please give us a star if you do, thank you very much.
        "#
        .to_string()
    });

    let tts = TTSKoko::new("checkpoints/kokoro-v0_19.onnx");

    tts.tts(&txt, "en-us", "af_sky");
}
