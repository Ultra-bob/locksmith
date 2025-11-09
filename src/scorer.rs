use std::{collections::HashSet, sync::Arc};

pub trait Scorer {
    // A stable, human-readable identifier for the category.
    fn name(&self) -> &'static str;
    // Return an integer score (e.g., 0..100). You can use negatives for penalties.
    fn score(&self, input: &str) -> i32;
}

// Engine that runs all registered scorers.
pub struct ScoringEngine {
    scorers: Vec<Box<dyn Scorer + Send + Sync>>,
}

impl ScoringEngine {
    pub fn new() -> Self {
        Self {
            scorers: Vec::new(),
        }
    }

    pub fn register<S>(&mut self, scorer: S)
    where
        S: Scorer + Send + Sync + 'static,
    {
        self.scorers.push(Box::new(scorer));
    }

    // Convenience for adding scorers as closures.
    pub fn register_fn<F>(&mut self, name: &'static str, f: F)
    where
        F: Fn(&str) -> i32 + Send + Sync + 'static,
    {
        self.register(FnScorer::new(name, f));
    }

    // Score with all categories, return (name, score) sorted by descending score.
    pub fn score_all(&self, input: &str) -> Vec<(&'static str, i32)> {
        let mut out: Vec<_> = self
            .scorers
            .iter()
            .map(|s| (s.name(), s.score(input)))
            .collect();
        out.sort_by(|a, b| b.1.cmp(&a.1));
        out
    }

    // The highest-scoring category (if any).
    pub fn best(&self, input: &str) -> Option<(&'static str, i32)> {
        // Empty input
        if input.is_empty() {
            return Some(("Empty", 0));
        }
        self.score_all(input).into_iter().max_by_key(|(_, s)| *s)
    }

    // Score many inputs, and return the one with the highest score
    pub fn score_many(&self, inputs: &[&str]) -> Option<(&'static str, i32)> {
        let mut max_score = None;
        for input in inputs {
            let score = self.best(input);
            if let Some((name, score)) = score {
                max_score = Some((name, score));
            }
        }
        max_score
    }
}

// A scorer implemented from a closure.
pub struct FnScorer {
    name: &'static str,
    f: Arc<dyn Fn(&str) -> i32 + Send + Sync>,
}

impl FnScorer {
    pub fn new<F>(name: &'static str, f: F) -> Self
    where
        F: Fn(&str) -> i32 + Send + Sync + 'static,
    {
        Self {
            name,
            f: Arc::new(f),
        }
    }
}

impl Scorer for FnScorer {
    fn name(&self) -> &'static str {
        self.name
    }
    fn score(&self, input: &str) -> i32 {
        (self.f)(input)
    }
}

// Built-in: URL scorer.
pub struct UrlScorer;

impl Scorer for UrlScorer {
    fn name(&self) -> &'static str {
        "url"
    }

    fn score(&self, input: &str) -> i32 {
        let mut score = 0;
        if input.contains("://") {
            score += 50
        }
        if input.contains("http") {
            score += 50
        }

        score
    }
}

pub struct EnglishScorer {
    wordlist: HashSet<String>,
}

impl EnglishScorer {
    pub fn new() -> Self {
        // Load the wordlist from the unix words file
        let wordlist = std::fs::read_to_string("/usr/share/dict/words")
            .expect("Failed to read wordlist")
            .lines()
            .map(|line| line.to_string())
            .filter(|line| !line.is_empty() && line.len() > 3) // Filter short words.
            .collect();

        EnglishScorer { wordlist }
    }

    pub fn new_with_wordlist(wordlist: HashSet<String>) -> Self {
        EnglishScorer { wordlist }
    }
}

impl Scorer for EnglishScorer {
    fn name(&self) -> &'static str {
        "English Text"
    }

    fn score(&self, input: &str) -> i32 {
        // Normalize to lowercase for case-insensitive matching
        let s = input.to_lowercase();

        // Character boundary positions -> byte indices
        let positions: Vec<usize> = s
            .char_indices()
            .map(|(i, _)| i)
            .chain(std::iter::once(s.len()))
            .collect();
        let n_chars = positions.len() - 1;
        let n_non_whitespace = s.chars().filter(|c| !c.is_whitespace()).count();

        // Compute min/max dictionary word lengths in chars
        let mut min_w = usize::MAX;
        let mut max_w = 0usize;
        for w in &self.wordlist {
            let len = w.chars().count();
            if len == 0 {
                continue;
            }
            min_w = min_w.min(len);
            max_w = max_w.max(len);
        }
        if max_w == 0 {
            return 0;
        }

        // dp[i] = max covered chars in the first i chars (0..i)
        let mut dp = vec![0usize; n_chars + 1];

        for i in 0..n_chars {
            // Option 1: skip this char
            if dp[i + 1] < dp[i] {
                dp[i + 1] = dp[i];
            }

            // Option 2: take any dictionary word starting at i
            let mut len = min_w;
            while len <= max_w {
                let j = i + len;
                if j > n_chars {
                    break;
                }
                let sub = &s[positions[i]..positions[j]];
                if self.wordlist.contains(sub) {
                    dp[j] = dp[j].max(dp[i] + len);
                }
                len += 1;
            }
        }

        ((dp[n_chars] as f64) / (n_non_whitespace as f64) * 100.0) as i32
    }
}

