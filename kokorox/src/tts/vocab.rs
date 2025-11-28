use lazy_static::lazy_static;
use std::collections::HashMap;

/// Get the default English/multilingual vocab (v1.0 model)
pub fn get_vocab() -> std::collections::HashMap<char, usize> {
    let pad = "$";
    let punctuation = ";:,.!?\u{00A1}\u{00BF}\u{2014}\u{2026}\"\u{00AB}\u{00BB}\u{201C}\u{201D} ";
    let letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let letters_ipa = "ɑɐɒæɓʙβɔɕçɗɖðʤəɘɚɛɜɝɞɟʄɡɠɢʛɦɧħɥʜɨɪʝɭɬɫɮʟɱɯɰŋɳɲɴøɵɸθœɶʘɹɺɾɻʀʁɽʂʃʈʧʉʊʋⱱʌɣɤʍχʎʏʑʐʒʔʡʕʢǀǁǂǃˈˌːˑʼʴʰʱʲʷˠˤ˞↓↑→↗↘'̩'ᵻ";

    let symbols: String = [pad, punctuation, letters, letters_ipa].concat();

    symbols
        .chars()
        .enumerate()
        .collect::<HashMap<_, _>>()
        .into_iter()
        .map(|(idx, c)| (c, idx))
        .collect()
}

