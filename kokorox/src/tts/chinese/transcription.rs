//! Pinyin to IPA/Bopomofo transcription mappings
//!
//! Adapted from misaki's transcription.py and zh_frontend.py

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// Pinyin initial consonants to IPA mapping
    static ref INITIAL_TO_IPA: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("b", "p");
        m.insert("p", "pʰ");
        m.insert("m", "m");
        m.insert("f", "f");
        m.insert("d", "t");
        m.insert("t", "tʰ");
        m.insert("n", "n");
        m.insert("l", "l");
        m.insert("g", "k");
        m.insert("k", "kʰ");
        m.insert("h", "x");
        m.insert("j", "ʨ");
        m.insert("q", "ʨʰ");
        m.insert("x", "ɕ");
        m.insert("zh", "ʈʂ");
        m.insert("ch", "ʈʂʰ");
        m.insert("sh", "ʂ");
        m.insert("r", "ɻ");
        m.insert("z", "ts");
        m.insert("c", "tsʰ");
        m.insert("s", "s");
        m
    };

    /// Pinyin finals to IPA mapping
    static ref FINAL_TO_IPA: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("a", "a");
        m.insert("ai", "ai̯");
        m.insert("an", "an");
        m.insert("ang", "aŋ");
        m.insert("ao", "au̯");
        m.insert("e", "ɤ");
        m.insert("ei", "ei̯");
        m.insert("en", "ən");
        m.insert("eng", "əŋ");
        m.insert("er", "ɚ");
        m.insert("i", "i");
        m.insert("ia", "ja");
        m.insert("ian", "jɛn");
        m.insert("iang", "jaŋ");
        m.insert("iao", "jau̯");
        m.insert("ie", "je");
        m.insert("in", "in");
        m.insert("ing", "iŋ");
        m.insert("iong", "jʊŋ");
        m.insert("iu", "jou̯");
        m.insert("iou", "jou̯");
        m.insert("o", "wo");
        m.insert("ong", "ʊŋ");
        m.insert("ou", "ou̯");
        m.insert("u", "u");
        m.insert("ua", "wa");
        m.insert("uai", "wai̯");
        m.insert("uan", "wan");
        m.insert("uang", "waŋ");
        m.insert("ue", "ɥe");
        m.insert("uei", "wei̯");
        m.insert("ui", "wei̯");
        m.insert("un", "wən");
        m.insert("uen", "wən");
        m.insert("ueng", "wəŋ");
        m.insert("uo", "wo");
        m.insert("v", "y");
        m.insert("ve", "ɥe");
        m.insert("van", "ɥɛn");
        m.insert("vn", "yn");
        // Special finals for zh/ch/sh/r + i
        m.insert("ii", "ɻ̩");
        // Special finals for z/c/s + i  
        m.insert("iii", "ɹ̩");
        m
    };

    /// Tone number to IPA tone marker
    static ref TONE_TO_IPA: HashMap<u8, &'static str> = {
        let mut m = HashMap::new();
        m.insert(1, "˥");      // First tone (high level)
        m.insert(2, "˧˥");     // Second tone (rising)
        m.insert(3, "˧˩˧");    // Third tone (dipping)
        m.insert(4, "˥˩");     // Fourth tone (falling)
        m.insert(5, "");       // Fifth tone (neutral)
        m
    };

    /// Pinyin to Bopomofo (Zhuyin) mapping
    /// This is the ZH_MAP from misaki's zh_frontend.py
    pub static ref ZH_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // Initials
        m.insert("b", "ㄅ");
        m.insert("p", "ㄆ");
        m.insert("m", "ㄇ");
        m.insert("f", "ㄈ");
        m.insert("d", "ㄉ");
        m.insert("t", "ㄊ");
        m.insert("n", "ㄋ");
        m.insert("l", "ㄌ");
        m.insert("g", "ㄍ");
        m.insert("k", "ㄎ");
        m.insert("h", "ㄏ");
        m.insert("j", "ㄐ");
        m.insert("q", "ㄑ");
        m.insert("x", "ㄒ");
        m.insert("zh", "ㄓ");
        m.insert("ch", "ㄔ");
        m.insert("sh", "ㄕ");
        m.insert("r", "ㄖ");
        m.insert("z", "ㄗ");
        m.insert("c", "ㄘ");
        m.insert("s", "ㄙ");
        // Finals
        m.insert("a", "ㄚ");
        m.insert("o", "ㄛ");
        m.insert("e", "ㄜ");
        m.insert("ie", "ㄝ");
        m.insert("ai", "ㄞ");
        m.insert("ei", "ㄟ");
        m.insert("ao", "ㄠ");
        m.insert("ou", "ㄡ");
        m.insert("an", "ㄢ");
        m.insert("en", "ㄣ");
        m.insert("ang", "ㄤ");
        m.insert("eng", "ㄥ");
        m.insert("er", "ㄦ");
        m.insert("i", "ㄧ");
        m.insert("u", "ㄨ");
        m.insert("v", "ㄩ");
        // Special finals
        m.insert("ii", "ㄭ");      // zi, ci, si
        m.insert("iii", "十");     // zhi, chi, shi, ri
        m.insert("ve", "月");
        m.insert("ia", "压");
        m.insert("ian", "言");
        m.insert("iang", "阳");
        m.insert("iao", "要");
        m.insert("in", "阴");
        m.insert("ing", "应");
        m.insert("iong", "用");
        m.insert("iou", "又");
        m.insert("ong", "中");
        m.insert("ua", "穵");
        m.insert("uai", "外");
        m.insert("uan", "万");
        m.insert("uang", "王");
        m.insert("uei", "为");
        m.insert("uen", "文");
        m.insert("ueng", "瓮");
        m.insert("uo", "我");
        m.insert("van", "元");
        m.insert("vn", "云");
        // Tones (as single characters)
        m.insert("1", "1");
        m.insert("2", "2");
        m.insert("3", "3");
        m.insert("4", "4");
        m.insert("5", "5");
        // Punctuation passthrough
        m.insert(";", ";");
        m.insert(":", ":");
        m.insert(",", ",");
        m.insert(".", ".");
        m.insert("!", "!");
        m.insert("?", "?");
        m.insert("/", "/");
        m.insert(" ", " ");
        m
    };

    /// List of valid initials
    static ref INITIALS: Vec<&'static str> = vec![
        "zh", "ch", "sh",  // Two-char initials first for matching
        "b", "p", "m", "f", "d", "t", "n", "l", 
        "g", "k", "h", "j", "q", "x", "r", "z", "c", "s",
        "y", "w"
    ];
}

