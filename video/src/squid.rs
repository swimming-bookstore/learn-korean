//! Squid Game official trailer, used as a Korean lesson.
//!
//! Source: https://www.youtube.com/watch?v=oqxAJKy0ii4
//!
//! The picture plays once, on the left. The right side is remarks only.
//! Korean lines come from the trailer audio, not from a translation.
//!
//! A single speech-recognition pass breaks after about 20 seconds: the rest
//! of the clip is dropped or garbled. Windows stay under that limit, and the
//! lines below are the ones those windows heard.
//!
//! The cut is a frame, recognized from the trailer audio: the quiet after the
//! line, still before the next syllable. Trailer picture and trailer audio stop
//! on that frame. The explanation plays only while that frame is held. Then
//! both resume together on the next frame. A guessed second, or a 40 ms trim,
//! lets the picture jump while the voice is still talking.
//!
//! A short pause is under one output tick if the held frames keep the source
//! clock. fps then emits nothing, the card stays on the line, and the notes
//! are gone. Each held frame is one tick, so a short pause still shows them.
//! N*1001/24000/TB is one second per frame, so the highlight lands on the
//! note above the one being read. N/TB is one tick per frame.
//!
//! On the pause the voice reads the whole line, and the card lights that line.
//! Then it walks vocabulary one by one — Korean, then English — then each
//! grammar point, then its meaning, then the meaning of the line. Same order
//! as 퇴사할게여, and the same note sizes. A card taller than the panel
//! scrolls to the word being read. The sung line stays where it is. The
//! notes scroll under that line, not over it: a note pulled up to the top
//! of the card covers the sentence. The sentence is painted again after the
//! notes, on a paper cover, so a scrolled line cannot show through it.
//!
//! Vocabulary is the dictionary stem. A verb ends in 다 (처리되다, not
//! 처리된다). Grammar is that stem plus the particle or ending, the same
//! split as 참교육. 의 is "of", not a slash and not "'s".

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

const ROOT: &str = "/home/packer/learn-korean/work";
const TRAILER: &str = "/home/packer/learn-korean/work/squid/trailer.mp4";
const OUT: &str = "/home/packer/learn-korean/docs/squid-game.mp4";
const FONT_KR: &str = "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc";
const FONT_B: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf";
const FONT: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf";
const FPS: &str = "24000/1001";
const FPS_NUM: f64 = 24000.0;
const FPS_DEN: f64 = 1001.0;
/// Source is 3289 frames at 24000/1001. Frame N starts at N * 1001/24000.
const SRC_FRAMES: i64 = 3289;
/// 48000 Hz * 1001/24000. A frame and its audio samples are the same length.
const SAMPLES_PER_FRAME: i64 = 2002;
const SRC_END: f64 = 137.18;

/// One recognition request past this comes back broken. Windows stay under it.
const RECOGNIZE_LIMIT: f64 = 15.0;
const RECOGNIZE_OVERLAP: f64 = 2.0;

/// Remarks start on the first frame of the pause. A delay here is the
/// explanation starting late, over the next line once the picture resumes.
const VOICE_DELAY: f64 = 0.0;
const BEAT_GAP: f64 = 0.22;
const MEANING: &str = "0xE7D3A1";
const PAPER: &str = "0xF6F1E8";
const INK: &str = "0x141210";
const MUTED: &str = "0x8A8378";
const GOLD: &str = "0xFFE58A";
const BLUE: &str = "0xB7D7F5";
const HANJA: &str = "0x8C2F2F";
const PX: i32 = 918;

const END_KO: &str = "무궁화   ·   상금   ·   탈락";
const END_EN: &str = "hibiscus   ·   prize   ·   elimination";
const END_SAY_KO: &str = "무궁화. 상금. 탈락.";
const END_SAY_EN: &str = "Hibiscus. Prize. Elimination. 탈락으로 처리된다 means you are eliminated. That was the rule.";

struct Word {
    ko: &'static str,
    en: &'static str,
    hanja: &'static str,
}

const fn w(ko: &'static str, en: &'static str, hanja: &'static str) -> Word {
    Word { ko, en, hanja }
}

struct Lesson {
    hear_at: f64,
    pause_at: f64,
    ko: &'static [&'static str],
    en: &'static [&'static str],
    words: &'static [Word],
    grammar: &'static [Word],
    say: &'static str,
}

const fn g(ko: &'static str, en: &'static str) -> Word {
    w(ko, en, "")
}

/// Split `duration` into recognition windows. Each window is at most
/// [`RECOGNIZE_LIMIT`] seconds, so a request never crosses the 20-second
/// point where one long pass comes back broken. Overlap keeps a word that
/// sits on the cut.
fn recognition_windows(duration: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    if duration <= 0.05 {
        return out;
    }
    let mut t = 0.0;
    loop {
        let end = (t + RECOGNIZE_LIMIT).min(duration);
        out.push((t, end));
        if end >= duration - 0.05 {
            break;
        }
        t = end - RECOGNIZE_OVERLAP;
    }
    out
}

