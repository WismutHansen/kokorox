<div align="center">
  <img src="https://img2023.cnblogs.com/blog/3572323/202501/3572323-20250112184100378-907988670.jpg" alt="Banner" width="400" height="190">
</div>
<br>
<h1 align="center">🔥🔥🔥 Kokoro Rust</h1>

## [Zonos Rust Is On The Way?](https://github.com/lucasjinreal/Kokoros/issues/60)
## [Spark-TTS On The Way?](https://github.com/lucasjinreal/Kokoros/issues/75)
## [Orpheus-TTS On The Way?](https://github.com/lucasjinreal/Kokoros/issues/75)


**AMSR**

https://github.com/user-attachments/assets/1043dfd3-969f-4e10-8b56-daf8285e7420

**Digital Human**

https://github.com/user-attachments/assets/9f5e8fe9-d352-47a9-b4a1-418ec1769567

<p align="center">
  <b>Give a star ⭐ if you like it!</b>
</p>

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

New Discord community: https://discord.gg/E566zfDWqD, Please join us if you interested in Rust Kokoro.

## Updates

- **_`2025.01.22`_**: 🔥🔥🔥 **Streaming mode supported.** You can now using `--stream` to have fun with stream mode, kudos to [mroigo](https://github.com/mrorigo);
- **_`2025.01.17`_**: 🔥🔥🔥 Style mixing supported! Now, listen the output AMSR effect by simply specific style: `af_sky.4+af_nicole.5`;
- **_`2025.01.15`_**: OpenAI compatible server supported, openai format still under polish!
- **_`2025.01.15`_**: Phonemizer supported! Now `Kokoros` can inference E2E without anyother dependencies! Kudos to [@tstm](https://github.com/tstm);
- **_`2025.01.13`_**: Espeak-ng tokenizer and phonemizer supported! Kudos to [@mindreframer](https://github.com/mindreframer) ;
- **_`2025.01.12`_**: Released `Kokoros`;

## Installation

1. Install required Python packages:

```bash
pip install -r scripts/requirements.txt
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

### View available options

```bash
./target/release/koko -h
```

### Generate speech for some text

```
./target/release/koko text "Hello, this is a TTS test"
```

The generated audio will be saved to `tmp/output.wav` by default. You can customize the save location with the `--output` or `-o` option:

```
./target/release/koko text "I hope you're having a great day today!" --output greeting.wav
```

### Generate speech for each line in a file

```
./target/release/koko file poem.txt
```

For a file with 3 lines of text, by default, speech audio files `tmp/output_0.wav`, `tmp/output_1.wav`, `tmp/output_2.wav` will be outputted. You can customize the save location with the `--output` or `-o` option, using `{line}` as the line number:

```
./target/release/koko file lyrics.txt -o "song/lyric_{line}.wav"
```

### OpenAI-Compatible Server

1. Start the server:

```bash
./target/release/koko openai
```

2. Make API requests using either curl or Python:

Using curl:

```bash
curl -X POST http://localhost:3000/v1/audio/speech \
  -H "Content-Type: application/json" \
  -d '{
    "model": "anything can go here",
    "input": "Hello, this is a test of the Kokoro TTS system!",
    "voice": "af_sky"
  }'
  --output sky-says-hello.wav
```

Using Python:

```bash
python scripts/run_openai.py
```

### Streaming

The `stream` option will start the program, reading for lines of input from stdin and outputting WAV audio to stdout.

Use it in conjunction with piping.

#### Typing manually

```
./target/release/koko stream > live-audio.wav
# Start typing some text to generate speech for and hit enter to submit
# Speech will append to `live-audio.wav` as it is generated
# Hit Ctrl D to exit
```

#### Input from another source

```
echo "Suppose some other program was outputting lines of text" | ./target/release/koko stream > programmatic-audio.wav
```

### With docker

1. Build the image

```bash
docker build -t kokoros .
```

2. Run the image, passing options as described above

```bash
# Basic text to speech
docker run -v ./tmp:/app/tmp kokoros text "Hello from docker!" -o tmp/hello.wav

# An OpenAI server (with appropriately bound port)
docker run -p 3000:3000 kokoros openai
```

## Roadmap

Due to Kokoro actually not finalizing it's ability, this repo will keep tracking the status of Kokoro, and helpfully we can have language support incuding: English, Mandarin, Japanese, German, French etc.

## Copyright

Copyright reserved by Lucas Jin under Apache License.
