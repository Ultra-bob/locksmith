use crate::engine::{Decoder, DecoderId, Policy, Step, TransformResult};

/// Decoder for Base32-encoded data.
///
/// Tries multiple common Base32 alphabets and gathers all successful decodes:
/// - RFC4648 (no padding)
/// - RFC4648 HEX (no padding)
/// - Crockford
/// - Crockford (lowercase)
///
/// Padding '=' is leniently stripped from the input before decoding.
pub struct Base32Decoder;

impl Decoder for Base32Decoder {
    fn id(&self) -> DecoderId {
        "base32".to_string()
    }

    fn group(&self) -> &'static str {
        "base"
    }

    fn policy(&self) -> Policy {
        // Allow repeated Base32 decodes if desired by exploration logic.
        Policy {
            no_consecutive_same_op: false,
        }
    }

    fn apply(&self, input: &str) -> Vec<TransformResult> {
        let alphabets = [
            ("RFC4648", fast32::base32::RFC4648_NOPAD),
            ("RFC4648HEX", fast32::base32::RFC4648_HEX_NOPAD),
            ("Crockford", fast32::base32::CROCKFORD),
            ("Crockford lowercase", fast32::base32::CROCKFORD_LOWER),
        ];

        let mut results = Vec::new();

        // Be lenient about trailing padding.
        let trimmed = input.trim_end_matches('=');

        for (desc_name, alphabet) in alphabets {
            if let Ok(decoded) = alphabet.decode(trimmed.as_bytes()) {
                let output = String::from_utf8_lossy(&decoded).into_owned();
                results.push(TransformResult {
                    output,
                    step: Step {
                        op_id: self.id(),
                        desc: format!("Base32 decode ({})", desc_name),
                    },
                });
            }
        }

        results
    }
}
