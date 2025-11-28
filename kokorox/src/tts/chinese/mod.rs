//! Chinese G2P (Grapheme-to-Phoneme) module for Kokoro TTS
//!
//! This module provides native Rust implementation of Chinese text to phoneme conversion,
//! adapted from misaki's ZHG2P for compatibility with Kokoro v1.1-zh model.

mod tone_sandhi;
mod transcription;

use jieba_rs::Jieba;
use lazy_static::lazy_static;
use pinyin::ToPinyin;
use regex::Regex;
use std::collections::HashSet;
use std::sync::Mutex;

pub use transcription::ZH_MAP;

lazy_static! {
    /// Global Jieba instance for word segmentation
    static ref JIEBA: Mutex<Jieba> = Mutex::new(Jieba::new());
    
    /// Chinese character range regex
    static ref ZH_CHAR_PATTERN: Regex = Regex::new(r"[\u4E00-\u9FFF]").unwrap();
    
    /// Chinese text segment pattern
    static ref ZH_SEGMENT_PATTERN: Regex = Regex::new(r"[\u4E00-\u9FFF]+|[^\u4E00-\u9FFF]+").unwrap();
    
    /// Punctuation mapping from Chinese to ASCII
    static ref PUNCTUATION_MAP: Vec<(&'static str, &'static str)> = vec![
        ("\u{3001}", ", "),  // 、
        ("\u{FF0C}", ", "),  // ，
        ("\u{3002}", ". "),  // 。
        ("\u{FF0E}", ". "),  // ．
        ("\u{FF01}", "! "),  // ！
        ("\u{FF1A}", ": "),  // ：
        ("\u{FF1B}", "; "),  // ；
        ("\u{FF1F}", "? "),  // ？
        ("\u{00AB}", " \""), // «
        ("\u{00BB}", "\" "), // »
        ("\u{300A}", " \""), // 《
        ("\u{300B}", "\" "), // 》
        ("\u{300C}", " \""), // 「
        ("\u{300D}", "\" "), // 」
        ("\u{3010}", " \""), // 【
        ("\u{3011}", "\" "), // 】
        ("\u{FF08}", " ("),  // （
        ("\u{FF09}", ") "),  // ）
    ];
    
    /// Words that must have neutral tone on last syllable
    static ref MUST_ERHUA: HashSet<&'static str> = {
        let mut s = HashSet::new();
        for word in &[
            "小院儿", "胡同儿", "范儿", "老汉儿", "撒欢儿", 
            "寻老礼儿", "妥妥儿", "媳妇儿"
        ] {
            s.insert(*word);
        }
        s
    };
    
    /// Words that should NOT have erhua applied
    static ref NOT_ERHUA: HashSet<&'static str> = {
        let mut s = HashSet::new();
        for word in &[
            "虐儿", "为儿", "护儿", "瞒儿", "救儿", "替儿", "有儿", "一儿", 
            "我儿", "俺儿", "妻儿", "拐儿", "聋儿", "乞儿", "患儿", "幼儿", 
            "孤儿", "婴儿", "婴幼儿", "连体儿", "脑瘫儿", "流浪儿", "体弱儿", 
            "混血儿", "蜜雪儿", "舫儿", "祖儿", "美儿", "应采儿", "可儿", 
            "侄儿", "孙儿", "侄孙儿", "女儿", "男儿", "红孩儿", "花儿", 
            "虫儿", "马儿", "鸟儿", "猪儿", "猫儿", "狗儿", "少儿"
        ] {
            s.insert(*word);
        }
        s
    };
}

/// Chinese G2P processor
pub struct ChineseG2P {
    /// Whether to use v1.1 frontend with Bopomofo output
    use_bopomofo: bool,
    /// Unknown character replacement
    unk: char,
}

impl Default for ChineseG2P {
    fn default() -> Self {
        Self::new()
    }
}

impl ChineseG2P {
    /// Create a new Chinese G2P processor
    pub fn new() -> Self {
        Self {
            use_bopomofo: true,
            unk: '?',
        }
    }
    
    /// Create with IPA output (legacy mode)
    pub fn with_ipa() -> Self {
        Self {
            use_bopomofo: false,
            unk: '?',
        }
    }
    
