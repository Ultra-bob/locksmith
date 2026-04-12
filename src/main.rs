use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos_use::use_clipboard;
use leptos_use::use_timestamp;
use leptos_workers::worker;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::spawn_local;

mod decoders;
mod engine;
mod scorer;
mod search;

use search::{Chain, SearchConfig, explore};

// --------------- Worker Request/Response ----------------------------------

#[derive(Clone, Serialize, Deserialize)]
pub struct DecryptRequest {
    input: String,
    max_depth: usize,
    beam_width: Option<usize>,
    dedup_on_text: bool,
    selected_decoder_ids: Vec<String>,
    wordlist: Option<HashSet<String>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DecryptResponse {
    results: Vec<Chain>,
}

// --------------- Helper functions -----------------------------------------

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
    let resp: Response = JsFuture::from(win.fetch_with_str(url))
        .await
        .ok()?
        .dyn_into()
        .ok()?;
    if !resp.ok() {
        return None;
    }
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

// --------------- The worker itself ----------------------------------------
// This runs off-main-thread. It rebuilds the decoder engine and scorer,
// then runs `explore` and returns the results.
#[worker(DecryptWorker)]
pub async fn decrypt_worker(req: DecryptRequest) -> DecryptResponse {
    let mut dec_engine = engine::DecoderEngine::new();
    crate::decoders::register_selected(
        &mut dec_engine,
        req.selected_decoder_ids.iter().map(|s| s.as_str()),
    );

    let scorer = build_scorer(req.wordlist);
    let cfg = SearchConfig {
        max_depth: req.max_depth,
        beam_width: req.beam_width,
        dedup_on_text: req.dedup_on_text,
    };

    let results = explore(&dec_engine, &scorer, &req.input, cfg);
    DecryptResponse { results }
}

// --------------- UI component ---------------------------------------------

#[component]
fn App() -> impl IntoView {
    let (input_text, set_input_text) = signal(String::new());
    let (depth_text, set_depth_text) = signal(String::from("4"));
    let (beam_text, set_beam_text) = signal(String::from("1000"));

    let timestamp = use_timestamp();
    let (decoding_start_timestamp, set_decoding_start_timestamp) = signal(0.0);
    let (decoding_time_ms, set_decoding_time_ms) = signal(0);

    // let leptos_use::UseClipboardReturn {
    //     is_supported: clipboard_supported,
    //     text: clipboard_text,
    //     copied: clipboard_copied,
    //     copy: clipboard_copy,
    // } = use_clipboard();

    let (results, set_results) = signal::<Vec<Chain>>(vec![]);
    let (wordlist, set_wordlist) = signal::<Option<HashSet<String>>>(None);

    // Selected decoders: default to all enabled
    let initial_selected: HashSet<String> = crate::decoders::all_decoders_info()
        .iter()
        .map(|d| d.id.to_string())
        .collect();
    let (selected_decoder_ids, set_selected_decoder_ids) =
        signal::<HashSet<String>>(initial_selected);

    {
        const WORDLIST_URL: &str =
            "https://raw.githubusercontent.com/jeremy-rifkin/Wordlist/refs/heads/master/master.txt";
        Effect::new(move |_| {
            spawn_local(async move {
                if let Some(set) = fetch_wordlist_from_url(WORDLIST_URL).await {
                    set_wordlist.set(Some(set));
                }
            });
        });
    }

    // Spinner signal
    let (solving, set_solving) = signal(false);

    // Submit handler now uses the worker
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_solving.set(true);
        set_decoding_start_timestamp.set(timestamp.get());

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

        let req = DecryptRequest {
            input: input_text.get_untracked().trim().to_string(),
            max_depth,
            beam_width: beam_opt,
            dedup_on_text: true,
            selected_decoder_ids: selected_decoder_ids
                .get_untracked()
                .into_iter()
                .collect::<Vec<_>>(),
            wordlist: wordlist.get_untracked().clone(),
        };

        spawn_local(async move {
            // Yield to the event loop so the spinner can render
            TimeoutFuture::new(0).await;

            match decrypt_worker(req).await {
                Ok(resp) => {
                    set_results.set(resp.results);
                    set_decoding_time_ms
                        .set((timestamp.get() - decoding_start_timestamp.get()) as i32); // If the decoding time is more than 2 billion milliseconds, we have bigger problems
                }
                Err(_e) => {
                    // Fallback: run on main thread if workers aren't supported
                    let mut dec_engine = engine::DecoderEngine::new();
                    let selected = selected_decoder_ids.get_untracked();
                    crate::decoders::register_selected(
                        &mut dec_engine,
                        selected.iter().map(|s| s.as_str()),
                    );
                    let scorer = build_scorer(wordlist.get_untracked().clone());
                    let cfg = SearchConfig {
                        max_depth,
                        beam_width: beam_opt,
                        dedup_on_text: true,
                    };
                    let input = input_text.get_untracked();
                    let res = explore(&dec_engine, &scorer, &input, cfg);
                    set_results.set(res);
                }
            }

            set_solving.set(false);
        });
    };

