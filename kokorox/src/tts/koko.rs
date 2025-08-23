use crate::tts::tokenize::tokenize;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

use ndarray::{Array3, ArrayBase, IxDyn, OwnedRepr};
use ndarray_npy::NpzReader;
use std::fs::File;

use crate::onn::ort_base::OrtBase;
use crate::onn::ort_koko;
use crate::utils;

use espeak_rs::text_to_phonemes;

pub struct TTSKoko {
    model: ort_koko::OrtKoko,
    styles: HashMap<String, Vec<[[f32; 256]; 1]>>,
}

impl TTSKoko {
    const MODEL_URL: &str =
        "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/kokoro-v1.0.onnx";
    const VOICES_URL: &str = "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/voices-v1.0.bin";

    const SAMPLE_RATE: u32 = 24000;

    pub async fn new(model_path: &str, voices_path: &str) -> Self {
        // Download model if it doesn't exist
        let p = Path::new(model_path);
        if !p.exists() {
            utils::fileio::download_file_from_url(TTSKoko::MODEL_URL, model_path)
                .await
                .expect("download model failed.");
        } else {
            eprintln!("load model from: {model_path}");
        }

        // Download voices if they don't exist
        let v = Path::new(voices_path);
        if !v.exists() {
            utils::fileio::download_file_from_url(TTSKoko::VOICES_URL, voices_path)
                .await
                .expect("download voices failed.");
        } else {
            eprintln!("load voices from: {voices_path}");
        }

        let model = ort_koko::OrtKoko::new(model_path.to_string())
            .expect("Failed to create Kokoro TTS model");

        model.print_info();

        let styles = Self::load_voices(voices_path);

        TTSKoko { model, styles }
    }

    pub fn tts(&self, txt: &str, language: &str, style_name: &str) {
        self.tts_with_output(txt, language, style_name, None);
    }

    pub fn tts_with_output(
        &self,
        txt: &str,
        language: &str,
        style_name: &str,
        output_path: Option<&str>,
    ) {
        println!("hello, going to tts. text: {txt}");

        // Split text into sentences and process them with pauses
        use crate::tts::segmentation::split_into_sentences;
        let sentences = split_into_sentences(txt);
        
        let mut all_tokens = Vec::new();
        let mut total_phonemes_len = 0;
        
        for (i, sentence) in sentences.iter().enumerate() {
            if sentence.trim().is_empty() {
                continue;
            }
            
            let phonemes = text_to_phonemes(sentence, language, None, true, false)
                .expect("Failed to phonemize given text using espeak-ng.")
                .join("");

            total_phonemes_len += phonemes.len();
            let mut sentence_tokens = tokenize(&phonemes);
            
            // Add pause tokens between sentences (except for the last one)
            if i < sentences.len() - 1 {
                // Token 30 is typically a space/pause, add multiple for longer pause
                sentence_tokens.extend(vec![30, 30, 30, 30]); // Add pause between sentences
            }
            
            all_tokens.extend(sentence_tokens);
        }

        let tokens = vec![all_tokens];

        if let Some(style) = self.styles.get(style_name) {
            let styles = vec![style[0][0].to_vec()];

            let start_t = Instant::now();

            let out = self.model.infer(tokens, styles, 0.8);
            println!("output: {out:?}");

            if let Ok(out) = out {
                let phonemes_len = total_phonemes_len;
                self.process_and_save_audio(start_t, out, phonemes_len, output_path)
                    .expect("save audio failed.");
            }
        } else {
            println!(
                "{style_name} not found, choose one from data/voices.json style key."
            );
        }
    }

