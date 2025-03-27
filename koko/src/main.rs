use clap::{Parser, Subcommand};
use kokoros::{
    tts::koko::{TTSKoko, TTSOpts},
    utils::wav::{write_audio_chunk, WavHeader},
};
use rodio::{OutputStream, Sink, Source};
// Removed unused Cow import
use std::net::{IpAddr, SocketAddr};
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::{
    fs::{self},
    io::Write,
};
use tokio::io::{AsyncBufReadExt, BufReader};
use sentence_segmentation;

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
    /// Common values: en-us, en-gb, es, fr-fr, de, zh, ja, pt-pt
    #[arg(
        short = 'l',
        long = "lan",
        value_name = "LANGUAGE",
        default_value = "en-us"
    )]
    lan: String,
    
    /// Auto-detect language from input text
    /// When enabled, the system will attempt to detect the language from the input text
    #[arg(
        short = 'a',
        long = "auto-detect",
        value_name = "AUTO_DETECT",
        default_value_t = false
    )]
    auto_detect: bool,
    
    /// Override style selection 
    /// When enabled, this will use the specified style (set with -s/--style)
    /// instead of automatically selecting a language-appropriate style.
    /// Without this flag, the system tries to use language-appropriate voices.
    #[arg(
        long = "force-style",
        value_name = "FORCE_STYLE",
        default_value_t = false
    )]
    force_style: bool,

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
    /// For Spanish: ef_dora (female) or em_alex (male)
    /// For Portuguese: pf_dora (female) or pm_alex (male)
    /// For English: af_* (US female), am_* (US male), bf_* (UK female), bm_* (UK male)
    /// For Japanese: jf_* (female) or jm_* (male)
    /// For Chinese: zf_* (female) or zm_* (male)
    #[arg(
        short = 's',
        long = "style",
        value_name = "STYLE",
        // if users use `af_sarah.4+af_nicole.6` as style name
        // then we blend it, with 0.4*af_sarah + 0.6*af_nicole
        default_value = "af_sky"
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
    
    /// Enable verbose debug output for text processing
    /// Especially useful for debugging issues with non-English text
    #[arg(
        short = 'v',
        long = "verbose",
        help = "Enable verbose debug logs for text processing",
        default_value_t = false
    )]
    verbose: bool,
    
    /// Enable detailed accent debugging for non-English languages
    /// Shows character-by-character analysis of accented characters
    #[arg(
        long = "debug-accents",
        help = "Enable detailed accent debugging for non-English languages",
        default_value_t = false
    )]
    debug_accents: bool,

    #[command(subcommand)]
    mode: Mode,
}

