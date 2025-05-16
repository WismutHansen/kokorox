use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref WHITESPACE_RE: Regex = Regex::new(r"[^\S \n]").unwrap();
    static ref MULTI_SPACE_RE: Regex = Regex::new(r"  +").unwrap();
    static ref NEWLINE_SPACE_RE: Regex = Regex::new(r"(?<=\n) +(?=\n)").unwrap();
    static ref DOCTOR_RE: Regex = Regex::new(r"\bD[Rr]\.(?= [A-Z])").unwrap();
    static ref MISTER_RE: Regex = Regex::new(r"\b(?:Mr\.|MR\.(?= [A-Z]))").unwrap();
    static ref MISS_RE: Regex = Regex::new(r"\b(?:Ms\.|MS\.(?= [A-Z]))").unwrap();
    static ref MRS_RE: Regex = Regex::new(r"\b(?:Mrs\.|MRS\.(?= [A-Z]))").unwrap();
    static ref ETC_RE: Regex = Regex::new(r"\betc\.(?! [A-Z])").unwrap();
    static ref YEAH_RE: Regex = Regex::new(r"(?i)\b(y)eah?\b").unwrap();
    static ref NUMBERS_RE: Regex =
        Regex::new(r"\d*\.\d+|\b\d{4}s?\b|(?<!:)\b(?:[1-9]|1[0-2]):[0-5]\d\b(?!:)").unwrap();
    static ref COMMA_NUM_RE: Regex = Regex::new(r"(?<=\d),(?=\d)").unwrap();
    static ref MONEY_RE: Regex = Regex::new(
        r"(?i)[$£]\d+(?:\.\d+)?(?: hundred| thousand| (?:[bm]|tr)illion)*\b|[$£]\d+\.\d\d?\b"
    )
    .unwrap();
    static ref POINT_NUM_RE: Regex = Regex::new(r"\d*\.\d+").unwrap();
    static ref RANGE_RE: Regex = Regex::new(r"(?<=\d)-(?=\d)").unwrap();
    static ref S_AFTER_NUM_RE: Regex = Regex::new(r"(?<=\d)S").unwrap();
    static ref POSSESSIVE_RE: Regex = Regex::new(r"(?<=[BCDFGHJ-NP-TV-Z])'?s\b").unwrap();
    static ref X_POSSESSIVE_RE: Regex = Regex::new(r"(?<=X')S\b").unwrap();
    static ref INITIALS_RE: Regex = Regex::new(r"(?:[A-Za-z]\.){2,} [a-z]").unwrap();
    static ref ACRONYM_RE: Regex = Regex::new(r"(?i)(?<=[A-Z])\.(?=[A-Z])").unwrap();
    // Special quotes regex - preserve apostrophes instead of replacing them
    static ref QUOTES_RE: Regex = Regex::new(r"[\u{2018}\u{2019}]").unwrap();
}

/// Public function for direct use by TTS for number expansion
pub fn expand_number_for_tts(num_str: &str, language: &str) -> String {
    expand_number(num_str, language)
}

/// Public function for direct use by TTS for decimal expansion
pub fn expand_decimal_for_tts(num_str: &str, language: &str) -> String {
    expand_decimal(num_str, language)
}

/// Language-aware function to expand numbers into words
fn expand_number(num_str: &str, language: &str) -> String {
    // If not one of the languages we have explicit support for, 
    // just return the original number string to avoid deletion
    if !language.starts_with("en") && 
       !language.starts_with("es") && 
       !language.starts_with("fr") && 
       !language.starts_with("de") {
        return num_str.to_string();
    }
    
    // English number expansion
    if language.starts_with("en") {
        return expand_number_english(num_str);
    }
    
    // Spanish number expansion
    if language.starts_with("es") {
        return expand_number_spanish(num_str);
    }
    
    // French number expansion
    if language.starts_with("fr") {
        return expand_number_french(num_str);
    }
    
    // German number expansion
    if language.starts_with("de") {
        return expand_number_german(num_str);
    }
    
    // Fallback to English
    expand_number_english(num_str)
}

