<h1 align="center">kokorox - kokoro text-to-speech in Rust</h1>

[Kokoro](https://huggingface.co/hexgrad/Kokoro-82M) is a trending top 2 TTS model on huggingface.
This repo provides **insanely fast Kokoro infer in Rust**, you can now have your built TTS engine powered by Kokoro and infer fast by only a command of `koko`.

`kokorox` (the library crate) and `koko` (the CLI application) provide easy-to-use Text-to-Speech capabilities.
`koko` uses a relatively small model (87M parameters) yet delivers extremely good quality voice results.

Language support includes:

- [x] English
- [x] Chinese (Mandarin)
- [x] Spanish (with improved accent and phoneme handling)
- [x] Japanese
- [x] German
- [x] French
- [x] And more via espeak-ng integration and automatic language detection.

## Updates

- **_`LATEST`_**:
  - **New CLI command `koko voices`**: List available voice styles with options for JSON, list, or table format, and filtering by language or gender.
  - **New OpenAI server endpoints**:
    - `/v1/audio/voices`: Lists available voice IDs.
    - `/v1/audio/voices/detailed`: Provides detailed information about each voice (name, description, language, gender).
  - **Improved Spanish Language Support**: Enhanced accent restoration and phoneme correction for more natural Spanish speech.
  - **Enhanced Text Processing**:
    - More robust sentence segmentation, especially for texts with year ranges and complex structures, and better UTF-8 handling for accented characters.
    - New debugging flags:
      - `--verbose`: Enable verbose debug logs for text processing stages.
      - `--debug-accents`: Enable detailed character-by-character analysis for non-English languages with accents.
  - **Pipe Mode Enhancement**: Added `--silent` flag to `pipe` mode to suppress audio playback, useful when only saving to file.
  - **Automatic `force-style`**: If you specify a `--style` different from the default (`af_heart`), `--force-style` is now automatically enabled to ensure your chosen style is used.
- **_`2025.03.23`_**: **Piping now supports incoming streamed text from LLMs** The audio generation will now start once the first complete sentence is detected, making audio playback even faster.
- **_`2025.03.22`_**: **Piping with direct playback supported.** You can now use `pipe` to send text to `koko`, which will be split into sentences and the output will start to play once the first sentence has been generated.
- **_`2025.01.22`_**: **Streaming mode supported.** You can now using `--stream` to have fun with stream mode, kudos to [mroigo](https://github.com/mrorigo);
- **_`2025.01.17`_**: Style mixing supported! Now, listen the output AMSR effect by simply specific style: `af_sky.4+af_nicole.5`;
- **_`2025.01.15`_**: OpenAI compatible server supported, openai format still under polish!
- **_`2025.01.15`_**: Phonemizer supported! Now `koko` can inference E2E without anyother dependencies! Kudos to [@tstm](https://github.com/tstm);
- **_`2025.01.13`_**: Espeak-ng tokenizer and phonemizer supported! Kudos to [@mindreframer](https://github.com/mindreframer) ;
- **_`2025.01.12`_**: Released `kokorox`;

## Installation

### 0. Install onnx runtime for your architecture

#### Ubuntu w/ NVIDIA GPU

```bash
# 1) extract (skip if already done)
tar -xzf onnxruntime-linux-x64-gpu-1.22.0.tgz

# 2) copy *preserving* symlinks & metadata
sudo cp -a onnxruntime-linux-x64-gpu-1.22.0/include /usr/local/
sudo cp -a onnxruntime-linux-x64-gpu-1.22.0/lib /usr/local/

# 3) refresh linker cache
sudo ldconfig

# 4) Make sure the headers are on PATH
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
```

### 1. Pre-built Binaries (Recommended for most users)

Pre-built binaries for Linux, macOS, and Windows are available on the [**GitHub Releases page**](https://github.com/WismutHansen/kokorox/releases). Download the appropriate archive for your system, extract it, and you'll find the `koko` executable.

### 2. Install System Dependencies (if building from source or using certain features)

These are primarily for `espeak-ng` which powers the phonemization:

- **Ubuntu/Debian**: `sudo apt-get install espeak-ng libespeak-ng-dev`
- **macOS**: `brew install espeak-ng`
- **Windows**: Install espeak-ng from [the official repository](https://github.com/espeak-ng/espeak-ng/releases)

### 3. Install Python and Download Models (if building from source or needing models)

If you plan to build from source or need to download the TTS models:
a. Install Python (3.10+ recommended).
b. Install required Python packages:
`bash
      pip install -r scripts/requirements.txt
      `
c. Download model and voices:

````bash # Download all required resources (recommended: model, default voices)
python scripts/download_voices.py --all

      # Or download specific resources
      # python scripts/download_voices.py --model   # Download only the model
      # python scripts/download_voices.py --voices  # Download only the voices

      # List supported languages for informational purposes
      # python scripts/download_voices.py --list-languages
      ```
      This will place `kokoro-v1.0.onnx` in `checkpoints/` and `voices-v1.0.bin` in `data/`.

### 4. Build from Source (Optional)

If you prefer to build from source:
a. Ensure you have Rust installed (see [rustup.rs](https://rustup.rs/)).
b. Clone the repository: `git clone https://github.com/WismutHansen/kokorox.git`
c. `cd kokorox`
d. Build the project:
`bash
      cargo build --release
      `
The executable will be at `target/release/koko`.

### 5. All-in-One Download and Build Script (for source builds)

Alternatively, you can use the `download_all.sh` script to automate Python dependency installation, model downloads, and the Rust build:

```bash
./download_all.sh --build
````

## Usage

After installation (either by downloading a pre-built binary or building from source), you can use the `koko` CLI. If you built from source, replace `./koko` with `target/release/koko`.

### View available options

```bash
./koko -h
```

### Generate speech for some text

```bash
./koko text "Hello, this is a TTS test"
```

The generated audio will be saved to `tmp/output.wav` by default. Customize the save location with `--output` or `-o`:

```bash
./koko text "I hope you're having a great day today!" -o greeting.wav
```

### Multi-language support

Kokorox supports multiple languages. You can either specify the language manually or use automatic detection.

**Language and Style Interaction:**

- Use `--lan <LANGUAGE_CODE>` to specify a language (e.g., `en-us`, `es`, `zh`).
- Use `--auto-detect` (or `-a`) to let `koko` try to determine the language from the input text. If detection fails, it falls back to the language specified by `--lan` (default `en-us`).
- Use `--style <VOICE_ID>` to choose a specific voice.
- By default, `koko` attempts to pick a voice appropriate for the detected or specified language.
- If you set `--style` to a value different from the default (`af_heart`), `--force-style` will be automatically enabled, meaning your chosen style will be used regardless of the detected/specified language.
- You can explicitly use `--force-style` to make `koko` use the voice specified by `--style` even if it doesn't match the language.

**Examples:**

```bash
# Specify language manually
./koko text "你好，世界!" --lan zh
./koko text "こんにちは、世界!" --lan ja
./koko text "Hola, mundo. Esto es una prueba." --lan es --style ef_dora # Good Spanish voice

# Automatic language detection
./koko -a text "Hello, world!"   # Will detect English
./koko -a text "你好，世界!"      # Will detect Chinese

# Using a specific style (implicitly enables --force-style if style is not default)
./koko text "This is a test." --style am_michael

# Explicitly force a style, even if language is different
./koko text "Esto es una prueba en español." --lan es --style af_sky --force-style
```

### Debugging Text Processing

For complex text or non-English languages, you might encounter issues. Use these flags to help debug:

- `--verbose`: Prints detailed logs about text normalization, phonemization, and segmentation steps.
- `--debug-accents`: Provides character-by-character analysis for text containing non-ASCII (e.g., accented) characters, showing how they are handled.

```bash
./koko text "Una política económica." --lan es --style ef_dora --verbose --debug-accents
```

### List Available Voices

Use the `voices` subcommand to see which voice styles are available in your `voices-v1.0.bin` (or custom voices file).

```bash
# Default table format
./koko voices

# JSON format
./koko voices --format json

# Simple list of IDs
./koko voices --format list

# Filter by language (e.g., English voices)
./koko voices --language en

# Filter by gender (e.g., male voices)
./koko voices --gender male

# Combine filters (e.g., Spanish female voices)
./koko voices --language es --gender female
```

### Generate speech for each line in a file

```bash
./koko file poem.txt
```

For a file with 3 lines, output files `tmp/output_0.wav`, `tmp/output_1.wav`, `tmp/output_2.wav` are created. Customize with `-o`:

```bash
./koko file lyrics.txt -o "song/lyric_{line}.wav"
```

### Use pipe for live audio playback from text streams

Route text from other programs (like LLMs) to `koko` and have it played back as sentences are generated.

```bash
ollama run llama3 "Tell me a short story about a brave robot." | ./koko pipe

# To save to file and not play audio:
ollama run llama3 "Explain quantum physics." | ./koko pipe --silent -o quantum.wav
```

### OpenAI-Compatible Server

1. Start the server:

   ```bash
   ./koko openai --ip 0.0.0.0 --port 3000
   ```

2. Make API requests:

   **Synthesize Speech:**

   ```bash
   curl -X POST http://localhost:3000/v1/audio/speech \
     -H "Content-Type: application/json" \
     -d '{
       "model": "kokoro",
       "input": "Hello, this is a test of the Kokoro TTS system!",
       "voice": "af_sky",
       "language": "en-us",
       "response_format": "wav"
     }' \
     --output sky-says-hello.wav
   ```

   Supported `response_format`: `wav` (default), `mp3`.

   **List Voice IDs:**

   ```bash
   curl http://localhost:3000/v1/audio/voices
   ```

   Example Output:

   ```json
   {
     "voices": ["af_heart", "af_sky", "ef_dora", "em_alex", "..."]
   }
   ```

   **List Detailed Voice Information:**

   ```bash
   curl http://localhost:3000/v1/audio/voices/detailed
   ```

   Example Output:

   ```json
   {
     "voices": [
       {
         "id": "af_heart",
         "name": "Heart (Female)",
         "description": "English (US) female voice",
         "language": "English (US)",
         "gender": "female"
       }
       // ... more voices
     ]
   }
   ```

   You can also use the Python test script: `python scripts/test_server.py speak "Test message"`

### Streaming (line-by-line from stdin to stdout WAV)

The `stream` option reads lines from stdin and outputs WAV audio to stdout.

```bash
# Typing manually, saving to file
./koko stream > live-audio.wav
# (Type text, press Enter. Ctrl+D to exit)

# Input from another source
echo "This is a line of text." | ./koko stream > programmatic-audio.wav
```

### Docker Usage

1. Build the image:

   ```bash
   docker build -t kokorox .
   ```

2. Run the image:

   ```bash
   # Basic text to speech (ensure ./tmp exists or change output path)
   docker run -v ./tmp:/app/tmp kokorox text "Hello from docker!" -o tmp/hello.wav

   # OpenAI server (bind port, models must be in image or mounted)
   # The default Dockerfile copies models from ./checkpoints and ./data
   docker run -p 3000:3000 kokorox openai --ip 0.0.0.0 --port 3000
   ```

### Additional Voices (from Hugging Face PyTorch models)

The default installation includes a standard set of voices. The original Kokoro model on Hugging Face has more voices (54 voices in 8 languages) that can be converted.

1. **List available languages/voices from Hugging Face:**

   ```bash
   python scripts/convert_pt_voices.py --list-languages
   python scripts/convert_pt_voices.py --list-voices en
   ```

2. **Download, convert, and combine all Hugging Face voices:**

   ```bash
   python scripts/convert_pt_voices.py --all
   ```

   This creates `data/voices-custom.bin`.

3. **Use custom voices file with `koko`:**

   ```bash
   ./koko -d data/voices-custom.bin text "Using a custom voice." --style en_sarah # Example custom style
   ```

   You can list voices from a custom file:

   ```bash
   ./koko -d data/voices-custom.bin voices
   ```

## Troubleshooting

### ONNX Runtime Mutex Errors / Crashes on Exit

Some users, particularly on certain Linux distributions or when interrupting the program, might experience crashes related to ONNX Runtime's internal thread management during shutdown. While the audio generation is usually complete, the crash can be disruptive.

You can use the `run_kokoros.sh` wrapper script (Linux/macOS) to mitigate this:

```bash
./run_kokoros.sh text "This should exit more gracefully."
```

This script catches the abrupt termination and allows your shell to continue normally.

## Roadmap

- Continue improving multi-language support, focusing on phoneme accuracy and naturalness for supported languages.
- Enhance text normalization for a wider range of inputs.
- Explore further optimizations for speed and resource usage.
- Community feedback will guide further development.

## Copyright

Copyright © Lucas Jin, Tommy Falkowski. Licensed under the Apache License, Version 2.0.
