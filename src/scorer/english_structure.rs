use super::Scorer;

/// Scores text based on its whitespace structure, favoring inputs where the
/// percentage of whitespace is close to typical English (~30%).
///
/// This is a coarse heuristic intended to help rank candidates; it does not
/// validate dictionary words or grammar.
pub struct EnglishStructureScorer;

impl Scorer for EnglishStructureScorer {
    fn name(&self) -> &'static str {
        "English Text Structure"
    }

    fn score(&self, input: &str) -> i32 {
        let total_chars = input.chars().count();
        if total_chars == 0 {
            return 0;
        }

        let spaces = input.chars().filter(|c| c.is_whitespace()).count();
        let percentage = (spaces as f32 / total_chars as f32) * 100.0;
        let diff_from_expected = (percentage - 30.0).abs();

        if diff_from_expected < 15.0 {
            30 // Good structure
        } else {
            10 // Poor structure
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EnglishStructureScorer, Scorer};

    #[test]
    fn scores_empty_as_zero() {
        let s = EnglishStructureScorer;
        assert_eq!(s.score(""), 0);
    }

    #[test]
    fn scores_space_heavy_text_reasonably() {
        let s = EnglishStructureScorer;
        // About 33% spaces (3 of 9)
        let input = "word word ";
        assert!(s.score(input) >= 10);
    }
}
