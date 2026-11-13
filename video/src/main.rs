//! 소연 "퇴사할게여" official MV, used as a Korean lesson.
//!
//! The picture plays once, on the left. The right side is remarks only.
//! Korean lines are the ones in the video, the same lines as `src/content/toesa.rs`.
//!
//! The source is 24 fps. Every cut is a frame index on that clock, and the
//! sung audio for that cut is the same range. The cut is the quiet after the
//! line, still before the next syllable. Sung picture and sung audio stop on
//! that frame. The explanation plays only while that frame is held. Then both
//! resume together on the next frame. A pause inside the next line is the
//! jump: the notes talk, and the song is already on the next syllable.
//! The last note is not the end of the MV. After it, the picture and the
//! song play out to the last source frame, and the card stays blank.
//!
//! The voice then says the whole line again in Korean, and the card lights
//! that line. Then it walks vocabulary one by one — Korean, then English —
//! then the grammar point, then its meaning, then the meaning of the line.
//! Hanja sits right after that Korean word.
//!
//! A repeated chorus line is shown while it is sung, marked 다시 in the body,
//! and is not taught again.
//!
//! Source: https://www.youtube.com/watch?v=7mDDM0eBWR0

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

const ROOT: &str = "/home/packer/learn-korean/work";
const TRAILER: &str = "/home/packer/learn-korean/work/toesa-mv.mp4";
const OUT: &str = "/home/packer/learn-korean/docs/toesa.mp4";
const FONT_KR: &str = "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc";
const FONT_B: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf";
const FONT: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf";
const FPS: &str = "24";
const FPS_I: i64 = 24;
/// Source video is 5072 frames, 211.333 s. Frame N starts at N/24.
const SRC_FRAMES: i64 = 5072;
/// 48000 Hz / 24 fps. A frame and its audio samples are the same length.
const SAMPLES_PER_FRAME: i64 = 2000;

const VOICE_DELAY: f64 = 0.0;
/// Remarks start on the first frame of the pause. A delay here is the
/// explanation starting late, which is what made the first line feel late.
const BEAT_GAP: f64 = 0.22;
const MEANING: &str = "0xE7D3A1";
const PAPER: &str = "0xF6F1E8";
const INK: &str = "0x141210";
const MUTED: &str = "0x8A8378";
const GOLD: &str = "0xFFE58A";
const BLUE: &str = "0xB7D7F5";
const HANJA: &str = "0x8C2F2F";
const PX: i32 = 918;

struct Word {
    ko: &'static str,
    en: &'static str,
    hanja: &'static str,
}

const fn w(ko: &'static str, en: &'static str, hanja: &'static str) -> Word {
    Word { ko, en, hanja }
}

struct Lesson {
    ko: &'static [&'static str],
    en: &'static [&'static str],
    pattern: &'static str,
    pattern_en: &'static str,
    words: &'static [Word],
    say: &'static str,
}

struct Cue {
    /// First source frame of this line, inclusive.
    start: i64,
    /// First source frame after this line, exclusive. The freeze is `end - 1`.
    end: i64,
    lesson: usize,
    /// First time this line is heard. Later chorus lines are shown, not retaught.
    teach: bool,
}