/// Parse a pinyin syllable into (initial, final, tone)
fn parse_pinyin(pinyin: &str) -> (Option<&str>, String, u8) {
    let pinyin = pinyin.to_lowercase();
    
    // Extract tone number from end
    let (base, tone) = if let Some(last) = pinyin.chars().last() {
        if last.is_ascii_digit() {
            let tone = last.to_digit(10).unwrap_or(5) as u8;
            (&pinyin[..pinyin.len()-1], tone)
        } else {
            (pinyin.as_str(), 5u8)
        }
    } else {
        return (None, String::new(), 5);
    };
    
    // Find initial
    let mut initial: Option<&str> = None;
    let mut final_start = 0;
    
    for init in INITIALS.iter() {
        if base.starts_with(init) {
            initial = Some(init);
            final_start = init.len();
            break;
        }
    }
    
    let final_part = &base[final_start..];
    
    // Handle special cases for finals
    let final_part = match (initial, final_part) {
        // zi, ci, si -> use "ii" final
        (Some("z"), "i") | (Some("c"), "i") | (Some("s"), "i") => "ii",
        // zhi, chi, shi, ri -> use "iii" final
        (Some("zh"), "i") | (Some("ch"), "i") | (Some("sh"), "i") | (Some("r"), "i") => "iii",
        // iu -> iou
        (_, "iu") => "iou",
        // ui -> uei
        (_, "ui") => "uei",
        // un -> uen
        (_, "un") => "uen",
        // Handle u after j, q, x, y -> v
        (Some("j"), f) | (Some("q"), f) | (Some("x"), f) | (Some("y"), f) 
            if f.starts_with('u') && !f.starts_with("ua") && !f.starts_with("uo") => {
            // Replace u with v
            &f.replacen('u', "v", 1).leak()
        },
        _ => final_part,
    };
    
    (initial, final_part.to_string(), tone)
}

