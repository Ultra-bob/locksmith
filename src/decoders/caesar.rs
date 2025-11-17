use crate::engine::{Decoder, DecoderId, Policy, Step, TransformResult};

/// Brute-force Caesar shift decoder (shifts 1..=25).
pub struct CaesarDecoder;

impl Decoder for CaesarDecoder {
    fn id(&self) -> DecoderId {
        "caesar".to_string()
    }

    fn policy(&self) -> Policy {
        // Avoid consecutive Caesar shifts (which are typically redundant)
        // and avoid other "shift" group decoders immediately after.
        Policy {
            no_consecutive_same_op: true,
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
                },
            });
        }

        out
    }
}
