use kokorox::tts::{tokenize::tokenize, vocab::VOCAB};

fn main() {
    let test_input = "ɑː juː fˈʌkɪŋ sˈiəɹiəs";
    println!("Input: {}", test_input);
    
    // Check which characters are in vocab and which are not
    for c in test_input.chars() {
        match VOCAB.get(&c) {
            Some(idx) => println!("'{}' -> token {}", c, idx),
            None => println!("'{}' -> NOT IN VOCAB", c),
        }
    }
    
    // Show tokenized result
    let tokens = tokenize(test_input);
    println!("Tokens: {:?}", tokens);
    
    // Show what the tokenizer actually keeps
    let kept_chars: String = test_input.chars()
        .filter(|c| VOCAB.contains_key(c))
        .collect();
    println!("Kept chars: '{}'", kept_chars);
}