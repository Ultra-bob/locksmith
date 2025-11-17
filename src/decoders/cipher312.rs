use crate::engine::{Decoder, DecoderId, Step, TransformResult};

pub struct ThreeOneTwoCipher;

impl Decoder for ThreeOneTwoCipher {
    fn id(&self) -> DecoderId {
        "cipher312".to_string()
    }

    fn apply(&self, input: &str) -> Vec<TransformResult> {
        if !input
            .chars()
            .all(|c| c.is_ascii_digit() || c.is_ascii_whitespace())
        {
            return vec![];
        }
        // 1. Reverse the "squishing" rule: 4→11, 5→22, 6→33.
        let mut expanded = String::with_capacity(input.len() * 2);
        for ch in input.chars() {
            match ch {
                '4' => expanded.push_str("11"),
                '5' => expanded.push_str("22"),
                '6' => expanded.push_str("33"),
                _ => expanded.push(ch),
            }
        }

        // 2. Decode triplets, treating '0' as a space and unknowns as '?'.
        let bytes = expanded.as_bytes();
        let mut i = 0;
        let mut result = String::new();

        while i < bytes.len() {
            // '0' => space, consumes only one digit
            if bytes[i] == b'0' {
                result.push(' ');
                i += 1;
                continue;
            }

            // Need a full triplet; if not enough bytes remain, treat as unknown.
            if i + 3 > bytes.len() {
                result.push('?');
                break;
            }

            // Safe because the data is ASCII digits.
            let triplet = std::str::from_utf8(&bytes[i..i + 3]).unwrap();
            i += 3;

            let ch = match triplet {
                "111" => 'A',
                "112" => 'B',
                "113" => 'C',
                "121" => 'D',
                "122" => 'E',
                "123" => 'F',
                "131" => 'G',
                "132" => 'H',
                "133" => 'I',
                "211" => 'J',
                "212" => 'K',
                "213" => 'L',
                "221" => 'M',
                "222" => 'N',
                "223" => 'O',
                "231" => 'P',
                "232" => 'Q',
                "233" => 'R',
                "311" => 'S',
                "312" => 'T',
                "313" => 'U',
                "321" => 'V',
                "322" => 'W',
                "323" => 'X',
                "331" => 'Y',
                "332" => 'Z',
                "118" => '1',
                "128" => '2',
                "138" => '3',
                "218" => '4',
                "228" => '5',
                "238" => '6',
                "318" => '7',
                "328" => '8',
                "338" => '9',
                _ => '?',
            };

            result.push(ch);
        }

        if result.trim_end_matches('?').is_empty() {
            return vec![];
        }
        return vec![TransformResult {
            output: result,
            step: Step {
                op_id: self.id(),
                desc: String::from("Decode 312 Cipher"),
            },
        }];
    }
}
