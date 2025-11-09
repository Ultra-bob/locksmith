use base64::Engine;
use std::collections::HashMap;

use base64;

use crate::engine::{Decoder, DecoderId, Policy, Step, TransformResult};

/// Decoder for Base64-encoded data.
pub struct Base64Decoder;

impl Decoder for Base64Decoder {
    fn id(&self) -> DecoderId {
        "base64"
    }

    fn group(&self) -> &'static str {
        "radix64"
    }

    fn policy(&self) -> Policy {
        // Prevent immediate repeated base64 decodes by default.
        // Adjust `no_group_repeat_within` if you want to allow chains like base64->base64.
        Policy {
            no_consecutive_same_op: true,
            no_group_repeat_within: 1,
        }
    }

    fn apply(&self, input: &str) -> Vec<TransformResult> {
        // Strip trailing '=' padding characters for leniency
        let stripped = input.trim().trim_end_matches('=');

        let try_decode = |input: &str| {
            base64::engine::general_purpose::STANDARD_NO_PAD
                .decode(input)
                .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(input))
        };

        // dbg!(try_decode(stripped));

        match try_decode(stripped) {
            Ok(decoded) => {
                let output = String::from_utf8_lossy(&decoded).into_owned();
                // Invalid characters
                if output.contains("\u{FFFD}") {
                    // dbg!("Invalid characters");
                    return vec![];
                }
                vec![TransformResult {
                    output,
                    step: Step {
                        op_id: self.id(),
                        desc: "Base64 decode".to_string(),
                        group: self.group(),
                    },
                }]
            }
            Err(_) => vec![],
        }
    }
}

/// Brute-force Caesar shift decoder (shifts 1..=25).
pub struct CaesarDecoder;

impl Decoder for CaesarDecoder {
    fn id(&self) -> DecoderId {
        "caesar"
    }

    fn group(&self) -> &'static str {
        "shift"
    }

    fn policy(&self) -> Policy {
        // Avoid consecutive Caesar shifts (which are typically redundant)
        // and avoid other "shift" group decoders immediately after.
        Policy {
            no_consecutive_same_op: true,
            no_group_repeat_within: 1,
        }
    }

    fn apply(&self, input: &str) -> Vec<TransformResult> {
        let mut out = Vec::with_capacity(25);

        for shift in 1..=25 {
            let decoded = input
                .chars()
                .map(|c| {
                    if c.is_ascii_alphabetic() {
                        let base = if c.is_ascii_lowercase() { b'a' } else { b'A' };
                        let x = ((c as u8 - base + (shift as u8)) % 26) + base;
                        x as char
                    } else {
                        c
                    }
                })
                .collect::<String>();

            out.push(TransformResult {
                output: decoded,
                step: Step {
                    op_id: self.id(),
                    desc: format!("Caesar shift {}", shift),
                    group: self.group(),
                },
            });
        }

        out
    }
}

pub struct BinaryDecoder;

impl Decoder for BinaryDecoder {
    fn id(&self) -> DecoderId {
        "binary"
    }

    fn group(&self) -> &'static str {
        "binary"
    }

    fn policy(&self) -> Policy {
        Policy {
            no_consecutive_same_op: true,
            no_group_repeat_within: 1,
        }
    }

    // Decodes written out binary (ex 00101011) as ASCII
    fn apply(&self, input: &str) -> Vec<TransformResult> {
        let mut out = String::with_capacity(input.len() / 8);
        let mut byte = 0u8;
        let mut bit_count = 0;

        for c in input.chars() {
            match c {
                '0' => {
                    byte = (byte << 1) | 0;
                    bit_count += 1;
                }
                '1' => {
                    byte = (byte << 1) | 1;
                    bit_count += 1;
                }
                _ => {
                    // Ignore non-binary characters (e.g. spaces or delimiters)
                    continue;
                }
            }

            if bit_count == 8 {
                out.push(byte as char);
                byte = 0;
                bit_count = 0;
            }
        }

        vec![TransformResult {
            output: out,
            step: Step {
                op_id: self.id(),
                desc: "Binary to ASCII".to_string(),
                group: self.group(),
            },
        }]
    }
}

pub struct ReverseDecoder;

impl Decoder for ReverseDecoder {
    fn id(&self) -> DecoderId {
        "reverse"
    }

    fn group(&self) -> &'static str {
        "reverse"
    }

    fn policy(&self) -> Policy {
        Policy {
            no_consecutive_same_op: true,
            no_group_repeat_within: 1,
        }
    }

    // Reverses the input string
    fn apply(&self, input: &str) -> Vec<TransformResult> {
        vec![TransformResult {
            output: input.chars().rev().collect(),
            step: Step {
                op_id: self.id(),
                desc: "Reverse".to_string(),
                group: self.group(),
            },
        }]
    }
}

pub struct HexDecoder;

impl Decoder for HexDecoder {
    fn id(&self) -> DecoderId {
        "hex"
    }