/// Custom sentence segmentation function that preserves UTF-8 characters
/// This is a replacement for the sentence_segmentation library to fix the
/// loss of accented characters during processing.
fn utf8_safe_sentence_segmentation(text: &str, language: &str, verbose: bool, debug_accents: bool) -> Vec<String> {
    // Only log when debug flags are enabled
    if verbose || debug_accents {
        // Log debug info for text with special characters
        let has_accents = text.contains('á') || text.contains('é') || 
                         text.contains('í') || text.contains('ó') || 
                         text.contains('ú') || text.contains('ñ') || 
                         text.contains('ü') ||
                         text.contains('à') || text.contains('è') || 
                         text.contains('ì') || text.contains('ò') || 
                         text.contains('ù') || text.contains('ë') || 
                         text.contains('ï') || text.contains('ç');
        if has_accents {
            if verbose {
                println!("UTF8-SAFE SEGMENTATION: Text with accents detected");
            }
            
            // If the detailed accent debugging is enabled, show each character
            if debug_accents {
                for (i, c) in text.char_indices() {
                    if !c.is_ascii() {
                        println!("  Special char at {}: '{}' (Unicode: U+{:04X})", i, c, c as u32);
                    }
                }
            }
        }
    }
    
    // IMPORTANT: The key issue with sentence segmentation is that it needs to correctly 
    // handle multi-byte UTF-8 characters. We need to carefully track strings through this process.
    
    // First, ensure the text is valid UTF-8 (it should be since it's a Rust &str)
    if !text.is_empty() {
        // Choose the appropriate segmentation function based on language
        let processed = if language.starts_with("es") || 
                          language.starts_with("fr") || 
                          language.starts_with("it") || 
                          language.starts_with("pt") {
            // Use the English processor for romance languages (for now)
            // In the future, we could implement language-specific segmentation
            sentence_segmentation::processor::english(text)
        } else if language.starts_with("de") {
            // Use the English processor for German (for now)
            sentence_segmentation::processor::english(text)
        } else {
            // Default to English processor
            sentence_segmentation::processor::english(text)
        };
        
        // Verify if the output retains accented characters, if debugging is enabled
        if verbose || debug_accents {
            // Check for languages that commonly use accents
            let check_accents = language.starts_with("es") || 
                               language.starts_with("fr") || 
                               language.starts_with("pt") || 
                               language.starts_with("it");
                               
            if check_accents {
                for (i, sentence) in processed.iter().enumerate() {
                    let has_accents = sentence.contains('á') || sentence.contains('é') || 
                                     sentence.contains('í') || sentence.contains('ó') || 
                                     sentence.contains('ú') || sentence.contains('ñ') || 
                                     sentence.contains('ü') ||
                                     sentence.contains('à') || sentence.contains('è') || 
                                     sentence.contains('ì') || sentence.contains('ò') || 
                                     sentence.contains('ù') || sentence.contains('ë') || 
                                     sentence.contains('ï') || sentence.contains('ç');
                                     
                    if has_accents {
                        if debug_accents {
                            println!("SEGMENT {}: Retained accents: {}", i, sentence);
                        }
                    } else if verbose {
                        // Try to identify potential accent loss by looking for common words
                        // that should have accents but don't
                        let potential_issues = language.starts_with("es") && (
                            sentence.contains("politica") || 
                            sentence.contains("aqu") || 
                            sentence.contains("economia") || 
                            sentence.contains("informacion") ||
                            sentence.contains("comunicacion")
                        );
                        
                        if potential_issues {
                            println!("POTENTIAL ACCENT LOSS in segment {}: {}", i, sentence);
                        }
                    }
                }
            }
        }
        
        processed
    } else {
        vec![]
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The segmentation fault seems to be related to ONNX runtime cleanup
    // We'll use a different approach to clean up
    
    // Tell Rust to just abort on panic instead of unwinding
    // This avoids complex cleanup issues with ONNX Runtime 
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("Application panic: {}", panic_info);
        std::process::abort();
    }));
    
    // Set up SIGTERM/SIGINT handlers for immediate exit
    ctrlc::set_handler(move || {
        println!("Received termination signal, exiting immediately.");
        std::process::exit(0); // Exit immediately on Ctrl+C
    }).expect("Error setting Ctrl-C handler");
    
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let Cli {
            lan,
            auto_detect,
            force_style,
            model_path,
            data_path,
            style,
            speed,
            initial_silence,
            mono,
            verbose,
            debug_accents,
            mode,
        } = Cli::parse();

        let tts = TTSKoko::new(&model_path, &data_path).await;

        match &mode {
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
                        force_style,
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
                    force_style,
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
                
                // Cleanup happens in the finally block at the end
                // Do a clean exit now
                return Ok(());
            }

            Mode::OpenAI { ip, port } => {
                let app = kokoros_openai::create_server(tts.clone()).await;
                let addr = SocketAddr::from((*ip, *port));
                let binding = tokio::net::TcpListener::bind(&addr).await?;
                println!("Starting OpenAI-compatible HTTP server on {addr}");
                kokoros_openai::serve(binding, app.into_make_service()).await?;
                
                // Clean up resources before exit
                tts.cleanup();
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

                    // Process the line and get audio data with proper language handling
                    match tts.tts_raw_audio(&stripped_line, &lan, &style, speed, initial_silence, auto_detect, force_style) {
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
                // Comment removed: "This buffer stores text as it comes in from stdin"
                // Unused variable removed
                
                // We don't need these variables anymore since we use session_language and session_style

                // Set up rodio for immediate streaming playback
                let (tx, rx) = std::sync::mpsc::channel::<Vec<f32>>();
                let (_stream, stream_handle) = OutputStream::try_default()?;
                let sink = Sink::try_new(&stream_handle)?;
                let source = ChannelSource::new(rx, tts.sample_rate());
                sink.append(source);
                
                // Configure TTS settings once at the beginning, but they can be updated
                let mut session_language = lan.clone();
                let mut session_style = style.clone();
                
                // Initialize language detection state.
                // If auto_detect is false, language is already "detected" (we're using the specified one)
                // If auto_detect is true, we need to perform detection
                let mut language_detected = !auto_detect;
                
                // Print language selection mode clearly (always show this regardless of verbosity)
                if auto_detect {
                    eprintln!("AUTO-DETECT MODE: Will determine language from text input");
                    eprintln!("Note: -l flag will only be used as fallback if detection fails");
                } else {
                    eprintln!("MANUAL LANGUAGE MODE: Using specified language: {}", lan);
                }

                // Also create a WAV file to write the output.
                let mut wav_file = std::fs::File::create(&output_path)?;
                let header = WavHeader::new(1, tts.sample_rate(), 32);
                header.write_header(&mut wav_file)?;
                wav_file.flush()?;

                // Streaming approach:
                // 1. Detect language from initial input
                // 2. Process complete sentences as they arrive
                // 3. Stream audio as soon as each sentence is processed
                
                // Keep track of accumulated text and sentence boundaries
                let mut buffer = String::new();
                
                loop {
                    // Read a new line from stdin
                    if verbose {
                        eprintln!("BEFORE READ: About to read from stdin");
                    }
                    let mut line = String::new();
                    
                    // Try to read using standard method first
                    let bytes_read = reader.read_line(&mut line).await?;
                    if bytes_read == 0 {
                        // EOF reached
                        break;
                    }
                    
                    // Immediately verify UTF-8 validity and fix any potential issues
                    if !String::from_utf8(line.clone().into_bytes()).is_ok() {
                        eprintln!("WARNING: Invalid UTF-8 detected in input. Attempting to fix...");
                        // Use the lossy conversion to handle invalid UTF-8
                        line = String::from_utf8_lossy(&line.as_bytes().to_vec()).to_string();
                    }
                    
                    if verbose || debug_accents {
                        // Check specifically for encoding issues by comparing bytes vs chars
                        let bytes_count = line.bytes().count();
                        let chars_count = line.chars().count();
                        eprintln!("ENCODING ANALYSIS: Bytes: {}, Chars: {}, Difference: {}", 
                                bytes_count, chars_count, bytes_count - chars_count);
                        
                        // If the string contains multi-byte characters (like accents), there will be a difference
                        if bytes_count != chars_count {
                            eprintln!("MULTI-BYTE CHARS DETECTED: Line likely contains accented characters");
                        }
                    }
                        
                    // Detailed logging for UTF-8 characters if debug_accents is enabled
                    if debug_accents {
                        for (i, c) in line.char_indices() {
                            if !c.is_ascii() {
                                let mut bytes = [0u8; 4];
                                let len = c.encode_utf8(&mut bytes).len();
                                let byte_str = bytes[0..len].iter()
                                    .map(|b| format!("{:02X}", b))
                                    .collect::<Vec<_>>()
                                    .join(" ");
                                
                                eprintln!("  Char at byte {}: '{}' (U+{:04X}) - UTF-8: {}", 
                                        i, c, c as u32, byte_str);
                            }
                        }
                    }
                    
                    // For Spanish, do a quick check on common accented words, but only in verbose mode
                    if verbose && (session_language.starts_with("es") || 
                        (auto_detect && !language_detected && 
                        (line.contains("Hola") || line.contains("español")))) {
                        
                        eprintln!("SPANISH CHECK: Looking for potential accent issues");
                        
                        // Check for words that are commonly missing accents
                        let suspicious_words = [
                            ("politica", "política"),
                            ("aqu", "aquí"),
                            ("economia", "economía"),
                            ("informacion", "información"),
                            ("educacion", "educación"),
                            ("comunicacion", "comunicación")
                        ];
                        
                        for (incorrect, correct) in suspicious_words.iter() {
                            if line.contains(incorrect) {
                                eprintln!("POTENTIAL MISSING ACCENT: Found '{}', should be '{}'", 
                                        incorrect, correct);
                            }
                        }
                    }
                    
                    // Debug the raw bytes received, but only in verbose mode
                    if verbose {
                        eprintln!("RAW INPUT DEBUG: Received {} bytes", bytes_read);
                        eprintln!("TEXT RECEIVED: {}", line.trim());
                        eprintln!("ENCODING CHECK: UTF-8 valid: {}", String::from_utf8(line.clone().into_bytes()).is_ok());
                    }
                    
                    // Detailed debugging for problematic Spanish words, only in debug_accents mode
                    if debug_accents && session_language.starts_with("es") {
                        // Check common problem characters with detailed byte analysis
                        if line.contains("poltica") || line.contains("politica") {
                            eprintln!("ENCODING DEBUG: Found 'poltica/politica' - might be missing 'í'");
                            eprintln!("Line: {}", line);
                            eprintln!("HEX: {}", line.bytes().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "));
                        }
                        
                        if line.contains("Aqu") || line.contains("aqu") {
                            eprintln!("ENCODING DEBUG: Found 'Aqu/aqu' - might be missing 'í'");
                            eprintln!("Line: {}", line);
                            eprintln!("HEX: {}", line.bytes().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "));
                        }
                        
                        if line.contains("comunicacin") || line.contains("comunicacion") {
                            eprintln!("ENCODING DEBUG: Found 'comunicacion' - might be missing 'ó'");
                            eprintln!("Line: {}", line);
                            eprintln!("HEX: {}", line.bytes().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "));
                        }
                        
                        // Check specifically for Spanish characters that should have accents
                        let spanish_words = [
                            ("poltica", "política"),
                            ("politica", "política"),
                            ("aqu", "aquí"),
                            ("Aqu", "Aquí"),
                            ("comunicacion", "comunicación"),
                            ("informacion", "información"),
                            ("educacion", "educación")
                        ];
                        
                        for (incorrect, correct) in spanish_words.iter() {
                            if line.contains(incorrect) {
                                eprintln!("ACCENT MISSING: Found '{}', should be '{}'", incorrect, correct);
                            }
                        }
                    }
                    
                    // Add to our text buffer
                    buffer.push_str(&line);
                    
                    // Only run language detection if we haven't detected yet and auto-detect is enabled
                    if !language_detected {
                        if auto_detect && buffer.len() > 60 {
                            // Only perform language detection when auto_detect is true
                            if verbose {
                                eprintln!("Auto-detecting language from initial text...");
                            }
                            
                            if let Some(detected) = kokoros::tts::phonemizer::detect_language(&buffer) {
                                eprintln!("Detected language: {}", detected);
                                session_language = detected;
                            } else {
                                eprintln!("Language detection failed, using specified: {}", lan);
                            }
                        } else if verbose {
                            // With auto_detect disabled, just use the specified language
                            eprintln!("Using specified language: {}", lan);
                        }
                        
                        // Select appropriate voice style if not forcing a specific one
                        if !force_style {
                            let is_custom = tts.is_using_custom_voices(tts.voices_path());
                            let default_style = kokoros::tts::phonemizer::get_default_voice_for_language(&session_language, is_custom);
                            // Always show the selected voice, this is important information
                            eprintln!("Selected voice style: {}", default_style);
                            session_style = default_style;
                        }
                        
                        language_detected = true;
                        // Always show the final language/voice selection as this is important information
                        eprintln!("Will use language: {} with voice: {}", session_language, session_style);
                    }
                    
                    // Extract and process complete sentences
                    let mut complete_sentences = Vec::new();
                    
                    // Process sentences based on language type
                    if session_language.starts_with("zh") || 
                       session_language.starts_with("ja") || 
                       session_language.starts_with("ko") 
                    {
                        // For CJK languages, extract based on special punctuation
                        let mut cjk_sentences = Vec::new();
                        let mut current = String::new();
                        
                        for c in buffer.chars() {
                            current.push(c);
                            if c == '。' || c == '！' || c == '？' || c == '.' || c == '!' || c == '?' {
                                if !current.trim().is_empty() {
                                    cjk_sentences.push(current.clone());
                                    current.clear();
                                }
                            }
                        }
                        
                        // Update complete_sentences with the extracted CJK sentences
                        complete_sentences = cjk_sentences;
                        
                        // Keep the incomplete sentence in the buffer
                        if !current.trim().is_empty() {
                            buffer = current;
                        } else {
                            buffer.clear();
                        }
                    } else {
                        // Check the buffer for accented characters before segmentation, but only if debug is enabled
                        if verbose && session_language.starts_with("es") {
                            let has_accents = buffer.contains('á') || buffer.contains('é') || 
                                             buffer.contains('í') || buffer.contains('ó') || 
                                             buffer.contains('ú') || buffer.contains('ñ') || 
                                             buffer.contains('ü');
                            if has_accents {
                                println!("BUFFER PRE-SEGMENTATION: Spanish text with accents detected");
                                if debug_accents {
                                    println!("Buffer: {}", buffer);
                                }
                            }
                        }
                        
                        // Use our UTF-8 safe sentence segmentation function with proper language handling
                        let sentences = utf8_safe_sentence_segmentation(&buffer, &session_language, verbose, debug_accents);
                        
                        if verbose {
                            println!("SEGMENTATION COMPLETE: Found {} potential sentences", sentences.len());
                        }
                        
                        // Handle buffer updates with UTF-8 safety
                        if !sentences.is_empty() {
                            // Check if the last sentence appears incomplete (no ending punctuation)
                            let last_sentence = sentences.last().unwrap();
                            
                            if verbose {
                                println!("Last segment: {}", last_sentence);
                            }
                            
                            if !(last_sentence.ends_with('.') || 
                                 last_sentence.ends_with('!') || 
                                 last_sentence.ends_with('?')) {
                                
                                if verbose {
                                    println!("Last segment appears incomplete - will keep in buffer");
                                }
                                
                                // Handle buffer update with careful UTF-8 byte handling
                                if sentences.len() > 1 {
                                    // Everything except the last sentence
                                    let complete_text: String = sentences[..sentences.len()-1]
                                        .iter()
                                        .fold(String::new(), |acc, s| acc + s + " ");
                                    
                                    // Try to find where the complete sentences end in the buffer
                                    if buffer.starts_with(&complete_text) {
                                        // Safe to remove the processed text and keep remainder
                                        buffer = buffer[complete_text.len()..].to_string();
                                        if verbose {
                                            println!("BUFFER UPDATE: Remaining text in buffer: '{}'", 
                                                    buffer.chars().take(30).collect::<String>());
                                        }
                                    } else {
                                        // Fallback: just keep the last incomplete sentence
                                        buffer = last_sentence.to_string();
                                        if verbose {
                                            println!("BUFFER FALLBACK: Keeping last segment: '{}'", 
                                                    last_sentence.chars().take(30).collect::<String>());
                                        }
                                    }
                                    
                                    // Only use complete sentences for processing
                                    complete_sentences = sentences[..sentences.len()-1].to_vec();
                                } else {
                                    // Only one sentence and it's incomplete - keep entire buffer
                                    if verbose {
                                        println!("Single incomplete sentence - keeping entire buffer");
                                    }
                                }
                            } else {
                                // All sentences are complete, including the last one
                                if verbose {
                                    println!("All segments appear complete - processing everything");
                                }
                                complete_sentences = sentences;
                                buffer.clear();
                            }
                            
                            // Check for accent preservation in Spanish text, but only in debug mode
                            if debug_accents && session_language.starts_with("es") {
                                for (i, sentence) in complete_sentences.iter().enumerate() {
                                    let has_accents = sentence.contains('á') || sentence.contains('é') || 
                                                     sentence.contains('í') || sentence.contains('ó') || 
                                                     sentence.contains('ú') || sentence.contains('ñ') || 
                                                     sentence.contains('ü');
                                    if has_accents {
                                        println!("SEGMENT {} RETAINS ACCENTS: {}", i, sentence);
                                    } else if verbose {
                                        println!("SEGMENT {} NO ACCENTS: {}", i, sentence);
                                    }
                                }
                            }
                        }
                    };
                    
                    // Handle special case: no complete sentences but substantial text
                    if complete_sentences.is_empty() && buffer.len() > 200 {
                        if verbose {
                            eprintln!("Processing substantial incomplete text segment...");
                        }
                        let end_index = if buffer.len() > 200 { 200 } else { buffer.len() };
                        let segment = buffer[..end_index].to_string();
                        complete_sentences.push(segment.clone());
                        buffer = buffer[end_index..].to_string();
                    }
                    
                    // Process complete sentences immediately
                    for (i, sentence) in complete_sentences.iter().enumerate() {
                        let sentence = sentence.trim();
                        if sentence.is_empty() {
                            continue;
                        }
                        
                        // Add proper punctuation if needed
                        let mut text_to_process = if !(sentence.ends_with('.') || 
                                                sentence.ends_with('!') || 
                                                sentence.ends_with('?')) {
                            format!("{}.", sentence)
                        } else {
                            sentence.to_string() 
                        };
                        
                        // Always check for UTF-8 validity before processing
                        if !String::from_utf8(text_to_process.clone().into_bytes()).is_ok() {
                            eprintln!("WARNING: Invalid UTF-8 detected in segment {}. Attempting to fix...", i);
                            // Use lossy conversion to replace invalid sequences
                            text_to_process = String::from_utf8_lossy(&text_to_process.as_bytes().to_vec()).to_string();
                        }
                        
                        // Check if there are accented characters already
                        let has_accents = text_to_process.contains('á') || text_to_process.contains('é') || 
                                         text_to_process.contains('í') || text_to_process.contains('ó') || 
                                         text_to_process.contains('ú') || text_to_process.contains('ñ') || 
                                         text_to_process.contains('ü');
                        
                        // For Spanish text, always try to restore accents
                        if session_language.starts_with("es") {
                            // Log pre-restoration state
                            if has_accents {
                                eprintln!("SEGMENT {} HAS ACCENTS: Accented characters present before restoration", i);
                            } else {
                                eprintln!("SEGMENT {} NO ACCENTS: No accented characters found, will attempt restoration", i);
                            }
                            
                            // Use kokoros restore_spanish_accents to fix lost accents
                            let restored = kokoros::tts::koko::restore_spanish_accents(&text_to_process);
                            
                            // Compare before and after restoration
                            if restored != text_to_process {
                                eprintln!("ACCENT RESTORATION: Fixed accents in segment {}", i);
                                eprintln!("  Before: {}", text_to_process);
                                eprintln!("  After: {}", restored);
                                
                                // Use the restored text
                                text_to_process = restored;
                            } else if !has_accents {
                                eprintln!("WARNING: Segment {} still has no accents after restoration attempt", i);
                                eprintln!("  Text: {}", text_to_process);
                            }
                        }
                        
                        eprintln!("Processing segment {}: {}", i+1, 
                            if text_to_process.len() > 50 {
                                format!("{}...", &text_to_process[..50])
                            } else {
                                text_to_process.clone() 
                            });
                        
                        // Debug log for Spanish special characters
                        if session_language.starts_with("es") {
                            let contains_special = text_to_process.contains('ñ') || 
                                                  text_to_process.contains('á') || 
                                                  text_to_process.contains('é') || 
                                                  text_to_process.contains('í') || 
                                                  text_to_process.contains('ó') || 
                                                  text_to_process.contains('ú') || 
                                                  text_to_process.contains('ü');
                            if contains_special {
                                eprintln!("DEBUG SPANISH CHARS: Found special characters in text");
                                for (i, c) in text_to_process.char_indices() {
                                    if !c.is_ascii() {
                                        eprintln!("  Pos {}: '{}' (Unicode: U+{:04X})", i, c, c as u32);
                                    }
                                }
                                eprintln!("Raw text with special chars: {}", text_to_process);
                            }
                        }
                        
                        // Generate audio with consistent language/voice
                        match tts.tts_raw_audio(
                            &text_to_process,
                            &session_language,
                            &session_style,
                            speed,
                            initial_silence,
                            false,  // Never auto-detect again
                            true    // Force the selected style
                        ) {
                            Ok(audio) => {
                                // Stream this chunk immediately
                                tx.send(audio.clone())?;
                                
                                // Also write to WAV file
                                write_audio_chunk(&mut wav_file, &audio)?;
                                wav_file.flush()?;
                                
                                eprintln!("Streaming audio for this segment...");
                            },
                            Err(e) => {
                                eprintln!("Error processing segment: {}", e);
                                // Continue with the next sentence
                            }
                        }
                    }
                }
                
                // Process any remaining text at EOF
                if !buffer.trim().is_empty() {
                    eprintln!("Processing final text: {}", buffer.trim());
                    
                    // Add period if needed
                    let mut final_text = if !(buffer.trim().ends_with('.') || 
                                        buffer.trim().ends_with('!') || 
                                        buffer.trim().ends_with('?')) {
                        format!("{}.", buffer.trim())
                    } else {
                        buffer.trim().to_string() 
                    };
                    
                    // Always check for UTF-8 validity before processing
                    if !String::from_utf8(final_text.clone().into_bytes()).is_ok() {
                        eprintln!("WARNING: Invalid UTF-8 detected in final text. Attempting to fix...");
                        // Use lossy conversion to replace invalid sequences
                        final_text = String::from_utf8_lossy(&final_text.as_bytes().to_vec()).to_string();
                    }
                    
                    // Check if there are already accented characters
                    let has_accents = final_text.contains('á') || final_text.contains('é') || 
                                     final_text.contains('í') || final_text.contains('ó') || 
                                     final_text.contains('ú') || final_text.contains('ñ') || 
                                     final_text.contains('ü');
                    
                    // For Spanish text, always try to restore accents
                    if session_language.starts_with("es") {
                        // Log pre-restoration state
                        if has_accents {
                            eprintln!("FINAL TEXT HAS ACCENTS: Accented characters present before restoration");
                        } else {
                            eprintln!("FINAL TEXT NO ACCENTS: No accented characters found, will attempt restoration");
                        }
                        
                        // Use our UTF-8 safe accent restoration
                        let restored = kokoros::tts::koko::restore_spanish_accents(&final_text);
                        
                        // Compare before and after restoration
                        if restored != final_text {
                            eprintln!("ACCENT RESTORATION: Fixed accents in final text");
                            eprintln!("  Before: {}", final_text);
                            eprintln!("  After: {}", restored);
                            
                            // Use the restored text
                            final_text = restored;
                        } else if !has_accents {
                            eprintln!("WARNING: Final text still has no accents after restoration attempt");
                            eprintln!("  Text: {}", final_text);
                        }
                        
                        // Show each accented character for debugging
                        for (i, c) in final_text.char_indices() {
                            if !c.is_ascii() {
                                eprintln!("  FINAL TEXT Pos {}: '{}' (Unicode: U+{:04X})", i, c, c as u32);
                            }
                        }
                    };
                    
                    // Generate audio with consistent settings
                    match tts.tts_raw_audio(
                        &final_text,
                        &session_language,
                        &session_style,
                        speed,
                        initial_silence,
                        false,
                        true
                    ) {
                        Ok(audio) => {
                            // Stream final chunk
                            tx.send(audio.clone())?;
                            
                            // Write to WAV file
                            write_audio_chunk(&mut wav_file, &audio)?;
                            wav_file.flush()?;
                            
                            eprintln!("Streaming final audio segment...");
                        },
                        Err(e) => {
                            eprintln!("Error processing final segment: {}", e);
                        }
                    }
                }
                
                // Drop the sender to close the channel
                drop(tx);
                
                // Wait for all audio to finish playing
                eprintln!("All text processed. Waiting for audio playback to complete...");
                sink.sleep_until_end();
            }
        }

        // Final cleanup before exiting
        println!("Performing final cleanup...");
        
        // Explicit cleanup to manage ONNX Runtime resources
        tts.cleanup();
        
        // Sleep to allow background threads to finish
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        println!("Cleanup complete, exiting normally");
        
        // Let the program exit normally instead of forcing termination
        Ok(())
    })
}