    /// Map Chinese punctuation to ASCII equivalents
    fn map_punctuation(text: &str) -> String {
        let mut result = text.to_string();
        for (from, to) in PUNCTUATION_MAP.iter() {
            result = result.replace(from, to);
        }
        result.trim().to_string()
    }
    
    /// Convert Arabic numbers to Chinese
    fn convert_numbers(text: &str) -> String {
        use chinese_number::{ChineseCase, ChineseCountMethod, ChineseVariant, NumberToChinese};
        
        let mut result = String::new();
        let mut num_buffer = String::new();
        
        for ch in text.chars() {
            if ch.is_ascii_digit() {
                num_buffer.push(ch);
            } else {
                if !num_buffer.is_empty() {
                    // Convert accumulated number
                    if let Ok(num) = num_buffer.parse::<i64>() {
                        if let Ok(chinese) = num.to_chinese(
                            ChineseVariant::Simple,
                            ChineseCase::Lower,
                            ChineseCountMethod::Low,
                        ) {
                            result.push_str(&chinese);
                        } else {
                            result.push_str(&num_buffer);
                        }
                    } else {
                        result.push_str(&num_buffer);
                    }
                    num_buffer.clear();
                }
                result.push(ch);
            }
        }
        
        // Handle trailing number
        if !num_buffer.is_empty() {
            if let Ok(num) = num_buffer.parse::<i64>() {
                if let Ok(chinese) = num.to_chinese(
                    ChineseVariant::Simple,
                    ChineseCase::Lower,
                    ChineseCountMethod::Low,
                ) {
                    result.push_str(&chinese);
                } else {
                    result.push_str(&num_buffer);
                }
            } else {
                result.push_str(&num_buffer);
            }
        }
        
        result
    }
    
    /// Segment Chinese text into words using jieba
    fn segment(text: &str) -> Vec<String> {
        let jieba = JIEBA.lock().unwrap();
        jieba.cut(text, false)
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }
    
    /// Segment with POS tags
    fn segment_with_pos(text: &str) -> Vec<(String, String)> {
        let jieba = JIEBA.lock().unwrap();
        jieba.tag(text, false)
            .into_iter()
            .map(|t| (t.word.to_string(), t.tag.to_string()))
            .collect()
    }
    
    /// Convert a single word to pinyin with tones
    fn word_to_pinyin(word: &str) -> Vec<String> {
        word.chars()
            .filter_map(|c| {
                c.to_pinyin().map(|p| {
                    // Get pinyin with tone number at end (e.g., "zhong1")
                    p.with_tone_num_end().to_string()
                })
            })
            .collect()
    }
    
    /// Convert a word to Bopomofo (Zhuyin) representation
    fn word_to_bopomofo(&self, word: &str, pos: &str) -> String {
        let pinyins = Self::word_to_pinyin(word);
        
        // Apply tone sandhi
        let modified_pinyins = tone_sandhi::apply_tone_sandhi(word, pos, &pinyins);
        
        // Convert each pinyin to bopomofo
        modified_pinyins
            .iter()
            .map(|py| transcription::pinyin_to_bopomofo(py))
            .collect::<Vec<_>>()
            .join("")
    }
    
    /// Convert a word to IPA representation (legacy mode)
    fn word_to_ipa(&self, word: &str) -> String {
        let pinyins = Self::word_to_pinyin(word);
        
        pinyins
            .iter()
            .map(|py| transcription::pinyin_to_ipa(py))
            .collect::<Vec<_>>()
            .join("")
    }
    
    /// Check if a character is Chinese
    fn is_chinese_char(c: char) -> bool {
        ('\u{4E00}'..='\u{9FFF}').contains(&c)
    }
    
    /// Process Chinese text to phonemes
    pub fn process(&self, text: &str) -> String {
        if text.trim().is_empty() {
            return String::new();
        }
        
        // Convert numbers to Chinese
        let text = Self::convert_numbers(text);
        
        // Map punctuation
        let text = Self::map_punctuation(&text);
        
        if self.use_bopomofo {
            self.process_bopomofo(&text)
        } else {
            self.process_ipa(&text)
        }
    }
    
