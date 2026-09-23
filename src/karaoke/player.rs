use super::script::Script;
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy)]
pub struct Player {
    pub cursor: RwSignal<usize>,
    pub playing: RwSignal<bool>,
    pub done: RwSignal<bool>,
    pub start_at: usize,
    n: usize,
}

impl Player {
    pub fn start(script: &Script, autoplay: bool) -> Self {
        let n = script.token_count();
        let p = Self {
            cursor: RwSignal::new(0),
            playing: RwSignal::new(false),
            done: RwSignal::new(false),
            start_at: 0,
            n,
        };
        if autoplay {
            later(1800, move || p.playing.set(true));
        }
        let durs: Vec<u32> = script
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| t.playable())
            .map(|t| t.dur_ms)
            .collect();
        p.tick(durs);
        p
    }

    pub fn restart(self) {
        self.done.set(false);
        self.cursor.set(self.start_at);
    }

    fn tick(self, durs: Vec<u32>) {
        let n = self.n;
        Effect::new(move |_| {
            if !self.playing.get() {
                return;
            }
            let i = self.cursor.get();
            if i >= n {
                self.playing.set(false);
                self.done.set(true);
                return;
            }
            let dur = durs.get(i).copied().unwrap_or(800).max(1);
            later(dur, move || {
                if self.playing.get_untracked() {
                    if i + 1 < n {
                        self.cursor.set(i + 1);
                    } else {
                        self.playing.set(false);
                        self.done.set(true);
                    }
                }
            });
        });
    }
}

fn later(ms: u32, f: impl FnOnce() + 'static) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let cb = Closure::once_into_js(f);
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        cb.as_ref().unchecked_ref(),
        ms as i32,
    );
}