    pub fn tts_pipe_to_stdout(
        &self,
        txt: &str,
        language: &str,
        style_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        eprintln!("TTS generating audio for: {txt}");

        // Split text into sentences and process them with pauses
        use crate::tts::segmentation::split_into_sentences;
        let sentences = split_into_sentences(txt);
        
        let mut all_tokens = Vec::new();
        let mut total_phonemes_len = 0;
        
        for (i, sentence) in sentences.iter().enumerate() {
            if sentence.trim().is_empty() {
                continue;
            }
            
            let phonemes = text_to_phonemes(sentence, language, None, true, false)
                .expect("Failed to phonemize given text using espeak-ng.")
                .join("");

            total_phonemes_len += phonemes.len();
            let mut sentence_tokens = tokenize(&phonemes);
            
            // Add pause tokens between sentences (except for the last one)
            if i < sentences.len() - 1 {
                sentence_tokens.extend(vec![30, 30, 30, 30]); // Add pause between sentences
            }
            
            all_tokens.extend(sentence_tokens);
        }

        let tokens = vec![all_tokens];

        if let Some(style) = self.styles.get(style_name) {
            let styles = vec![style[0][0].to_vec()];

            let start_t = Instant::now();

            let out = self.model.infer(tokens, styles, 0.8);

            if let Ok(out) = out {
                let phonemes_len = total_phonemes_len;
                self.stream_audio_to_stdout(start_t, out, phonemes_len)?;
            }
            Ok(())
        } else {
            eprintln!(
                "{style_name} not found, choose one from data/voices.json style key."
            );
            Err("Voice style not found".into())
        }
    }

    pub fn tts_pipe_play(
        &self,
        txt: &str,
        language: &str,
        style_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("TTS generating and playing audio for: {txt}");

        // Split text into sentences and process them with pauses
        use crate::tts::segmentation::split_into_sentences;
        let sentences = split_into_sentences(txt);
        
        let mut all_tokens = Vec::new();
        let mut total_phonemes_len = 0;
        
        for (i, sentence) in sentences.iter().enumerate() {
            if sentence.trim().is_empty() {
                continue;
            }
            
            let phonemes = text_to_phonemes(sentence, language, None, true, false)
                .expect("Failed to phonemize given text using espeak-ng.")
                .join("");

            total_phonemes_len += phonemes.len();
            let mut sentence_tokens = tokenize(&phonemes);
            
            // Add pause tokens between sentences (except for the last one)
            if i < sentences.len() - 1 {
                sentence_tokens.extend(vec![30, 30, 30, 30]); // Add pause between sentences
            }
            
            all_tokens.extend(sentence_tokens);
        }

        let tokens = vec![all_tokens];

        if let Some(style) = self.styles.get(style_name) {
            let styles = vec![style[0][0].to_vec()];

            let start_t = Instant::now();

            let out = self.model.infer(tokens, styles, 0.8);

            if let Ok(out) = out {
                let phonemes_len = total_phonemes_len;
                self.play_audio_directly(start_t, out, phonemes_len)?;
            }
            Ok(())
        } else {
            println!(
                "{style_name} not found, choose one from data/voices.json style key."
            );
            Err("Voice style not found".into())
        }
    }

    fn process_and_save_audio(
        &self,
        start_t: Instant,
        output: ArrayBase<OwnedRepr<f32>, IxDyn>,
        phonemes_len: usize,
        output_path: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Convert output to standard Vec
        let audio: Vec<f32> = output.iter().cloned().collect();

        let audio_duration = audio.len() as f32 / TTSKoko::SAMPLE_RATE as f32;
        let create_duration = start_t.elapsed().as_secs_f32();
        let speedup_factor = audio_duration / create_duration;

        println!(
            "Created audio in length of {audio_duration:.2}s for {phonemes_len} phonemes in {create_duration:.2}s ({speedup_factor:.2}x real-time)"
        );

        // Determine output path - use provided path, or fallback to sensible default
        let output_file = match output_path {
            Some(path) => path.to_string(),
            None => {
                // Use system temp directory as default
                let temp_dir = std::env::temp_dir();
                let default_path = temp_dir.join("kokoro_output.wav");
                default_path.to_string_lossy().to_string()
            }
        };

        // Ensure parent directory exists
        if let Some(parent) = Path::new(&output_file).parent() {
            fs::create_dir_all(parent)?;
        }

        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: TTSKoko::SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        let mut writer = hound::WavWriter::create(&output_file, spec)?;

        for &sample in &audio {
            writer.write_sample(sample)?;
        }

        writer.finalize()?;

        println!("Audio saved to {output_file}");
        Ok(())
    }