/// Get the Chinese vocab (v1.1-zh model)
/// This vocab is based on the tokenizer.json from onnx-community/Kokoro-82M-v1.1-zh-ONNX
pub fn get_zh_vocab() -> HashMap<char, usize> {
    let mut vocab = HashMap::new();
    
    // Based on tokenizer.json from Kokoro-82M-v1.1-zh-ONNX
    let mappings: &[(&str, usize)] = &[
        ("$", 0),
        (";", 1),
        (":", 2),
        (",", 3),
        (".", 4),
        ("!", 5),
        ("?", 6),
        ("/", 7),
        // 8 is missing
        ("—", 9),
        ("…", 10),
        ("\"", 11),
        ("(", 12),
        (")", 13),
        ("\u{201C}", 14), // left double quotation mark
        ("\u{201D}", 15), // right double quotation mark
        (" ", 16),
        ("\u{0303}", 17), // combining tilde
        ("ʣ", 18),
        ("ʥ", 19),
        ("ʦ", 20),
        ("ʨ", 21),
        ("ᵝ", 22),
        ("ㄓ", 23), // zh
        ("A", 24),
        ("I", 25),
        // 26-29 missing
        ("ㄅ", 30), // b
        ("O", 31),
        ("ㄆ", 32), // p
        ("Q", 33),
        ("R", 34),
        ("S", 35),
        ("T", 36),
        ("ㄇ", 37), // m
        ("ㄈ", 38), // f
        ("W", 39),
        ("ㄉ", 40), // d
        ("Y", 41),
        ("ᵊ", 42),
        ("a", 43),
        ("b", 44),
        ("c", 45),
        ("d", 46),
        ("e", 47),
        ("f", 48),
        ("ㄊ", 49), // t
        ("h", 50),
        ("i", 51),
        ("j", 52),
        ("k", 53),
        ("l", 54),
        ("m", 55),
        ("n", 56),
        ("o", 57),
        ("p", 58),
        ("q", 59),
        ("r", 60),
        ("s", 61),
        ("t", 62),
        ("u", 63),
        ("v", 64),
        ("w", 65),
        ("x", 66),
        ("y", 67),
        ("z", 68),
        ("ɑ", 69),
        ("ɐ", 70),
        ("ɒ", 71),
        ("æ", 72),
        ("ㄋ", 73), // n
        ("ㄌ", 74), // l
        ("β", 75),
        ("ɔ", 76),
        ("ɕ", 77),
        ("ç", 78),
        ("ㄍ", 79), // g
        ("ɖ", 80),
        ("ð", 81),
        ("ʤ", 82),
        ("ə", 83),
        ("ㄎ", 84), // k
        ("ㄦ", 85), // er
        ("ɛ", 86),
        ("ɜ", 87),
        ("ㄏ", 88), // h
        ("ㄐ", 89), // j
        ("ɟ", 90),
        ("ㄑ", 91), // q
        ("ɡ", 92),
        ("ㄒ", 93), // x
        ("ㄔ", 94), // ch
        ("ㄕ", 95), // sh
        ("ㄗ", 96), // z
        ("ㄘ", 97), // c
        ("ㄙ", 98), // s
        ("月", 99), // ve/ue
        ("ㄚ", 100), // a
        ("ɨ", 101),
        ("ɪ", 102),
        ("ʝ", 103),
        ("ㄛ", 104), // o
        ("ㄝ", 105), // ie
        ("ㄞ", 106), // ai
        ("ㄟ", 107), // ei
        ("ㄠ", 108), // ao
        ("ㄡ", 109), // ou
        ("ɯ", 110),
        ("ɰ", 111),
        ("ŋ", 112),
        ("ɳ", 113),
        ("ɲ", 114),
        ("ɴ", 115),
        ("ø", 116),
        ("ㄢ", 117), // an
        ("ɸ", 118),
        ("θ", 119),
        ("œ", 120),
        ("ㄣ", 121), // en
        ("ㄤ", 122), // ang
        ("ɹ", 123),
        ("ㄥ", 124), // eng
        ("ɾ", 125),
        ("ㄖ", 126), // r
        ("ㄧ", 127), // i
        ("ʁ", 128),
        ("ɽ", 129),
        ("ʂ", 130),
        ("ʃ", 131),
        ("ʈ", 132),
        ("ʧ", 133),
        ("ㄨ", 134), // u
        ("ʊ", 135),
        ("ʋ", 136),
        ("ㄩ", 137), // v/u
        ("ʌ", 138),
        ("ɣ", 139),
        ("ㄜ", 140), // e
        ("ㄭ", 141), // ii (zi, ci, si)
        ("χ", 142),
        ("ʎ", 143),
        ("十", 144), // iii (zhi, chi, shi, ri)
        ("压", 145), // ia
        ("言", 146), // ian
        ("ʒ", 147),
        ("ʔ", 148),
        ("阳", 149), // iang
        ("要", 150), // iao
        ("阴", 151), // in
        ("应", 152), // ing
        ("用", 153), // iong
        ("又", 154), // iou/iu
        ("中", 155), // ong
        ("ˈ", 156),
        ("ˌ", 157),
        ("ː", 158),
        ("穵", 159), // ua
        ("外", 160), // uai
        ("万", 161), // uan
        ("ʰ", 162),
        ("王", 163), // uang
        ("ʲ", 164),
        ("为", 165), // uei/ui
        ("文", 166), // uen/un
        ("瓮", 167), // ueng
        ("我", 168), // uo
        ("3", 169),
        ("5", 170),
        ("1", 171),
        ("2", 172),
        ("4", 173),
        // 174 missing
        ("元", 175), // van/yuan
        ("云", 176), // vn/yun
        ("ᵻ", 177),
    ];
    
    for (s, idx) in mappings {
        for c in s.chars() {
            vocab.insert(c, *idx);
        }
    }
    
    vocab
}

pub fn get_reverse_vocab() -> HashMap<usize, char> {
    VOCAB.iter().map(|(&c, &idx)| (idx, c)).collect()
}

pub fn get_zh_reverse_vocab() -> HashMap<usize, char> {
    ZH_VOCAB.iter().map(|(&c, &idx)| (idx, c)).collect()
}

#[allow(dead_code)]
pub fn print_sorted_reverse_vocab() {
    let mut sorted_keys: Vec<_> = REVERSE_VOCAB.keys().collect();
    sorted_keys.sort();

    for key in sorted_keys {
        eprintln!("{}: {}", key, REVERSE_VOCAB[key]);
    }
}

lazy_static! {
    /// Default vocab for English/multilingual model (v1.0)
    pub static ref VOCAB: HashMap<char, usize> = get_vocab();
    pub static ref REVERSE_VOCAB: HashMap<usize, char> = get_reverse_vocab();
    
    /// Chinese vocab for v1.1-zh model
    pub static ref ZH_VOCAB: HashMap<char, usize> = get_zh_vocab();
    pub static ref ZH_REVERSE_VOCAB: HashMap<usize, char> = get_zh_reverse_vocab();
}
