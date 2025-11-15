use super::Scorer;

/// Heuristic scorer for inputs that resemble Morse code.
/// Accepts only '.', '-', '/', and whitespace. Requires a minimal length to
/// avoid short accidental matches.
pub struct MorseCodeScorer;

impl Scorer for MorseCodeScorer {
    fn name(&self) -> &'static str {
        "Morse Code"
    }

    fn score(&self, input: &str) -> i32 {
        let valid = input
            .chars()
            .all(|c| c.is_whitespace() || c == '.' || c == '-' || c == '/');

        if valid && input.len() > 10 {
            // Good signal, but likely needs decoding
            50
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MorseCodeScorer, Scorer};

    #[test]
    fn scores_morse_like_text_positive() {
        let s = MorseCodeScorer;
        // "HELLO WORLD" in Morse with separators
        let input = ".... . .-.. .-.. --- / .-- --- .-. .-.. -..";
        assert!(s.score(input) > 0);
    }

    #[test]
    fn scores_non_morse_as_zero() {
        let s = MorseCodeScorer;
        assert_eq!(s.score("..-.- not morse ..-.-"), 0);
        assert_eq!(s.score(""), 0);
        // Short but valid chars should be below the length threshold
        assert_eq!(s.score("... --- ..."), 0); // "SOS" but too short by heuristic
    }
}
