use base64::{Engine as _, engine::general_purpose};

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
        if let Ok(decoded) = general_purpose::STANDARD.decode(input) {
            let output = String::from_utf8_lossy(&decoded).into_owned();
            vec![TransformResult {
                output,
                step: Step {
                    op_id: self.id(),
                    desc: "Base64 decode".to_string(),
                    group: self.group(),
                },
            }]
        } else {
            vec![]
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

/// Convenience to register all built-in decoders into a decoder engine.
pub fn register_all(engine: &mut crate::engine::DecoderEngine) {
    engine.register(CaesarDecoder);
    engine.register(Base64Decoder);
}
