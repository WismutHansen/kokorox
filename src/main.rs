mod onn;
mod serve;
mod tts;
mod utils;

use clap::Parser;
use std::net::SocketAddr;
use tts::koko::TTSKoko;

#[derive(Parser, Debug)]
#[command(name = "kokoros")]
#[command(version = "0.1")]
#[command(author = "Lucas Jin")]
struct Cli {
    #[arg(short = 't', long = "text", value_name = "TEXT")]
    text: Option<String>,

    #[arg(
        short = 'l',
        long = "lan",
        value_name = "LANGUAGE",
        help = "https://github.com/espeak-ng/espeak-ng/blob/master/docs/languages.md"
    )]
    lan: Option<String>,

    #[arg(short = 'm', long = "model", value_name = "MODEL")]
    model: Option<String>,

    #[arg(short = 's', long = "style", value_name = "STYLE")]
    style: Option<String>,

    #[arg(long = "oai", value_name = "OpenAI server")]
    oai: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let args = Cli::parse();

        // if users use `af_sky.4+af_nicho.3` as style name
        // then we blend it, with 0.4 af_sky + 0.3 af_nicho

        let model_path = args.model.unwrap_or_else(|| "checkpoints/kokoro-v0_19.onnx".to_string());
        let style = args.style.unwrap_or_else(|| "af_sky.4+af_nicole.5".to_string());
        let lan = args.lan.unwrap_or_else(|| { "en-us".to_string() });

        let tts = TTSKoko::new(&model_path);

        if args.oai {
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
            let _ = tts.tts(&txt, &lan, &style);
            Ok(())
        }
    })
}