    fn stream_audio_to_stdout(
        &self,
        start_t: Instant,
        output: ArrayBase<OwnedRepr<f32>, IxDyn>,
        phonemes_len: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Convert output to standard Vec
        let audio: Vec<f32> = output.iter().cloned().collect();

        let audio_duration = audio.len() as f32 / TTSKoko::SAMPLE_RATE as f32;
        let create_duration = start_t.elapsed().as_secs_f32();
        let speedup_factor = audio_duration / create_duration;

        eprintln!(
            "Created audio in length of {audio_duration:.2}s for {phonemes_len} phonemes in {create_duration:.2}s ({speedup_factor:.2}x real-time)"
        );

        // Calculate data size (4 bytes per sample for 32-bit float)
        let data_size = (audio.len() * 4) as u32;

        // Write WAV header to stdout
        let header = crate::utils::wav::WavHeader::new(1, TTSKoko::SAMPLE_RATE, 32);
        let mut stdout = io::stdout();
        header.write_header(&mut stdout, data_size)?;

        // Write audio data to stdout
        crate::utils::wav::write_audio_chunk(&mut stdout, &audio)?;
        stdout.flush()?;

        eprintln!("Audio streamed to stdout");
        Ok(())
    }

    pub fn tts_pipe_to_writer<W: Write>(
        &self,
        txt: &str,
        language: &str,
        style_name: &str,
        writer: &mut W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Split text into sentences and process them with pauses
        use crate::tts::segmentation::split_into_sentences;
        let sentences = split_into_sentences(txt);
        
        let mut all_tokens = Vec::new();
        let mut total_phonemes_len = 0;
        
        for (i, sentence) in sentences.iter().enumerate() {
            if sentence.trim().is_empty() {
                continue;
            }
            
            let phonemes = text_to_phonemes(sentence, language, None, true, false)
                .expect("Failed to phonemize given text using espeak-ng.")
                .join("");

            total_phonemes_len += phonemes.len();
            let mut sentence_tokens = tokenize(&phonemes);
            
            // Add pause tokens between sentences (except for the last one)
            if i < sentences.len() - 1 {
                sentence_tokens.extend(vec![30, 30, 30, 30]); // Add pause between sentences
            }
            
            all_tokens.extend(sentence_tokens);
        }

        let tokens = vec![all_tokens];

        if let Some(style) = self.styles.get(style_name) {
            let styles = vec![style[0][0].to_vec()];

            let start_t = Instant::now();

            let out = self.model.infer(tokens, styles, 0.8);

            if let Ok(out) = out {
                let phonemes_len = total_phonemes_len;
                self.stream_audio_to_writer(start_t, out, phonemes_len, writer)?;
            }
            Ok(())
        } else {
            eprintln!(
                "{style_name} not found, choose one from data/voices.json style key."
            );
            Err("Voice style not found".into())
        }
    }