    /// Process with Bopomofo output (v1.1 mode)
    fn process_bopomofo(&self, text: &str) -> String {
        let mut result = String::new();
        let mut prev_was_chinese = false;
        
        // Process segments (alternating Chinese and non-Chinese)
        for cap in ZH_SEGMENT_PATTERN.find_iter(text) {
            let segment = cap.as_str();
            let first_char = segment.chars().next().unwrap_or(' ');
            let is_chinese = Self::is_chinese_char(first_char);
            
            if is_chinese {
                // Segment and convert Chinese text
                let words_with_pos = Self::segment_with_pos(segment);
                
                // Apply pre-merge for tone sandhi
                let merged = tone_sandhi::pre_merge_for_modify(&words_with_pos);
                
                for (i, (word, pos)) in merged.iter().enumerate() {
                    // Add separator between words (but not before first)
                    if i > 0 || prev_was_chinese {
                        result.push('/');
                    }
                    
                    // Check if it's punctuation
                    if pos == "x" {
                        result.push_str(word);
                    } else {
                        result.push_str(&self.word_to_bopomofo(word, pos));
                    }
                }
                prev_was_chinese = true;
            } else {
                // Non-Chinese segment - keep as is (or process with English G2P later)
                result.push_str(segment);
                prev_was_chinese = false;
            }
        }
        
        result
    }
    
    /// Process with IPA output (legacy mode)
    fn process_ipa(&self, text: &str) -> String {
        let mut result = String::new();
        
        for cap in ZH_SEGMENT_PATTERN.find_iter(text) {
            let segment = cap.as_str();
            let first_char = segment.chars().next().unwrap_or(' ');
            let is_chinese = Self::is_chinese_char(first_char);
            
            if is_chinese {
                let words = Self::segment(segment);
                let ipa_parts: Vec<String> = words
                    .iter()
                    .map(|w| self.word_to_ipa(w))
                    .collect();
                result.push_str(&ipa_parts.join(" "));
            } else {
                result.push_str(segment);
            }
        }
        
        // Clean up some IPA artifacts
        result.replace('\u{032F}', "") // combining inverted breve below
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_punctuation_mapping() {
        let text = "你好，世界！";
        let mapped = ChineseG2P::map_punctuation(text);
        println!("Mapped: '{}'", mapped);
        assert!(mapped.contains(", "), "Should contain comma+space");
        assert!(mapped.contains("!"), "Should contain exclamation mark");
    }
    
    #[test]
    fn test_number_conversion() {
        let text = "我有123个苹果";
        let converted = ChineseG2P::convert_numbers(text);
        assert!(converted.contains("一百二十三"));
    }
    
    #[test]
    fn test_segmentation() {
        let text = "我爱北京天安门";
        let words = ChineseG2P::segment(text);
        assert!(!words.is_empty());
        println!("Segments: {:?}", words);
    }
    
    #[test]
    fn test_word_to_pinyin() {
        let word = "中国";
        let pinyins = ChineseG2P::word_to_pinyin(word);
        assert_eq!(pinyins.len(), 2);
        println!("Pinyin: {:?}", pinyins);
    }
    
    #[test]
    fn test_basic_g2p() {
        let g2p = ChineseG2P::new();
        let result = g2p.process("你好");
        assert!(!result.is_empty());
        println!("G2P result: {}", result);
    }
    
    #[test]
    fn test_full_sentence_g2p() {
        let g2p = ChineseG2P::new();
        
        // Test various Chinese sentences
        let sentences = [
            ("你好世界", "Hello world"),
            ("中国人民", "Chinese people"),
            ("我爱北京天安门", "I love Beijing Tiananmen"),
            ("今天天气很好", "The weather is good today"),
            ("一二三四五", "One two three four five"),
        ];
        
        for (chinese, description) in sentences {
            let result = g2p.process(chinese);
            println!("{} ({}): {} -> {}", chinese, description, chinese, result);
            assert!(!result.is_empty(), "G2P should produce output for: {}", chinese);
            // Bopomofo output should contain tone numbers
            assert!(result.chars().any(|c| c.is_ascii_digit()), 
                "Output should contain tone numbers for: {}", chinese);
        }
    }
    
    #[test]
    fn test_tone_sandhi_integration() {
        let g2p = ChineseG2P::new();
        
        // Test tone sandhi cases
        let test_cases = [
            "不是",    // bu sandhi
            "一个",    // yi sandhi  
            "你好",    // third tone sandhi
        ];
        
        for text in test_cases {
            let result = g2p.process(text);
            println!("Tone sandhi test: {} -> {}", text, result);
            assert!(!result.is_empty());
        }
    }
}
