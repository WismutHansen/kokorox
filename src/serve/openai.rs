use axum::{
    routing::post,
    Router,
    Json,
    extract::State,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use crate::tts::koko::TTSKoko;
use axum::http::StatusCode;

#[derive(Deserialize)]
struct TTSRequest {
    model: String,
    input: String,
    voice: Option<String>,
}

#[derive(Serialize)]
struct TTSResponse {
    status: String,
    file_path: String,
}

pub async fn create_server(tts: TTSKoko) -> Router {
    Router::new()
        .route("/v1/audio/speech", post(handle_tts))
        .layer(CorsLayer::permissive())
        .with_state(tts)
}

async fn handle_tts(
    State(tts): State<TTSKoko>,
    Json(payload): Json<TTSRequest>,
) -> Result<Json<TTSResponse>, StatusCode> {
    let voice = payload.voice.unwrap_or_else(|| "af_sky".to_string());
    
    // Generate unique output filename
    let output_path = format!("output_{}.wav", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs());

    // Process TTS request
    if let Err(_) = tts.tts(&payload.input, "en-us", &voice) {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(TTSResponse {
        status: "success".to_string(),
        file_path: output_path,
    }))
}
