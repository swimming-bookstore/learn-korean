use super::player::Player;
use super::script::{Line, Script};
use leptos::either::Either;
use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn KaraokePlay(
    player: Player,
    record: RwSignal<bool>,
    on_next: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="karaoke-play">
            <button
                type="button"
                class="play"
                on:click=move |_| {
                    if record.get() {
                        if player.playing.get() {
                            player.playing.set(false);
                        } else {
                            player.restart();
                            player.playing.set(true);
                        }
                    } else {
                        record.set(true);
                        player.restart();
                        player.playing.set(true);
                    }
                }
            >
                {move || {
                    if !record.get() {
                        "Play"
                    } else if player.playing.get() {
                        "Pause"
                    } else {
                        "Play"
                    }
                }}
            </button>
            {move || {
                record.get().then(|| view! {
                    <button
                        type="button"
                        class="mode"
                        on:click=move |_| {
                            player.playing.set(false);
                            record.set(false);
                        }
                    >
                        "Read"
                    </button>
                    <button type="button" class="mode" on:click=move |_| on_next()>
                        "Next"
                    </button>
                })
            }}
        </div>
    }
}

#[component]
pub fn KaraokeRead(script: Script) -> impl IntoView {
    let rows: Vec<_> = script
        .lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            (
                i == 0 || script.lines[i - 1].section != line.section,
                line.clone(),
            )
        })
        .collect();
    view! {
        <div class="proof">
            {rows.into_iter().map(|(head, line)| {
                let section = line.section;
                let word = line.word().unwrap_or("").to_string();
                let cite = line.cite().map(str::to_string);
                view! {
                    {head.then(|| view! { <p class="sec-lab">{section}</p> })}
                    <p class="entry">
                        <span class="word em">{word}</span>
                        {cite.map(|g| view! { <span class="cite">{g}</span> })}
                    </p>
                }
            }).collect_view()}
        </div>
    }
}

/// Sentence above the list. Gold walks the matching stretch with the current card.
#[component]
pub fn KaraokePlate(korean: &'static str, meaning: &'static str, script: Script, player: Player) -> impl IntoView {
    let plate_s = script.clone();
    view! {
        <div class="plate">
            <p class="ko-line">
                {move || {
                    let span = cur_line(&plate_s, player)
                        .and_then(|li| plate_s.lines.get(li))
                        .and_then(|l| l.span);
                    let grammar = cur_line(&plate_s, player)
                        .and_then(|li| plate_s.lines.get(li))
                        .is_some_and(|l| l.section == "Grammar");
                    paint(korean, span, grammar)
                }}
            </p>
            <p class="en-line">{meaning}</p>
        </div>
    }
}

fn paint(korean: &str, span: Option<(usize, usize)>, grammar: bool) -> impl IntoView {
    let Some((a, b)) = span else {
        return Either::Left(korean.to_string());
    };
    if !korean.is_char_boundary(a) || !korean.is_char_boundary(b) || a >= b || b > korean.len() {
        return Either::Left(korean.to_string());
    }
    Either::Right(view! {
        <span class="dim">{korean[..a].to_string()}</span>
        <span class="hit" class:gram=grammar>{korean[a..b].to_string()}</span>
        <span class="dim">{korean[b..].to_string()}</span>
    })
}

/// Record: vocab + grammar on one page; meaning under the word; gold walks.
#[component]
pub fn KaraokeLyrics(script: Script, player: Player) -> impl IntoView {
    let notes: Vec<(usize, Line)> = script
        .lines
        .iter()
        .enumerate()
        .map(|(li, l)| (li, l.clone()))
        .collect();
    let now_s = script.clone();
    let list = NodeRef::<html::Div>::new();
    Effect::new(move |_| {
        let _ = player.cursor.get();
        let _ = player.playing.get();
        scroll_now(list);
    });
    let rows: Vec<_> = notes
        .iter()
        .enumerate()
        .map(|(i, (li, line))| {
            (
                *li,
                line.clone(),
                i == 0 || notes[i - 1].1.section != line.section,
            )
        })
        .collect();
    view! {
        <div class="karaoke" aria-live="polite">
            <div class="karaoke-clip" node_ref=list>
                {rows.into_iter().map(|(li, line, head)| {
                    let cur_s = now_s.clone();
                    view! {
                        {head.then(|| view! { <p class="sec">{line.section}</p> })}
                        <div
                            class="line"
                            class:now=move || cur_line(&cur_s, player) == Some(li)
                            class:gram=line.section == "Grammar"
                        >
                            <span class="ko">{line.word().unwrap_or("").to_string()}</span>
                            {line.cite().map(|g| view! { <span class="en">{g.to_string()}</span> })}
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

fn scroll_now(list: NodeRef<html::Div>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
        let Some(root) = list.get() else {
            return;
        };
        let Ok(Some(el)) = root.query_selector(".line.now") else {
            return;
        };
        let Ok(now) = el.dyn_into::<web_sys::HtmlElement>() else {
            return;
        };
        let top = now.offset_top() as f64;
        let h = now.offset_height() as f64;
        let view = root.client_height() as f64;
        let y = (top - (view - h) * 0.22).max(0.0);
        root.set_scroll_top(y as i32);
    });
    let _ = window.request_animation_frame(cb.as_ref().unchecked_ref());
}

fn cur_line(script: &Script, player: Player) -> Option<usize> {
    if !player.playing.get() && !player.done.get() && player.cursor.get() == player.start_at {
        return None;
    }
    script.get(player.cursor.get()).map(|(li, _, _)| li)
}