/// Convert pinyin to IPA
pub fn pinyin_to_ipa(pinyin: &str) -> String {
    let (initial, final_part, tone) = parse_pinyin(pinyin);
    
    let mut result = String::new();
    
    // Add initial IPA
    if let Some(init) = initial {
        if let Some(ipa) = INITIAL_TO_IPA.get(init) {
            result.push_str(ipa);
        }
    }
    
    // Add final IPA
    if let Some(ipa) = FINAL_TO_IPA.get(final_part.as_str()) {
        result.push_str(ipa);
    } else {
        // Fallback: try to build from parts
        result.push_str(&final_part);
    }
    
    // Add tone marker
    if let Some(tone_marker) = TONE_TO_IPA.get(&tone) {
        result.push_str(tone_marker);
    }
    
    result
}

/// Convert pinyin to Bopomofo (Zhuyin)
pub fn pinyin_to_bopomofo(pinyin: &str) -> String {
    let (initial, final_part, tone) = parse_pinyin(pinyin);
    
    let mut result = String::new();
    
    // Add initial Bopomofo
    if let Some(init) = initial {
        if init != "y" && init != "w" {  // y and w are not real initials in Bopomofo
            if let Some(bpmf) = ZH_MAP.get(init) {
                result.push_str(bpmf);
            }
        }
    }
    
    // Add final Bopomofo
    if let Some(bpmf) = ZH_MAP.get(final_part.as_str()) {
        result.push_str(bpmf);
    } else {
        // Try to decompose the final
        // Handle common patterns
        let bpmf = match final_part.as_str() {
            "iu" | "iou" => "又",
            "ui" | "uei" => "为",
            "un" | "uen" => "文",
            f => {
                // Build character by character as fallback
                let mut s = String::new();
                for c in f.chars() {
                    let cs = c.to_string();
                    if let Some(b) = ZH_MAP.get(cs.as_str()) {
                        s.push_str(b);
                    }
                }
                return result + &s + &tone.to_string();
            }
        };
        result.push_str(bpmf);
    }
    
    // Add tone number
    result.push_str(&tone.to_string());
    
    result
}

/// Retone IPA output (convert tone markers to arrows for v1.1 compatibility)
pub fn retone_ipa(ipa: &str) -> String {
    ipa.replace("˧˩˧", "↓")   // third tone
       .replace("˧˥", "↗")    // second tone  
       .replace("˥˩", "↘")    // fourth tone
       .replace("˥", "→")     // first tone
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_pinyin() {
        let (init, fin, tone) = parse_pinyin("zhong1");
        assert_eq!(init, Some("zh"));
        assert_eq!(fin, "ong");
        assert_eq!(tone, 1);
        
        let (init, fin, tone) = parse_pinyin("guo2");
        assert_eq!(init, Some("g"));
        assert_eq!(fin, "uo");
        assert_eq!(tone, 2);
    }
    
    #[test]
    fn test_pinyin_to_bopomofo() {
        let result = pinyin_to_bopomofo("zhong1");
        println!("zhong1 -> {}", result);
        assert!(result.contains("ㄓ"));
        assert!(result.contains("中"));
        
        let result = pinyin_to_bopomofo("ni3");
        println!("ni3 -> {}", result);
        assert!(result.contains("ㄋ"));
    }
    
    #[test]
    fn test_pinyin_to_ipa() {
        let result = pinyin_to_ipa("ma1");
        println!("ma1 -> {}", result);
        assert!(result.contains("m"));
        assert!(result.contains("a"));
        assert!(result.contains("˥"));
    }
}
