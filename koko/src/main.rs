use clap::{Parser, Subcommand};
use kokoros::{
    tts::koko::{TTSKoko, TTSOpts},
    utils::wav::{write_audio_chunk, WavHeader},
};
use rodio::{OutputStream, Sink, Source};
use std::net::{IpAddr, SocketAddr};
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::{
    fs::{self},
    io::Write,
};
use tokio::io::{AsyncBufReadExt, BufReader};

struct ChannelSource {
    rx: Receiver<Vec<f32>>,
    current: std::vec::IntoIter<i16>,
    sample_rate: u32,
}

impl ChannelSource {
    fn new(rx: Receiver<Vec<f32>>, sample_rate: u32) -> Self {
        Self {
            rx,
            current: Vec::new().into_iter(),
            sample_rate,
        }
    }
}

impl Iterator for ChannelSource {
    type Item = i16;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sample) = self.current.next() {
            Some(sample)
        } else {
            // Block until a new chunk arrives (or channel is closed)
            match self.rx.recv() {
                Ok(chunk) => {
                    // Convert each f32 sample to i16 (scaling appropriately)
                    let samples: Vec<i16> = chunk.iter().map(|&s| (s * 32767.0) as i16).collect();
                    self.current = samples.into_iter();
                    self.current.next()
                }
                Err(_) => None, // Channel closed.
            }
        }
    }
}

impl Source for ChannelSource {
    fn current_frame_len(&self) -> Option<usize> {
        None // Unknown.
    }
    fn channels(&self) -> u16 {
        1 // Mono audio.
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn total_duration(&self) -> Option<Duration> {
        None // Stream is indefinite.
    }
}

#[derive(Subcommand, Debug)]
enum Mode {
    /// Generate speech for a string of text
    #[command(alias = "t", long_flag_alias = "text", short_flag_alias = 't')]
    Text {
        /// Text to generate speech for
        #[arg(
            default_value = "Hello, This is Kokoro, your remarkable AI TTS. It's a TTS model with merely 82 million parameters yet delivers incredible audio quality.
                This is one of the top notch Rust based inference models, and I'm sure you'll love it. If you do, please give us a star. Thank you very much.
                As the night falls, I wish you all a peaceful and restful sleep. May your dreams be filled with joy and happiness. Good night, and sweet dreams!"
        )]
        text: String,

        /// Path to output the WAV file to on the filesystem
        #[arg(
            short = 'o',
            long = "output",
            value_name = "OUTPUT_PATH",
            default_value = "tmp/output.wav"
        )]
        save_path: String,
    },

    /// Read from a file path and generate a speech file for each line
    #[command(alias = "f", long_flag_alias = "file", short_flag_alias = 'f')]
    File {
        /// Filesystem path to read lines from
        input_path: String,

        /// Format for the output path of each WAV file, where {line} will be replaced with the line number
        #[arg(
            short = 'o',
            long = "output",
            value_name = "OUTPUT_PATH_FORMAT",
            default_value = "tmp/output_{line}.wav"
        )]
        save_path_format: String,
    },

    /// Continuously read from stdin to generate speech, outputting to stdout, for each line
    #[command(aliases = ["stdio", "stdin", "-"], long_flag_aliases = ["stdio", "stdin"])]
    Stream,
    ///
    /// Continuously process piped input by splitting sentences and streaming audio output.
    Pipe {
        /// Output WAV file path
        #[arg(
            short = 'o',
            long = "output",
            value_name = "OUTPUT_PATH",
            default_value = "tmp/pipe_output.wav"
        )]
        output_path: String,
    },
    /// Start an OpenAI-compatible HTTP server
    #[command(name = "openai", alias = "oai", long_flag_aliases = ["oai", "openai"])]
    OpenAI {
        /// IP address to bind to (typically 127.0.0.1 or 0.0.0.0)
        #[arg(long, default_value_t = [0, 0, 0, 0].into())]
        ip: IpAddr,

        /// Port to expose the HTTP server on
        #[arg(long, default_value_t = 3000)]
        port: u16,
    },
}