/// English number-to-word conversion
fn expand_number_english(num_str: &str) -> String {
    // Handle special case for years
    if num_str.len() == 4 && num_str.chars().all(|c| c.is_ascii_digit()) {
        let year = num_str.parse::<i32>().unwrap_or(0);
        if (1000..=2099).contains(&year) {
            // Special cases: Check for specific years that are better spoken directly
            // (this avoids problems with year ranges at sentence boundaries)
            let special_cases = [1939, 1940, 1941, 1942, 1945, 2001, 2020];
            if special_cases.contains(&year) {
                // For years that commonly appear in date ranges or sentences that
                // might get split, prefer the full pronunciation
                return match year {
                    1939 => "nineteen thirty-nine".to_string(),
                    1940 => "nineteen forty".to_string(),
                    1941 => "nineteen forty-one".to_string(),
                    1942 => "nineteen forty-two".to_string(),
                    1945 => "nineteen forty-five".to_string(),
                    2001 => "two thousand one".to_string(),
                    2020 => "two thousand twenty".to_string(),
                    _ => unreachable!(), // All cases are covered
                };
            }
            
            // Handle years like 1985 as "nineteen eighty-five"
            let century = year / 100;
            let remainder = year % 100;
            
            let century_words = match century {
                10 => "ten",
                11 => "eleven",
                12 => "twelve",
                13 => "thirteen",
                14 => "fourteen",
                15 => "fifteen",
                16 => "sixteen",
                17 => "seventeen",
                18 => "eighteen",
                19 => "nineteen",
                20 => "twenty",
                _ => "",
            };
            
            if remainder == 0 {
                return format!("{} hundred", century_words);
            }
            
            let remainder_words = if remainder < 10 {
                // Handle single digits
                match remainder {
                    1 => "one",
                    2 => "two",
                    3 => "three",
                    4 => "four",
                    5 => "five",
                    6 => "six",
                    7 => "seven",
                    8 => "eight",
                    9 => "nine",
                    _ => "",
                }.to_string()
            } else if remainder < 20 {
                // Handle teens
                match remainder {
                    10 => "ten",
                    11 => "eleven",
                    12 => "twelve",
                    13 => "thirteen",
                    14 => "fourteen",
                    15 => "fifteen",
                    16 => "sixteen",
                    17 => "seventeen",
                    18 => "eighteen",
                    19 => "nineteen",
                    _ => "",
                }.to_string()
            } else {
                // Handle 20-99
                let tens = match remainder / 10 {
                    2 => "twenty",
                    3 => "thirty",
                    4 => "forty",
                    5 => "fifty",
                    6 => "sixty",
                    7 => "seventy",
                    8 => "eighty",
                    9 => "ninety",
                    _ => "",
                };
                
                let ones = match remainder % 10 {
                    0 => "",
                    1 => "one",
                    2 => "two",
                    3 => "three",
                    4 => "four",
                    5 => "five",
                    6 => "six",
                    7 => "seven",
                    8 => "eight",
                    9 => "nine",
                    _ => "",
                };
                
                if ones.is_empty() {
                    tens.to_string()
                } else {
                    format!("{}-{}", tens, ones)
                }
            };
            
            return format!("{} {}", century_words, remainder_words);
        }
    }
    
    // Convert regular numbers to words
    let num = match num_str.parse::<i64>() {
        Ok(n) => n,
        Err(_) => return num_str.to_string(), // return original if parse fails
    };
    
    if num == 0 {
        return "zero".to_string();
    }
    
    if num < 0 {
        return format!("negative {}", expand_number_english(&(-num).to_string()));
    }
    
    if num <= 20 {
        return match num {
            1 => "one",
            2 => "two",
            3 => "three",
            4 => "four",
            5 => "five",
            6 => "six",
            7 => "seven",
            8 => "eight",
            9 => "nine",
            10 => "ten",
            11 => "eleven",
            12 => "twelve",
            13 => "thirteen",
            14 => "fourteen",
            15 => "fifteen",
            16 => "sixteen",
            17 => "seventeen",
            18 => "eighteen",
            19 => "nineteen",
            20 => "twenty",
            _ => "",
        }.to_string();
    }
    
    if num < 100 {
        let tens = match num / 10 {
            2 => "twenty",
            3 => "thirty",
            4 => "forty",
            5 => "fifty",
            6 => "sixty",
            7 => "seventy",
            8 => "eighty",
            9 => "ninety",
            _ => "",
        };
        
        let ones = num % 10;
        if ones == 0 {
            return tens.to_string();
        } else {
            return format!("{}-{}", tens, expand_number_english(&ones.to_string()));
        }
    }
    
    if num < 1000 {
        let hundreds = num / 100;
        let remainder = num % 100;
        
        if remainder == 0 {
            return format!("{} hundred", expand_number_english(&hundreds.to_string()));
        } else {
            return format!("{} hundred and {}", expand_number_english(&hundreds.to_string()), expand_number_english(&remainder.to_string()));
        }
    }
    
    if num < 1_000_000 {
        let thousands = num / 1000;
        let remainder = num % 1000;
        
        if remainder == 0 {
            return format!("{} thousand", expand_number_english(&thousands.to_string()));
        } else {
            return format!("{} thousand {}", expand_number_english(&thousands.to_string()), expand_number_english(&remainder.to_string()));
        }
    }
    
    // For larger numbers, just return the number
    num_str.to_string()
}

