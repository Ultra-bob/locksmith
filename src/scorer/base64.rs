use super::Scorer;

/// Heuristic scorer for Base64-looking data.
/// This does not validate padding thoroughly; it's meant as a lightweight signal.
pub struct Base64Scorer;

impl Scorer for Base64Scorer {
    fn name(&self) -> &'static str {
        "Base64 Data"
    }

    fn score(&self, input: &str) -> i32 {
        if input.is_empty() {
            return 0;
        }

        // Accept typical Base64 charset plus whitespace.
        let valid = input
            .chars()
            .all(|c| c.is_whitespace() || c.is_ascii_alphanumeric() || "+/=".contains(c));

        if valid && input.chars().last().unwrap() == '=' {
            // Ending with '=' is a common Base64 padding indicator.
            30
        } else if valid {
            // Looks like Base64 but no padding at the end.
            20
        } else {
            0
        }
    }
}
