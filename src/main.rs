use leptos::prelude::*;

use leptos::mount::mount_to_body;

use wasm_bindgen::JsCast;

use wasm_bindgen_futures::spawn_local;

use std::collections::HashSet;

mod decoders;
mod engine;
mod scorer;
mod search;

use decoders::register_all;
use search::{Chain, SearchConfig, explore};

fn event_target_value(ev: &leptos::ev::Event) -> String {
    if let Some(target) = ev.target() {
        if let Some(input) = target.dyn_ref::<web_sys::HtmlInputElement>() {
            return input.value();
        }
        if let Some(textarea) = target.dyn_ref::<web_sys::HtmlTextAreaElement>() {
            return textarea.value();
        }
    }
    String::new()
}

fn build_scorer(wordlist: Option<HashSet<String>>) -> scorer::ScoringEngine {
    // Avoid filesystem-based scorers in WASM
    let mut engine = scorer::ScoringEngine::new();
    engine.register(scorer::UrlScorer);
    engine.register(scorer::YoutubeURLScorer);
    engine.register(scorer::BinaryScorer);
    engine.register(scorer::Base64Scorer);
    engine.register(scorer::EnglishStructureScorer);
    engine.register(scorer::MorseCodeScorer);
    if let Some(set) = wordlist {
        engine.register(scorer::EnglishScorer::new_with_wordlist(set));
    } else {
        engine.register(scorer::EnglishTextScorer);
    }

    engine
}

async fn fetch_wordlist_from_url(url: &str) -> Option<HashSet<String>> {
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Response, window};

    let win = window()?;
    // fetch(url) -> Promise<Response>
    let resp: Response = JsFuture::from(win.fetch_with_str(url))
        .await
        .ok()?
        .dyn_into()
        .ok()?;

    if !resp.ok() {
        return None;
    }

    // Response.text() -> Promise<string>
    let text_promise = resp.text().ok()?;
    let text_js = JsFuture::from(text_promise).await.ok()?;
    let text = text_js.as_string()?;

    let set: HashSet<String> = text
        .lines()
        .map(|w| w.trim().to_lowercase())
        .filter(|w| !w.is_empty() && w.len() > 3)
        .collect();

    Some(set)
}