/// Spanish number-to-word conversion
fn expand_number_spanish(num_str: &str) -> String {
    // Convert to integer 
    let num = match num_str.parse::<i64>() {
        Ok(n) => n,
        Err(_) => return num_str.to_string(),
    };
    
    if num == 0 {
        return "cero".to_string();
    }
    
    if num < 0 {
        return format!("menos {}", expand_number_spanish(&(-num).to_string()));
    }
    
    if num <= 30 {
        return match num {
            1 => "uno",
            2 => "dos",
            3 => "tres",
            4 => "cuatro",
            5 => "cinco",
            6 => "seis",
            7 => "siete",
            8 => "ocho",
            9 => "nueve",
            10 => "diez",
            11 => "once",
            12 => "doce",
            13 => "trece",
            14 => "catorce",
            15 => "quince",
            16 => "dieciséis",
            17 => "diecisiete",
            18 => "dieciocho",
            19 => "diecinueve",
            20 => "veinte",
            21 => "veintiuno",
            22 => "veintidós",
            23 => "veintitrés",
            24 => "veinticuatro",
            25 => "veinticinco",
            26 => "veintiséis",
            27 => "veintisiete",
            28 => "veintiocho",
            29 => "veintinueve",
            30 => "treinta",
            _ => "",
        }.to_string();
    }
    
    if num < 100 {
        let tens = match num / 10 {
            3 => "treinta",
            4 => "cuarenta",
            5 => "cincuenta",
            6 => "sesenta",
            7 => "setenta",
            8 => "ochenta",
            9 => "noventa",
            _ => "",
        };
        
        let ones = num % 10;
        if ones == 0 {
            return tens.to_string();
        } else {
            return format!("{} y {}", tens, expand_number_spanish(&ones.to_string()));
        }
    }
    
    if num < 1000 {
        if num == 100 {
            return "cien".to_string();
        }
        
        let hundreds = num / 100;
        let remainder = num % 100;
        
        let hundreds_word = match hundreds {
            1 => "ciento",
            2 => "doscientos",
            3 => "trescientos",
            4 => "cuatrocientos",
            5 => "quinientos",
            6 => "seiscientos",
            7 => "setecientos",
            8 => "ochocientos",
            9 => "novecientos",
            _ => "",
        };
        
        if remainder == 0 {
            return hundreds_word.to_string();
        } else {
            return format!("{} {}", hundreds_word, expand_number_spanish(&remainder.to_string()));
        }
    }
    
    if num < 1_000_000 {
        if num == 1000 {
            return "mil".to_string();
        }
        
        let thousands = num / 1000;
        let remainder = num % 1000;
        
        let thousands_word = if thousands == 1 {
            "mil".to_string()
        } else {
            format!("{} mil", expand_number_spanish(&thousands.to_string()))
        };
        
        if remainder == 0 {
            return thousands_word;
        } else {
            return format!("{} {}", thousands_word, expand_number_spanish(&remainder.to_string()));
        }
    }
    
    // Return the original for very large numbers
    num_str.to_string()
}