    fn group(&self) -> &'static str {
        "hex"
    }

    fn policy(&self) -> Policy {
        // Prevent immediate repeated hex decodes by default.
        Policy {
            no_consecutive_same_op: true,
            no_group_repeat_within: 1,
        }
    }

    // Decodes pairs of hexadecimal digits (optionally separated by spaces) to ASCII.
    fn apply(&self, input: &str) -> Vec<TransformResult> {
        let mut out = Vec::new();
        let mut chars = input.chars().filter(|c| !c.is_whitespace()).peekable();

        // If odd number of hex digits, cannot decode properly
        if chars.clone().count() % 2 != 0 {
            return vec![];
        }

        while let (Some(h), Some(l)) = (chars.next(), chars.next()) {
            let hex = [h, l];
            let byte_str: String = hex.iter().collect();
            match u8::from_str_radix(&byte_str, 16) {
                Ok(byte) => out.push(byte as char),
                Err(_) => return vec![], // Invalid hex input
            }
        }

        vec![TransformResult {
            output: out.into_iter().collect(),
            step: Step {
                op_id: self.id(),
                desc: "Hex decode".to_string(),
                group: self.group(),
            },
        }]
    }
}

pub struct MorseCodeDecoder {
    table: HashMap<&'static str, char>,
}

impl MorseCodeDecoder {
    pub fn new() -> Self {
        let mut table = HashMap::new();
        table.insert(".-", 'A');
        table.insert("-...", 'B');
        table.insert("-.-.", 'C');
        table.insert("-..", 'D');
        table.insert(".", 'E');
        table.insert("..-.", 'F');
        table.insert("--.", 'G');
        table.insert("....", 'H');
        table.insert("..", 'I');
        table.insert(".---", 'J');
        table.insert("-.-", 'K');
        table.insert(".-..", 'L');
        table.insert("--", 'M');
        table.insert("-.", 'N');
        table.insert("---", 'O');
        table.insert(".--.", 'P');
        table.insert("--.-", 'Q');
        table.insert(".-.", 'R');
        table.insert("...", 'S');
        table.insert("-", 'T');
        table.insert("..-", 'U');
        table.insert("...-", 'V');
        table.insert(".--", 'W');
        table.insert("-..-", 'X');
        table.insert("-.--", 'Y');
        table.insert("--..", 'Z');
        table.insert(".----", '1');
        table.insert("..---", '2');
        table.insert("...--", '3');
        table.insert("....-", '4');
        table.insert(".....", '5');
        table.insert("-....", '6');
        table.insert("--...", '7');
        table.insert("---..", '8');
        table.insert("----.", '9');
        table.insert("-----", '0');
        MorseCodeDecoder { table }
    }
}

impl Decoder for MorseCodeDecoder {
    fn id(&self) -> DecoderId {
        "morse"
    }

    fn group(&self) -> &'static str {
        "morse"
    }

    fn policy(&self) -> Policy {
        Policy {
            no_consecutive_same_op: true,
            no_group_repeat_within: 1,
        }
    }

    // Decodes Morse code to ASCII.
    fn apply(&self, input: &str) -> Vec<TransformResult> {
        let mut words = input.split_whitespace().peekable();

        let mut out = String::new();

        while let Some(word) = words.next() {
            let word_out = self.table.get(word).copied().unwrap_or('?');
            out.push(word_out);
        }

        if out.chars().all(|c| c == '?') || out.is_empty() {
            return vec![];
        }

        vec![TransformResult {
            output: out,
            step: Step {
                op_id: self.id(),
                desc: "Decode morse".to_string(),
                group: self.group(),
            },
        }]
    }
}

pub struct HTMLEntityDecoder;

impl Decoder for HTMLEntityDecoder {
    fn id(&self) -> DecoderId {
        "html_entity"
    }

    fn group(&self) -> &'static str {
        "html"
    }

    fn policy(&self) -> Policy {
        Policy {
            no_consecutive_same_op: true,
            no_group_repeat_within: 1,
        }
    }

    fn apply(&self, input: &str) -> Vec<TransformResult> {
        let mut out = String::new();

        for entity in input.split('&') {
            let entity = entity.trim_start_matches('#');
            let entity = entity.trim_end_matches(';');

            if entity.starts_with("x") {
                let hex = entity.strip_prefix("x").unwrap();
                let char = char::from_u32(u32::from_str_radix(hex, 16).unwrap_or(0));
                out.push(char.unwrap_or('?'));
            } else {
                let dec = entity.parse::<u32>().unwrap_or(0);
                let char = char::from_u32(dec);
                out.push(char.unwrap_or('?'));
            }
        }

        if out.chars().all(|c| c == '?') || out.is_empty() {
            return vec![];
        }

        vec![TransformResult {
            output: out,
            step: Step {
                op_id: self.id(),
                desc: "Decode HTML entity".to_string(),
                group: self.group(),
            },
        }]
    }
}

/// Convenience to register all built-in decoders into a decoder engine.
pub fn register_all(engine: &mut crate::engine::DecoderEngine) {
    engine.register(CaesarDecoder);
    engine.register(BinaryDecoder);
    engine.register(ReverseDecoder);
    engine.register(Base64Decoder);
    engine.register(MorseCodeDecoder::new());
    engine.register(HexDecoder);
    engine.register(HTMLEntityDecoder);
}
