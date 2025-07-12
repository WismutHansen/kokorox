use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt};
use kokorox::tts::koko::TTSKoko;
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[derive(Deserialize)]
struct ClientCommand {
    command: String,
    text: Option<String>,
    voice: Option<String>,
}

#[derive(Serialize)]
struct AudioChunk<'a> {
    #[serde(rename = "type")]
    msg_type: &'a str,
    chunk: &'a str,
    index: usize,
    total: usize,
    sample_rate: u32,
}

#[derive(Serialize)]
struct SimpleMsg<'a> {
    #[serde(rename = "type")]
    msg_type: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    voice: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    voices: Option<&'a [String]>,
}

async fn handle_connection(stream: TcpStream, tts: TTSKoko) {
    if let Ok(ws_stream) = accept_async(stream).await {
        let voices = tts.get_available_voices();
        let sample_rate = tts.sample_rate();
        let mut current_voice = voices
            .first()
            .cloned()
            .unwrap_or_else(|| "af_heart".to_string());
        let (mut write, mut read) = ws_stream.split();

        while let Some(Ok(msg)) = read.next().await {
            if let Message::Text(text) = msg {
                match serde_json::from_str::<ClientCommand>(&text) {
                    Ok(cmd) => match cmd.command.as_str() {
                        "list_voices" => {
                            let reply = SimpleMsg {
                                msg_type: "voices",
                                voice: Some(&current_voice),
                                voices: Some(&voices),
                            };
                            if let Ok(json) = serde_json::to_string(&reply) {
                                let _ = write.send(Message::Text(json)).await;
                            }
                        }
                        "set_voice" => {
                            if let Some(v) = cmd.voice {
                                if voices.contains(&v) {
                                    current_voice = v.clone();
                                    let reply = SimpleMsg {
                                        msg_type: "voice_changed",
                                        voice: Some(&current_voice),
                                        voices: None,
                                    };
                                    if let Ok(json) = serde_json::to_string(&reply) {
                                        let _ = write.send(Message::Text(json)).await;
                                    }
                                } else {
                                    let reply = SimpleMsg {
                                        msg_type: "error",
                                        voice: None,
                                        voices: None,
                                    };
                                    let _ = write
                                        .send(Message::Text(serde_json::to_string(&reply).unwrap()))
                                        .await;
                                }
                            }
                        }
                        "synthesize" => {
                            if let Some(text) = cmd.text {
                                let _ = write
                                    .send(Message::Text(
                                        serde_json::to_string(&SimpleMsg {
                                            msg_type: "synthesis_started",
                                            voice: None,
                                            voices: None,
                                        })
                                        .unwrap(),
                                    ))
                                    .await;
                                let audio_opt = {
                                    let tts_result = tts.tts_raw_audio(
                                        &text,
                                        "en-us",
                                        &current_voice,
                                        1.0,
                                        None,
                                        false,
                                        true,
                                    );
                                    match tts_result {
                                        Ok(audio) => Some(audio),
                                        Err(e) => {
                                            eprintln!("TTS error: {}", e);
                                            None
                                        }
                                    }
                                };
                                if let Some(audio) = audio_opt {
                                    let encoded = encode_audio(&audio);
                                    let chunk = AudioChunk {
                                        msg_type: "audio_chunk",
                                        chunk: &encoded,
                                        index: 0,
                                        total: 1,
                                        sample_rate,
                                    };
                                    if let Ok(json) = serde_json::to_string(&chunk) {
                                        let _ = write.send(Message::Text(json)).await;
                                    }
                                    let done = SimpleMsg {
                                        msg_type: "synthesis_completed",
                                        voice: None,
                                        voices: None,
                                    };
                                    let _ = write
                                        .send(Message::Text(
                                            serde_json::to_string(&done).unwrap(),
                                        ))
                                        .await;
                                } else {
                                    let err = SimpleMsg {
                                        msg_type: "error",
                                        voice: None,
                                        voices: None,
                                    };
                                    let _ = write
                                        .send(Message::Text(
                                            serde_json::to_string(&err).unwrap(),
                                        ))
                                        .await;
                                }
                            }
                        }
                        _ => {
                            let reply = SimpleMsg {
                                msg_type: "error",
                                voice: None,
                                voices: None,
                            };
                            let _ = write
                                .send(Message::Text(serde_json::to_string(&reply).unwrap()))
                                .await;
                        }
                    },
                    Err(_) => {
                        let reply = SimpleMsg {
                            msg_type: "error",
                            voice: None,
                            voices: None,
                        };
                        let _ = write
                            .send(Message::Text(serde_json::to_string(&reply).unwrap()))
                            .await;
                    }
                }
            }
        }
    }
}

fn encode_audio(samples: &[f32]) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let mut bytes = Vec::with_capacity(samples.len() * 2);
    for &s in samples {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    STANDARD.encode(bytes)
}

/// Start the WebSocket server
pub async fn start_server(tts: TTSKoko, addr: SocketAddr) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on {}", addr);
    loop {
        let (stream, _) = listener.accept().await?;
        let tts_clone = tts.clone();
        tokio::spawn(async move {
            handle_connection(stream, tts_clone).await;
        });
    }
}