#[derive(Parser, Debug)]
#[command(name = "kokoros")]
#[command(version = "0.1")]
#[command(author = "Lucas Jin")]
struct Cli {
    /// A language identifier from
    /// https://github.com/espeak-ng/espeak-ng/blob/master/docs/languages.md
    #[arg(
        short = 'l',
        long = "lan",
        value_name = "LANGUAGE",
        default_value = "en-us"
    )]
    lan: String,
    
    /// Auto-detect language from input text
    #[arg(
        short = 'a',
        long = "auto-detect",
        value_name = "AUTO_DETECT",
        default_value_t = false
    )]
    auto_detect: bool,

    /// Path to the Kokoro v1.0 ONNX model on the filesystem
    #[arg(
        short = 'm',
        long = "model",
        value_name = "MODEL_PATH",
        default_value = "checkpoints/kokoro-v1.0.onnx"
    )]
    model_path: String,

    /// Path to the voices data file on the filesystem
    #[arg(
        short = 'd',
        long = "data",
        value_name = "DATA_PATH",
        default_value = "data/voices-v1.0.bin"
    )]
    data_path: String,

    /// Which single voice to use or voices to combine to serve as the style of speech
    #[arg(
        short = 's',
        long = "style",
        value_name = "STYLE",
        // if users use `af_sarah.4+af_nicole.6` as style name
        // then we blend it, with 0.4*af_sarah + 0.6*af_nicole
        default_value = "af_sarah.4+af_nicole.6"
    )]
    style: String,

    /// Rate of speech, as a coefficient of the default
    /// (i.e. 0.0 to 1.0 is slower than default,
    /// whereas 1.0 and beyond is faster than default)
    #[arg(
        short = 'p',
        long = "speed",
        value_name = "SPEED",
        default_value_t = 1.0
    )]
    speed: f32,

    /// Output audio in mono (as opposed to stereo)
    #[arg(long = "mono", default_value_t = false)]
    mono: bool,

    /// Initial silence duration in tokens
    #[arg(long = "initial-silence", value_name = "INITIAL_SILENCE")]
    initial_silence: Option<usize>,

    #[command(subcommand)]
    mode: Mode,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The segmentation fault seems to be related to ONNX runtime cleanup
    // We'll use an exit handler to ensure clean process termination
    
    // Exit handler for clean shutdown
    extern "C" fn exit_handler() {
        // Sleep to let ONNX runtime clean up its resources
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    // Register our exit handler to run when the program exits
    extern "C" {
        fn atexit(cb: extern "C" fn()) -> i32;
    }
    unsafe {
        atexit(exit_handler);
    }
    
    // Also set panic behavior to abort rather than unwind
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("Application panic: {}", panic_info);
        std::process::abort();
    }));
    
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let Cli {
            lan,
            auto_detect,
            model_path,
            data_path,
            style,
            speed,
            initial_silence,
            mono,
            mode,
        } = Cli::parse();

        let tts = TTSKoko::new(&model_path, &data_path).await;

        match mode {
            Mode::File {
                input_path,
                save_path_format,
            } => {
                let file_content = fs::read_to_string(input_path)?;
                for (i, line) in file_content.lines().enumerate() {
                    let stripped_line = line.trim();
                    if stripped_line.is_empty() {
                        continue;
                    }

                    let save_path = save_path_format.replace("{line}", &i.to_string());
                    tts.tts(TTSOpts {
                        txt: stripped_line,
                        lan: &lan,
                        auto_detect_language: auto_detect,
                        style_name: &style,
                        save_path: &save_path,
                        mono,
                        speed,
                        initial_silence,
                    })?;
                }
            }

            Mode::Text { text, save_path } => {
                let s = std::time::Instant::now();
                tts.tts(TTSOpts {
                    txt: &text,
                    lan: &lan,
                    auto_detect_language: auto_detect,
                    style_name: &style,
                    save_path: &save_path,
                    mono,
                    speed,
                    initial_silence,
                })?;
                println!("Time taken: {:?}", s.elapsed());
                let words_per_second =
                    text.split_whitespace().count() as f32 / s.elapsed().as_secs_f32();
                println!("Words per second: {:.2}", words_per_second);
                
                // Force immediate exit to avoid segfault
                // This is a workaround for ONNX Runtime's mutex issues at program exit
                std::process::exit(0);
            }

            Mode::OpenAI { ip, port } => {
                let app = kokoros_openai::create_server(tts).await;
                let addr = SocketAddr::from((ip, port));
                let binding = tokio::net::TcpListener::bind(&addr).await?;
                println!("Starting OpenAI-compatible HTTP server on {addr}");
                kokoros_openai::serve(binding, app.into_make_service()).await?;
            }

            Mode::Stream => {
                let stdin = tokio::io::stdin();
                let reader = BufReader::new(stdin);
                let mut lines = reader.lines();

                // Use std::io::stdout() for sync writing
                let mut stdout = std::io::stdout();

                eprintln!(
                    "Entering streaming mode. Type text and press Enter. Use Ctrl+D to exit."
                );

                // Write WAV header first
                let header = WavHeader::new(1, 24000, 32);
                header.write_header(&mut stdout)?;
                stdout.flush()?;

                while let Some(line) = lines.next_line().await? {
                    let stripped_line = line.trim();
                    if stripped_line.is_empty() {
                        continue;
                    }

                    // Process the line and get audio data
                    match tts.tts_raw_audio(&stripped_line, &lan, &style, speed, initial_silence, auto_detect) {
                        Ok(raw_audio) => {
                            // Write the raw audio samples directly
                            write_audio_chunk(&mut stdout, &raw_audio)?;
                            stdout.flush()?;
                            eprintln!("Audio written to stdout. Ready for another line of text.");
                        }
                        Err(e) => eprintln!("Error processing line: {}", e),
                    }
                }
            }
            Mode::Pipe { output_path } => {
                // Create an asynchronous reader for stdin.
                let stdin = tokio::io::stdin();
                let mut reader = BufReader::new(stdin);
                let mut buffer = String::new();

                // Set up rodio for immediate playback.
                // This channel will receive raw audio chunks (Vec<f32>) from TTS generation.
                let (tx, rx) = std::sync::mpsc::channel::<Vec<f32>>();
                let (_stream, stream_handle) = OutputStream::try_default()?;
                let sink = Sink::try_new(&stream_handle)?;
                let source = ChannelSource::new(rx, tts.sample_rate());
                sink.append(source);

                // Also create a WAV file to write the output.
                let mut wav_file = std::fs::File::create(&output_path)?;
                let header = WavHeader::new(1, tts.sample_rate(), 32);
                header.write_header(&mut wav_file)?;
                wav_file.flush()?;

                loop {
                    // Read a new line from stdin.
                    let mut line = String::new();
                    let bytes_read = reader.read_line(&mut line).await?;
                    if bytes_read == 0 {
                        // EOF reached.
                        break;
                    }
                    buffer.push_str(&line);

                    // Use your sentence splitter (here we assume English; adjust if needed)
                    let sentences = sentence_segmentation::processor::english(&buffer);

                    if !sentences.is_empty() {
                        // Clone sentences so we can manipulate.
                        let mut complete_sentences = sentences.clone();

                        // Check if the last sentence appears incomplete.
                        if let Some(last_sentence) = complete_sentences.last() {
                            let trimmed = last_sentence.trim();
                            if !(trimmed.ends_with('.')
                                || trimmed.ends_with('!')
                                || trimmed.ends_with('?'))
                            {
                                // Remove the incomplete sentence from processing.
                                complete_sentences.pop();
                            }
                        }

                        // If there is at least one complete sentence, process it.
                        if !complete_sentences.is_empty() {
                            for sentence in complete_sentences.iter() {
                                let sentence = sentence.trim();
                                if sentence.is_empty() {
                                    continue;
                                }
                                eprintln!("Processing sentence: {}", sentence);

                                let audio_chunk = tts
                                    .tts_raw_audio(sentence, &lan, &style, speed, initial_silence, auto_detect)
                                    .map_err(|e| {
                                        eprintln!("Error generating audio for sentence: {}", e);
                                        e
                                    })?;
                                // Immediately send the audio chunk for playback.
                                tx.send(audio_chunk.clone())?;
                                // Also write the chunk to the WAV file.
                                write_audio_chunk(&mut wav_file, &audio_chunk)?;
                                wav_file.flush()?;
                            }
                            // Remove the processed text from the beginning of the buffer.
                            // One strategy is to join the complete sentences and then remove that prefix.
                            let processed_text = complete_sentences.join(" ");
                            if buffer.starts_with(&processed_text) {
                                buffer = buffer[processed_text.len()..].to_string();
                            } else {
                                // If matching fails (due to extra spaces etc.), clear the buffer.
                                buffer.clear();
                            }
                        }
                    }
                }

                // Process any remaining text (e.g. if EOF arrives with an incomplete sentence).
                if !buffer.trim().is_empty() {
                    eprintln!("Processing final text: {}", buffer.trim());
                    let audio_chunk =
                        tts.tts_raw_audio(&buffer, &lan, &style, speed, initial_silence, auto_detect)?;
                    tx.send(audio_chunk.clone())?;
                    write_audio_chunk(&mut wav_file, &audio_chunk)?;
                    wav_file.flush()?;
                }
                
                // Explicitly drop the sender to close the channel
                drop(tx);
                
                // At this point, we need a clean exit
                // This is the key part that prevents segfault:
                // Instead of waiting for audio to finish, we'll stop gracefully
                eprintln!("Processing complete. Stopping audio playback...");
                
                // First stop the sink immediately
                sink.stop();
                
                // This is critical: wait a moment for any internal threads to clean up
                std::thread::sleep(std::time::Duration::from_millis(100));
                
                eprintln!("Playback finished.");
            }
        }

        // Force clean exit to avoid segfault
        // This is a workaround for ONNX Runtime's mutex issues
        std::process::exit(0);
    })
}