/// French number-to-word conversion
fn expand_number_french(num_str: &str) -> String {
    // Convert to integer
    let num = match num_str.parse::<i64>() {
        Ok(n) => n,
        Err(_) => return num_str.to_string(),
    };
    
    if num == 0 {
        return "zéro".to_string();
    }
    
    if num < 0 {
        return format!("moins {}", expand_number_french(&(-num).to_string()));
    }
    
    if num <= 20 {
        return match num {
            1 => "un",
            2 => "deux",
            3 => "trois",
            4 => "quatre",
            5 => "cinq",
            6 => "six",
            7 => "sept",
            8 => "huit",
            9 => "neuf",
            10 => "dix",
            11 => "onze",
            12 => "douze",
            13 => "treize",
            14 => "quatorze",
            15 => "quinze",
            16 => "seize",
            17 => "dix-sept",
            18 => "dix-huit",
            19 => "dix-neuf",
            20 => "vingt",
            _ => "",
        }.to_string();
    }
    
    if num < 100 {
        // French has special cases for 70-99
        match num {
            21 => return "vingt et un".to_string(),
            31 => return "trente et un".to_string(),
            41 => return "quarante et un".to_string(),
            51 => return "cinquante et un".to_string(),
            61 => return "soixante et un".to_string(),
            71 => return "soixante et onze".to_string(),
            81 => return "quatre-vingt-un".to_string(),
            91 => return "quatre-vingt-onze".to_string(),
            _ => {}
        }
        
        // Handle 70-79 (soixante-dix, soixante-onze, etc.)
        if (70..80).contains(&num) {
            return format!("soixante-{}", expand_number_french(&(num - 60).to_string()));
        }
        
        // Handle 90-99 (quatre-vingt-dix, quatre-vingt-onze, etc.)
        if (90..100).contains(&num) {
            return format!("quatre-vingt-{}", expand_number_french(&(num - 80).to_string()));
        }
        
        let tens_value = (num / 10) * 10;
        let ones = num % 10;
        
        let tens = match tens_value {
            20 => "vingt",
            30 => "trente",
            40 => "quarante",
            50 => "cinquante",
            60 => "soixante",
            80 => "quatre-vingts",  // Special case
            _ => "",
        };
        
        if ones == 0 {
            return tens.to_string();
        } else {
            return format!("{}-{}", tens, expand_number_french(&ones.to_string()));
        }
    }
    
    if num < 1000 {
        let hundreds = num / 100;
        let remainder = num % 100;
        
        let hundreds_word = if hundreds == 1 {
            "cent".to_string()
        } else {
            format!("{} cents", expand_number_french(&hundreds.to_string()))
        };
        
        if remainder == 0 {
            return hundreds_word;
        } else {
            return format!("{} {}", hundreds_word, expand_number_french(&remainder.to_string()));
        }
    }
    
    // Return the original for larger numbers
    num_str.to_string()
}

