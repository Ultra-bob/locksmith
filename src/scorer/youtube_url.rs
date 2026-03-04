use super::Scorer;

/// Scores inputs that look like YouTube video URLs or IDs.
pub struct YoutubeURLScorer;

impl Scorer for YoutubeURLScorer {
    fn name(&self) -> &'static str {
        "Youtube URL"
    }

    fn score(&self, input: &str) -> i32 {
        // Strong signals: full YouTube watch URLs.
        if input.starts_with("https://www.youtube.com/watch?v=") {
            return 1000; // Basically certain
        }
        if input.starts_with("https://youtu.be/") {
            return 950; // Very likely
        }

        // Heuristic: a bare 11-char video ID consisting of allowed chars.
        let valid = input
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_".contains(c));
        if valid && input.len() == 11 {
            return 30; // Reasonable guess
        }
        if valid {
            return 10; // Weak signal
        }
        0
    }
}
