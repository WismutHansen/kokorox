/// Simple fallback phonemizer that doesn't require external dependencies
/// This provides basic phonemization when DeepPhonemizer is not available
use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    // Basic grapheme-to-phoneme mapping for English
    static ref SIMPLE_G2P: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        
        // Common English words and their IPA transcriptions
        m.insert("hello", "həˈloʊ");
        m.insert("world", "wɜːrld");
        m.insert("can", "kæn");
        m.insert("you", "juː");
        m.insert("add", "æd");
        m.insert("cheese", "tʃiːz");
        m.insert("to", "tuː");
        m.insert("my", "maɪ");
        m.insert("shopping", "ˈʃɑːpɪŋ");
        m.insert("list", "lɪst");
        m.insert("the", "ðə");
        m.insert("a", "ə");
        m.insert("an", "æn");
        m.insert("and", "ænd");
        m.insert("or", "ɔːr");
        m.insert("but", "bʌt");
        m.insert("for", "fɔːr");
        m.insert("with", "wɪð");
        m.insert("from", "frʌm");
        m.insert("into", "ˈɪntuː");
        m.insert("on", "ɑːn");
        m.insert("at", "æt");
        m.insert("by", "baɪ");
        m.insert("up", "ʌp");
        m.insert("out", "aʊt");
        m.insert("down", "daʊn");
        m.insert("over", "ˈoʊvər");
        m.insert("under", "ˈʌndər");
        m.insert("through", "θruː");
        m.insert("between", "bɪˈtwiːn");
        m.insert("among", "əˈmʌŋ");
        m.insert("above", "əˈbʌv");
        m.insert("below", "bɪˈloʊ");
        m.insert("inside", "ɪnˈsaɪd");
        m.insert("outside", "ˌaʊtˈsaɪd");
        
        m
    };
    
    // Basic character-level fallback for unknown words
    static ref CHAR_TO_PHONEME: HashMap<char, &'static str> = {
        let mut m = HashMap::new();
        
        m.insert('a', "æ");
        m.insert('b', "b");
        m.insert('c', "k");
        m.insert('d', "d");
        m.insert('e', "ɛ");
        m.insert('f', "f");
        m.insert('g', "g");
        m.insert('h', "h");
        m.insert('i', "ɪ");
        m.insert('j', "dʒ");
        m.insert('k', "k");
        m.insert('l', "l");
        m.insert('m', "m");
        m.insert('n', "n");
        m.insert('o', "ɑ");
        m.insert('p', "p");
        m.insert('q', "kw");
        m.insert('r', "r");
        m.insert('s', "s");
        m.insert('t', "t");
        m.insert('u', "ʌ");
        m.insert('v', "v");
        m.insert('w', "w");
        m.insert('x', "ks");
        m.insert('y', "j");
        m.insert('z', "z");
        
        m
    };
}

pub fn simple_phonemize(text: &str, _language: &str) -> String {
    let text = text.to_lowercase();
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut phonemes = Vec::new();
    
    for word in words {
        // Remove punctuation
        let clean_word = word.trim_matches(|c: char| !c.is_alphabetic());
        
        if clean_word.is_empty() {
            continue;
        }
        
        // Try exact word lookup first
        if let Some(&phoneme) = SIMPLE_G2P.get(clean_word) {
            phonemes.push(phoneme.to_string());
        } else {
            // Fallback to character-by-character mapping
            let mut word_phonemes = String::new();
            for ch in clean_word.chars() {
                if let Some(&phoneme) = CHAR_TO_PHONEME.get(&ch) {
                    word_phonemes.push_str(phoneme);
                } else {
                    // For unknown characters, just include them as-is
                    word_phonemes.push(ch);
                }
            }
            phonemes.push(word_phonemes);
        }
    }
    
    phonemes.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_phonemization() {
        assert_eq!(
            simple_phonemize("hello world", "en"),
            "həˈloʊ wɜːrld"
        );
        
        assert_eq!(
            simple_phonemize("can you add cheese", "en"),
            "kæn juː æd tʃiːz"
        );
    }
}