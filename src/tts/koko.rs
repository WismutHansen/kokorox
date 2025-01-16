use crate::tts::tokenize::tokenize;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use std::sync::Arc;

use ndarray::{ArrayBase, IxDyn, OwnedRepr};

use crate::onn::ort_base::OrtBase;
use crate::onn::ort_koko::{self};
use crate::utils;
use crate::utils::fileio::load_json_file;

use espeak_rs::text_to_phonemes;

#[derive(Clone)]
pub struct TTSKoko {
    model_path: String,
    model: Arc<ort_koko::OrtKoko>,
    styles: HashMap<String, [[[f32; 256]; 1]; 511]>,
}

impl TTSKoko {
    const MODEL_URL: &str =
        "https://huggingface.co/hexgrad/Kokoro-82M/resolve/main/kokoro-v0_19.onnx";
    const JSON_DATA_F: &str = "data/voices.json";

    const SAMPLE_RATE: u32 = 24000;

    pub fn new(model_path: &str) -> Self {
        let p = Path::new(model_path);
        if !p.exists() {
            utils::fileio::download_file_from_url(TTSKoko::MODEL_URL, model_path)
                .expect("download model failed.");
        } else {
            println!("load model from: {}", model_path);
        }

        let model = Arc::new(
            ort_koko::OrtKoko::new(model_path.to_string())
                .expect("Failed to create Kokoro TTS model")
        );

        model.print_info();

        let mut instance = TTSKoko {
            model_path: model_path.to_string(),
            model,
            styles: HashMap::new(),
        };
        instance.load_voices();
        instance
    }

    pub fn tts(&self, txt: &str, language: &str, style_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("hello, going to tts. text: {}", txt);

        let phonemes = text_to_phonemes(txt, language, None, true, false)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
            .join("");

        let tokens = vec![tokenize(&phonemes)];

        if let Some(style) = self.styles.get(style_name) {
            let styles = vec![style[0][0].to_vec()];

            let start_t = Instant::now();

            let out = self.model.infer(tokens, styles)?;
            println!("output: {:?}", out);

            let phonemes_len = phonemes.len();
            self.process_and_save_audio(start_t, out, phonemes_len)?;
            Ok(())
        } else {
            Err(format!("{} not found, choose one from data/voices.json style key.", style_name).into())
        }
    }

    fn process_and_save_audio(
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
            "Created audio in length of {:.2}s for {} phonemes in {:.2}s ({:.2}x real-time)",
            audio_duration, phonemes_len, create_duration, speedup_factor
        );

        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: TTSKoko::SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        let mut writer = hound::WavWriter::create("tmp/output.wav", spec)?;

        for &sample in &audio {
            writer.write_sample(sample)?;
        }

        writer.finalize()?;

        println!("Audio saved to tmp/output.wav");
        Ok(())
    }

    pub fn load_voices(&mut self) {
        // load from json, get styles
        let values = load_json_file(TTSKoko::JSON_DATA_F);
        if let Ok(values) = values {
            if let Some(obj) = values.as_object() {
                for (key, value) in obj {
                    // Check if value is an array
                    if let Some(outer_array) = value.as_array() {
                        // Define target multidimensional array
                        let mut array_3d = [[[0.0; 256]; 1]; 511];

                        // Iterate through outer array (511 elements)
                        for (i, inner_value) in outer_array.iter().enumerate() {
                            if let Some(middle_array) = inner_value.as_array() {
                                // Iterate through middle array (1 element)
                                for (j, inner_inner_value) in middle_array.iter().enumerate() {
                                    if let Some(inner_array) = inner_inner_value.as_array() {
                                        // Iterate through inner array (256 elements)
                                        for (k, number) in inner_array.iter().enumerate() {
                                            if let Some(num) = number.as_f64() {
                                                array_3d[i][j][k] = num as f32;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Insert multidimensional array into HashMap
                        self.styles.insert(key.clone(), array_3d);
                    }
                }
            }

            println!("voice styles loaded: {}", self.styles.len());
            println!("{:?}", self.styles.keys());
            println!(
                "{:?} {:?}",
                self.styles.keys().next(),
                self.styles.keys().nth(1)
            );
        }
    }
}
