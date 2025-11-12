use gloo_timers::future::TimeoutFuture;
use leptos::mount::mount_to_body;
use leptos::prelude::*;
use std::collections::HashSet;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

mod decoders;
mod engine;
mod scorer;
mod search;

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

#[component]
fn App() -> impl IntoView {
    let (input_text, set_input_text) = signal(String::new());
    let (depth_text, set_depth_text) = signal(String::from("4"));
    let (beam_text, set_beam_text) = signal(String::from("1000"));
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
        let selected = selected_decoder_ids.get_untracked();
        crate::decoders::register_selected(&mut dec_engine, selected.iter().map(|s| s.as_str()));
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

    // 1.  NEW SIGNAL ----------------------------------------------------------
    let (solving, set_solving) = signal(false);

    // 2.  WRAP on_submit TO TOGGLE THE FLAG -----------------------------------
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_solving.set(true); // <-- start spinner
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
        let selected = selected_decoder_ids.get_untracked();
        crate::decoders::register_selected(&mut dec_engine, selected.iter().map(|s| s.as_str()));
        let scorer = build_scorer(wordlist.get_untracked().clone());
        let cfg = SearchConfig {
            max_depth,
            beam_width: beam_opt,
            dedup_on_text: true,
        };
        let input = input_text.get_untracked();
        spawn_local(async move {
            // run solver off-thread
            TimeoutFuture::new(0).await; // yield to the event loop so the spinner can render
            let res = explore(&dec_engine, &scorer, &input, cfg);
            set_results.set(res);
            set_solving.set(false); // <-- stop spinner
        });
    };

    view! {
        <main class="min-h-screen bg-gray-50 text-gray-800">
            <div class="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 py-10">
                {/* --- Header ---------------------------------------------------- */}
                <div class="rounded-xl bg-gradient-to-r from-emerald-600 to-green-600 text-white p-6 mb-8 shadow-lg">
                    <h1 class="text-3xl font-bold tracking-tight">"Locksmith"</h1>
                    <p class="mt-1 text-emerald-100">"Automatic crypto-puzzle solver"</p>
                </div>

                <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
                    {/* --- Left column : controls -------------------------------- */}
                    <div class="lg:col-span-1">
                        <form on:submit=on_submit class="bg-white rounded-xl shadow p-6 space-y-5">
                            <div>
                                <label class="block text-sm font-semibold text-gray-700 mb-1">"Input"</label>
                                <textarea
                                    placeholder="Paste encoded text here…"
                                    rows="6"
                                    on:input=move |ev| set_input_text.set(event_target_value(&ev))
                                    class="w-full rounded-lg border border-gray-300 px-3 py-2 font-mono
                                           focus:outline-none focus:ring-2 focus:ring-emerald-500"
                                />
                            </div>

                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                <div>
                                    <label class="block text-sm font-semibold text-gray-700 mb-1">
                                        "Depth "
                                        <span class="font-normal text-gray-500">{move || depth_text.get()}</span>
                                    </label>
                                    <input
                                        type="range"
                                        min="0"
                                        max="10"
                                        value=move || depth_text.get()
                                        on:input=move |ev| set_depth_text.set(event_target_value(&ev))
                                        class="w-full accent-emerald-600"
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-semibold text-gray-700 mb-1">"Beam Width"</label>
                                    <input
                                        type="number"
                                        min="0"
                                        placeholder="e.g. 1000"
                                        value=move || beam_text.get()
                                        on:input=move |ev| set_beam_text.set(event_target_value(&ev))
                                        class="w-full rounded-lg border border-gray-300 px-3 py-2
                                               focus:outline-none focus:ring-2 focus:ring-emerald-500"
                                    />
                                </div>
                            </div>

                            <fieldset class="border border-gray-100 rounded-xl p-4 bg-gray-50/70">
                                <legend class="px-2 text-xs font-semibold tracking-wide text-gray-500 uppercase">
                                    "Decoders"
                                </legend>
                                <div class="mt-3 grid grid-cols-1 sm:grid-cols-2 gap-2.5">
                                    {move || {
                                        crate::decoders::all_decoders_info()
                                            .iter()
                                            .map(|info| {
                                                let id = info.id;
                                                let label = info.label;
                                                view! {
                                                    <label
                                                        class="group flex items-center gap-2.5 px-2.5 py-1.5 rounded-lg
                                                               bg-white/80 border border-gray-200
                                                               hover:border-emerald-300 hover:bg-emerald-50/70
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
                                                            class="h-4 w-4 rounded-md border-gray-300
                                                                   text-emerald-600
                                                                   focus:ring-2 focus:ring-emerald-500 focus:ring-offset-1
                                                                   cursor-pointer"
                                                        />
                                                        <span class="text-xs font-medium text-gray-800 group-hover:text-emerald-800">
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
                                class="w-full sm:w-auto px-5 py-2.5 rounded-lg bg-emerald-600 text-white
                                       font-semibold hover:bg-emerald-700 focus:outline-none focus:ring-2
                                       focus:ring-emerald-500 transition disabled:opacity-60
                                       flex items-center justify-center gap-2"
                            >
                                <Show when=move || solving.get() fallback=move || view!{ <span>"Decode"</span> }>
                                    <svg class="animate-spin h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
                                    </svg>
                                    <span>"Solving…"</span>
                                </Show>
                            </button>
                        </form>
                    </div>

                    {/* --- Right column : results -------------------------------- */}
                    <div class="lg:col-span-2 space-y-6">
                        {/* Best result card */}
                        <section class="bg-white rounded-xl shadow p-5">
                            <Show
                                when=move || !results.get().is_empty()
                                fallback=move || {
                                    view! {
                                        <>
                                            <h2 class="text-lg font-semibold text-gray-900 mb-2">"Best Result"</h2>
                                            <p class="text-gray-500 italic">"Run a search to see results."</p>
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
                                            path.push_str(" → ");
                                            path.push_str(&format!("{} ({})", s.desc, s.op_id));
                                        }
                                    } else {
                                        path.push_str(" (no transforms)");
                                    }
                                    view! {
                                        <>
                                            <h2 class="text-lg font-semibold text-gray-900 mb-2">"Best Result"</h2>
                                            <div class="flex items-center gap-3 mb-3 text-sm">
                                                <span class="inline-flex items-center rounded-full bg-emerald-100 text-emerald-800 px-3 py-0.5">
                                                    <strong class="mr-1">"Score:"</strong>
                                                    {best.score}
                                                </span>
                                                <span class="inline-flex items-center rounded-full bg-gray-100 text-gray-800 px-3 py-0.5">
                                                    <strong class="mr-1">"Detected as:"</strong>
                                                    {best.detected_as}
                                                </span>
                                            </div>
                                            <pre class="rounded-lg bg-gray-900 text-gray-100 p-4 text-sm
                                                        overflow-auto max-h-56">
                                                {best.text}
                                            </pre>
                                            <h3 class="mt-4 font-semibold text-gray-800">"Path"</h3>
                                            <p class="font-mono text-sm text-gray-700">{path}</p>
                                        </>
                                    }
                                }}
                            </Show>
                        </section>

                        {/* Top 5 section */}
                        // ---------  NEW  “Other Top 5 Results”  ---------
                        <section class="bg-white rounded-xl shadow p-5">
                            <h2 class="text-lg font-semibold text-gray-900 mb-4">"Other Results"</h2>

                            <Show
                                when=move || (results.get().len() > 1)
                                fallback=move || view! { <p class="text-gray-500 italic">"No additional top results yet."</p> }
                            >
                                <div class="grid gap-4 md:grid-cols-2">
                                    {move || {
                                        let res = results.get();
                                        res.into_iter()
                                            .skip(1)                 // best result already shown
                                            .take(4)                 // top-5
                                            .enumerate()
                                            .map(|(idx, chain)| {
                                                let steps = if chain.steps.is_empty() {
                                                    "none".to_string()
                                                } else {
                                                    chain.steps
                                                        .iter()
                                                        .map(|s| s.desc.as_str())
                                                        .collect::<Vec<_>>()
                                                        .join(" → ")
                                                };
                                                let preview = if chain.text.len() > 120 {
                                                    format!("{}…", &chain.text[..120])
                                                } else {
                                                    chain.text.clone()
                                                };
                                                view! {
                                                    <div class="border border-gray-200 rounded-lg p-4 hover:shadow-md transition">
                                                        <div class="flex items-center justify-between mb-2">
                                                            <span class="text-sm font-medium text-gray-500">
                                                                {idx + 1}
                                                            </span>
                                                            <div class="flex items-center gap-2">
                                                                <span class="inline-flex items-center rounded-full
                                                                               bg-emerald-100 text-emerald-800
                                                                               px-2 py-0.5 text-xs font-semibold">
                                                                    {chain.score}
                                                                </span>
                                                                <span class="inline-flex items-center rounded-full
                                                                               bg-gray-100 text-gray-700
                                                                               px-2 py-0.5 text-xs">
                                                                    {chain.detected_as}
                                                                </span>
                                                            </div>
                                                        </div>

                                                        <p class="text-sm text-gray-800 font-mono bg-gray-50 rounded
                                                                 px-3 py-2 mb-2 break-all">
                                                            {preview}
                                                        </p>

                                                        <p class="text-xs text-gray-500">
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
                <footer class="mt-10 text-sm text-gray-500">
                    "Tip: Increase beam width to explore more options at each depth. \
                     Depth controls how many chained transforms are allowed."
                </footer>
            </div>
        </main>
    }
}

pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}
