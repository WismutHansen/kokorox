use clap::Parser;
use kokoros::{
    tts::koko::{TTSKoko, TTSOpts},
    utils::wav::{write_audio_chunk, WavHeader},
};
use std::net::SocketAddr;
use std::{
    fs::{self},
    io::Write,
    path::Path,
};
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Parser, Debug)]
#[command(name = "kokoros")]
#[command(version = "0.1")]
#[command(author = "Lucas Jin")]
struct Cli {
    #[arg(
        short = 't',
        long = "text",
        value_name = "TEXT",
        default_value = "Hello, This is Kokoro, your remarkable AI TTS. It's a TTS model with merely 82 million parameters yet delivers incredible audio quality.
        This is one of the top notch Rust based inference models, and I'm sure you'll love it. If you do, please give us a star. Thank you very much.
        As the night falls, I wish you all a peaceful and restful sleep. May your dreams be filled with joy and happiness. Good night, and sweet dreams!"
    )]
    text: String,

    #[arg(
        short = 'l',
        long = "lan",
        value_name = "LANGUAGE",
        help = "https://github.com/espeak-ng/espeak-ng/blob/master/docs/languages.md",
        default_value = "en-us"
    )]
    lan: String,

    #[arg(
        short = 'm',
        long = "model",
        value_name = "MODEL",
        default_value = "checkpoints/kokoro-v0_19.onnx"
    )]
    model_path: String,

    #[arg(
        short = 's',
        long = "style",
        value_name = "STYLE",
        // if users use `af_sarah.4+af_nicole.6` as style name
        // then we blend it, with 0.4*af_sarah + 0.6*af_nicole
        default_value = "af_sarah.4+af_nicole.6"
    )]
    style: String,

    #[arg(
        short = 'p',
        long = "speed",
        value_name = "SPEED",
        default_value_t = 1.0
    )]
    speed: f32,

    #[arg(long = "mono", default_value_t = false)]
    mono: bool,

    #[arg(long = "oai", value_name = "OpenAI server")]
    oai: bool,

    #[arg(long = "stream", help = "Enable streaming mode")]
    stream: bool,

    #[arg(
        short = 'o',
        long = "output",
        value_name = "OUTPUT_PATH",
        help = "Output path for WAV file",
        default_value = "tmp/output.wav"
    )]
    save_path: String,
}

async fn handle_streaming_mode(
    tts: &TTSKoko,
    lan: &str,
    style: &str,
    speed: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    // Use std::io::stdout() for sync writing
    let mut stdout = std::io::stdout();

    // Write WAV header first
    eprintln!("Entering streaming mode. Type text and press Enter. Use Ctrl+D to exit.");

    let header = WavHeader::new(1, 24000, 32);
    header.write_header(&mut stdout)?;
    stdout.flush()?;

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }

        // Process the line and get audio data
        match tts.tts_raw_audio(&line, lan, style, speed) {
            Ok(raw_audio) => {
                // Write the raw audio samples directly
                write_audio_chunk(&mut stdout, &raw_audio)?;
                stdout.flush()?;
            }
            Err(e) => eprintln!("Error processing line: {}", e),
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let Cli {
            text,
            lan,
            model_path,
            style,
            speed,
            mono,
            oai,
            stream,
            save_path,
        } = Cli::parse();

        let tts = TTSKoko::new(&model_path).await;

        if stream {
            handle_streaming_mode(&tts, &lan, &style, speed).await?;
            Ok(())
        } else if oai {
            let app = kokoros_openai::create_server(tts).await;
            let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
            println!("Starting OpenAI-compatible server on http://localhost:3000");
            kokoros_openai::serve(
                tokio::net::TcpListener::bind(&addr).await?,
                app.into_make_service(),
            )
            .await?;
            Ok(())
        } else {
            let path = Path::new(&text);
            if path.exists() && path.is_file() {
                let file_content = fs::read_to_string(path)?;
                for (i, line) in file_content.lines().enumerate() {
                    let stripped_line = line.trim();
                    if !stripped_line.is_empty() {
                        let save_path = format!("{save_path}_{i}.wav");
                        tts.tts(TTSOpts {
                            txt: stripped_line,
                            lan: &lan,
                            style_name: &style,
                            save_path: &save_path,
                            mono,
                            speed: speed,
                        })?;
                    }
                }
                return Ok(());
            }

            tts.tts(TTSOpts {
                txt: &text,
                lan: &lan,
                style_name: &style,
                save_path: &save_path,
                mono,
                speed,
            })?;
            Ok(())
        }
    })
}
