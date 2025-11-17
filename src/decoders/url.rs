use crate::engine::{Decoder, DecoderId, Step, TransformResult};

/// Decoder that attempts to URL-decode the input string.
///
/// Behavior:
/// - Uses `urlencoding::decode` to percent-decode.
/// - After decoding, replaces '+' with a space to mimic
///   application/x-www-form-urlencoded-style decoding.
pub struct URLDecoder;

impl Decoder for URLDecoder {
    fn id(&self) -> DecoderId {
        "url".to_string()
    }

    fn group(&self) -> &'static str {
        "url"
    }

    fn apply(&self, input: &str) -> Vec<TransformResult> {
        match urlencoding::decode(input) {
            Ok(decoded) => vec![TransformResult {
                output: decoded.to_string().replace('+', " "),
                step: Step {
                    op_id: self.id(),
                    desc: "Decode URL".to_string(),
                },
            }],
            Err(_) => vec![],
        }
    }
}
