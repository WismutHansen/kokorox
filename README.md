# Kokoro Rust

![img](https://img2023.cnblogs.com/blog/3572323/202501/3572323-20250112184100378-907988670.jpg)

https://github.com/user-attachments/assets/1043dfd3-969f-4e10-8b56-daf8285e7420

🔥🔥🔥 **Introducing Kokoro Agents**

https://github.com/user-attachments/assets/9f5e8fe9-d352-47a9-b4a1-418ec1769567

Give a star if you like it!

[Kokoro](https://huggingface.co/hexgrad/Kokoro-82M) is a trending top 2 TTS model on huggingface.
This repo provides **insanely fast Kokoro infer in Rust**, you can now have your built TTS engine powered by Kokoro and infer fast by only a command of `koko`.

`kokoros` is a `rust` crate that provides easy to use TTS ability.
One can directly call `koko` in terminal to synthesize audio.

`kokoros` uses a relative small model 87M params, while results in extremly good quality voices results.

Languge support:

- [x] English;
- [x] Chinese (partly);
- [x] Japanese (partly);
- [x] German (partly);

> 🔥🔥🔥🔥🔥🔥🔥🔥🔥 Kokoros Rust version just got a lot attention now. If you also interested in insanely fast inference, embeded build, wasm support etc, please star this repo! We are keep updating it.

> Currently help wanted! Implement OpenAI compatible API in Rust, anyone interested? Send me PR!

New Discord community: https://discord.gg/nN4tCXC6, Please join us if you interested in Rust Kokoro.

## Updates

- **_`2025.01.22`_**: 🔥🔥🔥 **Streaming mode supported.** You can now using `--stream` to have fun with stream mode, kudos to [mroigo](https://github.com/mrorigo);
- **_`2025.01.17`_**: 🔥🔥🔥 Style mixing supported! Now, listen the output AMSR effect by simply specific style: `af_sky.4+af_nicole.5`;
- **_`2025.01.15`_**: OpenAI compatible server supported, openai format still under polish!
- **_`2025.01.15`_**: Phonemizer supported! Now `Kokoros` can inference E2E without anyother dependencies! Kudos to [@tstm](https://github.com/tstm);
- **_`2025.01.13`_**: Espeak-ng tokenizer and phonemizer supported! Kudos to [@mindreframer](https://github.com/mindreframer) ;
- **_`2025.01.12`_**: Released `Kokoros`;

## Installation

1. Install required python packages

```bash
pip install torch numpy requests
```

2. Initialize voice data:

```bash
python scripts/fetch_voices.py
```

This step fetches the required `voices.json` data file, which is necessary for voice synthesis.

3. Build the project:

```bash
cargo build --release
```

## Usage

Test the installation:

```bash
cargo run
```

For production use:

```bash
./target/release/koko -h        # View available options
./target/release/koko -t "Hello, this is a TTS test"
```

The generated audio will be saved to:

```
tmp/output.wav
```

### OpenAI-Compatible Server

1. Start the server:

```bash
cargo run -- --oai
```

2. Make API requests using either curl or Python:

Using curl:

```bash
curl -X POST http://localhost:3000/v1/audio/speech \
  -H "Content-Type: application/json" \
  -d '{
    "model": "tts-1",
    "input": "Hello, this is a test of the Kokoro TTS system!",
    "voice": "af_sky"
  }'
```

Using Python:

```bash
python scripts/run_openai.py
```

## Roadmap

Due to Kokoro actually not finalizing it's ability, this repo will keep tracking the status of Kokoro, and helpfully we can have language support incuding: English, Mandarin, Japanese, German, French etc.

## Copyright

Copyright reserved by Lucas Jin under Apache License.