static LESSONS: &[Lesson] = &[
    Lesson {
        ko: &["퇴사할게여"],
        en: &["I'm gonna quit~"],
        pattern: "−ㄹ게여",
        pattern_en: "−ㄹ게요 with 요 written 여",
        words: &[
            w("퇴사하다", "quit a company job", "退社"),
            w("−ㄹ게", "I decided / I promise", ""),
            w("여", "cute 요", ""),
        ],
        say: "퇴사할게여.",
    },
    Lesson {
        ko: &["아쉬울 거도", "없거든요"],
        en: &["There's nothing I'll", "even miss, you see."],
        pattern: "없거든요",
        pattern_en: "there isn't, you see",
        words: &[
            w("아쉽다", "feel a loss", ""),
            w("거", "thing, spoken 것", ""),
            w("도", "even", ""),
            w("없다", "not have", ""),
        ],
        say: "아쉬울 거도 없거든요.",
    },
    Lesson {
        ko: &["원래 이렇게", "열심히"],
        en: &["I was never going", "to live this hard."],
        pattern: "이렇게 열심히",
        pattern_en: "this hard",
        words: &[
            w("원래", "originally", "原來"),
            w("이렇게", "like this", ""),
            w("열심히", "hard", "熱心"),
        ],
        say: "원래 이렇게 열심히.",
    },
    Lesson {
        ko: &["살 생각", "없었거든요"],
        en: &["I never meant to", "live like that."],
        pattern: "없었거든요",
        pattern_en: "I didn't have, you see",
        words: &[
            w("살다", "to live", ""),
            w("생각", "a plan", "生覺"),
            w("없다", "not have", ""),
        ],
        say: "살 생각 없었거든요.",
    },
    Lesson {
        ko: &["오래 했어요"],
        en: &["I did it for a long time."],
        pattern: "했어요",
        pattern_en: "did",
        words: &[w("오래", "for a long time", ""), w("하다", "to do", "")],
        say: "오래 했어요.",
    },
    Lesson {
        ko: &["수백 번을", "참았거든요"],
        en: &["I held it in", "hundreds of times."],
        pattern: "참았거든요",
        pattern_en: "I endured, you see",
        words: &[
            w("수백", "hundreds", "數百"),
            w("번", "times", "番"),
            w("을", "object", ""),
            w("참다", "endure", ""),
        ],
        say: "수백 번을 참았거든요.",
    },
    Lesson {
        ko: &["쿨하게 짐을 싸고", "뒤돌아보지 않고"],
        en: &["Pack the bags cool,", "and don't look back."],
        pattern: "뒤돌아보지 않고",
        pattern_en: "without looking back",
        words: &[
            w("쿨하다", "be cool", ""),
            w("짐", "bags", ""),
            w("싸다", "to pack", ""),
            w("−지 않고", "without doing", ""),
        ],
        say: "쿨하게 짐을 싸고 뒤돌아보지 않고.",
    },
    Lesson {
        ko: &["모두 안녕히", "계세요"],
        en: &["Everyone, goodbye.", "She's the one leaving."],
        pattern: "안녕히 계세요",
        pattern_en: "goodbye to people who stay",
        words: &[
            w("모두", "everyone", ""),
            w("안녕히", "peacefully", "安寧"),
            w("계세요", "please stay", ""),
        ],
        say: "모두 안녕히 계세요.",
    },
    Lesson {
        ko: &["그래 이팔청춘", "후회하면 어때요"],
        en: &["So what if I regret it", "while I'm young?"],
        pattern: "후회하면 어때요",
        pattern_en: "so what if I regret",
        words: &[
            w("그래", "yeah / so", ""),
            w("이팔청춘", "youth, 16 to 25", "二八靑春"),
            w("후회하다", "to regret", "後悔"),
            w("어때요", "so what", ""),
        ],
        say: "그래 이팔청춘 후회하면 어때요.",
    },
    Lesson {
        ko: &["당장 저 문밖으로", "떠나요"],
        en: &["Leave out that door", "right now."],
        pattern: "문밖으로",
        pattern_en: "out the door",
        words: &[
            w("당장", "right away", "當場"),
            w("문", "door", "門"),
            w("밖", "outside", ""),
            w("떠나다", "to leave", ""),
        ],
        say: "당장 저 문밖으로 떠나요.",
    },
    Lesson {
        ko: &["날씨 좋군요", "바람 좋군요"],
        en: &["Oh, the weather's nice.", "Oh, the wind's nice."],
        pattern: "좋군요",
        pattern_en: "oh, it's nice",
        words: &[
            w("날씨", "weather", ""),
            w("좋다", "be good", ""),
            w("바람", "wind", ""),
        ],
        say: "날씨 좋군요. 바람 좋군요.",
    },
    Lesson {
        ko: &["결국 우린 모두", "할머니가 될 거고"],
        en: &["In the end we'll all", "be grandmas, and."],
        pattern: "될 거고",
        pattern_en: "will become, and",
        words: &[
            w("결국", "in the end", "結局"),
            w("우린", "we", ""),
            w("할머니", "grandma", ""),
            w("되다", "become", ""),
        ],
        say: "결국 우린 모두 할머니가 될 거고.",
    },
    Lesson {
        ko: &["뒤돌아보면", "오늘 하루는"],
        en: &["When we look back,", "today."],
        pattern: "뒤돌아보면",
        pattern_en: "when we look back",
        words: &[
            w("뒤돌아보다", "look back", ""),
            w("오늘", "today", ""),
            w("하루", "a day", ""),
        ],
        say: "뒤돌아보면 오늘 하루는.",
    },
    Lesson {
        ko: &["밥 먹을까요"],
        en: &["Shall we eat?"],
        pattern: "먹을까요",
        pattern_en: "shall we eat?",
        words: &[w("밥", "a meal", ""), w("먹다", "eat", "")],
        say: "밥 먹을까요.",
    },
    Lesson {
        ko: &["요즘 시간 좀", "많거든요"],
        en: &["I've got time", "these days, you see."],
        pattern: "많거든요",
        pattern_en: "I've got a lot, you see",
        words: &[
            w("요즘", "these days", ""),
            w("시간", "time", "時間"),
            w("좀", "a bit", ""),
            w("많다", "be a lot", ""),
        ],
        say: "요즘 시간 좀 많거든요.",
    },
    Lesson {
        ko: &["이제 전처럼 절대로", "약속 취소할 일은 없죠"],
        en: &["I won't cancel plans", "like I used to, right?"],
        pattern: "−ㄹ 일은 없죠",
        pattern_en: "there won't be any, right?",
        words: &[
            w("이제", "from now on", ""),
            w("전", "before", "前"),
            w("절대로", "never", "絕對"),
            w("약속", "plans", "約束"),
            w("취소하다", "cancel", "取消"),
        ],
        say: "이제 전처럼 절대로 약속 취소할 일은 없죠.",
    },
    Lesson {
        ko: &["너 너 이거 또", "거짓말하네"],
        en: &["Hey, you're lying again."],
        pattern: "거짓말하네",
        pattern_en: "you're lying, I see",
        words: &[
            w("너", "you", ""),
            w("이거", "this", ""),
            w("또", "again", ""),
            w("거짓말", "a lie", ""),
        ],
        say: "너 너 이거 또 거짓말하네.",
    },
    Lesson {
        ko: &["여행 갈까요"],
        en: &["Shall we travel?"],
        pattern: "갈까요",
        pattern_en: "shall we go?",
        words: &[w("여행", "a trip", "旅行"), w("가다", "go", "")],
        say: "여행 갈까요.",
    },
    Lesson {
        ko: &["버킷리스트가", "있거든요"],
        en: &["I've got a bucket list,", "you see."],
        pattern: "N가 있거든요",
        pattern_en: "I've got N, you see",
        words: &[
            w("버킷리스트", "bucket list", ""),
            w("가", "subject", ""),
            w("있다", "have", ""),
        ],
        say: "버킷리스트가 있거든요.",
    },
    Lesson {
        ko: &["전 세계 일주하기"],
        en: &["Go around the world."],
        pattern: "일주하기",
        pattern_en: "going around",
        words: &[
            w("전", "whole", "全"),
            w("세계", "world", "世界"),
            w("일주하다", "go around", "一周"),
        ],
        say: "전 세계 일주하기.",
    },
    Lesson {
        ko: &["소설책 작가 되기"],
        en: &["Become a novelist."],
        pattern: "되기",
        pattern_en: "becoming",
        words: &[
            w("소설", "a novel", "小說"),
            w("책", "a book", "冊"),
            w("작가", "a writer", "作家"),
            w("되다", "become", ""),
        ],
        say: "소설책 작가 되기.",
    },
    Lesson {
        ko: &["맘 편히", "노래 부르기"],
        en: &["Sing easy."],
        pattern: "노래 부르기",
        pattern_en: "singing",
        words: &[
            w("맘", "마음, short", ""),
            w("편히", "comfortably", "便"),
            w("노래", "a song", ""),
            w("부르다", "sing", ""),
        ],
        say: "맘 편히 노래 부르기.",
    },
    Lesson {
        ko: &["내 이런", "철없는 말이"],
        en: &["These childish words", "of mine."],
        pattern: "철없는",
        pattern_en: "childish, modifies 말",
        words: &[
            w("내", "my", ""),
            w("이런", "this kind of", ""),
            w("철없다", "immature", ""),
            w("말", "words", ""),
        ],
        say: "내 이런 철없는 말이.",
    },
    Lesson {
        ko: &["조금 당황스러워도"],
        en: &["Even if they throw you a bit."],
        pattern: "당황스러워도",
        pattern_en: "even if awkward",
        words: &[
            w("조금", "a little", ""),
            w("당황스럽다", "flustered", "唐慌"),
        ],
        say: "조금 당황스러워도.",
    },
    Lesson {
        ko: &["오늘은 날", "잡지는 말아요"],
        en: &["Don't hold me back today."],
        pattern: "잡지는 말아요",
        pattern_en: "don't hold me back",
        words: &[
            w("오늘", "today", ""),
            w("날", "me, 나를", ""),
            w("잡다", "hold back", ""),
        ],
        say: "오늘은 날 잡지는 말아요.",
    },
    Lesson {
        ko: &["언젠가 후회하는", "날이 와도"],
        en: &["Even if a day of regret", "comes someday."],
        pattern: "와도",
        pattern_en: "even if it comes",
        words: &[
            w("언젠가", "someday", ""),
            w("후회하다", "regret", "後悔"),
            w("날", "a day", ""),
            w("오다", "come", ""),
        ],
        say: "언젠가 후회하는 날이 와도.",
    },
    Lesson {
        ko: &["낭만이라", "할 테죠"],
        en: &["I'll still call it romance."],
        pattern: "할 테죠",
        pattern_en: "I will, you know",
        words: &[
            w("낭만", "romance", "浪漫"),
            w("−이라", "called", ""),
            w("하다", "call", ""),
        ],
        say: "낭만이라 할 테죠.",
    },
    Lesson {
        ko: &["나는 낭만적인", "할머니가 되어서"],
        en: &["I'll be a romantic grandma,", "and then."],
        pattern: "되어서",
        pattern_en: "become and then",
        words: &[
            w("나", "I", ""),
            w("낭만", "romance", "浪漫"),
            w("할머니", "grandma", ""),
            w("되다", "become", ""),
        ],
        say: "나는 낭만적인 할머니가 되어서.",
    },
    Lesson {
        ko: &["뒤돌아보곤", "오늘 하루를"],
        en: &["I'd look back", "on this one day."],
        pattern: "뒤돌아보곤",
        pattern_en: "would look back",
        words: &[
            w("뒤돌아보다", "look back", ""),
            w("−곤", "would", ""),
            w("오늘", "today", ""),
            w("하루", "a day", ""),
        ],
        say: "뒤돌아보곤 오늘 하루를.",
    },
];

