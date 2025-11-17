use crate::engine::{Decoder, DecoderId, Policy, Step, TransformResult};

/// Decoder that reverses the input string.
pub struct ReverseDecoder;

impl Decoder for ReverseDecoder {
    fn id(&self) -> DecoderId {
        "reverse".to_string()
    }

    fn policy(&self) -> Policy {
        Policy {
            no_consecutive_same_op: true,
        }
    }

    fn apply(&self, input: &str) -> Vec<TransformResult> {
        vec![TransformResult {
            output: input.chars().rev().collect(),
            step: Step {
                op_id: self.id(),
                desc: "Reverse".to_string(),
            },
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverses_simple_text() {
        let d = ReverseDecoder;
        let res = d.apply("abcd");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].output, "dcba");
    }

    #[test]
    fn reverses_unicode() {
        let d = ReverseDecoder;
        let res = d.apply("hé🍊");
        assert_eq!(res[0].output, "🍊éh");
    }
}
