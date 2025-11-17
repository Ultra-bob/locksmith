use crate::engine::{Decoder, DecoderId, Policy, Step, TransformResult};

/// Decoder for Base58-encoded data.
///
/// Uses the `bs58` crate to decode the input. On success, returns a single
/// `TransformResult` with the decoded bytes interpreted as UTF-8 (lossy).
pub struct Base58Decoder;

impl Decoder for Base58Decoder {
    fn id(&self) -> DecoderId {
        "base58".to_string()
    }

    fn policy(&self) -> Policy {
        // Allow repeated Base58 decodes if desired by exploration logic.
        Policy {
            no_consecutive_same_op: false,
        }
    }

    fn apply(&self, input: &str) -> Vec<TransformResult> {
        match bs58::decode(input).into_vec() {
            Ok(decoded) => {
                let output = String::from_utf8_lossy(&decoded).into_owned();
                vec![TransformResult {
                    output,
                    step: Step {
                        op_id: self.id(),
                        desc: "Base58 decode".to_string(),
                    },
                }]
            }
            Err(_) => vec![],
        }
    }
}