/// Frames of quiet source between a line's last syllable and the next onset.
/// The pause has to land in that quiet. Two frames, not four: the next line
/// often starts a fifth of a second later, and four frames lands back inside
/// the syllable just sung.
const GUARD_FRAMES: i64 = 2;

/// Times are the vocal onsets and the first quiet after each line, measured
/// on the official MV: center vocal against the side instruments, then checked
/// against the lyric captions.
///
/// `pause` is exclusive. The remarks freeze `pause - 1` and the picture
/// resumes at `pause`. That frame is still before the next syllable, so the
/// song is not already on the next line when the explanation starts.
///
/// 모두 안녕히 계세요 ends near 38.8 s. 그래 starts at 39.04 s. A pause at
/// 39.85 s was already inside 그래, so the notes talked over that line.
///
/// 쿨하게 짐을 싸고 뒤돌아보지 않고 ends on 않고 at 34.85–35.12 s. The pause
/// is 35.20 s, after that syllable and before 모두 at 35.34 s.
///
/// 거짓말 ends near 91.0 s. 여행 starts at 91.32 s. A pause at 92.16 s was
/// already inside 여행.
///
/// 뒤돌아보곤 오늘 하루를 is still being sung at 193.2 s. The pause is 194.48 s,
/// after that line and before the English tag.
fn cues() -> Vec<Cue> {
    // (onset, pause, lesson, teach)
    snap_cues(&[
        (7.04, 8.76, 0, true),
        (10.06, 13.16, 1, true),
        (15.28, 17.72, 2, true),
        (17.82, 20.36, 3, true),
        (23.28, 24.80, 4, true),
        (26.32, 29.50, 5, true),
        (31.32, 35.20, 6, true),
        (35.34, 38.88, 7, true),
        (39.04, 43.18, 8, true),
        (43.32, 46.64, 9, true),
        (46.80, 52.78, 10, true),
        (55.02, 59.22, 11, true),
        (59.52, 62.50, 12, true),
        (75.22, 76.78, 13, true),
        (78.28, 81.02, 14, true),
        (83.28, 87.72, 15, true),
        (87.84, 91.06, 16, true),
        (91.32, 92.82, 17, true),
        (94.26, 97.22, 18, true),
        (99.22, 101.18, 19, true),
        (101.32, 103.18, 20, true),
        (103.30, 106.96, 21, true),
        (107.04, 111.20, 8, false),
        (111.32, 114.54, 9, false),
        (114.78, 120.72, 10, false),
        (123.00, 127.18, 11, false),
        (127.52, 130.42, 12, false),
        (143.90, 148.42, 22, true),
        (149.06, 152.54, 23, true),
        (153.02, 158.72, 24, true),
        (158.84, 165.18, 25, true),
        (165.22, 171.28, 26, true),
        (186.80, 191.24, 27, true),
        (191.36, 194.48, 28, true),
    ])
}

