mod onn;
mod tts;
mod utils;

use tts::koko::TTSKoko;

fn main() {
    let tts = TTSKoko::new("checkpoints/kokoro-v0_19.onnx");

    let txt = r#"
    Hello, This is Kokoro. Your amazing AI TTS! A TTS model with only 82 million parameters that achieve incredable audio quality. 
    This is the one of the best Rust inference, I help you will like it. 
    Please give us a star if you do, thank you very much.
    "#;
    tts.tts(txt, "en-us", "bf_emma");
}
