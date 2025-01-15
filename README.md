# Kokoro Rust


![img](https://img2023.cnblogs.com/blog/3572323/202501/3572323-20250112184100378-907988670.jpg)




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


## Updates

- ***`2025.01.15`***: Phonemizer supported! Now `Kokoros` can inference E2E without anyother dependencies! Kudos to [@tstm](https://github.com/tstm);
- ***`2025.01.13`***: Espeak-ng tokenizer and phonemizer supported! Kudos to [@mindreframer](https://github.com/mindreframer) ;
- ***`2025.01.12`***: Released `Kokoros`;


## Build

First, fetch the `voices.json` data, this is need same as Kokoro official step.

```
python scripts/fetch_voices.py
```

Run:

```shell
cargo build --release

# test
cargo run
```

For production:

```shell

cargo build --release

./target/release/koko -h
./target/release/koko -t 'Hello, this is a TTS test'
```

For further development, for example, supports on embeded etc, please raise an issue to discuss your requirement.


## Roadmap

Due to Kokoro actually not finalizing it's ability, this repo will keep tracking the status of Kokoro, and helpfully we can have language support incuding: English, Mandarin, Japanese, German, French etc.


## Copyright

Copyright reserved by Lucas Jin under Apache License.