    let example_input = String::from(
        "==rW5G2lrPRTwUIl1QIKjMNoy42nw9RKrBIJzCSTjGRlhw2JrvymsMDKv92JwCRTgCSTwixJsMDKtMjm0MjnaMNms92KrXSoUarW5iRmsYHl0QHmgCIosMjny5Hlv92JfG2WkUIKzMIluMNKwUIK5QRmrnxmaUIKzMIluGRKrT3mxMNmg9RorPRTkwRTzCIleY3lu9RE",
    );

    view! {
        <main class="min-h-screen bg-black text-emerald-100" style="font-family: 'Fira Mono', monospace;">
            <div class="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
                {/* --- Header ---------------------------------------------------- */}
                <div class=" border border-emerald-600 bg-black/80 px-4 py-3 mb-6">
                    <h1 class="text-2xl font-bold tracking-[0.2em] uppercase text-emerald-300">
                        "< Locksmith >"
                    </h1>
                    <p class="mt-1 text-xs text-emerald-500">
                        "Automatic multi-step decoder"
                    </p>
                </div>

                <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    {/* --- Left column : controls -------------------------------- */}
                    <div class="lg:col-span-1">
                        <form
                            on:submit=on_submit
                            class="bg-black/90 border border-emerald-800 px-4 py-3 space-y-4"
                        >
                            <div>
                                <div class="flex items-center justify-between">
                                    <label class="block text-sm font-semibold text-emerald-300 uppercase tracking-wide">
                                        "> Input"
                                    </label>
                                    <button
                                        type="button"
                                        class="bg-emerald-700/50 border border-emerald-600 hover:bg-emerald-700 text-white px-2 py-1 my-2 text-xs"
                                        on:click=move |_| {
                                            set_input_text.set(example_input.clone());
                                            // Manually set textarea value to trigger visual update for controlled input
                                            // (Leptos signals should trigger rerender, but we force HTML update for robustness)
                                            if let Some(textarea) = web_sys::window()
                                                .and_then(|w| w.document())
                                                .and_then(|doc| doc.query_selector("textarea").ok().flatten())
                                                .and_then(|el| el.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
                                            {
                                                textarea.set_value(&example_input);
                                            }
                                        }
                                    >
                                        "Example"
                                    </button>
                                </div>
                                <textarea
                                    placeholder="Paste encoded text here…"
                                    rows="6"
                                    on:input=move |ev| set_input_text.set(event_target_value(&ev))
                                    class="w-full border border-emerald-800 bg-black/80 px-2 py-1.5
                                           text-xs text-emerald-100 placeholder:text-emerald-700
                                           focus:outline-none focus:ring-1 focus:ring-emerald-500"
                                />
                            </div>

                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                <div>
                                    <label class="block text-xs font-semibold text-emerald-300 mb-1 uppercase tracking-wide">
                                        "Depth "
                                        <span class="font-normal text-emerald-500">
                                            {move || depth_text.get()}
                                        </span>
                                    </label>
                                    <input
                                        type="range"
                                        min="0"
                                        max="25"
                                        value=move || depth_text.get()
                                        on:input=move |ev| set_depth_text.set(event_target_value(&ev))
                                        class="w-full accent-emerald-400 bg-black"
                                    />
                                </div>
                                <div>
                                    <label class="block text-xs font-semibold text-emerald-300 mb-1 uppercase tracking-wide">
                                        "Beam Width"
                                    </label>
                                    <input
                                        type="number"
                                        min="0"
                                        placeholder="e.g. 1000"
                                        value=move || beam_text.get()
                                        on:input=move |ev| set_beam_text.set(event_target_value(&ev))
                                        class="w-full border border-emerald-800 bg-black/80 px-2 py-1.5
                                               text-xs text-emerald-100 placeholder:text-emerald-700
                                               focus:outline-none focus:ring-1 focus:ring-emerald-500"
                                    />
                                </div>
                            </div>

                            <fieldset class="border border-emerald-800 px-3 py-2 bg-black/70">
                                <legend class="px-1 text-[0.6rem] font-semibold tracking-[0.25em] text-emerald-400 uppercase">
                                    "Decoders"
                                </legend>
                                <div class="mt-2 grid grid-cols-1 sm:grid-cols-2 gap-1.5">
                                    {move || {
                                        crate::decoders::all_decoders_info()
                                            .iter()
                                            .map(|info| {
                                                let id = info.id;
                                                let label = info.label;
                                                view! {
                                                    <label
                                                        class="group flex items-center gap-2 px-2 py-1
                                                               bg-black/60 border border-emerald-900
                                                               hover:border-emerald-400 hover:bg-emerald-950/40
                                                               transition-colors cursor-pointer"
                                                    >
                                                        <input
                                                            type="checkbox"
                                                            checked=move || selected_decoder_ids.get().contains(id)
                                                            on:change=move |ev| {
                                                                let checked = ev
                                                                    .target()
                                                                    .and_then(|t| t.dyn_ref::<web_sys::HtmlInputElement>().map(|i| i.checked()))
                                                                    .unwrap_or(false);
                                                                let id_str = id.to_string();
                                                                set_selected_decoder_ids.update(move |set| {
                                                                    if checked {
                                                                        set.insert(id_str.clone());
                                                                    } else {
                                                                        set.remove(&id_str);
                                                                    }
                                                                });
                                                            }
                                                            class="h-3.5 w-3.5  border-emerald-700
                                                                   accent-emerald-700
                                                                   bg-black text-emerald-400
                                                                   focus:ring-1 focus:ring-emerald-500 focus:ring-offset-0
                                                                   cursor-pointer"
                                                        />
                                                        <span class="text-[0.7rem] font-medium text-emerald-200 group-hover:text-emerald-100">
                                                            {label}
                                                        </span>
                                                    </label>
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                    }}
                                </div>
                            </fieldset>

                            <button
                                type="submit"
                                disabled=move || solving.get()
                                class="w-full px-4 py-1.5
                                       border border-emerald-500 bg-emerald-700/20 text-emerald-100
                                       text-xs font-semibold tracking-[0.25em] uppercase
                                       hover:bg-emerald-500/30 focus:outline-none focus:ring-1
                                       focus:ring-emerald-400 transition disabled:opacity-50
                                       flex items-center justify-center gap-2"
                            >
                                <Show when=move || solving.get() fallback=move || view!{ <span>"Automatic Decode"</span> }>
                                    <svg class="animate-spin h-4 w-4 text-emerald-200" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
                                    </svg>
                                    <span>"Decoding..."</span>
                                </Show>
                            </button>
                        </form>
                    </div>

                    {/* --- Right column : results -------------------------------- */}
                    <div class="lg:col-span-2 space-y-4">
                        {/* Best result card */}
                        <section class="bg-black/90 border border-emerald-800  px-4 py-3">
                            <Show
                                when=move || !results.get().is_empty()
                                fallback=move || {
                                    view! {
                                        <>
                                            <h2 class="text-sm font-semibold text-emerald-300 tracking-[0.2em] uppercase mb-1">
                                                "> Best Result"
                                            </h2>
                                            <p class="text-xs text-emerald-600 italic">
                                                "[ Run a search to see results ]"
                                            </p>
                                        </>
                                    }
                                }
                            >
                                {move || {
                                    let res = results.get();
                                    let best = res[0].clone();
                                    let mut path = String::from("Input");
                                    if !best.steps.is_empty() {
                                        for s in &best.steps {
                                            path.push_str(" -> ");
                                            path.push_str(&s.desc);
                                        }
                                    } else {
                                        path.push_str(" (no transforms)");
                                    }
                                    view! {
                                        <>
                                            <h2 class="text-sm font-semibold text-emerald-300 tracking-[0.2em] uppercase mb-2">
                                                "> Best Result"
                                            </h2>
                                            <div class="flex flex-wrap items-center gap-2 mb-2 text-[0.7rem]">
                                                <span class="inline-flex items-center  border border-emerald-600 bg-black px-2 py-0.5">
                                                    <strong class="mr-1 text-emerald-400">"Score:"</strong>
                                                    {best.score}
                                                </span>
                                                <span class="inline-flex items-center  border border-emerald-800 bg-black px-2 py-0.5">
                                                    <strong class="mr-1 text-emerald-400">"Time:"</strong>
                                                    {decoding_time_ms.get() as f64 / 1000.0}s
                                                </span>
                                                <span class="inline-flex items-center  border border-emerald-800 bg-black px-2 py-0.5">
                                                    <strong class="mr-1 text-emerald-400">"Detected:"</strong>
                                                    {best.detected_as}
                                                </span>
                                            </div>
                                            <pre class=" border border-emerald-900 bg-black text-emerald-100
                                                        px-3 py-2 text-xs whitespace-pre-wrap max-h-56">
                                                {best.text}
                                            </pre>
                                            <h3 class="mt-3 text-[0.7rem] font-semibold text-emerald-300 uppercase tracking-wide">
                                                "Path " <span class="font-normal">"("{best.steps.len()}" steps)"</span>
                                            </h3>
                                            <p class="text-[0.7rem] text-emerald-200">{path}</p>
                                        </>
                                    }
                                }}
                            </Show>
                        </section>

                        {/* Other results */}
                        <section class="bg-black/90 border border-emerald-800  px-4 py-3">
                            <h2 class="text-sm font-semibold text-emerald-300 tracking-[0.2em] uppercase mb-2">
                                "> Other Results"
                            </h2>
                            <Show

                                when=#[allow(unused_parens)] move || (results.get().len() > 1)
                                fallback=move || view! {
                                    <p class="text-xs text-emerald-600 italic">
                                        "[ No additional results yet ]"
                                    </p>
                                }
                            >
                                <div class="grid gap-3 md:grid-cols-2">
                                    {move || {
                                        let res = results.get();
                                        res.into_iter()
                                            .skip(1)
                                            .take(4)
                                            .enumerate()
                                            .map(|(idx, chain)| {
                                                let steps = if chain.steps.is_empty() {
                                                    "none".to_string()
                                                } else {
                                                    chain.steps
                                                        .iter()
                                                        .map(|s| s.desc.as_str())
                                                        .collect::<Vec<_>>()
                                                        .join(" -> ")
                                                };
                                                let preview = if chain.text.len() > 120 {
                                                    format!("{}…", &chain.text[..120])
                                                } else {
                                                    chain.text.clone()
                                                };
                                                view! {
                                                    <div class="border border-emerald-900  px-3 py-2 bg-black/70 hover:border-emerald-500 transition-colors">
                                                        <div class="flex items-center justify-between mb-1">
                                                            <span class="text-[0.7rem] font-medium text-emerald-500">
                                                                {"#" }{idx + 1}
                                                            </span>
                                                            <div class="flex items-center gap-1.5">
                                                                <span class="inline-flex items-center
                                                                               border border-emerald-700
                                                                               px-1.5 py-0.5 text-[0.65rem] text-emerald-200">
                                                                    {chain.score}
                                                                </span>
                                                                <span class="inline-flex items-center
                                                                               border border-emerald-900
                                                                               px-1.5 py-0.5 text-[0.65rem] text-emerald-300">
                                                                    {chain.detected_as}
                                                                </span>
                                                            </div>
                                                        </div>
                                                        <p class="text-[0.7rem] text-emerald-100 bg-black/60
                                                                 px-2 py-1 mb-1 break-all">
                                                            {preview}
                                                        </p>
                                                        <p class="text-[0.65rem] text-emerald-500">
                                                            <span class="font-semibold">"Steps: "</span>{steps}
                                                        </p>
                                                    </div>
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                    }}
                                </div>
                            </Show>
                        </section>
                    </div>
                </div>

                {/* Footer tip */}
                <footer class="mt-6 text-[0.7rem] text-emerald-600 border-t border-emerald-900 pt-2">
                    "Tip: Increase beam width to explore more options at each depth. "
                    "Depth controls how many chained operations are attempted."
                </footer>
            </div>
        </main>
    }
}

fn main() {
    // Build decoder engine and register available decoders.
    let mut dec_engine = engine::DecoderEngine::new();
    decoders::register_all(&mut dec_engine);

    // Build scoring engine.
    let scorer = scorer::default_scorer();

    // Read from input.txt
    let input = std::fs::read_to_string("input.txt").expect("Failed to read input.txt");

    // Configure search: explore up to 3 steps deep, keep best 100 candidates per depth,
    // and avoid revisiting identical output texts.
    let cfg = SearchConfig {
        max_depth: 6,
        beam_width: Some(1000),
        dedup_on_text: true,
    };

    // Run the exploration.
    let results = explore(&dec_engine, &scorer, &input, cfg);

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

#[wasm_bindgen(start)]
pub fn start() {
    // If we're in a Worker, web_sys::window() is None (workers have WorkerGlobalScope, not Window).
    if web_sys::window().is_none() {
        // Running inside a Web Worker: do not mount the Leptos app.
        // leptos_workers' generated worker harness will run instead.
        return;
    }

    // Browser main thread: mount the app as usual.
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <App/> })
}