/// Snap each sung span onto a frame. The pause stays at least `GUARD_FRAMES`
/// before the next onset, so it cannot land inside the next line.
fn snap_cues(raw: &[(f64, f64, usize, bool)]) -> Vec<Cue> {
    let mut cues: Vec<Cue> = raw
        .iter()
        .map(|(hear, pause, lesson, teach)| Cue {
            start: (*hear * FPS_I as f64).round() as i64,
            end: (*pause * FPS_I as f64).round() as i64,
            lesson: *lesson,
            teach: *teach,
        })
        .collect();
    for i in 0..cues.len() {
        if let Some(next) = cues.get(i + 1) {
            let limit = next.start - GUARD_FRAMES;
            if cues[i].end > limit {
                cues[i].end = limit;
            }
        }
        if cues[i].end > SRC_FRAMES {
            cues[i].end = SRC_FRAMES;
        }
        if cues[i].end <= cues[i].start {
            cues[i].end = cues[i].start + 1;
        }
    }
    cues
}

fn secs(frames: i64) -> f64 {
    frames as f64 / FPS_I as f64
}

fn frames_ceil(seconds: f64) -> i64 {
    let n = (seconds * FPS_I as f64).ceil();
    if n < 1.0 {
        1
    } else {
        n as i64
    }
}

fn samples_for(frames: i64) -> i64 {
    frames * SAMPLES_PER_FRAME
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
    let all = cues();
    validate(&all)?;
    let selected: Vec<Cue> = if draft {
        all.into_iter().take(2).collect()
    } else {
        all
    };
    if plan_only {
        print_plan(&selected);
        return Ok(());
    }
    if !Path::new(TRAILER).exists() {
        return Err(format!("missing source video {TRAILER}"));
    }
    let prepared = prepare_voices(&selected)?;
    let script = build(&selected, &prepared)?;
    let (script, expect_frames) = script;
    render(&script, &prepared, draft, expect_frames)?;
    Ok(())
}

fn validate(cues: &[Cue]) -> Result<(), String> {
    if LESSONS.len() != 29 {
        return Err(format!("expected 29 lessons, got {}", LESSONS.len()));
    }
    if cues.len() != 34 {
        return Err(format!("expected 34 sung lines, got {}", cues.len()));
    }
    let mut taught = vec![false; LESSONS.len()];
    let mut cursor = 0i64;
    for (i, cue) in cues.iter().enumerate() {
        if cue.lesson >= LESSONS.len() {
            return Err(format!("lesson {} out of range", cue.lesson));
        }
        if cue.start < cursor {
            return Err(format!(
                "cue overlap at frame {} ({:.3}s)",
                cue.start,
                secs(cue.start)
            ));
        }
        if cue.end <= cue.start {
            return Err(format!("empty cue at frame {}", cue.start));
        }
        if cue.end > SRC_FRAMES {
            return Err(format!(
                "cue ends at frame {} past source {}",
                cue.end, SRC_FRAMES
            ));
        }
        if cue.end - cue.start < 12 {
            return Err(format!("cue at frame {} is shorter than half a second", cue.start));
        }
        if let Some(next) = cues.get(i + 1) {
            if cue.end > next.start - GUARD_FRAMES {
                return Err(format!(
                    "pause at {:.3}s lands inside the next line at {:.3}s",
                    secs(cue.end),
                    secs(next.start)
                ));
            }
        }
        if cue.teach {
            if taught[cue.lesson] {
                return Err(format!("lesson {} taught twice", cue.lesson));
            }
            taught[cue.lesson] = true;
        }
        cursor = cue.end;
    }
    if let Some(missed) = taught.iter().position(|ok| !ok) {
        return Err(format!("lesson {missed} is never taught"));
    }
    Ok(())
}

fn print_plan(cues: &[Cue]) {
    for cue in cues {
        let lesson = &LESSONS[cue.lesson];
        let flag = if cue.teach { "teach" } else { "hear " };
        println!(
            "{flag} f{:05}-{:05}  {:7.3}-{:7.3}  {}",
            cue.start,
            cue.end,
            secs(cue.start),
            secs(cue.end),
            lesson.say
        );
    }
}

fn vo_dir() -> PathBuf {
    PathBuf::from(ROOT).join("ko_vo")
}

fn txt_dir() -> PathBuf {
    PathBuf::from(ROOT).join("ko")
}

