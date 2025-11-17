use std::collections::HashMap;

use crate::engine::{Decoder, DecoderId, Policy, Step, TransformResult};

/// Decoder for Morse code to ASCII/UTF-8 letters and digits.
///
/// Supported tokens:
/// - Letters A-Z
/// - Digits 0-9
/// - '/' as a word separator (mapped to ' ')
pub struct MorseCodeDecoder {
    table: HashMap<&'static str, char>,
}

impl MorseCodeDecoder {
    /// Construct a new Morse code decoder with a built-in lookup table.
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
        table.insert("/", ' ');
        MorseCodeDecoder { table }
    }
}

impl Decoder for MorseCodeDecoder {
    fn id(&self) -> DecoderId {
        "morse".to_string()
    }

    fn policy(&self) -> Policy {
        Policy {
            no_consecutive_same_op: true,
        }
    }

    /// Decodes Morse code tokens separated by whitespace.
    ///
    /// Unknown tokens are mapped to '?' during decoding, but if the entire output
    /// would consist only of '?' or be empty, this returns no outputs.
    fn apply(&self, input: &str) -> Vec<TransformResult> {
        let mut out = String::new();

        for token in input.split_whitespace() {
            let ch = self.table.get(token).copied().unwrap_or('?');
            out.push(ch);
        }

        if out.is_empty() || out.chars().all(|c| c == '?') {
            return vec![];
        }

        vec![TransformResult {
            output: out,
            step: Step {
                op_id: self.id(),
                desc: "Decode morse".to_string(),
            },
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_hello_world() {
        let d = MorseCodeDecoder::new();
        let res = d.apply(".... . .-.. .-.. --- / .-- --- .-. .-.. -..");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].output, "HELLO WORLD");
    }

    #[test]
    fn rejects_entirely_invalid_input() {
        let d = MorseCodeDecoder::new();
        // Single unknown token -> would decode to "?", so should be rejected.
        assert!(d.apply("..-.-").is_empty());
        // Empty input -> nothing to decode.
        assert!(d.apply("").is_empty());
    }
}
