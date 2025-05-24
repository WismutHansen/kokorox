use std::error::Error;
use std::io::{self};

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{extract::State, routing::get, routing::post, Json, Router};
use kokorox::{
    tts::koko::{InitConfig as TTSKokoInitConfig, TTSKoko},
    utils::mp3::pcm_to_mp3,
    utils::wav::{write_audio_chunk, WavHeader},
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "lowercase")]
enum AudioFormat {
    #[default]
    Wav,
    Mp3,
}

#[derive(Deserialize)]
struct Voice(String);

impl Default for Voice {
    fn default() -> Self {
        Self("af_sky".into())
    }
}

#[derive(Deserialize)]
struct Speed(f32);

impl Default for Speed {
    fn default() -> Self {
        Self(1.)
    }
}

#[derive(Deserialize)]
struct SpeechRequest {
    // Only one Kokoro model exists
    #[allow(dead_code)]
    model: String,

    input: String,

    #[serde(default)]
    voice: Voice,

    // Must be WAV
    #[allow(dead_code)]
    #[serde(default)]
    response_format: AudioFormat,

    #[serde(default)]
    speed: Speed,

    #[serde(default)]
    initial_silence: Option<usize>,

    // Language for the text to be synthesized
    #[serde(default)]
    language: Option<String>,

    // Enable automatic language detection
    #[serde(default)]
    auto_detect: Option<bool>,
}

#[derive(Serialize)]
struct VoiceInfo {
    id: String,
    name: String,
    description: String,
    language: String,
    gender: String,
}

#[derive(Serialize)]
struct VoicesDetailedResponse {
    voices: Vec<VoiceInfo>,
}

#[derive(Serialize)]
struct VoicesResponse {
    voices: Vec<String>,
}

pub async fn create_server(tts: TTSKoko) -> Router {
    println!("create_server()");

    Router::new()
        .route("/", get(handle_home))
        .route("/v1/audio/speech", post(handle_tts))
        .route("/v1/audio/voices", get(get_voices))
        .route("/v1/audio/voices/detailed", get(get_voices_detailed))
        .layer(CorsLayer::permissive())
        .with_state(tts)
}

pub use axum::serve;

#[derive(Debug)]
enum SpeechError {
    // Deciding to modify this example in order to see errors
    // (e.g. with tracing) is up to the developer
    #[allow(dead_code)]
    Koko(Box<dyn Error>),

    #[allow(dead_code)]
    Header(io::Error),

    #[allow(dead_code)]
    Chunk(io::Error),

    #[allow(dead_code)]
    Mp3Conversion(std::io::Error),
}

impl IntoResponse for SpeechError {
    fn into_response(self) -> Response {
        // None of these errors make sense to expose to the user of the API
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

/// Returns a 200 OK response to make it easier to check if the server is
/// running.
async fn handle_home() -> &'static str {
    "OK"
}

async fn get_voices(State(tts): State<TTSKoko>) -> Json<VoicesResponse> {
    // Get the available voices from the TTSKoko instance
    let mut voices = tts.get_available_voices();
    voices.sort();

    Json(VoicesResponse { voices })
}

async fn get_voices_detailed(State(tts): State<TTSKoko>) -> Json<VoicesDetailedResponse> {
    // Get the available voices from the TTSKoko instance
    let voice_styles = tts.get_available_voices();

    let mut voices = Vec::new();

    for voice_id in voice_styles {
        let (name, description, language, gender) = parse_voice_info(&voice_id);
        voices.push(VoiceInfo {
            id: voice_id.clone(),
            name,
            description,
            language,
            gender,
        });
    }

    // Sort voices by language and then by gender
    voices.sort_by(|a, b| {
        a.language
            .cmp(&b.language)
            .then(a.gender.cmp(&b.gender))
            .then(a.name.cmp(&b.name))
    });

    Json(VoicesDetailedResponse { voices })
}

fn parse_voice_info(voice_id: &str) -> (String, String, String, String) {
    // Parse voice ID patterns like "af_heart", "em_alex", "zf_xiaoxiao"
    // Format: [language_prefix][gender][_name]

    let parts: Vec<&str> = voice_id.split('_').collect();
    if parts.len() < 2 {
        return (
            voice_id.to_string(),
            format!("Voice {}", voice_id),
            "unknown".to_string(),
            "unknown".to_string(),
        );
    }

    let prefix = parts[0];
    let name = parts[1..].join("_");

    let (language_code, gender) = if prefix.len() >= 2 {
        let lang_part = &prefix[..prefix.len() - 1];
        let gender_part = &prefix[prefix.len() - 1..];

        let gender = match gender_part {
            "f" => "female",
            "m" => "male",
            _ => "unknown",
        };

        (lang_part, gender)
    } else {
        (prefix, "unknown")
    };

    let language = match language_code {
        "a" => "English (US)",
        "b" => "English (UK)",
        "e" => "Spanish",
        "p" => "Portuguese",
        "f" => "French",
        "i" => "Italian",
        "d" => "German",
        "z" => "Chinese",
        "j" => "Japanese",
        "k" => "Korean",
        "r" => "Russian",
        "h" => "Hindi",
        _ => "Unknown",
    };

    let display_name = format!(
        "{} ({})",
        name.chars()
            .next()
            .unwrap_or('a')
            .to_uppercase()
            .to_string()
            + &name[1..],
        gender
            .chars()
            .next()
            .unwrap_or('u')
            .to_uppercase()
            .to_string()
            + &gender[1..]
    );

    let description = format!("{} {} voice", language, gender);

    (
        display_name,
        description,
        language.to_string(),
        gender.to_string(),
    )
}

async fn handle_tts(
    State(tts): State<TTSKoko>,
    Json(SpeechRequest {
        model: _,
        input,
        voice: Voice(voice),
        response_format,
        speed: Speed(speed),
        initial_silence,
        language,
        auto_detect,
    }): Json<SpeechRequest>,
) -> Result<Response, SpeechError> {
    // Determine language - either specified, auto-detected, or default
    let lan = language.unwrap_or_else(|| "en-us".to_string());
    let auto_detect_language = auto_detect.unwrap_or(false);

    // Determine if we should force style (always false for API)
    let force_style = false;

    // Log info about the request
    if auto_detect_language {
        println!("API Request: Auto-detecting language for text: '{}'", input);
    } else {
        println!(
            "API Request: Using language '{}' for text: '{}'",
            lan, input
        );
    }

    let raw_audio = tts
        .tts_raw_audio(
            &input,
            &lan,
            &voice,
            speed,
            initial_silence,
            auto_detect_language,
            force_style,
        )
        .map_err(SpeechError::Koko)?;

    let sample_rate = TTSKokoInitConfig::default().sample_rate;

    let (content_type, audio_data) = match response_format {
        AudioFormat::Wav => {
            let mut wav_data = Vec::default();
            let header = WavHeader::new(1, sample_rate, 32);
            header
                .write_header(&mut wav_data)
                .map_err(SpeechError::Header)?;
            write_audio_chunk(&mut wav_data, &raw_audio).map_err(SpeechError::Chunk)?;

            ("audio/wav", wav_data)
        }
        AudioFormat::Mp3 => {
            let mp3_data =
                pcm_to_mp3(&raw_audio, sample_rate).map_err(SpeechError::Mp3Conversion)?;

            ("audio/mpeg", mp3_data)
        }
    };

    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .body(audio_data.into())
        .map_err(|e| SpeechError::Mp3Conversion(std::io::Error::new(std::io::ErrorKind::Other, e)))
}
