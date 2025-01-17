mod onn;
mod serve;
mod tts;
mod utils;

use clap::Parser;
use tts::koko::TTSKoko;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
#[command(name = "kokoros")]
#[command(version = "0.1")]
#[command(author = "Lucas Jin")]
struct Cli {
    #[arg(short = 't', long = "text", value_name = "TEXT")]
    text: Option<String>,

    #[arg(short = 'l', long = "language", value_name = "LANGUAGE", help="https://github.com/espeak-ng/espeak-ng/blob/master/docs/languages.md")]
    language: Option<String>,

    #[arg(long = "oai", value_name = "OpenAI server")]
    oai: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let args = Cli::parse();

        if args.oai {
            let tts = TTSKoko::new("checkpoints/kokoro-v0_19.onnx");
            let app = serve::openai::create_server(tts).await;
            
            let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
            println!("Starting OpenAI-compatible server on http://localhost:3000");
            axum::serve(
                tokio::net::TcpListener::bind(&addr).await?,
                app.into_make_service(),
            )
            .await?;
            Ok(())
        } else {
            let txt = args.text.unwrap_or_else(|| {
                r#"
                Hello, This is Kokoro. Your amazing AI TTS! A TTS model with only 82 million parameters that achieve incredible audio quality. 
                This is the one of the best Rust inference, I help you will like it. 
                Please give us a star if you do, thank you very much.
                "#
                .to_string()
            });
            let language = args.language.unwrap_or_else(|| { "en-us".to_string() });

            let tts = TTSKoko::new("checkpoints/kokoro-v0_19.onnx");
            tts.tts(&txt, &language, "af_sky");
            Ok(())
        }
    })
}
