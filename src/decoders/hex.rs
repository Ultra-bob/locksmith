use crate::engine::{Decoder, DecoderId, Policy, Step, TransformResult};

/// Decoder that converts hexadecimal-encoded text to ASCII/UTF-8.
///
/// - Whitespace is ignored (e.g., "68 69" decodes to "hi").
/// - Returns an empty list if the number of hex digits is odd or if any pair is invalid.
pub struct HexDecoder;

impl Decoder for HexDecoder {
    fn id(&self) -> DecoderId {
        "hex".to_string()
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

    fn apply(&self, input: &str) -> Vec<TransformResult> {
        // Remove whitespace from the input to get a continuous stream of hex digits.
        let filtered: Vec<char> = input.chars().filter(|c| !c.is_whitespace()).collect();

        // Hex decoding requires an even number of digits.
        if filtered.len() % 2 != 0 {
            return vec![];
        }

        let mut out = String::with_capacity(filtered.len() / 2);

        for pair in filtered.chunks(2) {
            let s: String = pair.iter().collect();
            match u8::from_str_radix(&s, 16) {
                Ok(byte) => out.push(byte as char),
                Err(_) => return vec![], // Invalid hex input -> no outputs
            }
        }

        vec![TransformResult {
            output: out,
            step: Step {
                op_id: self.id(),
                desc: "Hex decode".to_string(),
                group: self.group().to_string(),
            },
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Decoder;

    #[test]
    fn decodes_simple_hex() {
        let d = HexDecoder;
        let res = d.apply("6869"); // "hi"
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].output, "hi");
    }

    #[test]
    fn decodes_hex_with_spaces() {
        let d = HexDecoder;
        let res = d.apply("68 69"); // "hi"
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].output, "hi");
    }

    #[test]
    fn rejects_odd_digit_count() {
        let d = HexDecoder;
        assert!(d.apply("123").is_empty());
    }

    #[test]
    fn rejects_invalid_hex() {
        let d = HexDecoder;
        assert!(d.apply("ZZ").is_empty());
    }
}