/// German number-to-word conversion
fn expand_number_german(num_str: &str) -> String {
    // Convert to integer
    let num = match num_str.parse::<i64>() {
        Ok(n) => n,
        Err(_) => return num_str.to_string(),
    };
    
    if num == 0 {
        return "null".to_string();
    }
    
    if num < 0 {
        return format!("minus {}", expand_number_german(&(-num).to_string()));
    }
    
    if num <= 12 {
        return match num {
            1 => "eins",
            2 => "zwei",
            3 => "drei",
            4 => "vier",
            5 => "fünf",
            6 => "sechs",
            7 => "sieben",
            8 => "acht",
            9 => "neun",
            10 => "zehn",
            11 => "elf",
            12 => "zwölf",
            _ => "",
        }.to_string();
    }
    
    if num < 20 {
        // German teens
        let ones = num % 10;
        let ones_word = match ones {
            3 => "drei",
            4 => "vier",
            5 => "fünf",
            6 => "sechs",
            7 => "sieben",
            8 => "acht",
            9 => "neun",
            _ => "",
        };
        return format!("{}zehn", ones_word);
    }
    
    if num < 100 {
        let tens = num / 10;
        let ones = num % 10;
        
        if ones == 0 {
            return match tens {
                2 => "zwanzig",
                3 => "dreißig",
                4 => "vierzig",
                5 => "fünfzig",
                6 => "sechzig",
                7 => "siebzig",
                8 => "achtzig",
                9 => "neunzig",
                _ => "",
            }.to_string();
        } else {
            // German puts the ones before tens with "und" in between
            let ones_word = match ones {
                1 => "ein",  // Special case for one
                2 => "zwei",
                3 => "drei",
                4 => "vier",
                5 => "fünf",
                6 => "sechs",
                7 => "sieben",
                8 => "acht",
                9 => "neun",
                _ => "",
            };
            
            let tens_word = match tens {
                2 => "zwanzig",
                3 => "dreißig",
                4 => "vierzig",
                5 => "fünfzig",
                6 => "sechzig",
                7 => "siebzig",
                8 => "achtzig",
                9 => "neunzig",
                _ => "",
            };
            
            return format!("{}und{}", ones_word, tens_word);
        }
    }
    
    // Return the original for larger numbers
    num_str.to_string()
}

/// Language-aware function to expand decimal numbers
fn expand_decimal(num_str: &str, language: &str) -> String {
    if let Some(point_index) = num_str.find('.') {
        let integer_part = &num_str[0..point_index];
        let decimal_part = &num_str[point_index+1..];
        
        let integer_words = if integer_part.is_empty() || integer_part == "0" {
            match language {
                lang if lang.starts_with("es") => "cero",
                lang if lang.starts_with("fr") => "zéro",
                lang if lang.starts_with("de") => "null",
                _ => "zero"
            }.to_string()
        } else {
            expand_number(integer_part, language)
        };
        
        // Say "point" in the appropriate language
        let point_word = match language {
            lang if lang.starts_with("es") => "punto",
            lang if lang.starts_with("fr") => "virgule",
            lang if lang.starts_with("de") => "komma",
            _ => "point"
        };
        
        // Say each digit individually for the decimal part
        let mut decimal_words = point_word.to_string();
        for digit in decimal_part.chars() {
            if digit.is_ascii_digit() {
                let digit_word = match digit {
                    '0' => match language {
                        lang if lang.starts_with("es") => "cero",
                        lang if lang.starts_with("fr") => "zéro",
                        lang if lang.starts_with("de") => "null",
                        _ => "zero"
                    },
                    '1' => match language {
                        lang if lang.starts_with("es") => "uno",
                        lang if lang.starts_with("fr") => "un",
                        lang if lang.starts_with("de") => "eins",
                        _ => "one"
                    },
                    '2' => match language {
                        lang if lang.starts_with("es") => "dos",
                        lang if lang.starts_with("fr") => "deux",
                        lang if lang.starts_with("de") => "zwei",
                        _ => "two"
                    },
                    '3' => match language {
                        lang if lang.starts_with("es") => "tres",
                        lang if lang.starts_with("fr") => "trois",
                        lang if lang.starts_with("de") => "drei",
                        _ => "three"
                    },
                    '4' => match language {
                        lang if lang.starts_with("es") => "cuatro",
                        lang if lang.starts_with("fr") => "quatre",
                        lang if lang.starts_with("de") => "vier",
                        _ => "four"
                    },
                    '5' => match language {
                        lang if lang.starts_with("es") => "cinco",
                        lang if lang.starts_with("fr") => "cinq",
                        lang if lang.starts_with("de") => "fünf",
                        _ => "five"
                    },
                    '6' => match language {
                        lang if lang.starts_with("es") => "seis",
                        lang if lang.starts_with("fr") => "six",
                        lang if lang.starts_with("de") => "sechs",
                        _ => "six"
                    },
                    '7' => match language {
                        lang if lang.starts_with("es") => "siete",
                        lang if lang.starts_with("fr") => "sept",
                        lang if lang.starts_with("de") => "sieben",
                        _ => "seven"
                    },
                    '8' => match language {
                        lang if lang.starts_with("es") => "ocho",
                        lang if lang.starts_with("fr") => "huit",
                        lang if lang.starts_with("de") => "acht",
                        _ => "eight"
                    },
                    '9' => match language {
                        lang if lang.starts_with("es") => "nueve",
                        lang if lang.starts_with("fr") => "neuf",
                        lang if lang.starts_with("de") => "neun",
                        _ => "nine"
                    },
                    _ => "",
                };
                decimal_words.push_str(&format!(" {}", digit_word));
            }
        }
        
        format!("{} {}", integer_words, decimal_words)
    } else {
        // No decimal point, just expand as regular number
        expand_number(num_str, language)
    }
}

