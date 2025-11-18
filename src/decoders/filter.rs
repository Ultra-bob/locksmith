use crate::engine::{Decoder, DecoderId, Policy, Step, TransformResult};

pub struct Filter;

impl Decoder for Filter {
    fn id(&self) -> DecoderId {
        String::from("filter")
    }

    fn policy(&self) -> Policy {
        Policy {
            no_consecutive_same_op: true,
        }
    }

    fn apply(&self, input: &str) -> Vec<TransformResult> {
        vec![
            TransformResult {
                output: input.chars().filter(|c| c.is_alphabetic()).collect(),
                step: Step {
                    op_id: self.id(),
                    desc: String::from("Remove non-alphabetic"),
                },
            },
            TransformResult {
                output: input.chars().filter(|c| c.is_alphanumeric()).collect(),
                step: Step {
                    op_id: self.id(),
                    desc: String::from("Remove non-alphanumeric"),
                },
            },
            TransformResult {
                output: input.chars().filter(|c| c.is_numeric()).collect(),
                step: Step {
                    op_id: self.id(),
                    desc: String::from("Remove non-numeric"),
                },
            },
            TransformResult {
                output: input.chars().filter(|c| c.is_whitespace()).collect(),
                step: Step {
                    op_id: self.id(),
                    desc: String::from("Remove whitespace"),
                },
            },
        ]
    }
}
