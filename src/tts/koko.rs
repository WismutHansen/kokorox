use crate::tts::tokenize::tokenize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use crate::onn::ort_koko::{self};
use crate::utils;
use crate::utils::fileio::load_json_file;

use espeak_rs::text_to_phonemes;

#[derive(Clone)]
pub struct TTSKoko {
    #[allow(dead_code)]
    model_path: String,
    model: Arc<ort_koko::OrtKoko>,
    styles: HashMap<String, [[[f32; 256]; 1]; 511]>,
}

impl TTSKoko {
    const MODEL_URL: &str =
        "https://huggingface.co/hexgrad/Kokoro-82M/resolve/main/kokoro-v0_19.onnx";
    const JSON_DATA_F: &str = "data/voices.json";

    pub const SAMPLE_RATE: u32 = 24000;

    pub fn new(model_path: &str) -> Self {
        let p = Path::new(model_path);
        if !p.exists() {
            utils::fileio::download_file_from_url(TTSKoko::MODEL_URL, model_path)
                .expect("download model failed.");
        } else {
            eprintln!("load model from: {}", model_path);
        }

        let model = Arc::new(
            ort_koko::OrtKoko::new(model_path.to_string())
                .expect("Failed to create Kokoro TTS model"),
        );

        // TODO: if(not streaming) { model.print_info(); }
        // model.print_info();

        let mut instance = TTSKoko {
            model_path: model_path.to_string(),
            model,
            styles: HashMap::new(),
        };
        instance.load_voices();
        instance
    }

    pub fn tts_raw_audio(
        &self,
        txt: &str,
        lan: &str,
        style_name: &str,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let phonemes = text_to_phonemes(txt, lan, None, true, false)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
            .join("");

        let tokens = vec![tokenize(&phonemes)];

        if let Ok(styles) = self.mix_styles(style_name) {
            let start_t = Instant::now();
            let result = self.model.infer(tokens, styles);
            match result {
                Ok(out) => {
                    let audio: Vec<f32> = out.iter().cloned().collect();
                    Ok(audio)
                }
                Err(e) => {
                    eprintln!("An error occurred during inference: {:?}", e);
                    Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Inference failed",
                    )))
                }
            }
        } else {
            Err(format!("{} failed to parse this style_name.", style_name).into())
        }
    }

    pub fn tts(
        &self,
        txt: &str,
        lan: &str,
        style_name: &str,
        save_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let audio = self.tts_raw_audio(txt, lan, style_name)?;

        // Save to file
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: TTSKoko::SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        let mut writer = hound::WavWriter::create(save_path, spec)?;
        for &sample in &audio {
            writer.write_sample(sample)?;
        }
        writer.finalize()?;

        eprintln!("Audio saved to {}", save_path);
        Ok(())
    }

    pub fn mix_styles(
        &self,
        style_name: &str,
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        if !style_name.contains("+") {
            if let Some(style) = self.styles.get(style_name) {
                let styles = vec![style[0][0].to_vec()];
                Ok(styles)
            } else {
                Err(format!("can not found from styles_map: {}", style_name).into())
            }
        } else {
            eprintln!("parsing style mix");
            let styles: Vec<&str> = style_name.split('+').collect();

            let mut style_names = Vec::new();
            let mut style_portions = Vec::new();

            for style in styles {
                if let Some((name, portion)) = style.split_once('.') {
                    if let Ok(portion) = portion.parse::<f32>() {
                        style_names.push(name);
                        style_portions.push(portion * 0.1);
                    }
                }
            }
            eprintln!("styles: {:?}, portions: {:?}", style_names, style_portions);

            let mut blended_style = vec![vec![0.0; 256]; 1];

            for (name, portion) in style_names.iter().zip(style_portions.iter()) {
                if let Some(style) = self.styles.get(*name) {
                    let style_slice = &style[0][0]; // This is a [256] array
                                                    // Blend into the blended_style
                    for j in 0..256 {
                        blended_style[0][j] += style_slice[j] * portion;
                    }
                }
            }
            Ok(blended_style)
        }
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

            eprintln!("voice styles loaded: {}", self.styles.len());
            let mut keys: Vec<_> = self.styles.keys().cloned().collect();
            keys.sort();
            eprintln!("{:?}", keys);
            eprintln!(
                "{:?} {:?}",
                self.styles.keys().next(),
                self.styles.keys().nth(1)
            );
        }
    }
}
