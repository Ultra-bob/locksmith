use crate::engine::{Decoder, DecoderId, Policy, Step, TransformResult};

/// Decoder that interprets sequences of '0' and '1' (ignoring whitespace and other
/// non-binary characters) as bytes and converts them to a UTF-8 string by mapping
/// each full 8-bit chunk to a char.
///
/// Example:
/// "01101000 01101001" -> "hi"
pub struct BinaryDecoder;

impl Decoder for BinaryDecoder {
    fn id(&self) -> DecoderId {
        "binary".to_string()
    }

    fn group(&self) -> &'static str {
        "binary"
    }

    fn policy(&self) -> Policy {
        Policy {
            no_consecutive_same_op: true,
        }
    }

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
                    // Ignore non-binary characters (e.g., spaces or delimiters)
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
            },
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_to_ascii_basic() {
        let d = BinaryDecoder;
        let res = d.apply("01101000 01101001"); // "hi"
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].output, "hi");
    }

    #[test]
    fn ignores_non_binary_chars() {
        let d = BinaryDecoder;
        let res = d.apply("0110-1000, 0110-1001"); // "hi" with punctuation
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].output, "hi");
    }

    #[test]
    fn incomplete_final_byte_is_ignored() {
        let d = BinaryDecoder;
        // "h" is 01101000, we only give 7 bits -> should not produce 'h'
        let res = d.apply("0110100");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].output, "");
    }
}
