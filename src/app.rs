use crate::content::{days, get, next_day, Lesson, SERIES};
use crate::karaoke::{compile, KaraokeLyrics, KaraokePlay, KaraokeRead, Player, Timing};
use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::JsValue;

#[component]
pub fn App() -> impl IntoView {
    let start = read_route();
    let day = RwSignal::new(start);
    let record = RwSignal::new(wants_record());
    write_route(start, false);

    Effect::new(move |_| {
        set_title(day.get());
    });

    window_event_listener(ev::hashchange, move |_| {
        let d = read_route();
        if day.get_untracked() != d {
            day.set(d);
        }
    });

    view! {
        <div id="app">
            <header>
                <p class="brand">"Learn Korean · 수영 책방"</p>
                <h1>
                    {move || {
                        get(day.get())
                            .map(|l| format!("Day {} · {}", l.day, l.series))
                            .unwrap_or_else(|| "Learn Korean".into())
                    }}
                </h1>
            </header>
            <main>
                {move || {
                    match get(day.get()) {
                        Some(l) => view! { <LessonPage lesson=*l day=day record=record /> }.into_any(),
                        None => view! { <p class="empty">"This lesson is not yet written."</p> }.into_any(),
                    }
                }}
            </main>
            <footer>
                <nav class="books" aria-label="Series">
                    <span class="nav-lab">"Series"</span>
                    {SERIES.iter().map(|(name, lessons)| {
                        let first = lessons.first().map(|l| l.day).unwrap_or(1);
                        view! {
                            <button
                                class:active=move || {
                                    get(day.get()).map(|l| l.series == *name).unwrap_or(false)
                                }
                                on:click=move |_| {
                                    day.set(first);
                                    write_route(first, true);
                                }
                            >
                                {*name}
                            </button>
                        }
                    }).collect_view()}
                </nav>
                <nav class="props" aria-label="Days">
                    <span class="nav-lab">"Days"</span>
                    {days().into_iter().map(|n| {
                        view! {
                            <button
                                class:active=move || day.get() == n
                                on:click=move |_| {
                                    day.set(n);
                                    write_route(n, true);
                                }
                            >
                                {format!("{n}")}
                            </button>
                        }
                    }).collect_view()}
                </nav>
                <p class="copy">"© 수영 책방 Swimming Bookstore"</p>
            </footer>
        </div>
    }
}

#[component]
fn LessonPage(lesson: Lesson, day: RwSignal<u16>, record: RwSignal<bool>) -> impl IntoView {
    let script = compile(&lesson, Timing::default());
    let capture = wants_record();
    let player = Player::start(&script, record.get_untracked());
    let script_read = script.clone();
    let script_lyr = script.clone();
    let meaning = lesson.meaning;
    let korean = lesson.korean;

    view! {
        <article
            class="stage"
            class:record=move || record.get()
            class:capture=capture
            data-ready="1"
            data-done=move || if player.done.get() { "1" } else { "0" }
        >
            <KaraokePlay
                player=player
                record=record
                on_next=move || {
                    if let Some(n) = next_day(day.get()) {
                        day.set(n);
                        write_route(n, true);
                    }
                }
            />
            {move || {
                if record.get() {
                    view! {
                        <div class="plate">
                            <p class="ko-line">{korean}</p>
                            <p class="en-line">{meaning}</p>
                        </div>
                        <KaraokeLyrics script=script_lyr.clone() player=player />
                    }.into_any()
                } else {
                    view! {
                        <div class="hero">
                            <p class="ko-line">{korean}</p>
                            <p class="en-line">{meaning}</p>
                        </div>
                        <KaraokeRead script=script_read.clone() />
                    }.into_any()
                }
            }}
        </article>
    }
}

fn wants_record() -> bool {
    web_sys::window()
        .and_then(|w| w.location().search().ok())
        .map(|s| s.contains("record") || s.contains("autoplay"))
        .unwrap_or(false)
}

fn read_route() -> u16 {
    let loc = web_sys::window().map(|w| w.location());
    let Some(loc) = loc else {
        return 1;
    };
    if let Ok(hash) = loc.hash() {
        if let Some(d) = parse_day(&hash) {
            return d;
        }
    }
    if let Ok(search) = loc.search() {
        if let Some(d) = parse_query(&search) {
            return d;
        }
    }
    1
}

fn parse_query(search: &str) -> Option<u16> {
    let s = search.trim_start_matches('?');
    for part in s.split('&') {
        let (k, v) = part.split_once('=')?;
        if matches!(k, "day" | "d") {
            return v.parse().ok().map(|n: u16| n.max(1));
        }
    }
    None
}

fn parse_day(raw: &str) -> Option<u16> {
    let s = raw
        .trim()
        .trim_start_matches('#')
        .trim_start_matches('/')
        .split(['?', '&', '#'])
        .next()
        .unwrap_or("")
        .trim_end_matches('/');
    if s.is_empty() {
        return None;
    }
    let parts: Vec<&str> = s
        .split(['/', '.', '-', '_'])
        .filter(|p| !p.is_empty())
        .collect();
    let n: u16 = match parts.as_slice() {
        ["day", d] | ["d", d] => d.parse().ok()?,
        [d] => d.parse().ok()?,
        _ => return None,
    };
    Some(n.max(1))
}

fn write_route(day: u16, push: bool) {
    let Some(win) = web_sys::window() else {
        return;
    };
    let loc = win.location();
    let path = loc.pathname().unwrap_or_else(|_| "/".into());
    let search = loc.search().unwrap_or_default();
    let hash = format!("#/{day}");
    if loc.hash().ok().as_deref() == Some(hash.as_str()) {
        return;
    }
    let url = format!("{path}{search}{hash}");
    if let Ok(history) = win.history() {
        let r = if push {
            history.push_state_with_url(&JsValue::NULL, "", Some(&url))
        } else {
            history.replace_state_with_url(&JsValue::NULL, "", Some(&url))
        };
        if r.is_ok() {
            return;
        }
    }
    let _ = loc.set_hash(&format!("/{day}"));
}

fn set_title(day: u16) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        let title = get(day)
            .map(|l| format!("Day {} · {} — Learn Korean", l.day, l.korean))
            .unwrap_or_else(|| "Learn Korean".into());
        doc.set_title(&title);
    }
}
