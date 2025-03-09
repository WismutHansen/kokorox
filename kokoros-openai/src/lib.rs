use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{extract::State, routing::get, routing::post, Json, Router};
use kokoros::{
    tts::koko::{InitConfig as TTSKokoInitConfig, TTSKoko},
    utils::wav::{write_audio_chunk, WavHeader},
};
use serde::Deserialize;
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
}

pub async fn create_server(tts: TTSKoko) -> Router {
    println!("create_server()");
    
    Router::new()
        .route("/", get(handle_home))
        .route("/v1/audio/speech", post(handle_tts))
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

async fn handle_tts(
    State(tts): State<TTSKoko>,
    Json(SpeechRequest {
        model: _,
        input,
        voice: Voice(voice),
        response_format,
        speed: Speed(speed),
        initial_silence,
    }): Json<SpeechRequest>,
) -> Result<Response, SpeechError> {
    println!("handle_tts()");
    
    let raw_audio = tts
        .tts_raw_audio(&input, "en-us", &voice, speed, initial_silence)
        .map_err(SpeechError::Koko)?;
    println!("handle_tts() - raw data");

    let sample_rate = TTSKokoInitConfig::default().sample_rate;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    println!("response_format: {:?}", response_format);

    let (content_type, audio_data) = match response_format {
        AudioFormat::Wav => {
            // Используем текущую реализацию WAV
            let mut wav_data = Vec::default();
            let header = WavHeader::new(1, sample_rate, 32);
            header
                .write_header(&mut wav_data)
                .map_err(SpeechError::Header)?;
            write_audio_chunk(&mut wav_data, &raw_audio).map_err(SpeechError::Chunk)?;

            // Записываем WAV в файл для проверки
            let filename = format!("debug_output_{}_sample.wav", timestamp);
            if let Ok(mut file) = File::create(&filename) {
                if let Err(e) = file.write_all(&wav_data) {
                    eprintln!("Failed to write WAV file: {}", e);
                } else {
                    println!("Debug WAV file saved to: {}", filename);
                }
            }

            ("audio/wav", wav_data)
        }
        AudioFormat::Mp3 => {
            // Конвертация в MP3 с использованием mp3lame-encoder
            let mp3_data =
                pcm_to_mp3(&raw_audio, sample_rate).map_err(|e| SpeechError::Mp3Conversion(e))?;

            // Записываем MP3 в файл для проверки
            let filename = format!("debug_output_{}_sample.mp3", timestamp);
            if let Ok(mut file) = File::create(&filename) {
                if let Err(e) = file.write_all(&mp3_data) {
                    eprintln!("Failed to write MP3 file: {}", e);
                } else {
                    println!("Debug MP3 file saved to: {}", filename);
                }
            }

            ("audio/mpeg", mp3_data)
        }
    };

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .body(audio_data.into())
        .map_err(|e| {
            SpeechError::Mp3Conversion(std::io::Error::new(std::io::ErrorKind::Other, e))
        })?)
}

use mp3lame_encoder::{Builder, FlushNoGap, Id3Tag, MonoPcm};

// Функция конвертации PCM в MP3
fn pcm_to_mp3(pcm_data: &[f32], sample_rate: u32) -> Result<Vec<u8>, std::io::Error> {
    // Инициализация MP3-кодера
    let mut mp3_encoder = Builder::new().ok_or(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("Encoder init failed"),
    ))?;

    // Настройка параметров
    mp3_encoder.set_num_channels(1).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Set channels failed: {:?}", e),
        )
    })?;
    mp3_encoder.set_sample_rate(sample_rate).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Set sample rate failed: {:?}", e),
        )
    })?;
    mp3_encoder
        .set_brate(mp3lame_encoder::Bitrate::Kbps192)
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Set bitrate failed: {:?}", e),
            )
        })?;
    mp3_encoder
        .set_quality(mp3lame_encoder::Quality::Best)
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Set quality failed: {:?}", e),
            )
        })?;

    // Добавление ID3-тегов (опционально)
    let _ = mp3_encoder.set_id3_tag(Id3Tag {
        title: b"Generated Audio",
        artist: b"TTS Model",
        album: b"Synthesized Speech",
        year: b"Current year",
        album_art: &[],
        comment: b"Generated by TTS",
    });

    let mut mp3_encoder = mp3_encoder.build().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Build encoder failed: {:?}", e),
        )
    })?;

    // Преобразование f32 в i16 (mp3lame-encoder ожидает i16)
    let pcm_i16: Vec<i16> = pcm_data
        .iter()
        .map(|&x| (x * i16::MAX as f32) as i16)
        .collect();
    let pcm = MonoPcm(&pcm_i16);

    // Кодирование в MP3
    let mut mp3_out_buffer = Vec::new();
    mp3_out_buffer.reserve(mp3lame_encoder::max_required_buffer_size(pcm.0.len()));

    let encoded_size = mp3_encoder
        .encode(pcm, mp3_out_buffer.spare_capacity_mut())
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Encoding failed: {:?}", e),
            )
        })?;

    unsafe {
        mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(encoded_size));
    }

    // Завершение кодирования (flush)
    let flush_size = mp3_encoder
        .flush::<FlushNoGap>(mp3_out_buffer.spare_capacity_mut())
        .map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Flush failed: {:?}", e))
        })?;
    unsafe {
        mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(flush_size));
    }

    Ok(mp3_out_buffer)
}
