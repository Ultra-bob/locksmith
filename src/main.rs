mod decoders;
mod engine;
mod scorer;
mod search;

use decoders::register_all;
use search::{SearchConfig, explore};

fn main() {
    // Build decoder engine and register available decoders.
    let mut dec_engine = engine::DecoderEngine::new();
    register_all(&mut dec_engine);

    // Build scoring engine.
    let scorer = scorer::default_scorer();

    // Example input (Base64 then Caesar or vice versa scenarios are now naturally explored).
    let input = "lRGdmR8rK3G5nh==";

    // Configure search: explore up to 3 steps deep, keep best 100 candidates per depth,
    // and avoid revisiting identical output texts.
    let cfg = SearchConfig {
        max_depth: 3,
        beam_width: Some(100),
        dedup_on_text: true,
    };

    // Run the exploration.
    let results = explore(&dec_engine, &scorer, input, cfg);

    // Print the top 10 results by score.
    for r in results.iter().take(10) {
        let steps = if r.steps.is_empty() {
            "<none>".to_string()
        } else {
            r.steps
                .iter()
                .map(|s| s.desc.as_str())
                .collect::<Vec<_>>()
                .join(" -> ")
        };

        println!(
            "[score: {score:>4}] [{cat}] {text}\n  steps: {steps}",
            score = r.score,
            cat = r.detected_as,
            text = r.text,
            steps = steps
        );
    }
}
