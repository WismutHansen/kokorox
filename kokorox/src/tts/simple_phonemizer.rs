use lazy_static::lazy_static;
/// Simple fallback phonemizer that doesn't require external dependencies
/// This provides basic phonemization when DeepPhonemizer is not available
use std::collections::HashMap;

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

        // Common Spanish words and their IPA transcriptions
        m.insert("hola", "ˈola");
        m.insert("cómo", "ˈkomo");
        m.insert("estás", "esˈtas");
        m.insert("está", "esˈta");
        m.insert("hoy", "ˈoj");
        m.insert("qué", "ˈke");
        m.insert("dónde", "ˈdonde");
        m.insert("cuándo", "ˈkwando");
        m.insert("por", "ˈpor");
        m.insert("para", "ˈpara");
        m.insert("con", "ˈkon");
        m.insert("más", "ˈmas");
        m.insert("muy", "ˈmuj");
        m.insert("bien", "ˈbjen");
        m.insert("sí", "ˈsi");
        m.insert("no", "ˈno");
        m.insert("aquí", "aˈki");
        m.insert("allí", "aˈʎi");
        m.insert("ahora", "aˈora");
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

        // Basic Latin characters
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

        // Spanish accented characters
        m.insert('á', "a");
        m.insert('é', "e");
        m.insert('í', "i");
        m.insert('ó', "o");
        m.insert('ú', "u");
        m.insert('ñ', "ɲ");
        m.insert('ü', "u");

        // Uppercase versions
        m.insert('Á', "a");
        m.insert('É', "e");
        m.insert('Í', "i");
        m.insert('Ó', "o");
        m.insert('Ú', "u");
        m.insert('Ñ', "ɲ");
        m.insert('Ü', "u");

        m
    };
}

pub fn simple_phonemize(text: &str, _language: &str) -> String {
    let text = text.to_lowercase();
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut phonemes = Vec::new();

    for word in words {
        // Remove punctuation but preserve accented characters
        // The issue was that is_alphabetic() returns false for accented characters like ó, á, etc.
        // We need to include Unicode letters and combining marks
        let clean_word = word.trim_matches(|c: char| {
            !c.is_alphabetic()
                && !matches!(
                    c,
                    'á' | 'é'
                        | 'í'
                        | 'ó'
                        | 'ú'
                        | 'ñ'
                        | 'ü'
                        | 'Á'
                        | 'É'
                        | 'Í'
                        | 'Ó'
                        | 'Ú'
                        | 'Ñ'
                        | 'Ü'
                )
        });

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
        assert_eq!(simple_phonemize("hello world", "en"), "həˈloʊ wɜːrld");

        assert_eq!(
            simple_phonemize("can you add cheese", "en"),
            "kæn juː æd tʃiːz"
        );
    }

    #[test]
    fn test_spanish_accented_characters() {
        // Test the specific case that was failing: "Hola, ¿cómo estás hoy?"
        let result = simple_phonemize("Hola, ¿cómo estás hoy?", "es");

        // The phonemizer should preserve and handle accented characters
        // It should not strip out ó and á like it was doing before
        assert!(
            result.contains("ˈkomo"),
            "Should contain phoneme for 'cómo'"
        );
        assert!(
            result.contains("esˈtas"),
            "Should contain phoneme for 'estás'"
        );

        // Test individual words
        assert_eq!(simple_phonemize("cómo", "es"), "ˈkomo");

        assert_eq!(simple_phonemize("estás", "es"), "esˈtas");

        // Test that accented characters are preserved in unknown words
        let result_unknown = simple_phonemize("política", "es");
        assert!(
            result_unknown.contains("i"),
            "Should preserve accented characters in unknown words: got '{}'",
            result_unknown
        );
    }
}