#[component]
fn App() -> impl IntoView {
    // Form fields
    let (input_text, set_input_text) = signal(String::new());
    let (depth_text, set_depth_text) = signal(String::from("4"));
    let (beam_text, set_beam_text) = signal(String::from("1000"));

    // Results
    let (results, set_results) = signal::<Vec<Chain>>(vec![]);

    // Wordlist: fetched at runtime in WASM; None until loaded
    let (wordlist, set_wordlist) = signal::<Option<HashSet<String>>>(None);

    {
        // Change to your hosted wordlist URL (must allow CORS when cross-origin)
        const WORDLIST_URL: &str =
            "https://raw.githubusercontent.com/dwyl/english-words/refs/heads/master/words.txt";

        Effect::new(move |_| {
            spawn_local(async move {
                if let Some(set) = fetch_wordlist_from_url(WORDLIST_URL).await {
                    set_wordlist.set(Some(set));
                }
            });
        });
    }

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let max_depth: usize = depth_text.get_untracked().trim().parse().unwrap_or(4);
        let beam_opt: Option<usize> = {
            let s = beam_text.get_untracked();
            let s = s.trim();
            if s.is_empty() {
                None
            } else {
                s.parse::<usize>().ok()
            }
        };

        let mut dec_engine = engine::DecoderEngine::new();
        register_all(&mut dec_engine);
        let scorer = build_scorer(wordlist.get_untracked().clone());

        let cfg = SearchConfig {
            max_depth,
            beam_width: beam_opt,
            dedup_on_text: true,
        };

        let input = input_text.get_untracked();
        let res = explore(&dec_engine, &scorer, &input, cfg);
        set_results.set(res);
    };

    view! {
        <main style="max-width: 960px; margin: 2rem auto; padding: 0 1rem; font-family: system-ui, -apple-system, Segoe UI, Roboto, Ubuntu, Cantarell, Noto Sans, Helvetica, Arial;">
            <h1 style="margin: 0 0 1rem 0;">"Locksmith"</h1>

            <div style="display: flex; gap: 1rem; align-items: flex-start; flex-wrap: wrap;">
                <div style="flex: 1; min-width: 280px; max-width: 420px;">
                    <form on:submit=on_submit style="display: grid; gap: .75rem;">
                <label style="display: grid; gap: .25rem;">
                    <span style="font-weight: 600;">"Input"</span>
                    <textarea
                        placeholder="Paste encoded text here…"
                        rows="6"
                        on:input=move |ev| set_input_text.set(event_target_value(&ev))
                        style="width: 100%; padding: .5rem; border-radius: .25rem; border: 1px solid #ccc; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;"
                    />
                </label>

                <div style="display: flex; gap: .75rem; flex-wrap: wrap;">
                    <label style="display: grid; gap: .25rem;">
                        <span style="font-weight: 600;">"Depth: "<span style="font-weight: 400;">{move || depth_text.get()}</span></span>
                        <input
                            type="range"
                            min="0"
                            max="5"
                            placeholder="e.g. 4"
                            value=move || depth_text.get()
                            on:input=move |ev| set_depth_text.set(event_target_value(&ev))
                            style="padding: .4rem; border: 1px solid #ccc; border-radius: .25rem; width: 12ch;"
                        />
                    </label>

                    <label style="display: grid; gap: .25rem;">
                        <span style="font-weight: 600;">"Beam Width"</span>
                        <input
                            type="number"
                            min="0"
                            placeholder="e.g. 1000"
                            value=move || beam_text.get()
                            on:input=move |ev| set_beam_text.set(event_target_value(&ev))
                            style="padding: .4rem; border: 1px solid #ccc; border-radius: .25rem; width: 16ch;"
                        />
                    </label>
                </div>

                <div>
                    <button
                        type="submit"
                        style="padding: .5rem .9rem; border: 1px solid #3b82f6; background: #3b82f6; color: white; border-radius: .35rem; cursor: pointer;"
                    >
                        "Decode"
                    </button>
                </div>
            </form>
                </div>

                <div style="flex: 2; min-width: 340px;">

            <section style="border: 1px solid #ccc; border-radius: .5rem; padding: 1rem; background: #fafafa;">
                <Show
                    when=move || !results.get().is_empty()
                    fallback=move || view! {
                        <div>
                            <h2 style="margin: 0 0 .5rem 0;">"Best Result"</h2>
                            <div style="color: #666;"><em>"Run a search to see results."</em></div>
                        </div>
                    }
                >
                    {move || {
                        let res = results.get();
                        let best = res[0].clone();
                        let mut path = String::from("Input");
                        if !best.steps.is_empty() {
                            for s in &best.steps {
                                path.push_str(" -> ");
                                path.push_str(&format!("{} ({})", s.desc, s.op_id));
                            }
                        } else {
                            path.push_str(" (no transforms)");
                        }
                        view! {
                            <div>
                                <h2 style="margin: 0 0 .5rem 0;">"Best Result"</h2>
                                <div style="font-size: .9rem; color: #444; margin-bottom: .5rem;">
                                    <strong>"Score: "</strong>{best.score}
                                    " · "
                                    <strong>"Detected as: "</strong>{best.detected_as}
                                </div>
                                <div style="white-space: pre-wrap; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace; background: #fff; border: 1px solid #ddd; border-radius: .25rem; padding: .5rem; max-height: 12rem; overflow: auto;">
                                    {best.text}
                                </div>
                                <h3 style="margin: 1rem 0 .5rem 0; font-size: 1rem; color: #222;">"Path to this decoding"</h3>
                                <div style="font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;">
                                    {path}
                                </div>
                            </div>
                        }
                    }}
                </Show>
            </section>

            <section style="margin-top: 1.5rem;">
                <h2 style="margin: 0 0 .5rem 0; font-size: 1.15rem;">"Other Top 5 Results"</h2>
                <Show
                    when=move || { results.get().len() > 1 }
                    fallback=move || {
                        view! {
                        <div>
                            <pre style="white-space: pre-wrap; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace; background: #fff; border: 1px solid #e2e2e2; border-radius: .5rem; padding: .75rem; color: #333;">
                                "No additional top results yet."
                            </pre>
                        </div>
                        }
                    }
                >
                    {move || {
                        let res = results.get();
                        let mut buf = String::new();
                        let take_n = usize::min(5, res.len().saturating_sub(1));
                        for (i, c) in res.into_iter().skip(1).take(take_n).enumerate() {
                            let steps_label = if c.steps.is_empty() {
                                "<none>".to_string()
                            } else {
                                c.steps.iter().map(|s| s.desc.as_str()).collect::<Vec<_>>().join(" -> ")
                            };
                            let preview = if c.text.len() > 240 {
                                format!("{}…", &c.text[..240])
                            } else {
                                c.text.clone()
                            };
                            let line = format!(
                                "{}. [score: {}] [{}]\n   steps: {}\n   text: {}\n\n",
                                i + 1, c.score, c.detected_as, steps_label, preview
                            );
                            buf.push_str(&line);
                        }
                        view! {
                            <div>
                                <pre style="white-space: pre-wrap; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace; background: #fff; border: 1px solid #e2e2e2; border-radius: .5rem; padding: .75rem; color: #333;">
                                    {buf}
                                </pre>
                            </div>
                        }
                    }}
                </Show>
            </section>

                </div>
            </div>

            <footer style="margin-top: 2rem; color: #666; font-size: .9rem;">
                "Tip: Increase beam width to explore more options at each depth. Depth controls how many chained transforms are allowed."
            </footer>
        </main>
    }
}

pub fn main() {
    mount_to_body(|| view! { <App/> })
}