pub struct YoutubeURLScorer;

impl Scorer for YoutubeURLScorer {
    fn name(&self) -> &'static str {
        "Youtube URL"
    }

    fn score(&self, input: &str) -> i32 {
        if input.starts_with("https://www.youtube.com/watch?v=") {
            return 1000; // Basically certain
        }
        if input.starts_with("https://youtu.be/") {
            return 950; // Very likely
        }
        let valid = input
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_".contains(c));
        if valid && input.len() == 11 {
            return 100; // Decent
        }
        if valid {
            return 10; // Terrible
        }
        0
    }
}

pub fn default_scorer() -> ScoringEngine {
    let mut engine = ScoringEngine::new();

    // Register built-ins.
    engine.register(UrlScorer);
    engine.register(YoutubeURLScorer);
    engine.register(EnglishScorer::new());
    engine.register(BinaryScorer);
    engine.register(Base64Scorer);
    engine.register(EnglishStructureScorer);
    engine.register(MorseCodeScorer);

    engine
}

pub struct BinaryScorer;

impl Scorer for BinaryScorer {
    fn name(&self) -> &'static str {
        "Binary Data"
    }
    fn score(&self, input: &str) -> i32 {
        let valid = input.chars().all(|c| c.is_whitespace() || "01".contains(c));
        if valid {
            return 40; // Good, but should be further decoded
        }
        0
    }
}

pub struct Base64Scorer;

impl Scorer for Base64Scorer {
    fn name(&self) -> &'static str {
        "Base64 Data"
    }
    fn score(&self, input: &str) -> i32 {
        if input.is_empty() {
            return 0;
        }
        let valid = input
            .chars()
            .all(|c| c.is_whitespace() || c.is_ascii_alphanumeric() || "+/=".contains(c));
        if valid && input.chars().last().expect("Input should not be empty") == '=' {
            return 30; // Ending with '=' is good, but should be further decoded
        }
        if valid {
            return 20; // Alright, but should be further decoded
        }
        0
    }
}

pub struct EnglishStructureScorer;

impl Scorer for EnglishStructureScorer {
    fn name(&self) -> &'static str {
        "English Text Structure"
    }
    fn score(&self, input: &str) -> i32 {
        // Calculate % of spaces
        let spaces = input.chars().filter(|c| c.is_whitespace()).count();
        let total_chars = input.chars().count();
        let percentage = (spaces as f32 / total_chars as f32) * 100.0;
        let diff_from_expected = (percentage - 30.0).abs();
        // dbg!(diff_from_expected);
        if diff_from_expected < 15.0 {
            30 // Good structure
        } else {
            10 // Poor structure
        }
    }
}

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
            return 50; // Good, but should be further decoded
        }
        0
    }
}

pub struct MorseCodeScorer;

impl Scorer for MorseCodeScorer {
    fn name(&self) -> &'static str {
        "Morse Code"
    }
    fn score(&self, input: &str) -> i32 {
        let valid = input
            .chars()
            .all(|c| c.is_whitespace() || c == '.' || c == '-' || c == '/');
        if valid {
            return 50; // Good, but should be further decoded
        }
        0
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    fn assert_best(engine: &ScoringEngine, input: &str, expected: &str) {
        assert_eq!(engine.best(input).unwrap().0, expected)
    }

    #[test]
    fn test_default_scorer() {
        let engine = default_scorer();
        assert_best(
            &engine,
            "https://www.youtube.com/watch?v=U8DHPd4dAl0",
            "Youtube URL",
        );
        assert_best(&engine, "https://youtu.be/U8DHPd4dAl0", "Youtube URL");

        assert_best(&engine, "regular words", "English Text");

        assert_best(&engine, "cmVndWxhciB3b3Jkcw==", "Base64 Data");

        assert_best(&engine, "110110110 100010101", "Binary Data");

        assert_best(
            &engine,
            "wnzvr naq nouv frrz gb or serr sbe c3 fb ubcrshyyl gurl pna gnxr gubfr fuvsgf",
            "English Text Structure",
        );
    }
}
