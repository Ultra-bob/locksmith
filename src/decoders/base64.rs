use crate::engine::{Decoder, DecoderId, Policy, Step, TransformResult};

/// Decoder for Base64-encoded data.
pub struct Base64Decoder;

impl Decoder for Base64Decoder {
    fn id(&self) -> DecoderId {
        "base64".to_string()
    }

    fn group(&self) -> &'static str {
        "radix64"
    }

    fn policy(&self) -> Policy {
        // Prevent immediate repeated base64 decodes by default.
        // Adjust `no_group_repeat_within` if you want to allow chains like base64->base64.
        Policy {
            no_consecutive_same_op: false,
            no_group_repeat_within: 0,
        }
    }

    fn apply(&self, input: &str) -> Vec<TransformResult> {
        // Strip trailing '=' padding characters for leniency
        let stripped = input.trim().trim_end_matches('=');

        let try_decode = |s: &str| {
            fast32::base64::RFC4648_NOPAD
                .decode(s.as_bytes())
                .or_else(|_| fast32::base64::RFC4648_URL_NOPAD.decode(s.as_bytes()))
        };

        match try_decode(stripped) {
            Ok(decoded) => {
                let output = String::from_utf8_lossy(&decoded).into_owned();
                // Filter out lossy conversions (presence of the UTF-8 replacement char).
                if output.contains('\u{FFFD}') {
                    return vec![];
                }
                vec![TransformResult {
                    output,
                    step: Step {
                        op_id: self.id(),
                        desc: "Base64 decode".to_string(),
                        group: self.group().to_string(),
                    },
                }]
            }
            Err(_) => vec![],
        }
    }
}