static LESSONS: &[Lesson] = &[
    Lesson {
        hear_at: 27.85,
        pause_at: 32.55,
        ko: &["여러분들은 모두", "감당할 수 없는 빚을 지고"],
        en: &["You are all carrying", "debts you cannot bear."],
        words: &[
            w("여러분", "you all", ""),
            w("모두", "all", ""),
            w("감당하다", "to bear", ""),
            w("수", "a way", ""),
            w("없다", "to not exist", ""),
            w("빚", "a debt", ""),
            w("지다", "to carry", ""),
        ],
        grammar: &[
            g("여러분들은", "여러분 + 들 + 은 (you all, topic)"),
            g("감당할 수 없는", "감당하다 + ㄹ 수 없다 + 는 (that you cannot bear)"),
            g("빚을", "빚 + 을 (the debt, object)"),
            g("지고", "지다 + 고 (carrying, and)"),
        ],
        say: "여러분들은 모두 감당할 수 없는 빚을 지고",
    },
    Lesson {
        hear_at: 33.95,
        pause_at: 35.55,
        ko: &["벼랑 끝에", "서 계신 분들입니다"],
        en: &["You are people standing", "at the edge of a cliff."],
        words: &[
            w("벼랑", "a cliff", ""),
            w("끝", "an edge", ""),
            w("서다", "to stand", ""),
            w("계시다", "to be", ""),
            w("분", "a person", ""),
            w("이다", "to be", ""),
        ],
        grammar: &[
            g("끝에", "끝 + 에 (at the edge)"),
            g("서 계신", "서다 + 계시다 + ㄴ (who are standing)"),
            g("분들입니다", "분 + 들 + 이다 + ㅂ니다 (are people)"),
        ],
        say: "벼랑 끝에 서 계신 분들입니다.",
    },
    Lesson {
        hear_at: 37.85,
        pause_at: 41.20,
        ko: &["게임의 참가를", "원하지 않는 분은"],
        en: &["Anyone who does not want", "to join the game."],
        words: &[
            w("게임", "a game", ""),
            w("참가", "joining", "參加"),
            w("원하다", "to want", ""),
            w("않다", "to not do", ""),
            w("분", "a person", ""),
        ],
        grammar: &[
            g("게임의", "게임 + 의 (of the game)"),
            g("참가를", "참가 + 를 (joining, object)"),
            g("원하지 않는", "원하다 + 지 않다 + 는 (who does not want)"),
            g("분은", "분 + 은 (a person, topic)"),
        ],
        say: "게임의 참가를 원하지 않는 분은",
    },
    Lesson {
        hear_at: 41.35,
        pause_at: 44.55,
        ko: &["지금 말씀해", "주시길 바랍니다"],
        en: &["Please say so", "now."],
        words: &[
            w("지금", "now", ""),
            w("말씀하다", "to say", ""),
            w("주시다", "to give", ""),
            w("바라다", "to hope", ""),
        ],
        grammar: &[
            g("말씀해", "말씀하다 + 어 (say)"),
            g("주시길", "주시다 + 기 + 를 (that you give)"),
            g("바랍니다", "바라다 + ㅂ니다 (we hope)"),
        ],
        say: "지금 말씀해 주시길 바랍니다.",
    },
    Lesson {
        hear_at: 49.85,
        pause_at: 52.45,
        ko: &["무궁화 꽃이", "피었습니다"],
        en: &["The hibiscus flower", "has bloomed."],
        words: &[
            w("무궁화", "hibiscus", "無窮花"),
            w("꽃", "a flower", ""),
            w("피다", "to bloom", ""),
        ],
        grammar: &[
            g("꽃이", "꽃 + 이 (the flower, subject)"),
            g("피었습니다", "피다 + 었 + 습니다 (has bloomed)"),
        ],
        say: "무궁화 꽃이 피었습니다.",
    },
    Lesson {
        hear_at: 53.20,
        pause_at: 55.85,
        ko: &["그 옛날에 어릴 때", "하던 거 말이야?"],
        en: &["You mean the one", "we played as kids?"],
        words: &[
            w("그", "that", ""),
            w("옛날", "the old days", ""),
            w("어리다", "to be young", ""),
            w("때", "a time", ""),
            w("하다", "to do", ""),
            w("거", "a thing", ""),
            w("말", "a word", ""),
        ],
        grammar: &[
            g("옛날에", "옛날 + 에 (in the old days)"),
            g("어릴 때", "어리다 + ㄹ 때 (when one was young)"),
            g("하던", "하다 + 던 (that we used to do)"),
            g("말이야", "말 + 이다 + 야 (you mean)"),
        ],
        say: "그 옛날에 어릴 때 하던 거 말이야?",
    },
    Lesson {
        hear_at: 59.95,
        pause_at: 61.55,
        ko: &["이 사람을", "그렇게 죽여 놓고"],
        en: &["After killing", "this person like that."],
        words: &[
            w("이", "this", ""),
            w("사람", "a person", ""),
            w("그렇게", "like that", ""),
            w("죽이다", "to kill", ""),
            w("놓다", "to leave", ""),
        ],
        grammar: &[
            g("사람을", "사람 + 을 (the person, object)"),
            g("죽여", "죽이다 + 어 (kill)"),
            g("놓고", "놓다 + 고 (and leave it)"),
        ],
        say: "이 사람을 그렇게 죽여 놓고.",
    },
    Lesson {
        hear_at: 63.15,
        pause_at: 66.95,
        ko: &["여섯 개의 게임을", "모두 이긴 분들께는"],
        en: &["To those who win", "all six games."],
        words: &[
            w("여섯", "six", ""),
            w("개", "a piece", ""),
            w("게임", "a game", ""),
            w("모두", "all", ""),
            w("이기다", "to win", ""),
            w("분", "a person", ""),
        ],
        grammar: &[
            g("개의", "개 + 의 (of pieces)"),
            g("게임을", "게임 + 을 (the games, object)"),
            g("이긴", "이기다 + ㄴ (who won)"),
            g("분들께는", "분 + 들 + 께 + 는 (to those people)"),
        ],
        say: "여섯 개의 게임을 모두 이긴 분들께는",
    },
    Lesson {
        hear_at: 67.05,
        pause_at: 70.15,
        ko: &["456억 원의", "상금이 지급됩니다"],
        en: &["A prize of 45.6 billion won", "will be paid."],
        words: &[
            w("456억", "45.6 billion", "億"),
            w("원", "won", "圓"),
            w("상금", "a prize", "賞金"),
            w("지급되다", "to be paid", "支給"),
        ],
        grammar: &[
            g("원의", "원 + 의 (of won)"),
            g("상금이", "상금 + 이 (the prize, subject)"),
            g("지급됩니다", "지급되다 + ㅂ니다 (is paid)"),
        ],
        say: "456억 원의 상금이 지급됩니다.",
    },
    Lesson {
        hear_at: 87.55,
        pause_at: 89.55,
        ko: &["눈 바짝", "뜨고 계세요"],
        en: &["Keep your eyes", "wide open."],
        words: &[
            w("눈", "an eye", ""),
            w("바짝", "wide", ""),
            w("뜨다", "to open", ""),
            w("계시다", "to stay", ""),
        ],
        grammar: &[
            g("뜨고", "뜨다 + 고 (open, and)"),
            g("계세요", "계시다 + 세요 (please stay)"),
        ],
        say: "눈 바짝 뜨고 계세요.",
    },
    Lesson {
        hear_at: 89.70,
        pause_at: 91.45,
        ko: &["제일 센 쪽에", "붙어"],
        en: &["Stick to", "the strongest side."],
        words: &[
            w("제일", "the most", "第一"),
            w("세다", "to be strong", ""),
            w("쪽", "a side", ""),
            w("붙다", "to stick", ""),
        ],
        grammar: &[
            g("센", "세다 + ㄴ (that is strong)"),
            g("쪽에", "쪽 + 에 (to the side)"),
            g("붙어", "붙다 + 어 (stick)"),
        ],
        say: "제일 센 쪽에 붙어.",
    },
    Lesson {
        hear_at: 91.55,
        pause_at: 92.85,
        ko: &["그게 살 길이야"],
        en: &["That is how you live."],
        words: &[
            w("그게", "that", ""),
            w("살", "life", "生"),
            w("길", "a way", ""),
            w("이다", "to be", ""),
        ],
        grammar: &[
            g("살 길이야", "살 + 길 + 이다 + 야 (is the way to live)"),
        ],
        say: "그게 살 길이야.",
    },
    Lesson {
        hear_at: 94.70,
        pause_at: 96.45,
        ko: &["애들 게임", "규칙 단순해"],
        en: &["A kids' game.", "The rules are simple."],
        words: &[
            w("애들", "kids", ""),
            w("게임", "a game", ""),
            w("규칙", "a rule", "規則"),
            w("단순하다", "to be simple", "單純"),
        ],
        grammar: &[
            g("단순해", "단순하다 + 어 (is simple)"),
        ],
        say: "애들 게임 규칙 단순해.",
    },
    Lesson {
        hear_at: 97.55,
        pause_at: 99.45,
        ko: &["내가 무조건", "이기게 해 줄게"],
        en: &["I will make sure", "you win."],
        words: &[
            w("내", "I", ""),
            w("무조건", "no matter what", "無條件"),
            w("이기다", "to win", ""),
            w("하다", "to do", ""),
            w("주다", "to give", ""),
        ],
        grammar: &[
            g("내가", "내 + 가 (I, subject)"),
            g("이기게", "이기다 + 게 (so that you win)"),
            g("해 줄게", "하다 + 어 주다 + ㄹ게 (I will do it for you)"),
        ],
        say: "내가 무조건 이기게 해 줄게.",
    },
    Lesson {
        hear_at: 99.55,
        pause_at: 100.55,
        ko: &["무슨 수로?"],
        en: &["By what means?"],
        words: &[
            w("무슨", "what", ""),
            w("수", "a means", ""),
        ],
        grammar: &[
            g("수로", "수 + 로 (by means)"),
        ],
        say: "무슨 수로?",
    },
    Lesson {
        hear_at: 100.70,
        pause_at: 102.55,
        ko: &["내가 앞을", "봐 줄게"],
        en: &["I will watch", "the front for you."],
        words: &[
            w("내", "I", ""),
            w("앞", "the front", ""),
            w("보다", "to look", ""),
            w("주다", "to give", ""),
        ],
        grammar: &[
            g("내가", "내 + 가 (I, subject)"),
            g("앞을", "앞 + 을 (the front, object)"),
            g("봐 줄게", "보다 + 아 주다 + ㄹ게 (I will look for you)"),
        ],
        say: "내가 앞을 봐 줄게.",
    },
    Lesson {
        hear_at: 102.85,
        pause_at: 105.15,
        ko: &["이러면 안 되는", "거잖아"],
        en: &["We are not supposed", "to do this."],
        words: &[
            w("이러다", "to do this", ""),
            w("안", "not", ""),
            w("되다", "to be allowed", ""),
            w("거", "a thing", ""),
        ],
        grammar: &[
            g("이러면", "이러다 + 면 (if we do this)"),
            g("안 되는", "되다 + 는 (that is not allowed)"),
            g("거잖아", "거 + 이다 + 잖아 (you know it is)"),
        ],
        say: "이러면 안 되는 거잖아.",
    },
    Lesson {
        hear_at: 105.30,
        pause_at: 107.55,
        ko: &["우리끼리 이렇게", "죽이면 안 되는 거잖아"],
        en: &["We are not supposed to", "kill each other like this."],
        words: &[
            w("우리", "we", ""),
            w("끼리", "among", ""),
            w("이렇게", "like this", ""),
            w("죽이다", "to kill", ""),
            w("안", "not", ""),
            w("되다", "to be allowed", ""),
            w("거", "a thing", ""),
        ],
        grammar: &[
            g("우리끼리", "우리 + 끼리 (among us)"),
            g("죽이면", "죽이다 + 면 (if we kill)"),
            g("안 되는", "되다 + 는 (that is not allowed)"),
            g("거잖아", "거 + 이다 + 잖아 (you know it is)"),
        ],
        say: "우리끼리 이렇게 죽이면 안 되는 거잖아.",
    },
    Lesson {
        hear_at: 113.35,
        pause_at: 116.55,
        ko: &["게임을 거부하는", "참가자는", "탈락으로 처리된다"],
        en: &["A player who refuses the game", "is treated as eliminated."],
        words: &[
            w("게임", "a game", ""),
            w("거부하다", "to refuse", "拒否"),
            w("참가자", "a player", "參加者"),
            w("탈락", "elimination", "脫落"),
            w("처리되다", "to be treated", "處理"),
        ],
        grammar: &[
            g("게임을", "게임 + 을 (the game, object)"),
            g("거부하는", "거부하다 + 는 (who refuses)"),
            g("참가자는", "참가자 + 는 (a player, topic)"),
            g("탈락으로", "탈락 + 으로 (as elimination)"),
            g("처리된다", "처리되다 + ㄴ다 (is treated)"),
        ],
        say: "게임을 거부하는 참가자는 탈락으로 처리된다.",
    },
];