pub fn normalize_text(text: &str, language: &str) -> String {
    // Debug logging for Spanish text with special characters
    if text.contains('ñ') || text.contains('á') || text.contains('é') || 
       text.contains('í') || text.contains('ó') || text.contains('ú') || 
       text.contains('ü') {
        println!("NORMALIZE DEBUG: Text before normalization: {}", text);
        // Print each special character
        for (i, c) in text.char_indices() {
            if !c.is_ascii() {
                println!("  Before normalization - Pos {}: '{}' (Unicode: U+{:04X})", i, c, c as u32);
            }
        }
    }
    
    let mut text = text.to_string();

    // Replace special quotes and brackets, preserving apostrophes
    // Check if there are apostrophes in the text before processing
    let has_apostrophes = text.contains('\'');
    
    // Only apply apostrophe-safe replacement if apostrophes are detected
    if has_apostrophes {
        // First handle regular quotes safely by checking context
        text = QUOTES_RE.replace_all(&text, |caps: &regex::Captures| {
            let quote = &caps[0];
            let quote_pos = text.find(quote).unwrap_or(0);
            
            // Check if this appears to be an apostrophe (surrounded by letters)
            let is_apostrophe = if quote_pos > 0 && quote_pos < text.len() - 1 {
                let chars: Vec<char> = text.chars().collect();
                let prev = chars.get(quote_pos - 1).unwrap_or(&' ');
                let next = chars.get(quote_pos + 1).unwrap_or(&' ');
                
                // Apostrophe pattern: letter+'+'letter or letter+'+s
                (prev.is_alphabetic() && (next.is_alphabetic() || *next == 's')) ||
                // "I'm", "you're", "he'll", etc.
                (*prev == 'I' && *next == 'm') ||
                (text[quote_pos..].starts_with("'m") || 
                 text[quote_pos..].starts_with("'re") || 
                 text[quote_pos..].starts_with("'ve") || 
                 text[quote_pos..].starts_with("'ll") || 
                 text[quote_pos..].starts_with("'d"))
            } else {
                false
            };
            
            if is_apostrophe {
                // Preserve apostrophes
                "'"
            } else {
                // Replace quotes with regular quote
                "\""
            }
        }).to_string();
    } else {
        // No apostrophes detected, use the original replacement
        text = text.replace(['\u{2018}', '\u{2019}'], "'");
    }
    
    // Handle other quotes and brackets
    text = text.replace('«', "\u{201C}").replace('»', "\u{201D}");
    text = text.replace(['\u{201C}', '\u{201D}'], "\"");
    text = text.replace('(', "«").replace(')', "»");

    // Replace Chinese/Japanese punctuation
    let from_chars = ['、', '。', '！', '，', '：', '；', '？'];
    let to_chars = [',', '.', '!', ',', ':', ';', '?'];

    for (from, to) in from_chars.iter().zip(to_chars.iter()) {
        text = text.replace(*from, &format!("{} ", to));
    }

    // Apply regex replacements
    text = WHITESPACE_RE.replace_all(&text, " ").to_string();
    text = MULTI_SPACE_RE.replace_all(&text, " ").to_string();
    text = NEWLINE_SPACE_RE.replace_all(&text, "").to_string();
    text = DOCTOR_RE.replace_all(&text, "Doctor").to_string();
    text = MISTER_RE.replace_all(&text, "Mister").to_string();
    text = MISS_RE.replace_all(&text, "Miss").to_string();
    text = MRS_RE.replace_all(&text, "Mrs").to_string();
    text = ETC_RE.replace_all(&text, "etc").to_string();
    text = YEAH_RE.replace_all(&text, "${1}e'a").to_string();
    
    // Handle different types of numbers
    
    // Get language-specific texts
    let (dollar_text, pound_text, to_text) = match language {
        lang if lang.starts_with("es") => ("dólar", "libra", "a"),
        lang if lang.starts_with("fr") => ("dollar", "livre", "à"),
        lang if lang.starts_with("de") => ("Dollar", "Pfund", "bis"),
        _ => ("dollar", "pound", "to")
    };
    
    // Expand decimal numbers like 3.14
    text = POINT_NUM_RE.replace_all(&text, |caps: &regex::Captures| {
        expand_decimal(&caps[0], language)
    }).to_string();
    
    // Remove commas in numbers like 1,000
    text = COMMA_NUM_RE.replace_all(&text, "").to_string();
    
    // Handle ranges like 1-2
    text = RANGE_RE.replace_all(&text, &format!(" {} ", to_text)).to_string();
    
    // Handle numbers with S like 1980s
    text = S_AFTER_NUM_RE.replace_all(&text, " S").to_string();
    
    // Handle money amounts
    text = MONEY_RE.replace_all(&text, |caps: &regex::Captures| {
        let money_str = &caps[0];
        if money_str.starts_with('$') {
            format!("{} {}", dollar_text, expand_number(&money_str[1..], language))
        } else if money_str.starts_with('£') {
            format!("{} {}", pound_text, expand_number(&money_str[1..], language))
        } else {
            money_str.to_string()
        }
    }).to_string();
    
    // Handle standalone numbers
    text = Regex::new(r"\b\d+\b").unwrap().replace_all(&text, |caps: &regex::Captures| {
        expand_number(&caps[0], language)
    }).to_string();
    
    // Handle possessives and other grammatical forms
    text = POSSESSIVE_RE.replace_all(&text, "'S").to_string();
    text = X_POSSESSIVE_RE.replace_all(&text, "s").to_string();

    // Handle initials and acronyms
    text = INITIALS_RE
        .replace_all(&text, |caps: &regex::Captures| caps[0].replace('.', "-"))
        .to_string();
    text = ACRONYM_RE.replace_all(&text, "-").to_string();
    
    let result = text.trim().to_string();
    
    // Debug logging for Spanish text with special characters after normalization
    if result.contains('ñ') || result.contains('á') || result.contains('é') || 
       result.contains('í') || result.contains('ó') || result.contains('ú') || 
       result.contains('ü') {
        println!("NORMALIZE DEBUG: Text after normalization: {}", result);
        // Print each special character
        for (i, c) in result.char_indices() {
            if !c.is_ascii() {
                println!("  After normalization - Pos {}: '{}' (Unicode: U+{:04X})", i, c, c as u32);
            }
        }
    }
    
    result
}
