use crate::tts::vocab::{VOCAB, ZH_VOCAB};

/// Model variant for tokenization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModelVariant {
    /// English/multilingual model (v1.0)
    #[default]
    English,
    /// Chinese model (v1.1-zh)
    Chinese,
}

/// Tokenizes the given phonemes string into a vector of token indices.
///
/// This function takes a text string as input and converts it into a vector of token indices
/// by looking up each character in the appropriate vocabulary map based on the model variant.
///
/// # Arguments
/// * `phonemes` - The input phoneme string to be tokenized.
///
/// # Returns
/// A vector of `i64` token indices representing the input phonemes.
pub fn tokenize(phonemes: &str) -> Vec<i64> {
    tokenize_with_variant(phonemes, ModelVariant::English)
}

/// Tokenizes the given phonemes string into a vector of token indices using the specified model variant.
///
/// # Arguments
/// * `phonemes` - The input phoneme string to be tokenized.
/// * `variant` - The model variant to use for vocabulary lookup.
///
/// # Returns
/// A vector of `i64` token indices representing the input phonemes.
pub fn tokenize_with_variant(phonemes: &str, variant: ModelVariant) -> Vec<i64> {
    let vocab = match variant {
        ModelVariant::English => &*VOCAB,
        ModelVariant::Chinese => &*ZH_VOCAB,
    };
    
    let mut tokens = Vec::new();
    let mut dropped_chars = Vec::new();
    
    for c in phonemes.chars() {
        match vocab.get(&c) {
            Some(&idx) => tokens.push(idx as i64),
            None => {
                dropped_chars.push(c);
                // For now, skip unknown characters but log them
                eprintln!("WARNING: Character '{}' (U+{:04X}) not in vocabulary, skipping", c, c as u32);
            }
        }
    }
    
    if !dropped_chars.is_empty() {
        eprintln!("TOKENIZE: Dropped {} characters from input: {:?}", dropped_chars.len(), dropped_chars);
    }
    
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        // Test IPA phoneme tokenization
        let text = "heɪ ðɪs ɪz ˈlʌvliː!";
        let tokens = tokenize(text);

        // Verify that we get a non-empty result with the right length
        // (character counts: h, e, ɪ, space, ð, ɪ, s, space, ɪ, z, space, ˈ, l, ʌ, v, l, i, ː, !)
        assert!(!tokens.is_empty());
        assert_eq!(tokens.len(), 19); // 19 characters in the IPA string

        // Test empty string
        let empty = "";
        let empty_tokens = tokenize(empty);
        assert!(empty_tokens.is_empty());

        // Test punctuation
        let punct = "...";
        let punct_tokens = tokenize(punct);
        assert_eq!(punct_tokens.len(), 3);
    }
}

use crate::tts::vocab::{REVERSE_VOCAB, ZH_REVERSE_VOCAB};

pub fn tokens_to_phonemes(tokens: &[i64]) -> String {
    tokens_to_phonemes_with_variant(tokens, ModelVariant::English)
}

pub fn tokens_to_phonemes_with_variant(tokens: &[i64], variant: ModelVariant) -> String {
    let reverse_vocab = match variant {
        ModelVariant::English => &*REVERSE_VOCAB,
        ModelVariant::Chinese => &*ZH_REVERSE_VOCAB,
    };
    
    tokens
        .iter()
        .filter_map(|&t| reverse_vocab.get(&(t as usize)))
        .collect()
}

#[cfg(test)]
mod tests2 {
    use super::*;

    #[test]
    fn test_tokens_to_phonemes() {
        let tokens = vec![24, 47, 54, 54, 57, 5];
        let text = tokens_to_phonemes(&tokens);
        assert_eq!(text, "Hello!");

        let tokens = vec![
            0, 50, 83, 54, 156, 57, 135, 3, 16, 65, 156, 87, 158, 54, 46, 5, 0,
        ];

        let text = tokens_to_phonemes(&tokens);
        assert_eq!(text, "$həlˈoʊ, wˈɜːld!$");

        // Test empty vector
        let empty_tokens: Vec<i64> = vec![];
        assert_eq!(tokens_to_phonemes(&empty_tokens), "");
    }
}
