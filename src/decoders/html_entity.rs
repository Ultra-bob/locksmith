use crate::engine::{Decoder, DecoderId, Policy, Step, TransformResult};

/// Decoder that attempts to interpret numeric HTML entities (decimal or hex) and
/// convert them to characters.
///
/// Notes:
/// - This is a very naive implementation that splits on '&' and decodes each
///   token as either decimal (e.g., `#65;`) or hexadecimal (e.g., `#x41;`).
/// - Unknown or invalid tokens are mapped to `\0` due to the current parsing
///   strategy (mirroring the previous behavior).
/// - If the result would be empty or only contain '?' characters, no output is produced.
pub struct HTMLEntityDecoder;

impl Decoder for HTMLEntityDecoder {
    fn id(&self) -> DecoderId {
        "html_entity".to_string()
    }

    fn policy(&self) -> Policy {
        Policy {
            no_consecutive_same_op: true,
        }
    }

    fn apply(&self, input: &str) -> Vec<TransformResult> {
        let mut out = String::new();

        for entity in input.split('&') {
            let entity = entity.trim_start_matches('#');
            let entity = entity.trim_end_matches(';');

            if entity.starts_with('x') {
                // Hex form: #xNNNN;
                let hex = entity.strip_prefix('x').unwrap_or_default();
                let ch = char::from_u32(u32::from_str_radix(hex, 16).unwrap_or(0));
                out.push(ch.unwrap_or('?'));
            } else {
                // Decimal form: #NNNN;
                let dec = entity.parse::<u32>().unwrap_or(0);
                let ch = char::from_u32(dec);
                out.push(ch.unwrap_or('?'));
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
            },
        }]
    }
}
