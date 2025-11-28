use std::net::SocketAddr;

use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use kokorox::tts::koko::{TTSKoko, TTSManager};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

#[derive(Deserialize)]
struct ClientCommand {
    command: String,
    text: Option<String>,
    voice: Option<String>,
    language: Option<String>,
    speed: Option<f32>,
    /// If true, auto-detect language and switch models accordingly
    #[serde(default)]
    auto_detect: bool,
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

async fn handle_connection(stream: TcpStream, manager: TTSManager) {
    if let Ok(ws_stream) = accept_async(stream).await {
        let voices = manager.get_available_voices().await;
        let _sample_rate = manager.sample_rate();
        let mut current_voice = voices
            .first()
            .cloned()
            .unwrap_or_else(|| "af_heart".to_string());
        let mut current_language = "en-us".to_string();
        let mut current_speed = 1.0f32;
        let mut auto_detect = false;
        let (mut write, mut read) = ws_stream.split();

        while let Some(Ok(msg)) = read.next().await {
            if let Message::Text(text) = msg {
                match serde_json::from_str::<ClientCommand>(&text) {
                    Ok(cmd) => match cmd.command.as_str() {
                        "list_voices" => {
                            // Get voices from current model
                            let current_voices = manager.get_available_voices().await;
                            let reply = SimpleMsg {
                                msg_type: "voices",
                                voice: Some(&current_voice),
                                voices: Some(&current_voices),
                            };
                            if let Ok(json) = serde_json::to_string(&reply) {
                                let _ = write.send(Message::Text(json)).await;
                            }
                        }
                        "set_voice" => {
                            if let Some(v) = cmd.voice {
                                // For now, accept voice without strict validation
                                // since voices change when model switches
                                current_voice = v.clone();
                                let reply = SimpleMsg {
                                    msg_type: "voice_changed",
                                    voice: Some(&current_voice),
                                    voices: None,
                                };
                                if let Ok(json) = serde_json::to_string(&reply) {
                                    let _ = write.send(Message::Text(json)).await;
                                }
                            }
                        }
                        "set_language" => {
                            if let Some(lang) = cmd.language {
                                current_language = lang.clone();
                                // Pre-load the appropriate model for this language
                                let _ = manager.get_tts_for_language(&current_language).await;
                                let reply = SimpleMsg {
                                    msg_type: "language_changed",
                                    voice: None,
                                    voices: None,
                                };
                                if let Ok(json) = serde_json::to_string(&reply) {
                                    let _ = write.send(Message::Text(json)).await;
                                }
                            }
                        }
                        "set_auto_detect" => {
                            auto_detect = cmd.auto_detect;
                            let reply = SimpleMsg {
                                msg_type: "auto_detect_changed",
                                voice: None,
                                voices: None,
                            };
                            if let Ok(json) = serde_json::to_string(&reply) {
                                let _ = write.send(Message::Text(json)).await;
                            }
                        }
                        "set_speed" => {
                            if let Some(speed) = cmd.speed {
                                current_speed = speed.clamp(0.1, 3.0);
                                let reply = SimpleMsg {
                                    msg_type: "speed_changed",
                                    voice: None,
                                    voices: None,
                                };
                                if let Ok(json) = serde_json::to_string(&reply) {
                                    let _ = write.send(Message::Text(json)).await;
                                }
                            }
                        }
                        "synthesize" => {
                            if let Some(synth_text) = cmd.text {
                                let language = cmd.language.as_deref().unwrap_or(&current_language);
                                let speed = cmd.speed.unwrap_or(current_speed);
                                let use_auto_detect = cmd.auto_detect || auto_detect;
                                
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

                                let result = synthesize_streaming_with_manager(
                                    &manager,
                                    &synth_text,
                                    &current_voice,
                                    language,
                                    speed,
                                    use_auto_detect,
                                    &mut write,
                                )
                                .await;

                                if result.is_ok() {
                                    let done = SimpleMsg {
                                        msg_type: "synthesis_completed",
                                        voice: None,
                                        voices: None,
                                    };
                                    let _ = write
                                        .send(Message::Text(serde_json::to_string(&done).unwrap()))
                                        .await;
                                } else {
                                    let err = SimpleMsg {
                                        msg_type: "error",
                                        voice: None,
                                        voices: None,
                                    };
                                    let _ = write
                                        .send(Message::Text(serde_json::to_string(&err).unwrap()))
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

/// Legacy handler for single TTSKoko instance (for backward compatibility)
async fn handle_connection_legacy(stream: TcpStream, tts: TTSKoko) {
    if let Ok(ws_stream) = accept_async(stream).await {
        let voices = tts.get_available_voices();
        let _sample_rate = tts.sample_rate();
        let mut current_voice = voices
            .first()
            .cloned()
            .unwrap_or_else(|| "af_heart".to_string());
        let mut current_language = "en-us".to_string();
        let mut current_speed = 1.0f32;
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
                                let is_valid = if v.contains('+') {
                                    v.split('+')
                                        .all(|part| {
                                            part.split_once('.')
                                                .map(|(name, _)| voices.contains(&name.to_string()))
                                                .unwrap_or(false)
                                        })
                                } else {
                                    voices.contains(&v)
                                };

                                if is_valid {
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
                        "set_language" => {
                            if let Some(lang) = cmd.language {
                                current_language = lang.clone();
                                let reply = SimpleMsg {
                                    msg_type: "language_changed",
                                    voice: None,
                                    voices: None,
                                };
                                if let Ok(json) = serde_json::to_string(&reply) {
                                    let _ = write.send(Message::Text(json)).await;
                                }
                            }
                        }
                        "set_speed" => {
                            if let Some(speed) = cmd.speed {
                                current_speed = speed.clamp(0.1, 3.0);
                                let reply = SimpleMsg {
                                    msg_type: "speed_changed",
                                    voice: None,
                                    voices: None,
                                };
                                if let Ok(json) = serde_json::to_string(&reply) {
                                    let _ = write.send(Message::Text(json)).await;
                                }
                            }
                        }
                        "synthesize" => {
                            if let Some(synth_text) = cmd.text {
                                let language = cmd.language.as_deref().unwrap_or(&current_language);
                                let speed = cmd.speed.unwrap_or(current_speed);
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

                                let result = synthesize_streaming(
                                    &tts,
                                    &synth_text,
                                    &current_voice,
                                    language,
                                    speed,
                                    &mut write,
                                )
                                .await;

                                if result.is_ok() {
                                    let done = SimpleMsg {
                                        msg_type: "synthesis_completed",
                                        voice: None,
                                        voices: None,
                                    };
                                    let _ = write
                                        .send(Message::Text(serde_json::to_string(&done).unwrap()))
                                        .await;
                                } else {
                                    let err = SimpleMsg {
                                        msg_type: "error",
                                        voice: None,
                                        voices: None,
                                    };
                                    let _ = write
                                        .send(Message::Text(serde_json::to_string(&err).unwrap()))
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

async fn synthesize_streaming(
    tts: &TTSKoko,
    text: &str,
    voice: &str,
    language: &str,
    speed: f32,
    write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use kokorox::tts::segmentation::split_into_sentences;

    let sentences = split_into_sentences(text);
    let total_chunks = sentences.len();

    for (index, sentence) in sentences.iter().enumerate() {
        if sentence.trim().is_empty() {
            continue;
        }

        let audio_opt =
            match tts.tts_raw_audio(sentence, language, voice, speed, None, false, true, false) {
                Ok(audio) => Some(audio),
                Err(_) => {
                    eprintln!("TTS error for sentence '{}'", sentence);
                    None
                }
            };

        if let Some(audio) = audio_opt {
            // Send this chunk immediately
            let encoded = encode_audio(&audio);
            let chunk = AudioChunk {
                msg_type: "audio_chunk",
                chunk: &encoded,
                index,
                total: total_chunks,
                sample_rate: 24000,
            };

            if let Ok(json) = serde_json::to_string(&chunk) {
                let _ = write.send(Message::Text(json)).await;
            }
        }
    }

    Ok(())
}

async fn synthesize_streaming_with_manager(
    manager: &TTSManager,
    text: &str,
    voice: &str,
    language: &str,
    speed: f32,
    auto_detect: bool,
    write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use kokorox::tts::segmentation::split_into_sentences;

    let sentences = split_into_sentences(text);
    let total_chunks = sentences.len();

    for (index, sentence) in sentences.iter().enumerate() {
        if sentence.trim().is_empty() {
            continue;
        }

        let audio_opt = match manager
            .tts_raw_audio(sentence, language, voice, speed, None, auto_detect, true, false)
            .await
        {
            Ok(audio) => Some(audio),
            Err(e) => {
                eprintln!("TTS error for sentence '{}': {}", sentence, e);
                None
            }
        };

        if let Some(audio) = audio_opt {
            let encoded = encode_audio(&audio);
            let chunk = AudioChunk {
                msg_type: "audio_chunk",
                chunk: &encoded,
                index,
                total: total_chunks,
                sample_rate: 24000,
            };

            if let Ok(json) = serde_json::to_string(&chunk) {
                let _ = write.send(Message::Text(json)).await;
            }
        }
    }

    Ok(())
}

fn encode_audio(samples: &[f32]) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    // Convert f32 samples to i16 PCM
    let mut pcm_data = Vec::with_capacity(samples.len() * 2);
    for &s in samples {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        pcm_data.extend_from_slice(&v.to_le_bytes());
    }

    // Create WAV file with proper header
    let sample_rate = 24000u32;
    let num_channels = 1u16;
    let bits_per_sample = 16u16;
    let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align = num_channels * bits_per_sample / 8;

    let mut wav_data = Vec::with_capacity(44 + pcm_data.len());

    // RIFF header
    wav_data.extend_from_slice(b"RIFF");
    wav_data.extend_from_slice(&(36 + pcm_data.len() as u32).to_le_bytes());
    wav_data.extend_from_slice(b"WAVE");

    // fmt chunk
    wav_data.extend_from_slice(b"fmt ");
    wav_data.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    wav_data.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    wav_data.extend_from_slice(&num_channels.to_le_bytes());
    wav_data.extend_from_slice(&sample_rate.to_le_bytes());
    wav_data.extend_from_slice(&byte_rate.to_le_bytes());
    wav_data.extend_from_slice(&block_align.to_le_bytes());
    wav_data.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data chunk
    wav_data.extend_from_slice(b"data");
    wav_data.extend_from_slice(&(pcm_data.len() as u32).to_le_bytes());
    wav_data.extend_from_slice(&pcm_data);

    STANDARD.encode(wav_data)
}

/// Start the WebSocket server with TTSManager (supports dynamic model switching)
pub async fn start_server_with_manager(manager: TTSManager, addr: SocketAddr) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on {} (dynamic model switching enabled)", addr);
    loop {
        let (stream, _) = listener.accept().await?;
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            handle_connection(stream, manager_clone).await;
        });
    }
}

/// Start the WebSocket server with a single TTSKoko instance (legacy, no model switching)
pub async fn start_server(tts: TTSKoko, addr: SocketAddr) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on {}", addr);
    loop {
        let (stream, _) = listener.accept().await?;
        let tts_clone = tts.clone();
        tokio::spawn(async move {
            handle_connection_legacy(stream, tts_clone).await;
        });
    }
}
