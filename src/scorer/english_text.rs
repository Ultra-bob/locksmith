use super::Scorer;

/// Lightweight heuristic that scores inputs which look like plain English text
/// comprised only of ASCII alphanumeric characters and whitespace.
///
/// This is a permissive signal and does not try to validate dictionary words
/// or punctuation; it simply checks the character classes.
pub struct EnglishTextScorer;

impl Scorer for EnglishTextScorer {
    fn name(&self) -> &'static str {
        "English Text"
    }

    fn score(&self, input: &str) -> i32 {
        let valid = input
            .chars()
            .all(|c| c.is_whitespace() || c.is_ascii_alphanumeric());
        if valid {
            // Good, but likely needs further decoding/validation.
            50
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EnglishTextScorer;
    use super::Scorer;

    #[test]
    fn scores_alnum_whitespace_as_positive() {
        let s = EnglishTextScorer;
        assert!(s.score("hello world 123") > 0);
        assert!(s.score("ThisIsATest") > 0);
        assert!(s.score("   \n\t") > 0); // whitespace-only still passes the predicate
    }

    #[test]
    fn scores_with_punctuation_as_zero() {
        let s = EnglishTextScorer;
        assert_eq!(s.score("hello, world!"), 0);
        assert_eq!(s.score("email@example.com"), 0);
    }
}
