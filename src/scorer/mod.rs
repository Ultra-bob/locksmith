use std::sync::Arc;

/// Trait for heuristic scoring of input strings.
/// Higher is better. Negative scores are allowed for penalties.
pub trait Scorer: Send + Sync {
    /// Stable, human-readable scorer name/category.
    fn name(&self) -> &'static str;

    /// Return an integer score for the given input.
    fn score(&self, input: &str) -> i32;
}

/// Engine that runs all registered scorers and aggregates their results.
pub struct ScoringEngine {
    scorers: Vec<Box<dyn Scorer + Send + Sync>>,
}

impl ScoringEngine {
    pub fn new() -> Self {
        Self {
            scorers: Vec::new(),
        }
    }

    /// Register a concrete scorer instance.
    pub fn register<S>(&mut self, scorer: S)
    where
        S: Scorer + Send + Sync + 'static,
    {
        self.scorers.push(Box::new(scorer));
    }

    /// Convenience for adding scorers as closures.
    pub fn register_fn<F>(&mut self, name: &'static str, f: F)
    where
        F: Fn(&str) -> i32 + Send + Sync + 'static,
    {
        self.register(FnScorer::new(name, f));
    }

    /// Score with all categories, returning (name, score) sorted by descending score.
    pub fn score_all(&self, input: &str) -> Vec<(&'static str, i32)> {
        let mut out: Vec<_> = self
            .scorers
            .iter()
            .map(|s| (s.name(), s.score(input)))
            .collect();
        out.sort_by(|a, b| b.1.cmp(&a.1));
        out
    }

    /// The highest-scoring category (if any).
    pub fn best(&self, input: &str) -> Option<(&'static str, i32)> {
        // Keep the previous quick paths for empty/short strings.
        if input.is_empty() {
            return Some(("Empty", 0));
        }
        if input.len() < 5 {
            return Some(("Short", 10));
        }
        self.scorers
            .iter()
            .map(|s| (s.name(), s.score(input)))
            .max_by_key(|(_, s)| *s)
    }

    /// Score many inputs, and return the one with the highest score.
    pub fn score_many(&self, inputs: &[&str]) -> Option<(&'static str, i32)> {
        let mut max_score: Option<(&'static str, i32)> = None;
        for input in inputs {
            if let Some((name, score)) = self.best(input) {
                match max_score {
                    Some((_, best)) if best >= score => {}
                    _ => max_score = Some((name, score)),
                }
            }
        }
        max_score
    }
}

/// A scorer implemented from a closure.
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

// ---------------------------------------------------------------------------
// Submodules for individual scorers. Each file defines one scorer struct
// (and any private helpers) and implements `Scorer` for it.
// ---------------------------------------------------------------------------

mod base64;
mod binary;
mod english;
mod english_structure;
mod english_text;
mod morse_code;
mod url;
mod youtube_url;

// Re-exports so existing `use scorer::XxxScorer` keeps working.
pub use base64::Base64Scorer;
pub use binary::BinaryScorer;
pub use english::EnglishScorer;
pub use english_structure::EnglishStructureScorer;
pub use english_text::EnglishTextScorer;
pub use morse_code::MorseCodeScorer;
pub use url::UrlScorer;
pub use youtube_url::YoutubeURLScorer;

// ---------------------------------------------------------------------------
// Default scorer set
// ---------------------------------------------------------------------------

/// Construct a `ScoringEngine` with all built-in scorers registered.
pub fn default_scorer() -> ScoringEngine {
    let mut engine = ScoringEngine::new();

    engine.register(UrlScorer);
    engine.register(YoutubeURLScorer);
    engine.register(EnglishScorer::new());
    engine.register(BinaryScorer);
    engine.register(Base64Scorer);
    engine.register(EnglishStructureScorer);
    engine.register(MorseCodeScorer);

    engine
}
