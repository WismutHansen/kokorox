mod onn;
mod tts;
mod utils;

fn main() {
    let tts = tts::koko::TTSKoko::new("checkpoints/kokoro-v0_19.onnx");
    tts.tts(
        "Hello from Kokoro, your amazing AI TTS!",
        "en-us",
        "af_bella",
    );
}
