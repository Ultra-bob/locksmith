use super::Scorer;

/// Heuristic scorer that detects URLs by simple pattern checks.
/// Scores higher when the input looks like a URL.
pub struct UrlScorer;

impl Scorer for UrlScorer {
    fn name(&self) -> &'static str {
        "url"
    }

    fn score(&self, input: &str) -> i32 {
        let mut score = 0;
        // Very common URL indicator.
        if input.contains("://") {
            score += 50;
        }
        // Recognize common schemes like http/https by substring.
        if input.contains("http") {
            score += 50;
        }
        score
    }
}