/// Text from windows that start at or after `at`. A long shot that dies at
/// 20 seconds does not contain these lines.
#[cfg(test)]
fn heard_from(chunks: &[(f64, &str)], at: f64) -> String {
    chunks
        .iter()
        .filter(|(start, text)| *start + 0.01 >= at && !text.is_empty())
        .map(|(_, text)| text.trim())
        .collect::<Vec<_>>()
        .join(" ")
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let draft = args.iter().any(|a| a == "--draft");
    let plan_only = args.iter().any(|a| a == "--plan");
    let only = args.iter().find_map(|a| a.strip_prefix("--only="));
    validate()?;
    let selected: Vec<usize> = if let Some(only) = only {
        let i: usize = only.parse().map_err(|_| format!("bad --only={only}"))?;
        if i >= LESSONS.len() {
            return Err(format!("--only={i} past the last line"));
        }
        vec![i]
    } else if draft {
        (0..2).collect()
    } else {
        (0..LESSONS.len()).collect()
    };
    if plan_only {
        print_plan(&selected);
        return Ok(());
    }
    if !Path::new(TRAILER).exists() {
        return Err(format!("missing source video {TRAILER}"));
    }
    let prepared = prepare_voices(&selected)?;
    let end = if draft || only.is_some() {
        None
    } else {
        Some(prepare_end()?)
    };
    let (script, frames) = build(&selected, &prepared, end.as_ref())?;
    render(&script, &prepared, end.as_ref(), draft, frames)?;
    Ok(())
}

fn validate() -> Result<(), String> {
    if LESSONS.len() != 19 {
        return Err(format!("expected 19 lessons, got {}", LESSONS.len()));
    }
    let mut t = 0.0;
    for lesson in LESSONS {
        if lesson.hear_at + 0.001 < t {
            return Err(format!("cue overlap at {}", lesson.hear_at));
        }
        if lesson.pause_at <= lesson.hear_at {
            return Err(format!("empty cue at {}", lesson.hear_at));
        }
        if lesson.say.contains("비자에") || lesson.say.contains("설매") {
            return Err(format!("broken recognition kept: {}", lesson.say));
        }
        t = lesson.pause_at;
    }
    if t > SRC_END {
        return Err(format!("last cue {t} past source {SRC_END}"));
    }
    for (start, end) in recognition_windows(SRC_END) {
        if end - start > RECOGNIZE_LIMIT + 0.001 || end - start >= 20.0 {
            return Err(format!("recognition window {start:.2}-{end:.2} crosses 20s"));
        }
    }
    Ok(())
}

fn print_plan(indexes: &[usize]) {
    for (start, end) in recognition_windows(SRC_END) {
        println!("recognize {start:6.2}-{end:6.2}");
    }
    for i in indexes {
        let lesson = &LESSONS[*i];
        println!(
            "teach {:7.3}-{:7.3}  {}",
            lesson.hear_at, lesson.pause_at, lesson.say
        );
    }
}

fn vo_dir() -> PathBuf {
    PathBuf::from(ROOT).join("squid-vo")
}

fn txt_dir() -> PathBuf {
    PathBuf::from(ROOT).join("squid-ko")
}

struct Beat {
    kind: char,
    lang: &'static str,
    spoken: String,
    shown: String,
    hanja: String,
}

struct Row {
    kind: char,
    ko: String,
    en: String,
    hanja: String,
    ko_t: f64,
    hi_end: f64,
}

struct Prepared {
    beats: Vec<Beat>,
    beat_times: Vec<f64>,
    hold_frames: i64,
    wav: PathBuf,
}

fn prepare_voices(indexes: &[usize]) -> Result<Vec<Prepared>, String> {
    let dir = vo_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let gap = dir.join("beat_gap.wav");
    silence(&gap, BEAT_GAP)?;
    let mut out = Vec::new();
    for i in indexes {
        let lesson = &LESSONS[*i];
        let beats = lesson_beats(lesson);
        let mut parts = Vec::new();
        let mut durs = Vec::new();
        for (j, beat) in beats.iter().enumerate() {
            let clip = cached_voice(&beat.spoken, beat.lang)?;
            durs.push(wav_duration(&clip)?);
            parts.push(clip);
            if j + 1 < beats.len() {
                parts.push(gap.clone());
            }
        }
        let path = dir.join(format!("{i:02}.wav"));
        concat_wavs(&parts, &path)?;
        let dur = wav_duration(&path)?;
        let times = walk_card(&durs);
        let hold_frames = frames_for(dur + 0.40).max(frames_for(2.0));
        println!(
            "vo {i:02} {dur:5.2}s  hold {hold_frames}f  {} beats  {}",
            beats.len(),
            lesson.say
        );
        out.push(Prepared {
            beats,
            beat_times: times,
            hold_frames,
            wav: path,
        });
    }
    Ok(out)
}

fn prepare_end() -> Result<(PathBuf, i64), String> {
    let dir = vo_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let phrase = cached_voice(END_SAY_KO, "ko")?;
    let note = cached_voice(END_SAY_EN, "en")?;
    let gap = dir.join("end_gap.wav");
    silence(&gap, 0.40)?;
    let path = dir.join("end.wav");
    concat_wavs(&[phrase, gap, note], &path)?;
    let dur = wav_duration(&path)?;
    let card = frames_for(dur + 0.80).max(frames_for(5.0));
    println!("vo end {dur:5.2}s  card {card}f");
    Ok((path, card))
}

fn lesson_beats(lesson: &Lesson) -> Vec<Beat> {
    let mut beats = Vec::new();
    // The trailer just said it. Read the whole line again before the notes,
    // not the first vocab card alone.
    beats.push(Beat {
        kind: 's',
        lang: "ko",
        spoken: lesson
            .say
            .trim()
            .trim_end_matches(['.', '。'])
            .to_string(),
        shown: lesson.ko.join(" "),
        hanja: String::new(),
    });
    for word in lesson.words {
        let spoken = if word.ko.trim_start().starts_with(['-', '−']) {
            speakable(word.ko)
        } else {
            word.ko.trim().to_string()
        };
        beats.push(Beat {
            kind: 'v',
            lang: "ko",
            spoken,
            shown: word.ko.to_string(),
            hanja: word.hanja.split_whitespace().collect(),
        });
        beats.push(Beat {
            kind: 'v',
            lang: "en",
            spoken: word.en.trim().to_string(),
            shown: word.en.to_string(),
            hanja: String::new(),
        });
    }
    for note in lesson.grammar {
        beats.push(Beat {
            kind: 'g',
            lang: "ko",
            spoken: speakable(note.ko),
            shown: note.ko.to_string(),
            hanja: String::new(),
        });
        beats.push(Beat {
            kind: 'g',
            lang: "en",
            spoken: speakable(note.en),
            shown: note.en.to_string(),
            hanja: String::new(),
        });
    }
    let meaning = lesson
        .en
        .iter()
        .map(|s| s.trim())
        .collect::<Vec<_>>()
        .join(" ");
    beats.push(Beat {
        kind: 'm',
        lang: "en",
        spoken: meaning.clone(),
        shown: meaning,
        hanja: String::new(),
    });
    beats
}

fn walk_card(durs: &[f64]) -> Vec<f64> {
    let mut times = Vec::new();
    let mut cursor = VOICE_DELAY;
    for (i, dur) in durs.iter().enumerate() {
        times.push(cursor);
        cursor += dur;
        if i + 1 < durs.len() {
            cursor += BEAT_GAP;
        }
    }
    times
}

fn card_plan(prep: &Prepared, lesson: &Lesson) -> Vec<Row> {
    let beats = &prep.beats;
    let times = &prep.beat_times;
    let mut rows = Vec::new();
    let mut i = 0;
    while i < beats.len() {
        if beats[i].kind == 'm' || beats[i].kind == 's' {
            rows.push(Row {
                kind: beats[i].kind,
                ko: beats[i].shown.clone(),
                en: if beats[i].kind == 'm' {
                    lesson.en.join("\n")
                } else {
                    String::new()
                },
                hanja: String::new(),
                ko_t: times[i],
                hi_end: times.get(i + 1).copied().unwrap_or(999.0),
            });
            i += 1;
            continue;
        }
        rows.push(Row {
            kind: beats[i].kind,
            ko: beats[i].shown.clone(),
            en: beats[i + 1].shown.clone(),
            hanja: beats[i].hanja.clone(),
            ko_t: times[i],
            hi_end: times.get(i + 2).copied().unwrap_or(999.0),
        });
        i += 2;
    }
    rows
}

fn speakable(text: &str) -> String {
    // `+` is the parse mark. A slash is not a particle, and parentheses are
    // not spoken, or the voice says "or" and "parenthesis".
    let cleaned = text
        .replace(['−', '-', '/', '+', '(', ')'], " ")
        .replace("...", " ");
    let joined = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    if joined.is_empty() {
        text.trim().to_string()
    } else {
        joined
    }
}

fn font_for(text: &str, bold: bool) -> &'static str {
    if text.chars().any(|ch| !ch.is_ascii()) {
        FONT_KR
    } else if bold {
        FONT_B
    } else {
        FONT
    }
}

fn text_px(text: &str, size: i32) -> i32 {
    let mut px = 0.0;
    for ch in text.chars() {
        px += if ch == ' ' {
            size as f32 * 0.33
        } else if ch.is_ascii() {
            size as f32 * 0.55
        } else {
            size as f32
        };
    }
    px.round() as i32
}

/// Hanja sits just after that Korean word, not on the card's right edge.
fn hanja_x(ko_x: i32, ko: &str, size: i32) -> i32 {
    ko_x + text_px(ko, size) + 8
}