    fn stream_audio_to_writer<W: Write>(
        &self,
        start_t: Instant,
        output: ArrayBase<OwnedRepr<f32>, IxDyn>,
        phonemes_len: usize,
        writer: &mut W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Convert output to standard Vec
        let audio: Vec<f32> = output.iter().cloned().collect();

        let audio_duration = audio.len() as f32 / TTSKoko::SAMPLE_RATE as f32;
        let create_duration = start_t.elapsed().as_secs_f32();
        let speedup_factor = audio_duration / create_duration;

        eprintln!(
            "Created audio in length of {audio_duration:.2}s for {phonemes_len} phonemes in {create_duration:.2}s ({speedup_factor:.2}x real-time)"
        );

        // Calculate data size (4 bytes per sample for 32-bit float)
        let data_size = (audio.len() * 4) as u32;

        // Write WAV header to writer
        let header = crate::utils::wav::WavHeader::new(1, TTSKoko::SAMPLE_RATE, 32);
        header.write_header(writer, data_size)?;

        // Write audio data to writer
        crate::utils::wav::write_audio_chunk(writer, &audio)?;
        writer.flush()?;

        eprintln!("Audio streamed to player");
        Ok(())
    }

    fn play_audio_directly(
        &self,
        start_t: Instant,
        output: ArrayBase<OwnedRepr<f32>, IxDyn>,
        phonemes_len: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Convert output to standard Vec
        let audio: Vec<f32> = output.iter().cloned().collect();

        let audio_duration = audio.len() as f32 / TTSKoko::SAMPLE_RATE as f32;
        let create_duration = start_t.elapsed().as_secs_f32();
        let speedup_factor = audio_duration / create_duration;

        println!(
            "Created audio in length of {audio_duration:.2}s for {phonemes_len} phonemes in {create_duration:.2}s ({speedup_factor:.2}x real-time)"
        );

        // Try different audio players in order of preference
        let players = ["play", "aplay", "paplay", "afplay"];
        
        for player in &players {
            if let Ok(mut child) = Command::new(player)
                .arg("-t")
                .arg("wav")
                .arg("-")
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    // Calculate data size (4 bytes per sample for 32-bit float)
                    let data_size = (audio.len() * 4) as u32;

                    // Write WAV header
                    let header = crate::utils::wav::WavHeader::new(1, TTSKoko::SAMPLE_RATE, 32);
                    if let Err(_) = header.write_header(&mut stdin, data_size) {
                        continue; // Try next player
                    }

                    // Write audio data
                    if let Err(_) = crate::utils::wav::write_audio_chunk(&mut stdin, &audio) {
                        continue; // Try next player
                    }

                    drop(stdin); // Close stdin to signal end of input
                    
                    // Wait for player to finish
                    if let Ok(status) = child.wait() {
                        if status.success() {
                            println!("Audio played successfully with {}", player);
                            return Ok(());
                        }
                    }
                }
            }
        }

        // Fallback: save to temp file and try to open it
        println!("No compatible audio player found, saving to temp file...");
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("kokoro_temp.wav");
        
        // Save audio to temp file
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: TTSKoko::SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        let mut writer = hound::WavWriter::create(&temp_file, spec)?;
        for &sample in &audio {
            writer.write_sample(sample)?;
        }
        writer.finalize()?;

        println!("Audio saved to: {}", temp_file.display());
        println!("You can play it manually with: play {}", temp_file.display());

        Ok(())
    }

    fn load_voices(voices_path: &str) -> HashMap<String, Vec<[[f32; 256]; 1]>> {
        let mut npz = NpzReader::new(File::open(voices_path).unwrap()).unwrap();
        let mut map = HashMap::new();

        for voice in npz.names().unwrap() {
            let voice_data: Result<Array3<f32>, _> = npz.by_name(&voice);
            let voice_data = voice_data.unwrap();
            let mut tensor = vec![[[0.0; 256]; 1]; 511];
            for (i, inner_value) in voice_data.outer_iter().enumerate() {
                for (j, inner_inner_value) in inner_value.outer_iter().enumerate() {
                    for (k, number) in inner_inner_value.iter().enumerate() {
                        tensor[i][j][k] = *number;
                    }
                }
            }
            map.insert(voice, tensor);
        }

        let sorted_voices = {
            let mut voices = map.keys().collect::<Vec<_>>();
            voices.sort();
            voices
        };

        eprintln!("voice styles loaded: {sorted_voices:?}");
        map
    }
}
