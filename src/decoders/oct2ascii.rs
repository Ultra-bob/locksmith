use crate::engine::{Decoder, DecoderId, Policy, Step, TransformResult};

pub struct Oct2AsciiDecoder;

impl Decoder for Oct2AsciiDecoder {
    fn id(&self) -> DecoderId {
        String::from("oct2ascii")
    }

    fn apply(&self, input: &str) -> Vec<TransformResult> {
        let mut chars = input.chars().peekable();
        let mut decoded = String::new();

        while let Some(&c) = chars.peek() {
            if !c.is_ascii_digit() {
                chars.next(); // Skip non-digits
                continue;
            }

            // Determine length: if starts with '1', take 3, else take 2
            let len = if c == '1' { 3 } else { 2 };

            // Collect 'len' characters into a temporary string
            let num_str: String = (0..len)
                .filter_map(|_| chars.next()) // consuming items
                .collect();

            // Parse and push
            if let Ok(code) = u32::from_str_radix(&num_str, 8) {
                if let Some(ch) = std::char::from_u32(code) {
                    decoded.push(ch);
                }
            }
        }

        return vec![TransformResult {
            output: decoded,
            step: Step {
                op_id: self.id(),
                desc: String::from("Decimal to ASCII"),
            },
        }];
    }
}
