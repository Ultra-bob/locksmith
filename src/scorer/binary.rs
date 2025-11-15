use super::Scorer;

/// Heuristic scorer for inputs that look like binary (0/1) data.
/// Whitespace is allowed and ignored for validation purposes.
pub struct BinaryScorer;

impl Scorer for BinaryScorer {
    fn name(&self) -> &'static str {
        "Binary Data"
    }

    fn score(&self, input: &str) -> i32 {
        let valid = input
            .chars()
            .all(|c| c.is_whitespace() || c == '0' || c == '1');

        if valid {
            // Looks like binary text; decent hint but likely needs further decoding.
            40
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scores_binary_as_positive() {
        let s = BinaryScorer;
        assert!(s.score("01010101 11110000") > 0);
        assert!(s.score("1010\t0101\n0000") > 0);
    }

    #[test]
    fn scores_non_binary_as_zero() {
        let s = BinaryScorer;
        assert_eq!(s.score("01020101"), 0);
        assert_eq!(s.score("not binary"), 0);
    }
}
