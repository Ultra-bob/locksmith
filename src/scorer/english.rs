use crate::scorer::EnglishStructureScorer;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use super::Scorer;

/// Trie node used for dictionary word matching.
struct TrieNode {
    children: HashMap<char, usize>,
    terminal: bool,
}

/// Scorer that estimates how much of the input can be segmented into
/// dictionary words using a trie and dynamic programming.
///
/// The score is the percentage of non-whitespace characters covered by
/// dictionary words (0..=100).
pub struct EnglishScorer {
    trie: Vec<TrieNode>,
    max_word_len: usize,       // maximum word length (in chars)
    dp_buf: Mutex<Vec<usize>>, // reused DP buffer
    structure: EnglishStructureScorer,
}

impl EnglishScorer {
    /// Build the scorer from any iterator of words.
    fn from_words<I, S>(words: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut trie = vec![TrieNode {
            children: HashMap::new(),
            terminal: false,
        }];
        let mut max_word_len = 0usize;

        for w in words {
            let w = w.as_ref().to_lowercase(); // normalize once
            if w.is_empty() {
                continue;
            }

            let mut node_idx = 0usize;
            let mut len_chars = 0usize;

            for ch in w.chars() {
                len_chars += 1;

                // Avoid borrow checker pitfalls by not using `or_insert_with` capturing `trie`.
                let next_idx = match trie[node_idx].children.get(&ch) {
                    Some(&idx) => idx,
                    None => {
                        let new_idx = trie.len();
                        trie.push(TrieNode {
                            children: HashMap::new(),
                            terminal: false,
                        });
                        trie[node_idx].children.insert(ch, new_idx);
                        new_idx
                    }
                };

                node_idx = next_idx;
            }

            trie[node_idx].terminal = true;
            max_word_len = max_word_len.max(len_chars);
        }

        EnglishScorer {
            trie,
            max_word_len,
            dp_buf: Mutex::new(Vec::new()),
            structure: EnglishStructureScorer,
        }
    }

    /// Load from `/usr/share/dict/words`, filtering out short words (<=3 chars).
    ///
    /// Note: This will panic if the file cannot be read.
    pub fn new() -> Self {
        let wordlist: HashSet<String> = std::fs::read_to_string("/usr/share/dict/words")
            .expect("Failed to read wordlist")
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty() && line.len() > 3)
            .collect();

        Self::new_with_wordlist(wordlist)
    }

    /// Construct from an existing set of words.
    pub fn new_with_wordlist(wordlist: HashSet<String>) -> Self {
        Self::from_words(wordlist.into_iter())
    }
}

impl Scorer for EnglishScorer {
    fn name(&self) -> &'static str {
        "English Text"
    }

    fn score(&self, input: &str) -> i32 {
        if self.max_word_len == 0 {
            return 0;
        }

        // Lowercase once for case-insensitive matching.
        let lower = input.to_lowercase();

        // Build a char vector and count non-whitespace characters.
        let mut chars = Vec::with_capacity(lower.chars().count());
        let mut n_non_whitespace = 0usize;
        for ch in lower.chars() {
            if !ch.is_whitespace() {
                n_non_whitespace += 1;
            }
            chars.push(ch);
        }

        if n_non_whitespace == 0 {
            return 0;
        }

        let n = chars.len();

        // Reuse DP buffer (thread-safe).
        let mut dp_guard = self.dp_buf.lock().unwrap();
        if dp_guard.len() < n + 1 {
            dp_guard.resize(n + 1, 0);
        } else {
            for v in &mut dp_guard[..=n] {
                *v = 0;
            }
        }
        let dp = &mut *dp_guard;

        // dp[i] = max covered chars among the first i chars (0..i)
        for i in 0..n {
            // Option 1: skip this char
            if dp[i + 1] < dp[i] {
                dp[i + 1] = dp[i];
            }

            // Option 2: follow trie starting at position i
            let mut node_idx = 0usize;
            for (offset, &ch) in chars[i..].iter().enumerate() {
                if let Some(&next_idx) = self.trie[node_idx].children.get(&ch) {
                    node_idx = next_idx;

                    if self.trie[node_idx].terminal {
                        let word_len = offset + 1; // in chars
                        let j = i + word_len;
                        let covered = dp[i] + word_len;
                        if j <= n && dp[j] < covered {
                            dp[j] = covered;
                        }
                    }
                } else {
                    break;
                }
            }
        }

        let covered = dp[n] as f64;
        let score = covered / (n_non_whitespace as f64) * 100.0;
        score as i32 + self.structure.score(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score() {
        let scorer = EnglishScorer::new();
        let score = scorer.score("Hello, world!");
        assert_eq!(score, 100);
    }
}