fn prepare_voices(cues: &[Cue]) -> Result<Vec<Prepared>, String> {
    let dir = vo_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let gap = dir.join("beat_gap.wav");
    silence(&gap, BEAT_GAP)?;
    let mut out = Vec::new();
    for cue in cues.iter().filter(|c| c.teach) {
        let lesson = &LESSONS[cue.lesson];
        let beats = lesson_beats(lesson);
        let mut parts = Vec::new();
        let mut durs = Vec::new();
        for (j, beat) in beats.iter().enumerate() {
            let clip = tight_voice(&cached_voice(&beat.spoken, beat.lang)?)?;
            durs.push(wav_duration(&clip)?);
            parts.push(clip);
            if j + 1 < beats.len() {
                parts.push(gap.clone());
            }
        }
        let path = dir.join(format!("{:02}.wav", out.len()));
        concat_wavs(&parts, &path)?;
        let dur = wav_duration(&path)?;
        let times = walk_card(&durs);
        let hold_frames = frames_ceil(dur + VOICE_DELAY + 0.55).max(frames_ceil(2.5));
        println!(
            "vo {:02} {:5.2}s  hold {:5.2} ({}f)  {} beats  {}",
            out.len(),
            dur,
            secs(hold_frames),
            hold_frames,
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

fn lesson_beats(lesson: &Lesson) -> Vec<Beat> {
    let mut beats = Vec::new();
    // She just sang it. Say the whole line again before the notes.
    beats.push(Beat {
        kind: 's',
        lang: "ko",
        spoken: lesson.say.trim().trim_end_matches(['.', '。']).to_string(),
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
            spoken: without_tilde(word.en),
            shown: word.en.to_string(),
            hanja: String::new(),
        });
    }
    if has_hangul(lesson.pattern) {
        beats.push(Beat {
            kind: 'g',
            lang: "ko",
            spoken: speakable(lesson.pattern),
            shown: lesson.pattern.to_string(),
            hanja: String::new(),
        });
        beats.push(Beat {
            kind: 'g',
            lang: "en",
            spoken: speakable(lesson.pattern_en),
            shown: lesson.pattern_en.to_string(),
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
        spoken: without_tilde(&meaning),
        shown: meaning,
        hanja: String::new(),
    });
    beats
}

fn tight_voice(src: &Path) -> Result<PathBuf, String> {
    let name = src
        .file_name()
        .ok_or("voice clip has no name")?
        .to_string_lossy();
    let dest = src.with_file_name(format!("t_{name}"));
    if dest.exists() && fs::metadata(&dest).map(|m| m.len() > 400).unwrap_or(false) {
        return Ok(dest);
    }
    // The synthesizer leads every clip by 0.2–0.4 s of silence. That silence
    // is the explanation starting late. Leave a short attack and a short tail.
    ffmpeg(&[
        "-y",
        "-i",
        src.to_str().unwrap(),
        "-af",
        "silenceremove=start_periods=1:start_threshold=-36dB:start_silence=0.02:\
         stop_periods=-1:stop_threshold=-36dB:stop_silence=0.08",
        "-ar",
        "22050",
        "-ac",
        "1",
        dest.to_str().unwrap(),
    ])?;
    Ok(dest)
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

fn without_tilde(text: &str) -> String {
    // A tone mark. Left in, the voice says "tilde".
    let cleaned = text.replace(['~', '～'], " ");
    let joined = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    if joined.is_empty() {
        text.trim().to_string()
    } else {
        joined
    }
}

fn speakable(text: &str) -> String {
    let cleaned = without_tilde(text)
        .replace(['−', '-', '/', '+'], " ")
        .replace("...", " ");
    let joined = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    if joined.is_empty() {
        text.trim().to_string()
    } else {
        joined
    }
}

fn has_hangul(text: &str) -> bool {
    text.chars().any(|ch| ('\u{ac00}'..='\u{d7a3}').contains(&ch))
}

/// DejaVu has no Hangul, jamo, or hanja. Those come out as empty boxes.
/// Any line that is not plain Latin uses the CJK font.
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
    y: i32,
    start: f64,
    end: f64,
) -> String {
    format!(
        "drawtext=fontfile={font}:textfile={}:fontsize={size}:fontcolor={color}:x={x}:y={y}:enable='between(t,{start:.3},{end:.3})'",
        file.display()
    )
}

fn header_draws() -> Result<Vec<String>, String> {
    let mut draws = vec![
        format!("drawbox=x=0:y=0:w=iw:h=58:color={PAPER}:t=fill"),
        "drawbox=x=0:y=56:w=900:h=3:color=0x141210:t=fill".to_string(),
        "drawbox=x=898:y=58:w=3:h=662:color=0xD9D0C2:t=fill".to_string(),
    ];
    let kicker = write_text("h_kicker.txt", "Learn Korean with")?;
    draws.push(draw_line(FONT_B, &kicker, 16, "0x1E6F8A", 22, 20));
    let title = write_text("h_title.txt", "퇴사할게여")?;
    draws.push(draw_line(FONT_KR, &title, 22, INK, 196, 16));
    Ok(draws)
}

fn panel_draws(
    mode: &str,
    lesson: &Lesson,
    prep: Option<&Prepared>,
    index: usize,
    n: usize,
    repeat: bool,
) -> Result<Vec<String>, String> {
    let mut draws = Vec::new();
    let x = PX;
    // After the last note the MV is still playing. The card stays blank.
    if mode == "quiet" {
        return Ok(draws);
    }
    let (tag, tag_c) = if repeat {
        ("REPEAT", "0x8A5A00")
    } else {
        match mode {
            "hear" => ("HEAR THIS", "0x1E6F8A"),
            "hold" => ("PAUSE", "0x1E6F8A"),
            _ => ("NEXT", MUTED),
        }
    };
    let stem = if repeat {
        format!("r{index:02}_{mode}")
    } else {
        format!("{index:02}_{mode}")
    };
    let tag_file = write_text(&format!("{stem}_tag.txt"), tag)?;
    draws.push(draw_line(FONT_B, &tag_file, 15, tag_c, x, 78));
    let num = write_text(&format!("{stem}_num.txt"), &format!("{:02}  /  {n:02}", index + 1))?;
    draws.push(draw_line(FONT, &num, 14, MUTED, x, 100));

    let longest = lesson.ko.iter().map(|s| s.chars().count()).max().unwrap_or(0);
    let size: i32 = if longest > 11 { 24 } else if longest > 8 { 28 } else { 32 };
    let title_y = if repeat { 164 } else { 138 };
    if repeat {
        let again = write_text(&format!("{stem}_again.txt"), "다시")?;
        draws.push(draw_line(FONT_KR, &again, 18, "0x8A5A00", x, 132));
        let heard = write_text(&format!("{stem}_heard.txt"), "same lines, heard before")?;
        draws.push(draw_line(FONT, &heard, 13, MUTED, x + 48, 136));
    }
    let rows = if mode == "hold" {
        let prep = prep.ok_or("hold without a voice")?;
        Some(card_plan(prep, lesson))
    } else {
        None
    };
    if let Some(rows) = rows.as_ref() {
        if let Some(row) = rows.iter().find(|r| r.kind == 's') {
            let box_h = 10 + lesson.ko.len() as i32 * (size + 6);
            draws.push(format!(
                "drawbox=x={}:y={}:w=352:h={box_h}:color={GOLD}:t=fill:enable='between(t,{:.3},{:.3})'",
                x - 8,
                title_y - 6,
                row.ko_t,
                row.hi_end
            ));
        }
    }
    let mut y = title_y;
    for (j, line) in lesson.ko.iter().enumerate() {
        let file = write_text(&format!("{stem}_ko{j}.txt"), line)?;
        draws.push(draw_line(FONT_KR, &file, size, INK, x, y));
        y += size + 6;
    }
    let Some(rows) = rows else {
        return Ok(draws);
    };
    y += 18;
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
            let file = write_text(&format!("{index:02}_sec{j}.txt"), label)?;
            draws.push(draw_timed(FONT_B, &file, 12, MUTED, x, y, row.ko_t, 999.0));
            y += 22;
            head = row.kind;
        }
        if row.kind == 'm' {
            let lines = lesson.en;
            let box_h = 8 + 22 * lines.len() as i32;
            draws.push(format!(
                "drawbox=x={}:y={}:w=352:h={box_h}:color={MEANING}:t=fill:enable='between(t,{:.3},{:.3})'",
                x - 8,
                y - 4,
                row.ko_t,
                row.hi_end
            ));
            for (k, line) in lines.iter().enumerate() {
                let file = write_text(&format!("{index:02}_m{k}.txt"), line)?;
                draws.push(draw_timed(font_for(line, false), &file, 16, INK, x, y, row.ko_t, 999.0));
                y += 22;
            }
            y += 8;
            continue;
        }
        let ko_font = font_for(&row.ko, false);
        let en_lines = wrap_en(&row.en, 32);
        let box_h = 26 + 18 * en_lines.len() as i32;
        let color = if row.kind == 'v' { GOLD } else { BLUE };
        draws.push(format!(
            "drawbox=x={}:y={}:w=352:h={box_h}:color={color}:t=fill:enable='between(t,{:.3},{:.3})'",
            x - 8,
            y - 4,
            row.ko_t,
            row.hi_end
        ));
        let file = write_text(&format!("{index:02}_w{j}.txt"), &row.ko)?;
        draws.push(draw_timed(ko_font, &file, 18, INK, x, y, row.ko_t, 999.0));
        if !row.hanja.is_empty() {
            let file = write_text(&format!("{index:02}_hz{j}.txt"), &row.hanja)?;
            let hx = x + text_px(&row.ko, 18) + 8;
            draws.push(draw_timed(FONT_KR, &file, 16, HANJA, hx, y + 1, row.ko_t, 999.0));
        }
        let mut ey = y + 22;
        for (k, line) in en_lines.iter().enumerate() {
            let file = write_text(&format!("{index:02}_we{j}_{k}.txt"), line)?;
            draws.push(draw_timed(
                font_for(line, false),
                &file,
                14,
                "0x5C574E",
                x,
                ey,
                row.ko_t,
                999.0,
            ));
            ey += 18;
        }
        y = ey + 10;
    }
    Ok(draws)
}

fn stage(label: &str, src: &str, frames: i64) -> (String, String) {
    let out = format!("{label}s");
    let pad = secs(frames) + 1.0;
    let chain = format!(
        "[{src}]scale=900:506:flags=lanczos:force_original_aspect_ratio=disable,setsar=1,\
         pad=900:720:0:107:color={PAPER},fps={FPS},settb=1/24,setpts=N/24/TB,trim=end_frame={frames},setpts=N/24/TB[{label}p];\
         color=c={PAPER}:s=380x720:r={FPS}:d={pad:.3},fps={FPS},trim=end_frame={frames},setpts=N/24/TB[{label}c];\
         [{label}p][{label}c]hstack=inputs=2[{out}]"
    );
    (chain, out)
}

/// Draws, then cut to exactly `frames` so a filter cannot invent or drop time.
fn lock_picture(staged: &str, tag: &str, draws: &[String], frames: i64) -> String {
    format!(
        "[{staged}]{},fps={FPS},settb=1/24,trim=end_frame={frames},setpts=N/24/TB,format=yuv420p[{tag}]",
        draws.join(",")
    )
}

fn lock_song_audio(tag: &str, start: i64, end: i64) -> String {
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

fn lock_voice_audio(tag: &str, input: usize, frames: i64) -> String {
    let samples = samples_for(frames);
    format!(
        "[{input}:a]aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,\
         volume=2.4,afade=t=in:d=0.02,\
         apad=whole_len={samples},atrim=end_sample={samples},alimiter=limit=0.9[{tag}a]"
    )
}

fn build(
    cues: &[Cue],
    prepared: &[Prepared],
) -> Result<(PathBuf, i64), String> {
    let n = LESSONS.len();
    let mut chains = Vec::new();
    let mut vlabels = Vec::new();
    let mut alabels = Vec::new();
    let mut cursor = 0i64;
    let mut total = 0i64;
    let mut seg_i = 0usize;
    let mut teach_i = 0usize;
    let header = header_draws()?;

    for cue in cues {
        let lesson = &LESSONS[cue.lesson];
        let prep = if cue.teach {
            let item = &prepared[teach_i];
            teach_i += 1;
            Some(item)
        } else {
            None
        };
        // A long instrumental keeps the next line on the card without pretending
        // she is already singing it.
        if cue.start - cursor > 58 {
            total += add_play(
                &mut chains,
                &mut vlabels,
                &mut alabels,
                &mut seg_i,
                cursor,
                cue.start,
                "nxt",
                lesson,
                cue.lesson,
                n,
                &header,
                !cue.teach,
            )?;
            cursor = cue.start;
        }
        total += add_play(
            &mut chains,
            &mut vlabels,
            &mut alabels,
            &mut seg_i,
            cursor,
            cue.end,
            "hear",
            lesson,
            cue.lesson,
            n,
            &header,
            !cue.teach,
        )?;
        if let Some(prep) = prep {
            let frozen = cue.end - 1;
            total += add_hold(
                &mut chains,
                &mut vlabels,
                &mut alabels,
                &mut seg_i,
                frozen,
                prep,
                lesson,
                cue.lesson,
                n,
                &header,
                teach_i,
            )?;
        }
        cursor = cue.end;
    }

    // The last note is not the end of the MV. Play the rest of the picture
    // and the song, with the card blank, the same as the trailer tail.
    // A draft stops a second later so the cut can be checked without the outro.
    let tail = if cues.len() < 34 {
        (cursor + frames_ceil(1.0)).min(SRC_FRAMES)
    } else {
        SRC_FRAMES
    };
    if tail - cursor > 2 {
        let index = cues.last().map(|c| c.lesson).unwrap_or(0);
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
            false,
        )?;
    }

    let nseg = vlabels.len();
    if nseg == 0 {
        return Err("no segments".to_string());
    }
    let mix: String = vlabels
        .iter()
        .zip(&alabels)
        .map(|(v, a)| format!("{v}{a}"))
        .collect();
    chains.push(format!("{mix}concat=n={nseg}:v=1:a=1[vcat][aout]"));
    // The muxer drops a last frame that has no duration. Clone one, then cut
    // back, so the picture length is the audio length.
    chains.push(format!(
        "[vcat]tpad=stop_mode=clone:stop_duration=0.08,fps={FPS},trim=end_frame={total},setpts=N/24/TB,format=yuv420p,setsar=1[vout]"
    ));
    let script = PathBuf::from(ROOT).join("filter.txt");
    fs::write(&script, chains.join(";\n")).map_err(|e| e.to_string())?;
    println!(
        "filter {nseg} segments, {total} frames ({:.3}s) -> {}",
        secs(total),
        script.display()
    );
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
    repeat: bool,
) -> Result<i64, String> {
    let frames = end - start;
    if frames < 1 {
        return Ok(0);
    }
    if start < 0 || end > SRC_FRAMES {
        return Err(format!("play {start}..{end} outside source"));
    }
    let tag = format!("p{seg_i}");
    chains.push(format!(
        "[0:v]trim=start_frame={start}:end_frame={end},setpts=N/24/TB,fps={FPS},settb=1/24,format=yuv420p[{tag}v]"
    ));
    let (hstack, staged) = stage(&tag, &format!("{tag}v"), frames);
    chains.push(hstack);
    let mut draws = header.to_vec();
    draws.extend(panel_draws(mode, lesson, None, index, n, repeat)?);
    chains.push(lock_picture(&staged, &tag, &draws, frames));
    chains.push(lock_song_audio(&tag, start, end));
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
    let tag = format!("h{seg_i}");
    let extra = frames - 1;
    // One real source frame, looped. Not a 40 ms trim: that window is empty
    // when the cut lands on a frame, and the picture then jumps the whole pause.
    chains.push(format!(
        "[0:v]trim=start_frame={frame}:end_frame={next},setpts=PTS-STARTPTS,\
         loop=loop={extra}:size=1:start=0,trim=end_frame={frames},setpts=N/24/TB,fps={FPS},settb=1/24,format=yuv420p[{tag}v]",
        next = frame + 1
    ));
    let (hstack, staged) = stage(&tag, &format!("{tag}v"), frames);
    chains.push(hstack);
    let mut draws = header.to_vec();
    draws.extend(panel_draws("hold", lesson, Some(prep), index, n, false)?);
    chains.push(lock_picture(&staged, &tag, &draws, frames));
    chains.push(lock_voice_audio(&tag, voice_input, frames));
    vlabels.push(format!("[{tag}]"));
    alabels.push(format!("[{tag}a]"));
    *seg_i += 1;
    Ok(frames)
}

fn render(
    script: &Path,
    prepared: &[Prepared],
    draft: bool,
    expect_frames: i64,
) -> Result<(), String> {
    let out = if draft {
        PathBuf::from(ROOT).join("draft.mp4")
    } else {
        PathBuf::from(OUT)
    };
    let building = PathBuf::from(ROOT).join(if draft {
        "draft-build.mp4"
    } else {
        "toesa-build.mp4"
    });
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y").arg("-i").arg(TRAILER);
    for prep in prepared {
        cmd.arg("-i").arg(&prep.wav);
    }
    cmd.arg("-filter_complex_script")
        .arg(script)
        .args(["-map", "[vout]", "-map", "[aout]"])
        .args(["-c:v", "libx264", "-preset"])
        .arg(if draft { "ultrafast" } else { "veryfast" })
        .args(["-crf"])
        .arg(if draft { "23" } else { "18" })
        .args([
            "-pix_fmt",
            "yuv420p",
            "-r",
            "24",
            "-fps_mode",
            "cfr",
            "-video_track_timescale",
            "12288",
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
        let t: f64 = line
            .parse()
            .map_err(|_| format!("bad pts {line}"))?;
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
    for (a, b) in times.iter().zip(times.iter().skip(1)) {
        let gap = b - a;
        if gap > 0.05 {
            return Err(format!(
                "sync: picture jumps {gap:.3}s at {a:.3}s — a pause emitted no frames"
            ));
        }
        if (gap - 1.0 / 24.0).abs() > 0.002 {
            return Err(format!("sync: frame step {gap:.5}s at {a:.3}s, want 1/24"));
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
    println!(
        "sync ok  {expect_frames} frames  {video_dur:.3}s  audio {audio_dur:.3}s"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tilde_stays_on_the_card_and_is_not_spoken() {
        let lesson = &LESSONS[0];
        let beats = lesson_beats(lesson);
        let meaning = beats.iter().find(|b| b.kind == 'm').unwrap();
        assert_eq!(meaning.shown, "I'm gonna quit~");
        assert_eq!(meaning.spoken, "I'm gonna quit");
        assert!(!meaning.spoken.contains('~'));
    }

    #[test]
    fn two_korean_lines_get_two_english_lines() {
        let lesson = &LESSONS[6];
        assert_eq!(lesson.ko.len(), lesson.en.len());
        assert!(lesson.en[0].to_lowercase().contains("bag"));
        assert!(lesson.en[1].to_lowercase().contains("look back"));
        let beats = lesson_beats(lesson);
        let meaning = beats.iter().find(|b| b.kind == 'm').unwrap();
        assert!(meaning.spoken.contains("bags"));
        assert!(meaning.spoken.contains("look back"));
    }

    #[test]
    fn pause_is_the_quiet_after_the_line() {
        let cues = cues();
        let by_lesson = |n: usize| cues.iter().find(|c| c.lesson == n && c.teach).unwrap();
        // 않고 is still inside the 쿨하게 cut, and the pause is before 모두.
        let line6 = by_lesson(6);
        let line7 = by_lesson(7);
        assert!(secs(line6.end) >= 35.16, "07 still cuts early at {}", secs(line6.end));
        assert!(line6.end <= line7.start - GUARD_FRAMES);
        // 계세요 ends near 38.8. A pause at 39.85 was already inside 그래.
        let line8 = by_lesson(8);
        assert!(secs(line7.end) >= 38.80, "{}", secs(line7.end));
        assert!(
            secs(line7.end) < 39.04,
            "pause landed in the next line at {}",
            secs(line7.end)
        );
        assert!(line7.end <= line8.start - GUARD_FRAMES);
        // 거짓말 ends before 여행. A pause at 92.16 was already inside 여행.
        let lie = by_lesson(16);
        let trip = by_lesson(17);
        assert!(secs(lie.end) >= 90.90, "{}", secs(lie.end));
        assert!(secs(lie.end) < 91.30, "{}", secs(lie.end));
        assert!(lie.end <= trip.start - GUARD_FRAMES);
        // 소설's last syllable is still inside the cut, and the pause is before 맘 편히.
        let novel = by_lesson(20);
        assert!(secs(novel.end) >= 103.10, "{}", secs(novel.end));
        assert!(novel.end <= by_lesson(21).start - GUARD_FRAMES);
        // 뒤돌아보곤 is still being sung at 193.2. The pause is after that line.
        let last = by_lesson(28);
        assert!(secs(last.end) >= 194.30, "last line still cut at {}", secs(last.end));
        // The note is not the end of the MV. The song still has the outro.
        assert!(last.end < SRC_FRAMES - 24, "no tail left after the last note");
    }

    #[test]
    fn quiet_tail_leaves_the_card_blank() {
        let src = include_str!("main.rs");
        let build = src
            .split("fn build(")
            .nth(1)
            .unwrap()
            .split("fn add_play")
            .next()
            .unwrap();
        assert!(build.contains("SRC_FRAMES"), "the full song must play out");
        assert!(build.contains("\"quiet\""), "the outro is a quiet card");
        let panel = src
            .split("fn panel_draws")
            .nth(1)
            .unwrap()
            .split("fn stage")
            .next()
            .unwrap();
        assert!(panel.contains("mode == \"quiet\""));
    }

    #[test]
    fn pause_stops_the_sung_audio() {
        let src = include_str!("main.rs");
        let hold = src
            .split("fn add_hold")
            .nth(1)
            .unwrap()
            .split("fn render")
            .next()
            .unwrap();
        assert!(hold.contains("trim=start_frame="), "the freeze must be one source frame");
        assert!(hold.contains("loop=loop="), "that frame is held, not skipped");
        assert!(!hold.contains("[0:a]"), "sung audio stays paused");
        assert!(!hold.contains("adelay"), "the notes start on the pause, not late");
        let play = src
            .split("fn add_play")
            .nth(1)
            .unwrap()
            .split("fn add_hold")
            .next()
            .unwrap();
        assert!(play.contains("lock_song_audio"), "the line itself still plays");
    }
}
