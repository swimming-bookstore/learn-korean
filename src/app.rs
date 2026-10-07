use crate::content::{episodes_in, get, next_episode, series_name, Lesson, SERIES};
use crate::karaoke::{
    compile, KaraokeLyrics, KaraokePlate, KaraokePlay, KaraokeRead, Player, Timing,
};
use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::JsValue;

/// `#/2` is series 2 episode 0. `#/2/1` is episode 1. `#/2/3` is episode 3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Route {
    series: u16,
    episode: u16,
}

#[component]
pub fn App() -> impl IntoView {
    let start = read_route();
    let route = RwSignal::new(start);
    let record = RwSignal::new(wants_record());
    write_route(start, false);

    Effect::new(move |_| {
        set_title(route.get());
    });

    window_event_listener(ev::hashchange, move |_| {
        let d = read_route();
        if route.get_untracked() != d {
            route.set(d);
        }
    });

    view! {
        <div id="app">
            <header>
                <p class="brand">"Learn Korean · 수영 책방"</p>
                <h1>
                    {move || {
                        get(route.get().series, route.get().episode)
                            .map(|l| format!("{} {:02}", l.series, l.episode))
                            .unwrap_or_else(|| "Learn Korean".into())
                    }}
                </h1>
            </header>
            <main>
                {move || {
                    let r = route.get();
                    match get(r.series, r.episode) {
                        Some(l) => view! { <LessonPage lesson=*l route=route record=record /> }.into_any(),
                        None => view! { <p class="empty">"This lesson is not yet written."</p> }.into_any(),
                    }
                }}
            </main>
            <footer>
                <nav class="books" aria-label="Series">
                    <span class="nav-lab">"Series"</span>
                    {SERIES.iter().enumerate().map(|(i, (name, lessons))| {
                        let n = (i + 1) as u16;
                        let first = lessons.first().map(|l| l.episode).unwrap_or(1);
                        view! {
                            <button
                                class:active=move || route.get().series == n
                                on:click=move |_| {
                                    let r = Route { series: n, episode: first };
                                    route.set(r);
                                    write_route(r, true);
                                }
                            >
                                {*name}
                            </button>
                        }
                    }).collect_view()}
                </nav>
                <nav class="props" aria-label="Days">
                    <span class="nav-lab">"Days"</span>
                    {move || {
                        let series = route.get().series;
                        episodes_in(series).into_iter().map(|n| {
                            view! {
                                <button
                                    class:active=move || {
                                        let r = route.get();
                                        r.series == series && r.episode == n
                                    }
                                    on:click=move |_| {
                                        let r = Route { series, episode: n };
                                        route.set(r);
                                        write_route(r, true);
                                    }
                                >
                                    {format!("{n:02}")}
                                </button>
                            }
                        }).collect_view()
                    }}
                </nav>
                <p class="copy">"© 수영 책방 Swimming Bookstore"</p>
            </footer>
        </div>
    }
}

#[component]
fn LessonPage(lesson: Lesson, route: RwSignal<Route>, record: RwSignal<bool>) -> impl IntoView {
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
                    let r = route.get();
                    if let Some((series, episode)) = next_episode(r.series, r.episode) {
                        let next = Route { series, episode };
                        route.set(next);
                        write_route(next, true);
                    }
                }
            />
            {move || {
                if record.get() {
                    view! {
                        <KaraokePlate
                            korean=korean
                            meaning=meaning
                            script=script_lyr.clone()
                            player=player
                        />
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

fn read_route() -> Route {
    let loc = web_sys::window().map(|w| w.location());
    let Some(loc) = loc else {
        return Route {
            series: 1,
            episode: 1,
        };
    };
    if let Ok(hash) = loc.hash() {
        if let Some(r) = parse_hash(&hash) {
            return r;
        }
    }
    if let Ok(search) = loc.search() {
        if let Some(r) = parse_query(&search) {
            return r;
        }
    }
    Route {
        series: 1,
        episode: 1,
    }
}

fn parse_query(search: &str) -> Option<Route> {
    let s = search.trim_start_matches('?');
    let mut series = None;
    let mut episode = None;
    let mut day = None;
    for part in s.split('&') {
        let Some((k, v)) = part.split_once('=') else {
            continue;
        };
        let n: u16 = v.parse().ok()?;
        match k {
            "series" | "s" => series = Some(n.max(1)),
            "episode" | "e" => episode = Some(n),
            "day" | "d" => day = Some(n),
            _ => {}
        }
    }
    if series.is_some() || episode.is_some() {
        return Some(Route {
            series: series.unwrap_or(1),
            episode: episode.or(day).unwrap_or(1),
        });
    }
    day.map(|episode| Route { series: 1, episode })
}

fn parse_hash(raw: &str) -> Option<Route> {
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
    let parts: Vec<&str> = s.split('/').filter(|p| !p.is_empty()).collect();
    let num = |p: &str| -> Option<u16> {
        let t = p
            .trim_start_matches("day")
            .trim_start_matches("series")
            .trim_start_matches('s')
            .trim_start_matches('d')
            .trim_start_matches(['-', '_', '.']);
        let n: u16 = t.parse().ok()?;
        Some(n)
    };
    match parts.as_slice() {
        ["day", d] | ["d", d] => Some(Route {
            series: 1,
            episode: num(d)?,
        }),
        [series, episode] => Some(Route {
            series: num(series)?,
            episode: num(episode)?,
        }),
        [series] => {
            let series = num(series)?;
            let episode = crate::content::episodes_in(series)
                .into_iter()
                .next()
                .unwrap_or(1);
            Some(Route { series, episode })
        }
        _ => None,
    }
}

fn write_route(route: Route, push: bool) {
    let Some(win) = web_sys::window() else {
        return;
    };
    let loc = win.location();
    let path = loc.pathname().unwrap_or_else(|_| "/".into());
    let search = loc.search().unwrap_or_default();
    let hash = format!("#/{}/{}", route.series, route.episode);
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
    let _ = loc.set_hash(&format!("/{}/{}", route.series, route.episode));
}

fn set_title(route: Route) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        let title = get(route.series, route.episode)
            .map(|l| {
                format!(
                    "{} {:02} · {} — Learn Korean",
                    l.series, l.episode, l.korean
                )
            })
            .or_else(|| {
                series_name(route.series)
                    .map(|name| format!("{name} {:02} — Learn Korean", route.episode))
            })
            .unwrap_or_else(|| "Learn Korean".into());
        doc.set_title(&title);
    }
}