fn wrap_en(text: &str, limit: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        let nxt = if cur.is_empty() {
            word.to_string()
        } else {
            format!("{cur} {word}")
        };
        if nxt.chars().count() > limit && !cur.is_empty() {
            lines.push(cur);
            cur = word.to_string();
        } else {
            cur = nxt;
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    if lines.is_empty() {
        lines.push(text.to_string());
    }
    lines
}

fn lock_dur(dur: f64) -> f64 {
    secs(frames_for(dur))
}

fn frames_for(dur: f64) -> i64 {
    (dur * FPS_NUM / FPS_DEN).ceil().max(1.0) as i64
}

fn secs(frame: i64) -> f64 {
    frame as f64 * FPS_DEN / FPS_NUM
}

fn samples_for(frames: i64) -> i64 {
    frames * SAMPLES_PER_FRAME
}

fn percent_encode(text: &str) -> String {
    let mut out = String::new();
    for b in text.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn cache_key(lang: &str, text: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in format!("{lang}\n{text}").bytes() {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn cached_voice(text: &str, lang: &str) -> Result<PathBuf, String> {
    let path = vo_dir().join(format!("c_{lang}_{}.wav", cache_key(lang, text)));
    if path.exists() && fs::metadata(&path).map(|m| m.len() > 400).unwrap_or(false) {
        return Ok(path);
    }
    synth(text, &path, lang)?;
    Ok(path)
}

fn synth(text: &str, path: &Path, lang: &str) -> Result<(), String> {
    let mp3 = path.with_extension("mp3");
    let mut last = "no attempt".to_string();
    for attempt in 0..4 {
        let url = format!(
            "https://translate.google.com/translate_tts?ie=UTF-8&client=tw-ob&tl={}&q={}",
            lang,
            percent_encode(text.trim())
        );
        let status = Command::new("curl")
            .args(["-fsSL", "-A", "Mozilla/5.0", "--max-time", "30", "-o"])
            .arg(&mp3)
            .arg(&url)
            .status();
        match status {
            Ok(code) if code.success() => {
                let len = fs::metadata(&mp3).map(|m| m.len()).unwrap_or(0);
                if len >= 800 {
                    ffmpeg(&[
                        "-y",
                        "-i",
                        mp3.to_str().unwrap(),
                        "-ar",
                        "22050",
                        "-ac",
                        "1",
                        path.to_str().unwrap(),
                    ])?;
                    thread::sleep(Duration::from_millis(350));
                    return Ok(());
                }
                last = format!("short reply {len}");
            }
            Ok(code) => last = format!("curl {code}"),
            Err(err) => last = err.to_string(),
        }
        thread::sleep(Duration::from_millis(1400 * (attempt + 1)));
    }
    Err(format!("tts failed ({lang}): {last}: {text}"))
}

fn silence(path: &Path, seconds: f64) -> Result<(), String> {
    if path.exists() && fs::metadata(path).map(|m| m.len() > 400).unwrap_or(false) {
        return Ok(());
    }
    ffmpeg(&[
        "-y",
        "-f",
        "lavfi",
        "-i",
        &format!("anullsrc=r=22050:cl=mono:d={seconds:.2}"),
        path.to_str().unwrap(),
    ])
}

fn concat_wavs(parts: &[PathBuf], out: &Path) -> Result<(), String> {
    let list = out.with_extension("txt");
    let mut body = String::new();
    for part in parts {
        body.push_str("file '");
        body.push_str(part.to_str().unwrap());
        body.push_str("'\n");
    }
    fs::write(&list, body).map_err(|e| e.to_string())?;
    ffmpeg(&[
        "-y",
        "-f",
        "concat",
        "-safe",
        "0",
        "-i",
        list.to_str().unwrap(),
        "-ar",
        "22050",
        "-ac",
        "1",
        "-af",
        "loudnorm=I=-16:TP=-1.5:LRA=7",
        out.to_str().unwrap(),
    ])
}

fn wav_duration(path: &Path) -> Result<f64, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hdr = [0u8; 12];
    file.read_exact(&mut hdr).map_err(|e| e.to_string())?;
    if &hdr[0..4] != b"RIFF" || &hdr[8..12] != b"WAVE" {
        return Err(format!("not a wav: {}", path.display()));
    }
    let mut rate = 0u32;
    let mut channels = 0u16;
    let mut bits = 0u16;
    let mut data_len = 0u32;
    loop {
        let mut chunk = [0u8; 8];
        if file.read_exact(&mut chunk).is_err() {
            break;
        }
        let size = u32::from_le_bytes(chunk[4..8].try_into().unwrap());
        if &chunk[0..4] == b"fmt " {
            let mut fmt = vec![0u8; size as usize];
            file.read_exact(&mut fmt).map_err(|e| e.to_string())?;
            if fmt.len() < 16 {
                return Err(format!("short fmt: {}", path.display()));
            }
            channels = u16::from_le_bytes(fmt[2..4].try_into().unwrap());
            rate = u32::from_le_bytes(fmt[4..8].try_into().unwrap());
            bits = u16::from_le_bytes(fmt[14..16].try_into().unwrap());
            if size % 2 == 1 {
                let mut pad = [0u8; 1];
                file.read_exact(&mut pad).ok();
            }
        } else if &chunk[0..4] == b"data" {
            data_len = size;
            break;
        } else {
            let skip = size + (size % 2);
            let mut buf = vec![0u8; skip as usize];
            file.read_exact(&mut buf).map_err(|e| e.to_string())?;
        }
    }
    if rate == 0 || channels == 0 || bits == 0 {
        return Err(format!("bad wav header: {}", path.display()));
    }
    let frame = u32::from(channels) * u32::from(bits) / 8;
    Ok(f64::from(data_len) / f64::from(frame) / f64::from(rate))
}

fn ffmpeg(args: &[&str]) -> Result<(), String> {
    let out = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-loglevel")
        .arg("error")
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        return Ok(());
    }
    let err = String::from_utf8_lossy(&out.stderr);
    Err(format!("ffmpeg failed: {}", tail(&err, 1800)))
}

fn tail(text: &str, n: usize) -> String {
    let count = text.chars().count();
    if count <= n {
        text.to_string()
    } else {
        text.chars().skip(count - n).collect()
    }
}

fn write_text(name: &str, text: &str) -> Result<PathBuf, String> {
    let dir = txt_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(name);
    let mut file = File::create(&path).map_err(|e| e.to_string())?;
    writeln!(file, "{text}").map_err(|e| e.to_string())?;
    Ok(path)
}

fn draw_line(font: &str, file: &Path, size: i32, color: &str, x: i32, y: i32) -> String {
    format!(
        "drawtext=fontfile={font}:textfile={}:fontsize={size}:fontcolor={color}:x={x}:y={y}",
        file.display()
    )
}

fn draw_timed(
    font: &str,
    file: &Path,
    size: i32,
    color: &str,
    x: i32,
    y: &str,
    start: f64,
    end: f64,
) -> String {
    format!(
        "drawtext=fontfile={font}:textfile={}:fontsize={size}:fontcolor={color}:x={x}:y={y}:enable='between(t,{start:.3},{end:.3})'",
        file.display()
    )
}

fn draw_box(x: i32, y: &str, h: i32, color: &str, start: f64, end: f64) -> String {
    format!(
        "drawbox=x={x}:y={y}:w=352:h={h}:color={color}:t=fill:enable='between(t,{start:.3},{end:.3})'"
    )
}

/// Notes use the same sizes as 퇴사할게여. A long card is taller than the
/// panel, so it scrolls to the word being read. The sung line stays put.
/// A note stops at the gap under that line. Scrolling it to the top of the
/// card is what laid the vocab box over the sentence.
/// `stops` are (time, shift). Shift is how far the notes move up.
fn note_shift(layout: &[Note], floor: i32) -> Vec<(f64, i32)> {
    let mut shift = 0i32;
    let mut stops = Vec::new();
    for note in layout {
        let bottom = note.y + note.h - shift;
        if bottom > 700 {
            let next = shift + (bottom - 700) + 8;
            // Each note keeps its own gap under the line. A shift saved from
            // a taller note above it pulls this one up onto the sentence.
            let max_shift = (note.y - floor).max(0);
            let capped = next.min(max_shift);
            if capped != shift {
                stops.push((note.at, capped));
            }
            shift = capped;
        }
    }
    stops
}

/// `drawbox` reads `t` once, when the filter starts, and that `t` is the
/// whole pause. A scroll expression then jumps to the last stop, and the
/// first word is drawn on the sentence. The position has to be a number.
/// The pause is split at each stop so each stretch has its own number.
fn scroll_spans(stops: &[(f64, i32)], frames: i64) -> Vec<(i64, i64, i32)> {
    let mut spans = Vec::new();
    let mut cursor = 0i64;
    let mut shift = 0i32;
    for (t, next) in stops {
        let at = frame_at(*t).clamp(0, frames);
        if at > cursor {
            spans.push((cursor, at, shift));
            cursor = at;
        }
        shift = *next;
    }
    if cursor < frames {
        spans.push((cursor, frames, shift));
    }
    spans
}

fn pause_stops(layout: &[Note], stops: &[(f64, i32)]) -> Vec<(f64, i32)> {
    // The pause clock starts at the first note, not at the first note that
    // overflows. Measuring from the overflow jumps the card before that word
    // is read, and the vocab box lands on the sentence.
    let origin = layout.iter().map(|n| n.at).fold(f64::MAX, f64::min);
    stops
        .iter()
        .map(|(t, shift)| ((t - origin).max(0.0), *shift))
        .collect()
}

fn header_draws() -> Result<Vec<String>, String> {
    let mut draws = vec![
        format!("drawbox=x=0:y=0:w=iw:h=58:color={PAPER}:t=fill"),
        "drawbox=x=0:y=56:w=900:h=3:color=0x141210:t=fill".to_string(),
        "drawbox=x=898:y=58:w=3:h=662:color=0xD9D0C2:t=fill".to_string(),
    ];
    let kicker = write_text("h_kicker.txt", "Learn Korean with")?;
    draws.push(draw_line(FONT_B, &kicker, 16, "0x1E6F8A", 22, 20));
    let title = write_text("h_title.txt", "오징어 게임")?;
    draws.push(draw_line(FONT_KR, &title, 22, INK, 196, 16));
    Ok(draws)
}

fn panel_draws(
    mode: &str,
    lesson: &Lesson,
    prep: Option<&Prepared>,
    index: usize,
    n: usize,
    shift: i32,
    origin: f64,
) -> Result<Vec<String>, String> {
    let mut draws = Vec::new();
    let x = PX;
    if mode == "quiet" {
        return Ok(draws);
    }
    let (tag, tag_c) = match mode {
        "hear" => ("HEAR THIS", "0x1E6F8A"),
        "hold" => ("PAUSE", "0x1E6F8A"),
        _ => ("NEXT", MUTED),
    };
    let tag_file = write_text(&format!("{index:02}_{mode}_tag.txt"), tag)?;
    draws.push(draw_line(FONT_B, &tag_file, 15, tag_c, x, 78));
    let num = write_text(
        &format!("{index:02}_{mode}_num.txt"),
        &format!("{:02}  /  {n:02}", index + 1),
    )?;
    draws.push(draw_line(FONT, &num, 14, MUTED, x, 100));

    let longest = lesson.ko.iter().map(|s| s.chars().count()).max().unwrap_or(0);
    let size: i32 = if lesson.ko.len() > 2 || longest > 11 {
        24
    } else if longest > 8 {
        28
    } else {
        32
    };
    let title_y = 138;
    let rows = if mode == "hold" {
        let prep = prep.ok_or("hold without a voice")?;
        Some(card_plan(prep, lesson))
    } else {
        None
    };
    let mut sentence_box = None;
    if let Some(rows) = rows.as_ref() {
        if let Some(row) = rows.iter().find(|r| r.kind == 's') {
            let box_h = 10 + lesson.ko.len() as i32 * (size + 6);
            sentence_box = Some(format!(
                "drawbox=x={}:y={}:w=352:h={box_h}:color={GOLD}:t=fill:enable='between(t,{:.3},{:.3})'",
                x - 8,
                title_y - 6,
                (row.ko_t - origin).max(0.0),
                (row.hi_end - origin).max(0.0)
            ));
        }
    }
    let mut y = title_y;
    let mut sentence = Vec::new();
    for (j, line) in lesson.ko.iter().enumerate() {
        let file = write_text(&format!("{index:02}_{mode}_ko{j}.txt"), line)?;
        sentence.push(draw_line(FONT_KR, &file, size, INK, x, y));
        y += size + 6;
    }
    let Some(rows) = rows else {
        draws.extend(sentence);
        return Ok(draws);
    };
    // Same gap and sizes as 퇴사할게여. Shrinking the notes was what made
    // vocabulary, grammar, and meaning look smaller than that lesson.
    let layout = note_layout(lesson, &rows);
    // A number, not an expression. drawbox bakes a `t` expression at the
    // end of the pause, which is what put the first word on the sentence.
    for note in &layout {
        let y = note.y - shift;
        let yexpr = y.to_string();
        let at = (note.at - origin).max(0.0);
        let hi = (note.hi_end - origin).max(0.0);
        if note.kind == 'h' {
            let file = write_text(&format!("{index:02}_sec{}.txt", note.j), &note.text)?;
            draws.push(draw_timed(FONT_B, &file, 12, MUTED, x, &yexpr, at, 999.0));
            continue;
        }
        if note.kind == 'm' {
            draws.push(draw_box(x - 8, &yexpr, note.h, MEANING, at, hi));
            for (k, line) in note.en.iter().enumerate() {
                let file = write_text(&format!("{index:02}_m{k}.txt"), line)?;
                let dy = 4 + 22 * k as i32;
                draws.push(draw_timed(
                    font_for(line, false),
                    &file,
                    16,
                    INK,
                    x,
                    &format!("({yexpr})+{dy}"),
                    at,
                    999.0,
                ));
            }
            continue;
        }
        let color = if note.kind == 'v' { GOLD } else { BLUE };
        draws.push(draw_box(x - 8, &yexpr, note.h, color, at, hi));
        let file = write_text(&format!("{index:02}_w{}.txt", note.j), &note.text)?;
        draws.push(draw_timed(
            font_for(&note.text, false),
            &file,
            18,
            INK,
            x,
            &format!("({yexpr})+4"),
            at,
            999.0,
        ));
        if !note.hanja.is_empty() {
            let file = write_text(&format!("{index:02}_hz{}.txt", note.j), &note.hanja)?;
            let hx = hanja_x(x, &note.text, 18);
            draws.push(draw_timed(
                FONT_KR,
                &file,
                16,
                HANJA,
                hx,
                &format!("({yexpr})+5"),
                at,
                999.0,
            ));
        }
        for (k, line) in note.en.iter().enumerate() {
            let file = write_text(&format!("{index:02}_we{}_{k}.txt", note.j), line)?;
            let dy = 26 + 18 * k as i32;
            draws.push(draw_timed(
                font_for(line, false),
                &file,
                14,
                "0x5C574E",
                x,
                &format!("({yexpr})+{dy}"),
                at,
                999.0,
            ));
        }
    }
    // Notes are drawn after the sentence, so a scrolled line shows through it.
    // The gap under the line is part of that band. Cover both, then put the
    // sentence back on top.
    let cover_h = y + 18 - (title_y - 6);
    draws.push(format!(
        "drawbox=x={}:y={}:w=364:h={cover_h}:color={PAPER}:t=fill",
        x - 8,
        title_y - 6
    ));
    if let Some(box_draw) = sentence_box {
        draws.push(box_draw);
    }
    draws.extend(sentence);
    Ok(draws)
}

struct Note {
    at: f64,
    y: i32,
    h: i32,
    kind: char,
    text: String,
    en: Vec<String>,
    hanja: String,
    hi_end: f64,
    j: usize,
}

fn note_layout(lesson: &Lesson, rows: &[Row]) -> Vec<Note> {
    let longest = lesson.ko.iter().map(|s| s.chars().count()).max().unwrap_or(0);
    let size: i32 = if lesson.ko.len() > 2 || longest > 11 {
        24
    } else if longest > 8 {
        28
    } else {
        32
    };
    let mut y = 138 + lesson.ko.len() as i32 * (size + 6) + 18;
    let mut layout = Vec::new();
    let mut head = ' ';
    for (j, row) in rows.iter().enumerate() {
        if row.kind == 's' {
            continue;
        }
        if row.kind != head {
            let label = match row.kind {
                'v' => "VOCABULARY",
                'g' => "GRAMMAR",
                _ => "MEANING",
            };
            layout.push(Note {
                at: row.ko_t,
                y,
                h: 18,
                kind: 'h',
                text: label.to_string(),
                en: Vec::new(),
                hanja: String::new(),
                hi_end: row.hi_end,
                j,
            });
            y += 22;
            head = row.kind;
        }
        if row.kind == 'm' {
            let n = lesson.en.len() as i32;
            let box_h = 8 + 22 * n;
            layout.push(Note {
                at: row.ko_t,
                y: y - 4,
                h: box_h,
                kind: 'm',
                text: String::new(),
                en: lesson.en.iter().map(|s| s.to_string()).collect(),
                hanja: String::new(),
                hi_end: row.hi_end,
                j,
            });
            y += box_h + 4;
            continue;
        }
        let en_lines = wrap_en(&row.en, 32);
        let box_h = 26 + 18 * en_lines.len() as i32;
        layout.push(Note {
            at: row.ko_t,
            y: y - 4,
            h: box_h,
            kind: row.kind,
            text: row.ko.clone(),
            en: en_lines,
            hanja: row.hanja.clone(),
            hi_end: row.hi_end,
            j,
        });
        y += box_h + 6;
    }
    layout
}

fn stage(label: &str, src: &str, frames: i64) -> (String, String) {
    let out = format!("{label}s");
    let pad = secs(frames) + 1.0;
    let chain = format!(
        "[{src}]scale=900:506:flags=lanczos:force_original_aspect_ratio=disable,setsar=1,\
         pad=900:720:0:107:color={PAPER},fps={FPS},settb=1001/24000,setpts=N,trim=end_frame={frames},setpts=N[{label}p];\
         color=c={PAPER}:s=380x720:r={FPS}:d={pad:.3},fps={FPS},settb=1001/24000,trim=end_frame={frames},setpts=N[{label}c];\
         [{label}p][{label}c]hstack=inputs=2[{out}]"
    );
    (chain, out)
}

/// Draws, then cut to exactly `frames` so a filter cannot invent or drop time.
fn lock_picture(staged: &str, tag: &str, draws: &[String], frames: i64) -> String {
    format!(
        "[{staged}]{},fps={FPS},settb=1001/24000,trim=end_frame={frames},setpts=N,format=yuv420p[{tag}]",
        draws.join(",")
    )
}

fn lock_trailer_audio(tag: &str, start: i64, end: i64) -> String {
    let frames = end - start;
    let samples = samples_for(frames);
    format!(
        "[0:a]atrim=start={:.6}:end={:.6},asetpts=PTS-STARTPTS,\
         aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,\
         apad=whole_len={samples},atrim=end_sample={samples}[{tag}a]",
        secs(start),
        secs(end)
    )
}

/// The explanation is the only sound. Trailer audio is not in this chain, so it stays paused.
fn lock_voice_audio(tag: &str, input: usize, frames: i64) -> String {
    let samples = samples_for(frames);
    format!(
        "[{input}:a]aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,\
         volume=2.4,afade=t=in:d=0.02,\
         apad=whole_len={samples},atrim=end_sample={samples},alimiter=limit=0.9[{tag}a]"
    )
}

fn lock_voice_slice(tag: &str, input: usize, start: i64, end: i64) -> String {
    let frames = end - start;
    let samples = samples_for(frames);
    // Fade only the first stretch. A fade on every scroll cut dips the voice.
    let fade = if start == 0 { ",afade=t=in:d=0.02" } else { "" };
    format!(
        "[{input}:a]aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,\
         atrim=start={:.6}:end={:.6},asetpts=PTS-STARTPTS,volume=2.4{fade},\
         apad=whole_len={samples},atrim=end_sample={samples},alimiter=limit=0.9[{tag}a]",
        secs(start),
        secs(end)
    )
}

fn frame_at(t: f64) -> i64 {
    (t * FPS_NUM / FPS_DEN).round() as i64
}

fn build(
    indexes: &[usize],
    prepared: &[Prepared],
    end: Option<&(PathBuf, i64)>,
) -> Result<(PathBuf, i64), String> {
    let n = LESSONS.len();
    let mut chains = Vec::new();
    let mut vlabels = Vec::new();
    let mut alabels = Vec::new();
    let mut cursor = 0i64;
    let mut total = 0i64;
    let mut seg_i = 0usize;
    let header = header_draws()?;

    for (prep_i, index) in indexes.iter().copied().enumerate() {
        let lesson = &LESSONS[index];
        let prep = &prepared[prep_i];
        let start = frame_at(lesson.hear_at);
        let end_frame = frame_at(lesson.pause_at);
        if end_frame <= start {
            return Err(format!("empty cut at {}", lesson.hear_at));
        }
        if end_frame > SRC_FRAMES {
            return Err(format!("cut {end_frame} past source {SRC_FRAMES}"));
        }
        // A long gap keeps playing. The next line is not on the card yet, and
        // the explanation does not start until this line has actually ended.
        if start - cursor > 58 {
            total += add_play(
                &mut chains,
                &mut vlabels,
                &mut alabels,
                &mut seg_i,
                cursor,
                start,
                "nxt",
                lesson,
                index,
                n,
                &header,
            )?;
            cursor = start;
        }
        total += add_play(
            &mut chains,
            &mut vlabels,
            &mut alabels,
            &mut seg_i,
            cursor,
            end_frame,
            "hear",
            lesson,
            index,
            n,
            &header,
        )?;
        // Freeze the last frame of the line. Resume at `end_frame`, which is
        // still before the next syllable.
        total += add_hold(
            &mut chains,
            &mut vlabels,
            &mut alabels,
            &mut seg_i,
            end_frame - 1,
            prep,
            lesson,
            index,
            n,
            &header,
            prep_i + 1,
        )?;
        cursor = end_frame;
    }

    let tail = if indexes.len() < LESSONS.len() {
        (cursor + frames_for(1.0)).min(SRC_FRAMES)
    } else {
        SRC_FRAMES
    };
    if tail - cursor > 2 {
        let index = *indexes.last().unwrap();
        total += add_play(
            &mut chains,
            &mut vlabels,
            &mut alabels,
            &mut seg_i,
            cursor,
            tail,
            "quiet",
            &LESSONS[index],
            index,
            n,
            &header,
        )?;
    }

    if let Some((_, end_frames)) = end {
        if indexes.len() != n {
            return Err("end card is only part of the full lesson".to_string());
        }
        let tag = format!("e{seg_i}");
        let end_frames = (*end_frames).max(frames_for(5.0));
        let pad = secs(end_frames) + 0.5;
        chains.push(format!(
            "color=c={PAPER}:s=1280x720:r={FPS}:d={pad:.3},fps={FPS},settb=1001/24000,trim=end_frame={end_frames},setpts=N,format=yuv420p[{tag}bg]"
        ));
        let mut draws = Vec::new();
        let lines = [
            (FONT_KR, 32, INK, 80, 196, "end_ko.txt", END_KO),
            (FONT, 20, "0x5C574E", 80, 258, "end_en.txt", END_EN),
            (FONT_KR, 26, INK, 80, 330, "end_p.txt", "탈락으로 처리된다"),
            (FONT, 18, "0x5C574E", 80, 370, "end_pe.txt", "treated as eliminated"),
            (FONT_B, 18, "0x8A5A00", 80, 450, "end_date.txt", "SEPTEMBER 17    ·    NETFLIX"),
        ];
        for (font, size, color, x, y, name, text) in lines {
            let file = write_text(name, text)?;
            draws.push(draw_line(font_for(text, font == FONT_B), &file, size, color, x, y));
        }
        chains.push(lock_picture(&format!("{tag}bg"), &tag, &draws, end_frames));
        let ei = 1 + prepared.len();
        chains.push(lock_voice_audio(&tag, ei, end_frames));
        vlabels.push(format!("[{tag}]"));
        alabels.push(format!("[{tag}a]"));
        total += end_frames;
    }

    let nseg = vlabels.len();
    let mix: String = vlabels
        .iter()
        .zip(&alabels)
        .map(|(v, a)| format!("{v}{a}"))
        .collect();
    chains.push(format!("{mix}concat=n={nseg}:v=1:a=1[vcat][aout]"));
    // The muxer drops a last frame that has no duration. Clone one, then cut
    // back, so the picture length is the audio length.
    chains.push(format!(
        "[vcat]tpad=stop_mode=clone:stop_duration=0.08,fps={FPS},trim=end_frame={total},setpts=N,format=yuv420p,setsar=1[vout]"
    ));
    let script = PathBuf::from(ROOT).join("filter-squid.txt");
    fs::write(&script, chains.join(";\n")).map_err(|e| e.to_string())?;
    println!("filter {nseg} segments  {total} frames -> {}", script.display());
    Ok((script, total))
}

fn add_play(
    chains: &mut Vec<String>,
    vlabels: &mut Vec<String>,
    alabels: &mut Vec<String>,
    seg_i: &mut usize,
    start: i64,
    end: i64,
    mode: &str,
    lesson: &Lesson,
    index: usize,
    n: usize,
    header: &[String],
) -> Result<i64, String> {
    if end <= start {
        return Ok(0);
    }
    if start < 0 || end > SRC_FRAMES {
        return Err(format!("play {start}..{end} outside source"));
    }
    let frames = end - start;
    let tag = format!("p{seg_i}");
    chains.push(format!(
        "[0:v]trim=start_frame={start}:end_frame={end},setpts=PTS-STARTPTS,fps={FPS},settb=1001/24000,setpts=N,format=yuv420p[{tag}v]"
    ));
    let (hstack, staged) = stage(&tag, &format!("{tag}v"), frames);
    chains.push(hstack);
    let mut draws = header.to_vec();
    draws.extend(panel_draws(mode, lesson, None, index, n, 0, 0.0)?);
    chains.push(lock_picture(&staged, &tag, &draws, frames));
    chains.push(lock_trailer_audio(&tag, start, end));
    vlabels.push(format!("[{tag}]"));
    alabels.push(format!("[{tag}a]"));
    *seg_i += 1;
    Ok(frames)
}

fn add_hold(
    chains: &mut Vec<String>,
    vlabels: &mut Vec<String>,
    alabels: &mut Vec<String>,
    seg_i: &mut usize,
    frame: i64,
    prep: &Prepared,
    lesson: &Lesson,
    index: usize,
    n: usize,
    header: &[String],
    voice_input: usize,
) -> Result<i64, String> {
    let frames = prep.hold_frames;
    if frames < 2 {
        return Err(format!("hold at frame {frame} is too short"));
    }
    if frame < 0 || frame + 1 > SRC_FRAMES {
        return Err(format!("freeze frame {frame} outside source"));
    }
    let rows = card_plan(prep, lesson);
    let layout = note_layout(lesson, &rows);
    let longest = lesson.ko.iter().map(|s| s.chars().count()).max().unwrap_or(0);
    let size: i32 = if lesson.ko.len() > 2 || longest > 11 {
        24
    } else if longest > 8 {
        28
    } else {
        32
    };
    let line_bottom = 138 + lesson.ko.len() as i32 * (size + 6);
    let stops = pause_stops(&layout, &note_shift(&layout, line_bottom + 18));
    let spans = scroll_spans(&stops, frames);
    let mut held = 0i64;
    for (start, end, shift) in spans {
        let span = end - start;
        if span < 1 {
            continue;
        }
        let tag = format!("h{seg_i}");
        let extra = span - 1;
        // One real source frame, looped. The scroll is a number on this stretch.
        // A `t` expression is read once, at the end of the pause, and that is
        // what put the first word on the sentence.
        chains.push(format!(
            "[0:v]trim=start_frame={frame}:end_frame={next},setpts=PTS-STARTPTS,\
             loop=loop={extra}:size=1:start=0,trim=end_frame={span},setpts=N*1001/24000/TB,fps={FPS},settb=1001/24000,format=yuv420p[{tag}v]",
            next = frame + 1
        ));
        let (hstack, staged) = stage(&tag, &format!("{tag}v"), span);
        chains.push(hstack);
        let mut draws = header.to_vec();
        draws.extend(panel_draws(
            "hold",
            lesson,
            Some(prep),
            index,
            n,
            shift,
            secs(start),
        )?);
        chains.push(lock_picture(&staged, &tag, &draws, span));
        chains.push(lock_voice_slice(&tag, voice_input, start, end));
        vlabels.push(format!("[{tag}]"));
        alabels.push(format!("[{tag}a]"));
        *seg_i += 1;
        held += span;
    }
    Ok(held)
}

fn render(
    script: &Path,
    prepared: &[Prepared],
    end: Option<&(PathBuf, i64)>,
    draft: bool,
    expect_frames: i64,
) -> Result<(), String> {
    let out = if draft || prepared.len() != LESSONS.len() {
        PathBuf::from(ROOT).join("squid-draft.mp4")
    } else {
        PathBuf::from(OUT)
    };
    let building = PathBuf::from(ROOT).join(if draft || prepared.len() != LESSONS.len() {
        "squid-draft-build.mp4"
    } else {
        "squid-build.mp4"
    });
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y").arg("-i").arg(TRAILER);
    for prep in prepared {
        cmd.arg("-i").arg(&prep.wav);
    }
    if let Some((path, _)) = end {
        cmd.arg("-i").arg(path);
    }
    cmd.arg("-filter_complex_script")
        .arg(script)
        .args(["-map", "[vout]", "-map", "[aout]"])
        .args(["-c:v", "libx264", "-preset"])
        .arg(if draft || prepared.len() != LESSONS.len() { "ultrafast" } else { "veryfast" })
        .args(["-crf"])
        .arg(if draft || prepared.len() != LESSONS.len() { "23" } else { "18" })
        .args([
            "-pix_fmt",
            "yuv420p",
            "-r",
            FPS,
            "-fps_mode",
            "cfr",
            "-video_track_timescale",
            "24000",
            "-c:a",
            "aac",
            "-b:a",
            "160k",
            "-muxdelay",
            "0",
            "-muxpreload",
            "0",
            "-movflags",
            "+faststart",
        ])
        .arg(&building);
    println!(
        "ffmpeg {} -> {}",
        if draft { "draft" } else { "final" },
        building.display()
    );
    let output = cmd.output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("render failed: {}", tail(&err, 2500)));
    }
    verify_lock(&building, expect_frames)?;
    fs::rename(&building, &out).map_err(|e| e.to_string())?;
    println!("wrote {}", out.display());
    Ok(())
}

