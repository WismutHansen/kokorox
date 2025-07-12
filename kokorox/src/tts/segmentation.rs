/// Simple sentence segmentation for streaming TTS
/// This is a simplified version for WebSocket streaming use
pub fn split_into_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current_sentence = String::new();
    
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let ch = chars[i];
        current_sentence.push(ch);
        
        // Check for sentence endings
        if ch == '.' || ch == '!' || ch == '?' {
            // Look ahead to see if this might be an abbreviation or decimal
            let mut is_sentence_end = true;
            
            if ch == '.' && i + 1 < chars.len() {
                let next_char = chars[i + 1];
                // If followed by whitespace and then a lowercase letter, might not be sentence end
                if next_char.is_whitespace() && i + 2 < chars.len() {
                    let char_after_space = chars[i + 2];
                    if char_after_space.is_lowercase() {
                        is_sentence_end = false;
                    }
                }
                // If followed immediately by a digit, it's probably a decimal
                else if next_char.is_ascii_digit() {
                    is_sentence_end = false;
                }
            }
            
            if is_sentence_end {
                // Consume any trailing whitespace
                while i + 1 < chars.len() && chars[i + 1].is_whitespace() {
                    i += 1;
                    current_sentence.push(chars[i]);
                }
                
                let trimmed = current_sentence.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
                current_sentence.clear();
            }
        }
        
        i += 1;
    }
    
    // Add any remaining text as the last sentence
    let trimmed = current_sentence.trim().to_string();
    if !trimmed.is_empty() {
        sentences.push(trimmed);
    }
    
    sentences
}