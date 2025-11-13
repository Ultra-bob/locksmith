use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::engine::{DecoderEngine, Step};
use crate::scorer::ScoringEngine;

/// A decoded candidate along a chain of transformation steps.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chain {
    pub text: String,
    pub steps: Vec<Step>,
    pub score: i32,
    pub detected_as: String,
}

/// Configuration for the search through decoder expansions.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Maximum number of transformation steps to apply.
    pub max_depth: usize,
    /// Keep at most this many best-scoring nodes per depth. `None` = no pruning.
    pub beam_width: Option<usize>,
    /// If true, do not revisit the exact same output text again.
    pub dedup_on_text: bool,
}

/// Explore the search space of transformations using layered BFS with optional beam pruning.
///
/// Strategy:
/// - Start from the original input (depth 0) and score it.
/// - For each depth:
///   - Optionally beam-prune the current layer by score.
///   - Record the nodes in the results.
///   - If not at max depth, expand each node through the decoder engine.
///   - Compute scores of children, apply dedup if enabled, and form the next layer.
/// - Return all seen nodes, sorted by descending score.
pub fn explore(
    engine: &DecoderEngine,
    scorer: &ScoringEngine,
    input: &str,
    cfg: SearchConfig,
) -> Vec<Chain> {
    let mut results: Vec<Chain> = Vec::new();
    let mut seen_texts: HashSet<String> = HashSet::new();

    // Seed layer (depth 0)
    let (cat, score) = scorer.best(input).unwrap_or(("unknown", 0));
    let mut current_layer: Vec<Chain> = vec![Chain {
        text: input.to_string(),
        steps: vec![],
        score,
        detected_as: cat.to_string(),
    }];
    if cfg.dedup_on_text {
        seen_texts.insert(input.to_string());
    }

    for depth in 0..=cfg.max_depth {
        // Beam prune (keep top-K by score) if requested
        if let Some(k) = cfg.beam_width {
            current_layer.sort_by(|a, b| b.score.cmp(&a.score));
            if current_layer.len() > k {
                current_layer.truncate(k);
            }
        }

        // Record this layer's nodes as results
        results.extend(current_layer.iter().cloned());

        // Stop expanding at the maximum depth
        if depth == cfg.max_depth {
            break;
        }

        // Expand to next layer
        let mut next_layer: Vec<Chain> = Vec::new();

        for node in current_layer.into_iter() {
            let expansions = engine.expand(&node.text, &node.steps);
            for tr in expansions {
                if cfg.dedup_on_text {
                    if !seen_texts.insert(tr.output.clone()) {
                        // Already seen this text, skip re-exploration
                        continue;
                    }
                }

                let (cat, score) = scorer.best(&tr.output).unwrap_or(("unknown", 0));
                let mut steps = node.steps.clone();
                steps.push(tr.step);

                next_layer.push(Chain {
                    text: tr.output,
                    steps,
                    score,
                    detected_as: cat.to_string(),
                });
            }
        }

        current_layer = next_layer;
    }

    // Return all candidates, highest score first
    results.sort_by(|a, b| b.score.cmp(&a.score));
    results
}