/// The picture and the voice share one length. A hole here is the sync bug:
/// the player jumps the picture forward while the voice keeps going.
fn verify_lock(path: &Path, expect_frames: i64) -> Result<(), String> {
    let packets = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "packet=pts_time",
            "-of",
            "csv=p=0",
        ])
        .arg(path)
        .output()
        .map_err(|e| e.to_string())?;
    if !packets.status.success() {
        return Err(format!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&packets.stderr)
        ));
    }
    let text = String::from_utf8_lossy(&packets.stdout);
    let mut times = Vec::new();
    for line in text.lines() {
        let line = line.trim().trim_end_matches(',');
        if line.is_empty() {
            continue;
        }
        let t: f64 = line.parse().map_err(|_| format!("bad pts {line}"))?;
        times.push(t);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if times.len() as i64 != expect_frames {
        return Err(format!(
            "sync: expected {expect_frames} frames, file has {}",
            times.len()
        ));
    }
    if times.first().copied().unwrap_or(1.0) > 0.001 {
        return Err(format!("sync: picture does not start at 0 ({})", times[0]));
    }
    let step = FPS_DEN / FPS_NUM;
    for (a, b) in times.iter().zip(times.iter().skip(1)) {
        let gap = b - a;
        if gap > 0.05 {
            return Err(format!(
                "sync: picture jumps {gap:.3}s at {a:.3}s — a pause emitted no frames"
            ));
        }
        if (gap - step).abs() > 0.002 {
            return Err(format!("sync: frame step {gap:.5}s at {a:.3}s, want {step:.5}"));
        }
    }
    let audio = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=duration",
            "-of",
            "csv=p=0",
        ])
        .arg(path)
        .output()
        .map_err(|e| e.to_string())?;
    let audio_text = String::from_utf8_lossy(&audio.stdout);
    let audio_dur: f64 = audio_text
        .trim()
        .trim_end_matches(',')
        .parse()
        .map_err(|_| format!("bad audio duration {}", audio_text.trim()))?;
    let video_dur = secs(expect_frames);
    if (audio_dur - video_dur).abs() > 0.08 {
        return Err(format!(
            "sync: audio {audio_dur:.3}s vs picture {video_dur:.3}s"
        ));
    }
    println!("sync ok  {expect_frames} frames  {video_dur:.3}s  audio {audio_dur:.3}s");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What one long request actually returned. It smears the opening across
    /// the music and garbles 빚을 지고 as 빚을 치고.
    const LONG_SHOT: &str = "지금 비자에 계신 여러분들은 모두 감당할 수 없는 빚을 치고 설매 변함 끝에 석이신 분들입니다";

    #[test]
    fn recognition_windows_stay_under_twenty_seconds() {
        let windows = recognition_windows(SRC_END);
        assert!(windows.len() > 4, "one window is the broken pass");
        for (start, end) in &windows {
            let span = end - start;
            assert!(span <= RECOGNIZE_LIMIT + 0.001, "{start}-{end}");
            assert!(span < 20.0, "{start}-{end}");
            assert!(*end > *start);
        }
        assert_eq!(windows[0].0, 0.0);
        assert!((windows.last().unwrap().1 - SRC_END).abs() < 0.01);
        assert!(
            windows.iter().any(|(start, _)| *start >= 20.0),
            "nothing is recognized after 20s"
        );
        let mut covered_to = 0.0;
        for (start, end) in &windows {
            assert!(*start <= covered_to + 0.01, "gap before {start}");
            covered_to = covered_to.max(*end);
        }
    }

    #[test]
    fn chunked_hearing_keeps_lines_a_long_shot_drops() {
        let chunks = [
            (0.0, "지금 비자에 계신 여러분들은 빚을 치고"),
            (24.0, "벼랑 끝에 서 계신 분들입니다"),
            (36.0, "게임의 참가를 원하지 않는 분은"),
            (48.0, "무궁화 꽃이 피었습니다"),
            (60.0, "여섯 개의 게임을 모두 이긴 분들께는"),
            (108.0, "탈락으로 처리된다"),
        ];
        let later = heard_from(&chunks, 36.0);
        assert!(later.contains("무궁화"));
        assert!(later.contains("여섯 개의"));
        assert!(later.contains("탈락으로"));
        assert!(!LONG_SHOT.contains("무궁화"), "long shot missed the game");
        assert!(!LONG_SHOT.contains("탈락"));
        assert!(LONG_SHOT.contains("비자에"), "long shot garbled the opening");
        assert!(LONG_SHOT.contains("빚을 치고"));
        assert!(!later.contains("비자에"));
    }

    #[test]
    fn lines_after_twenty_match_the_short_windows() {
        validate().unwrap();
        let after: Vec<_> = LESSONS.iter().filter(|l| l.hear_at >= 20.0).collect();
        assert!(after.len() > 10);
        assert!(after.iter().any(|l| l.say.contains("무궁화")));
        assert!(after.iter().any(|l| l.say.contains("456억")));
        assert!(after.iter().any(|l| l.say.contains("무슨 수로")));
        assert!(after.iter().any(|l| l.say.contains("탈락으로")));
        assert!(LESSONS.iter().all(|l| !l.say.contains("비자에")));
        assert!(LESSONS.iter().all(|l| !l.say.contains("빚을 치고")));
        assert!(LESSONS.iter().all(|l| !l.say.contains("설매")));
        assert!(LESSONS.iter().all(|l| !l.say.contains("우궁화")));
        let prize = LESSONS.iter().find(|l| l.say.contains("456억")).unwrap();
        assert!(
            prize.hear_at >= 66.0 && prize.hear_at < 68.0,
            "the prize was cut at {}",
            prize.hear_at
        );
        let rule = LESSONS.iter().find(|l| l.say.contains("탈락으로")).unwrap();
        assert!(
            rule.hear_at >= 112.0 && rule.hear_at < 115.0,
            "the rule was early at {}",
            rule.hear_at
        );
        assert!(rule.hear_at > prize.pause_at);
        // 빚을 지고 is still in the line at 32.0. A pause at 30 kept only 감당.
        let first = &LESSONS[0];
        assert!(first.pause_at >= 32.2, "01 still cuts 빚을 지고 at {}", first.pause_at);
        assert!(first.pause_at < 33.2, "01 pause landed in the next sound");
        assert!(first.pause_at + 0.05 < LESSONS[1].hear_at);
        assert_eq!(
            LESSONS.iter().find(|l| l.hear_at >= 49.0 && l.hear_at < 53.0).unwrap().say,
            "무궁화 꽃이 피었습니다."
        );
    }

    #[test]
    fn output_name_has_no_korean_suffix() {
        assert!(OUT.ends_with("docs/squid-game.mp4"));
        let name = OUT.rsplit('/').next().unwrap();
        assert!(!name.contains("-korean"));
    }

    #[test]
    fn card_walks_vocab_then_grammar_then_meaning() {
        let lesson = &LESSONS[0];
        let beats = lesson_beats(lesson);
        assert_eq!(beats.first().unwrap().kind, 's');
        assert_eq!(beats.first().unwrap().lang, "ko");
        assert_eq!(beats.first().unwrap().spoken, "여러분들은 모두 감당할 수 없는 빚을 지고");
        assert!(beats.iter().any(|b| b.kind == 'g'));
        assert_eq!(beats.last().unwrap().kind, 'm');
        let grammar_at = beats.iter().position(|b| b.kind == 'g').unwrap();
        let meaning_at = beats.iter().position(|b| b.kind == 'm').unwrap();
        assert!(grammar_at < meaning_at);
        let flower = &LESSONS[4];
        let flower_beats = lesson_beats(flower);
        assert!(flower_beats.iter().any(|b| b.kind == 'g'));
        assert_eq!(flower.say, "무궁화 꽃이 피었습니다.");
        assert!(flower.words.iter().any(|w| w.ko == "무궁화" && w.hanja == "無窮花"));
        assert_eq!(flower_beats.first().unwrap().kind, 's');
        assert_eq!(flower_beats.first().unwrap().spoken, "무궁화 꽃이 피었습니다");
    }

    #[test]
    fn pause_reads_the_full_sentence_before_vocab() {
        let lesson = &LESSONS[18];
        let beats = lesson_beats(lesson);
        assert_eq!(lesson.say, "게임을 거부하는 참가자는 탈락으로 처리된다.");
        assert_eq!(beats[0].kind, 's');
        assert_eq!(beats[0].spoken, "게임을 거부하는 참가자는 탈락으로 처리된다");
        assert!(beats[0].spoken.contains("거부하는"));
        assert!(beats[0].spoken.contains("처리된다"));
        assert_eq!(beats[1].kind, 'v');
        assert_eq!(beats[1].spoken, "게임");
        assert!(beats[0].spoken.chars().count() > beats[1].spoken.chars().count());
        let grammar_at = beats.iter().position(|b| b.kind == 'g').unwrap();
        let meaning_at = beats.iter().position(|b| b.kind == 'm').unwrap();
        assert!(grammar_at < meaning_at);
    }

    #[test]
    fn quiet_tail_and_end_card_have_no_watching_or_trailer_line() {
        let src = include_str!("squid.rs");
        let panel = src
            .split("fn panel_draws")
            .nth(1)
            .unwrap()
            .split("fn stage")
            .next()
            .unwrap();
        assert!(panel.contains("mode == \"quiet\""));
        assert!(!panel.contains("watching"));
        assert!(!panel.contains("보는 중"));
        let end = src
            .split("if let Some((_, end_frames)) = end")
            .nth(1)
            .unwrap()
            .split("let nseg")
            .next()
            .unwrap();
        assert!(end.contains("END_KO"));
        assert!(!end.contains("FROM THE TRAILER"));
        assert!(!end.contains("end_kicker"));
    }

    #[test]
    fn notes_match_toesa_and_a_long_card_scrolls() {
        let src = include_str!("squid.rs");
        let panel = src
            .split("fn panel_draws")
            .nth(1)
            .unwrap()
            .split("fn stage")
            .next()
            .unwrap();
        assert!(panel.contains("18"), "vocab is 18, same as 퇴사");
        assert!(panel.contains("16"), "meaning is 16, same as 퇴사");
        assert!(!panel.contains(", 11, MUTED"), "section labels were shrunk to 11");
        assert!(panel.contains("note.y - shift"), "a long card scrolls instead of shrinking");
        assert!(
            panel.contains("color={PAPER}:t=fill"),
            "the sentence cover is missing, so a scrolled note shows through it"
        );
        let cover_at = panel.rfind("color={PAPER}:t=fill").unwrap();
        let redraw_at = panel.rfind("draws.extend(sentence)").unwrap();
        assert!(cover_at < redraw_at, "the sentence must be painted after the cover");
        let lesson = &LESSONS[0];
        let prep = Prepared {
            beats: lesson_beats(lesson),
            beat_times: (0..40).map(|i| i as f64).collect(),
            hold_frames: 48,
            wav: PathBuf::new(),
        };
        let rows = card_plan(&prep, &lesson);
        let notes = note_layout(lesson, &rows);
        let line_bottom = {
            let longest = lesson.ko.iter().map(|s| s.chars().count()).max().unwrap_or(0);
            let size: i32 = if lesson.ko.len() > 2 || longest > 11 {
                24
            } else if longest > 8 {
                28
            } else {
                32
            };
            138 + lesson.ko.len() as i32 * (size + 6)
        };
        let floor = line_bottom + 18;
        let stops = note_shift(&notes, floor);
        assert!(!stops.is_empty(), "the long card never needed a scroll");
        let mut shift = 0i32;
        let mut si = 0usize;
        for note in &notes {
            while si < stops.len() && stops[si].0 <= note.at + 0.001 {
                shift = stops[si].1;
                si += 1;
            }
            let top = note.y - shift;
            let bottom = top + note.h;
            assert!(bottom <= 708, "note left the panel: {bottom}");
            // The gap under the line is not the line. A note may sit in it.
            // It may not cross the bottom of the sentence.
            assert!(top >= line_bottom, "vocab scrolled onto the sentence: {top}");
        }
    }

    #[test]
    fn hanja_follows_the_word_not_the_card_edge() {
        let hx = hanja_x(PX, "인력", 18);
        assert!(hx > PX);
        assert!(hx < PX + 120, "hanja was pinned to the right edge: {hx}");
        assert_ne!(hx, 1200);
    }

    #[test]
    fn screen_five_verb_ends_in_da() {
        let lesson = &LESSONS[4];
        assert_eq!(lesson.say, "무궁화 꽃이 피었습니다.");
        assert!(lesson
            .words
            .iter()
            .any(|w| w.ko == "피다" && w.en == "to bloom"));
        assert!(lesson.words.iter().all(|w| w.ko != "피었습니다"));
        assert!(lesson.words.iter().all(|w| w.en != "has bloomed"));
        assert!(lesson
            .grammar
            .iter()
            .any(|g| g.ko == "피었습니다" && g.en.starts_with("피다 +")));
    }

    #[test]
    fn vocab_is_the_dictionary_stem() {
        for (i, lesson) in LESSONS.iter().enumerate() {
            let screen = i + 1;
            for word in lesson.words {
                assert!(!word.ko.contains('/') && !word.ko.contains('+'), "{screen}");
                assert!(!word.en.contains('/'), "{screen} {}", word.en);
                assert!(
                    lesson.grammar.iter().all(|g| g.ko != word.ko),
                    "screen {screen} vocab repeats inflected {}",
                    word.ko
                );
            }
            for note in lesson.grammar {
                assert!(!note.en.contains('/'), "screen {screen} {}", note.en);
                if note.ko.contains('의') {
                    assert!(
                        note.en.contains("of ") && !note.en.contains("'s"),
                        "의 is of, not 's: {}",
                        note.en
                    );
                }
                let cited = note.en.split(" + ").next().unwrap_or("");
                if cited.ends_with('다') {
                    assert!(
                        lesson.words.iter().any(|w| w.ko == cited),
                        "screen {screen} {} cites {cited}, which is not a 다 vocab stem",
                        note.ko
                    );
                }
            }
        }
        let spoken = speakable("피다 + 었 + 습니다 (has bloomed)");
        assert_eq!(spoken, "피다 었 습니다 has bloomed");
        assert!(!spoken.contains('/'));
    }

    #[test]
    fn pause_stops_the_trailer_audio() {
        let src = include_str!("squid.rs");
        let hold = src
            .split("fn add_hold")
            .nth(1)
            .unwrap()
            .split("fn render")
            .next()
            .unwrap();
        assert!(hold.contains("trim=start_frame="), "the freeze must be one source frame");
        assert!(hold.contains("loop=loop="), "that frame is held, not skipped");
        assert!(
            hold.contains("setpts=N*1001/24000/TB"),
            "the pause clock must be one output frame per spoken frame"
        );
        assert!(hold.contains("scroll_spans"), "a long card is split, not scrolled by t");
        assert!(
            !hold.contains("setpts=N,fps"),
            "bare setpts=N drops a short pause, and the notes never appear"
        );
        assert!(!hold.contains("tpad"), "a 40 ms trim is the jump");
        assert!(!hold.contains("[0:a]"), "trailer audio stays paused");
        assert!(!hold.contains("adelay"), "the notes start on the pause, not late");
        let play = src
            .split("fn add_play")
            .nth(1)
            .unwrap()
            .split("fn add_hold")
            .next()
            .unwrap();
        assert!(play.contains("start_frame"));
        assert!(play.contains("lock_trailer_audio"), "the line itself still plays");
        let trailer = src
            .split("fn lock_trailer_audio")
            .nth(1)
            .unwrap()
            .split("fn lock_voice_audio")
            .next()
            .unwrap();
        assert!(trailer.contains("[0:a]"));
    }
}
