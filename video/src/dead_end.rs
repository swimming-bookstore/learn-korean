//! Dead-End Job trailer, used as a Korean lesson.
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
//! The first line's last two words, 것 같지, are quiet. A pause on the loud
//! syllable 망한 drops them. The cut is after 같지, before the next sound.
//!
//! A short pause is under one output tick if the held frames keep the source
//! clock. fps then emits nothing, the card stays on the line, and the notes
//! are gone. Each held frame is one tick, so a short pause still shows them.
//!
//! On the pause the voice reads the whole line, and the card lights that line.
//! Then it walks vocabulary one by one — Korean, then English — then each
//! grammar point, then its meaning, then the meaning of the line. Same order
//! as 퇴사할게여.
//!
//! Vocabulary is the dictionary stem. A verb ends in 다 (시작하다, not
//! 시작하고요). Grammar is that stem plus the particle or ending, the same
//! split as 참교육. 의 is "of", not a slash and not "'s".

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

const ROOT: &str = "/home/packer/learn-korean/work";
const TRAILER: &str = "/home/packer/ai-reaction/work/trailer.mp4";
const OUT: &str = "/home/packer/learn-korean/docs/dead-end-job.mp4";
const FONT_KR: &str = "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc";
const FONT_B: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf";
const FONT: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf";
const FPS: &str = "24000/1001";
const FPS_NUM: f64 = 24000.0;
const FPS_DEN: f64 = 1001.0;
/// Source is 2754 frames at 24000/1001. Frame N starts at N * 1001/24000.
const SRC_FRAMES: i64 = 2754;
/// 48000 Hz * 1001/24000. A frame and its audio samples are the same length.
const SAMPLES_PER_FRAME: i64 = 2002;
const SRC_END: f64 = 114.80;

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

const END_KO: &str = "못 나가   ·   일하라고   ·   살려";
const END_EN: &str = "can't get out   ·   work   ·   help";
const END_SAY_KO: &str = "못 나가. 일하라고. 살려.";
const END_SAY_EN: &str = "Can't get out. Work. Help. 면 됩니다 means you just have to. That was the deal.";

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

static LESSONS: &[Lesson] = &[
    Lesson {
        hear_at: 0.29,
        // 것 같지 is still being said at 1.95. A pause at 1.54 kept only 망한.
        pause_at: 2.16,
        ko: &["나 이번 생은", "망한 것 같지"],
        en: &["I think this life", "is ruined."],
        words: &[
            w("나", "I", ""),
            w("이번", "this time", ""),
            w("생", "a life", "生"),
            w("망하다", "to be ruined", "亡"),
            w("것", "a thing", ""),
            w("같다", "to seem", ""),
        ],
        grammar: &[
            g("생은", "생 + 은 (life, topic)"),
            g("망한", "망하다 + ㄴ (that is ruined)"),
            g("같지", "같다 + 지 (I think)"),
        ],
        say: "나 이번 생은 망한 것 같지.",
    },
    Lesson {
        hear_at: 3.71,
        pause_at: 5.17,
        ko: &["마지막 반전의 기회"],
        en: &["One last chance", "to turn it around."],
        words: &[
            w("마지막", "last", ""),
            w("반전", "a reversal", "反轉"),
            w("기회", "a chance", "機會"),
        ],
        grammar: &[g("반전의", "반전 + 의 (of a reversal)")],
        say: "마지막 반전의 기회.",
    },
    Lesson {
        hear_at: 5.51,
        pause_at: 6.76,
        ko: &["거미인력"],
        en: &["Spider Staffing."],
        words: &[w("거미", "spider", ""), w("인력", "manpower", "人力")],
        grammar: &[],
        say: "거미인력.",
    },
    Lesson {
        hear_at: 10.39,
        pause_at: 12.01,
        ko: &["일자리 알아보러", "오셨어요?"],
        en: &["Did you come to", "look for work?"],
        words: &[
            w("일자리", "a job", ""),
            w("알아보다", "to look into", ""),
            w("오다", "to come", ""),
        ],
        grammar: &[
            g("알아보러", "알아보다 + 러 (in order to look)"),
            g("오셨어요", "오다 + 시 + 었 + 어요 (you came)"),
        ],
        say: "일자리 알아보러 오셨어요?",
    },
    Lesson {
        hear_at: 12.18,
        pause_at: 15.10,
        ko: &["최저 시급의", "기본 10배부터", "시작하고요"],
        en: &["And it starts from ten times", "the base minimum wage."],
        words: &[
            w("최저", "the lowest", "最低"),
            w("시급", "hourly wage", "時給"),
            w("기본", "base", "基本"),
            w("10", "ten", ""),
            w("배", "times", "倍"),
            w("시작하다", "to start", "始作"),
        ],
        grammar: &[
            g("시급의", "시급 + 의 (of the wage)"),
            g("10배부터", "10 + 배 + 부터 (from ten times)"),
            g("시작하고요", "시작하다 + 고 + 요 (starts, and)"),
        ],
        say: "최저 시급의 기본 10배부터 시작하고요.",
    },
    Lesson {
        hear_at: 15.27,
        pause_at: 17.93,
        ko: &["하루 일하고", "하루 일당으로", "바로 드립니다"],
        en: &["Work a day, and we pay", "a day's wage right away."],
        words: &[
            w("하루", "one day", ""),
            w("일하다", "to work", ""),
            w("일당", "a day's pay", "日當"),
            w("바로", "right away", ""),
            w("드리다", "to give", ""),
        ],
        grammar: &[
            g("일하고", "일하다 + 고 (work, and)"),
            g("일당으로", "일당 + 으로 (as a day's pay)"),
            g("드립니다", "드리다 + ㅂ니다 (we give)"),
        ],
        say: "하루 일하고 하루 일당으로 바로 드립니다.",
    },
    Lesson {
        hear_at: 18.10,
        pause_at: 19.48,
        ko: &["하루 뜨고 오세요"],
        en: &["Come for a day."],
        words: &[w("하루", "one day", ""), w("뜨다", "to show up", ""), w("오다", "to come", "")],
        grammar: &[
            g("뜨고", "뜨다 + 고 (show up, and)"),
            g("오세요", "오다 + 세요 (please come)"),
        ],
        say: "하루 뜨고 오세요.",
    },
    Lesson {
        hear_at: 20.40,
        pause_at: 21.31,
        ko: &["돈 벌어야죠"],
        en: &["You've got to", "make money."],
        words: &[w("돈", "money", ""), w("벌다", "to earn", "")],
        grammar: &[g("벌어야죠", "벌다 + 어야죠 (you've got to earn)")],
        say: "돈 벌어야죠.",
    },
    Lesson {
        hear_at: 21.56,
        pause_at: 23.57,
        ko: &["알바를 성공하시면"],
        en: &["If you succeed", "at the side job."],
        words: &[w("알바", "a side job", ""), w("성공하다", "to succeed", "成功")],
        grammar: &[
            g("알바를", "알바 + 를 (the side job, object)"),
            g("성공하시면", "성공하다 + 시 + 면 (if you succeed)"),
        ],
        say: "알바를 성공하시면.",
    },
    Lesson {
        hear_at: 23.90,
        pause_at: 25.80,
        ko: &["일당이 최소 두 배", "높아집니다"],
        en: &["The day's pay goes up", "at least double."],
        words: &[
            w("일당", "a day's pay", "日當"),
            w("최소", "at least", "最小"),
            w("두", "two", ""),
            w("배", "times", "倍"),
            w("높아지다", "to get higher", ""),
        ],
        grammar: &[
            g("일당이", "일당 + 이 (the pay, subject)"),
            g("높아집니다", "높아지다 + ㅂ니다 (goes up)"),
        ],
        say: "일당이 최소 두 배 높아집니다.",
    },
    Lesson {
        hear_at: 26.40,
        pause_at: 27.19,
        ko: &["이 알바가", "졸라 불길해"],
        en: &["This side job", "feels really wrong."],
        words: &[
            w("이", "this", ""),
            w("알바", "a side job", ""),
            w("졸라", "really", ""),
            w("불길하다", "ominous", "不吉"),
        ],
        grammar: &[
            g("알바가", "알바 + 가 (the gig, subject)"),
            g("불길해", "불길하다 + 어 (feels ominous)"),
        ],
        say: "이 알바가 졸라 불길해.",
    },
    Lesson {
        hear_at: 28.30,
        pause_at: 28.85,
        ko: &["저주예요"],
        en: &["It's a curse."],
        words: &[w("저주", "a curse", "詛呪"), w("이다", "to be", "")],
        grammar: &[g("저주예요", "저주 + 이다 + 예요 (it is a curse)")],
        say: "저주예요.",
    },
    Lesson {
        hear_at: 29.70,
        pause_at: 32.36,
        ko: &["살려! 살려!"],
        en: &["Help! Help!"],
        words: &[w("살리다", "to save", "")],
        grammar: &[g("살려", "살리다 + 어 (save me)")],
        say: "살려! 살려!",
    },
    Lesson {
        hear_at: 37.20,
        pause_at: 38.71,
        ko: &["이거 다 뭐예요?"],
        en: &["What was all of that?"],
        words: &[
            w("이거", "this", ""),
            w("다", "all", ""),
            w("뭐", "what", ""),
            w("이다", "to be", ""),
        ],
        grammar: &[g("뭐예요", "뭐 + 이다 + 예요 (what is it)")],
        say: "이거 다 뭐예요?",
    },
    Lesson {
        hear_at: 38.96,
        pause_at: 40.33,
        ko: &["환상이에요?"],
        en: &["Is it a hallucination?"],
        words: &[w("환상", "a hallucination", "幻想"), w("이다", "to be", "")],
        grammar: &[g("환상이에요", "환상 + 이다 + 이에요 (is it a hallucination)")],
        say: "환상이에요?",
    },
    Lesson {
        hear_at: 40.50,
        pause_at: 42.21,
        ko: &["귀신, 괴물"],
        en: &["Ghosts. Monsters."],
        words: &[w("귀신", "ghost", "鬼神"), w("괴물", "monster", "怪物")],
        grammar: &[],
        say: "귀신, 괴물.",
    },
    Lesson {
        hear_at: 42.38,
        pause_at: 44.92,
        ko: &["귀신 무섭냐?", "그지가 더 무섭지"],
        en: &["Scared of ghosts?", "Being broke is scarier."],
        words: &[
            w("귀신", "ghost", "鬼神"),
            w("무섭다", "to be scary", ""),
            w("그지", "broke", ""),
            w("더", "more", ""),
        ],
        grammar: &[
            g("무섭냐", "무섭다 + 냐 (are you scared)"),
            g("그지가", "그지 + 가 (being broke, subject)"),
            g("무섭지", "무섭다 + 지 (is scarier)"),
        ],
        say: "귀신 무섭냐? 그지가 더 무섭지.",
    },
    Lesson {
        hear_at: 45.80,
        pause_at: 48.47,
        ko: &["꿀알바", "시작하겠습니다"],
        en: &["Starting the sweet gig."],
        words: &[w("꿀알바", "a sweet gig", ""), w("시작하다", "to start", "始作")],
        grammar: &[g("시작하겠습니다", "시작하다 + 겠 + 습니다 (I will start)")],
        say: "꿀알바 시작하겠습니다.",
    },
    Lesson {
        hear_at: 48.63,
        pause_at: 50.72,
        ko: &["여덟 시간만", "버티면 됩니다"],
        en: &["Just hold on", "for eight hours."],
        words: &[
            w("여덟", "eight", ""),
            w("시간", "hours", "時間"),
            w("버티다", "to hold on", ""),
            w("되다", "to become", ""),
        ],
        grammar: &[
            g("시간만", "시간 + 만 (just the hours)"),
            g("버티면", "버티다 + 면 (if you hold on)"),
            g("됩니다", "되다 + ㅂ니다 (that's enough)"),
        ],
        say: "여덟 시간만 버티면 됩니다.",
    },
    Lesson {
        hear_at: 50.88,
        pause_at: 53.39,
        ko: &["안 그러면", "여기서 못 나갈 수도 있어"],
        en: &["If you don't,", "you may not get out of here."],
        words: &[
            w("안", "not", ""),
            w("그렇다", "to be so", ""),
            w("여기", "here", ""),
            w("못", "cannot", ""),
            w("나가다", "to get out", ""),
            w("수", "a way", ""),
            w("있다", "to exist", ""),
        ],
        grammar: &[
            g("그러면", "그렇다 + 면 (if so)"),
            g("여기서", "여기 + 서 (from here)"),
            g("나갈", "나가다 + ㄹ (to get out)"),
            g("수도", "수 + 도 (might)"),
            g("있어", "있다 + 어 (there is)"),
        ],
        say: "안 그러면 여기서 못 나갈 수도 있어.",
    },
    Lesson {
        hear_at: 53.55,
        pause_at: 55.14,
        ko: &["할 수 있다"],
        en: &["I can do it."],
        words: &[w("하다", "to do", ""), w("수", "a way", ""), w("있다", "to exist", "")],
        grammar: &[g("할 수 있다", "하다 + ㄹ 수 있다 (can)")],
        say: "할 수 있다.",
    },
    Lesson {
        hear_at: 59.60,
        pause_at: 61.31,
        ko: &["궁금하지 않아요?"],
        en: &["Aren't you curious?"],
        words: &[w("궁금하다", "to be curious", ""), w("않다", "to not", "")],
        grammar: &[
            g("궁금하지", "궁금하다 + 지 (curious, and)"),
            g("않아요", "않다 + 아요 (not, polite)"),
        ],
        say: "궁금하지 않아요?",
    },
    Lesson {
        hear_at: 61.48,
        pause_at: 63.10,
        ko: &["거미인력이", "뭐 하는 곳인지?"],
        en: &["What kind of place", "Spider Staffing is."],
        words: &[
            w("거미인력", "Spider Staffing", ""),
            w("뭐", "what", ""),
            w("하다", "to do", ""),
            w("곳", "a place", ""),
            w("이다", "to be", ""),
        ],
        grammar: &[
            g("거미인력이", "거미인력 + 이 (Spider Staffing, subject)"),
            g("하는", "하다 + 는 (that does)"),
            g("곳인지", "곳 + 이다 + ㄴ지 (whether it is a place)"),
        ],
        say: "거미인력이 뭐 하는 곳인지?",
    },
    Lesson {
        hear_at: 63.48,
        pause_at: 65.32,
        ko: &["혹시 여기서", "일했었나요?"],
        en: &["Did he work here,", "by any chance?"],
        words: &[
            w("혹시", "by any chance", ""),
            w("여기", "here", ""),
            w("일하다", "to work", ""),
        ],
        grammar: &[
            g("여기서", "여기 + 서 (here)"),
            g("일했었나요", "일하다 + 었었 + 나요 (did you work)"),
        ],
        say: "혹시 여기서 일했었나요?",
    },
    Lesson {
        hear_at: 66.11,
        pause_at: 67.11,
        ko: &["문을 세우라고"],
        en: &["Stop. I said stop."],
        words: &[w("문", "door", "門"), w("세우다", "to stop", "")],
        grammar: &[
            g("문을", "문 + 을 (the door, object)"),
            g("세우라고", "세우다 + 라고 (saying to stop)"),
        ],
        say: "문을 세우라고.",
    },
    Lesson {
        hear_at: 72.20,
        pause_at: 74.12,
        ko: &["일하라고"],
        en: &["Work."],
        words: &[w("일하다", "to work", "")],
        grammar: &[g("일하라고", "일하다 + 라고 (saying to work)")],
        say: "일하라고.",
    },
    Lesson {
        hear_at: 75.78,
        pause_at: 79.79,
        ko: &["혁준 씨,", "어디까지 갈 수 있나", "한번 해보시죠"],
        en: &["Hyeok-jun, let's see", "how far you can go."],
        words: &[
            w("혁준", "Hyeok-jun", ""),
            w("씨", "honorific", ""),
            w("어디", "where", ""),
            w("가다", "to go", ""),
            w("수", "a way", ""),
            w("있다", "to exist", ""),
            w("한번", "one try", ""),
            w("해보다", "to try", ""),
        ],
        grammar: &[
            g("어디까지", "어디 + 까지 (how far)"),
            g("갈 수 있나", "가다 + ㄹ 수 있다 + 나 (whether one can go)"),
            g("해보시죠", "해보다 + 시 + 죠 (please try)"),
        ],
        say: "혁준 씨, 어디까지 갈 수 있나 한번 해보시죠.",
    },
    Lesson {
        hear_at: 89.71,
        pause_at: 94.05,
        ko: &["악몽이라뇨.", "알바죠, 꿀알바"],
        en: &["A nightmare? Please.", "A side job. A sweet one."],
        words: &[
            w("악몽", "a nightmare", "惡夢"),
            w("이다", "to be", ""),
            w("알바", "a side job", ""),
            w("꿀알바", "a sweet gig", ""),
        ],
        grammar: &[
            g("악몽이라뇨", "악몽 + 이다 + 라뇨 (what do you mean, a nightmare)"),
            g("알바죠", "알바 + 죠 (it's a side job)"),
        ],
        say: "악몽이라뇨. 알바죠, 꿀알바.",
    },
    Lesson {
        hear_at: 95.09,
        pause_at: 96.76,
        ko: &["뭘 알바?"],
        en: &["What side job?"],
        words: &[w("뭐", "what", ""), w("알바", "a side job", "")],
        grammar: &[g("뭘", "뭐 + ㄹ (what, object)")],
        say: "뭘 알바?",
    },
];


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
    validate()?;
    let selected: Vec<usize> = if draft {
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
    let end = if draft {
        None
    } else {
        Some(prepare_end()?)
    };
    let (script, frames) = build(&selected, &prepared, end.as_ref())?;
    render(&script, &prepared, end.as_ref(), draft, frames)?;
    Ok(())
}

fn validate() -> Result<(), String> {
    if LESSONS.len() != 29 {
        return Err(format!("expected 29 lessons, got {}", LESSONS.len()));
    }
    let mut t = 0.0;
    for lesson in LESSONS {
        if lesson.hear_at + 0.001 < t {
            return Err(format!("cue overlap at {}", lesson.hear_at));
        }
        if lesson.pause_at <= lesson.hear_at {
            return Err(format!("empty cue at {}", lesson.hear_at));
        }
        if lesson.say.contains("죽어") || lesson.say.contains("하루만 버티") {
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
    PathBuf::from(ROOT).join("dead-vo")
}

fn txt_dir() -> PathBuf {
    PathBuf::from(ROOT).join("dead-ko")
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
    let title = write_text("h_title.txt", "막다른 일자리")?;
    draws.push(draw_line(FONT_KR, &title, 22, INK, 196, 16));
    Ok(draws)
}

fn panel_draws(
    mode: &str,
    lesson: &Lesson,
    prep: Option<&Prepared>,
    index: usize,
    n: usize,
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
        30
    };
    let title_y = 138;
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
        let file = write_text(&format!("{index:02}_{mode}_ko{j}.txt"), line)?;
        draws.push(draw_line(FONT_KR, &file, size, INK, x, y));
        y += size + 6;
    }
    let Some(rows) = rows else {
        return Ok(draws);
    };
    y = (y + 8).max(188);
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
            draws.push(draw_timed(FONT_B, &file, 11, MUTED, x, y, row.ko_t, 999.0));
            y += 14;
            head = row.kind;
        }
        if row.kind == 'm' {
            let lines = lesson.en;
            let box_h = 4 + 16 * lines.len() as i32;
            draws.push(format!(
                "drawbox=x={}:y={}:w=352:h={box_h}:color={MEANING}:t=fill:enable='between(t,{:.3},{:.3})'",
                x - 8,
                y - 4,
                row.ko_t,
                row.hi_end
            ));
            for (k, line) in lines.iter().enumerate() {
                let file = write_text(&format!("{index:02}_m{k}.txt"), line)?;
                draws.push(draw_timed(font_for(line, false), &file, 14, INK, x, y, row.ko_t, 999.0));
                y += 16;
            }
            y += 8;
            continue;
        }
        let ko_font = font_for(&row.ko, false);
        let en_lines = wrap_en(&row.en, 38);
        let box_h = 18 + 13 * en_lines.len() as i32;
        let color = if row.kind == 'v' { GOLD } else { BLUE };
        draws.push(format!(
            "drawbox=x={}:y={}:w=352:h={box_h}:color={color}:t=fill:enable='between(t,{:.3},{:.3})'",
            x - 8,
            y - 4,
            row.ko_t,
            row.hi_end
        ));
        let file = write_text(&format!("{index:02}_w{j}.txt"), &row.ko)?;
        draws.push(draw_timed(ko_font, &file, 16, INK, x, y, row.ko_t, 999.0));
        if !row.hanja.is_empty() {
            let file = write_text(&format!("{index:02}_hz{j}.txt"), &row.hanja)?;
            let hx = hanja_x(x, &row.ko, 16);
            draws.push(draw_timed(FONT_KR, &file, 16, HANJA, hx, y + 1, row.ko_t, 999.0));
        }
        let mut ey = y + 16;
        for (k, line) in en_lines.iter().enumerate() {
            let file = write_text(&format!("{index:02}_we{j}_{k}.txt"), line)?;
            draws.push(draw_timed(
                font_for(line, false),
                &file,
                12,
                "0x5C574E",
                x,
                ey,
                row.ko_t,
                999.0,
            ));
            ey += 13;
        }
        y = ey + 2;
    }
    Ok(draws)
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
            (FONT_KR, 26, INK, 80, 330, "end_p.txt", "−면 됩니다"),
            (FONT, 18, "0x5C574E", 80, 370, "end_pe.txt", "you just have to"),
            (FONT_B, 18, "0x8A5A00", 80, 450, "end_date.txt", "OCTOBER 30    ·    NETFLIX"),
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
    let script = PathBuf::from(ROOT).join("filter-dead.txt");
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
    draws.extend(panel_draws(mode, lesson, None, index, n)?);
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
    let tag = format!("h{seg_i}");
    let extra = frames - 1;
    // One real source frame, looped. Not a 40 ms trim: that window is empty
    // when the cut lands on a frame, and the picture then jumps the whole pause.
    // setpts=N keeps the source clock, 1/24000. A short pause is then under
    // one output tick, fps emits nothing, and the notes never appear.
    // N*1001/24000/TB is one tick per frame, so a short pause still holds.
    chains.push(format!(
        "[0:v]trim=start_frame={frame}:end_frame={next},setpts=PTS-STARTPTS,\
         loop=loop={extra}:size=1:start=0,trim=end_frame={frames},setpts=N*1001/24000/TB,fps={FPS},settb=1001/24000,format=yuv420p[{tag}v]",
        next = frame + 1
    ));
    let (hstack, staged) = stage(&tag, &format!("{tag}v"), frames);
    chains.push(hstack);
    let mut draws = header.to_vec();
    draws.extend(panel_draws("hold", lesson, Some(prep), index, n)?);
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
    end: Option<&(PathBuf, i64)>,
    draft: bool,
    expect_frames: i64,
) -> Result<(), String> {
    let out = if draft {
        PathBuf::from(ROOT).join("dead-draft.mp4")
    } else {
        PathBuf::from(OUT)
    };
    let building = PathBuf::from(ROOT).join(if draft {
        "dead-draft-build.mp4"
    } else {
        "dead-end-build.mp4"
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
        .arg(if draft { "ultrafast" } else { "veryfast" })
        .args(["-crf"])
        .arg(if draft { "23" } else { "18" })
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

    /// What one 40-second request actually returned. It never reaches the
    /// lines after the first half-minute, and it garbles the company name.
    const LONG_SHOT: &str = "나 이번 생은 망한 거 같지 마지막 반전의 기회 거미 열려라 일자리 알아보러 오셨어요 최저 시급의 기본 10배 시작하고요 하루 일하고 하루 일당으로 바로 드립니다 돈 벌어야죠 알바를 성공하시면 일상이 높아집니다";

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
            (0.0, "나 이번 생은 망한 거 같지 거미인력"),
            (15.0, "돈 벌어야죠"),
            (20.0, "알바를 성공하시면"),
            (30.0, "살려 환상이에요 귀신 괴물"),
            (60.0, "문을 세우라고 일하라고"),
            (90.0, "악몽이라뇨 알바죠 꿀알바"),
        ];
        let later = heard_from(&chunks, 20.0);
        assert!(later.contains("성공하시면"));
        assert!(later.contains("살려"));
        assert!(later.contains("문을 세우라고"));
        assert!(later.contains("악몽이라뇨"));
        assert!(!LONG_SHOT.contains("살려"), "long shot missed the plea");
        assert!(!LONG_SHOT.contains("문을 세우라고"));
        assert!(!LONG_SHOT.contains("악몽"));
        assert!(LONG_SHOT.contains("거미 열려라"), "long shot garbled the name");
        assert!(!later.contains("거미 열려라"));
    }

    #[test]
    fn lines_after_twenty_match_the_short_windows() {
        validate().unwrap();
        let after: Vec<_> = LESSONS.iter().filter(|l| l.hear_at >= 20.0).collect();
        assert!(after.len() > 10);
        assert!(after.iter().any(|l| l.say.contains("성공하시면")));
        assert!(after.iter().any(|l| l.say.starts_with("돈 벌어야죠")));
        assert!(after.iter().any(|l| l.say.contains("문을 세우라고")));
        assert!(after.iter().any(|l| l.say.contains("살려")));
        assert!(LESSONS.iter().all(|l| !l.say.contains("죽어")));
        assert!(LESSONS.iter().all(|l| !l.say.contains("하루만 버티")));
        assert!(LESSONS.iter().all(|l| !l.say.contains("열 배부터")));
        assert!(LESSONS.iter().all(|l| !l.say.contains("수행하시면")));
        let nightmare = LESSONS.iter().find(|l| l.say.contains("악몽")).unwrap();
        assert!(
            nightmare.hear_at < 91.0 && nightmare.hear_at >= 89.0,
            "the broken pass cut 악몽 late at {}",
            nightmare.hear_at
        );
        let reply = LESSONS.iter().find(|l| l.say == "뭘 알바?").unwrap();
        assert!(
            reply.hear_at >= 94.5 && reply.hear_at < 96.5,
            "28 was early at {}, the reply is around 8:55",
            reply.hear_at
        );
        assert!(reply.hear_at + 0.05 > nightmare.pause_at);
        // 것 같지 is still in the line at 1.95. A pause at 1.54 kept only 망한.
        let first = &LESSONS[0];
        assert!(first.pause_at >= 2.08, "01 still cuts 것 같지 at {}", first.pause_at);
        assert!(first.pause_at < 2.60, "01 pause landed in the next sound");
        assert!(first.pause_at + 0.05 < LESSONS[1].hear_at);
        assert_eq!(
            LESSONS.iter().find(|l| l.hear_at >= 12.0 && l.hear_at < 16.0).unwrap().say,
            "최저 시급의 기본 10배부터 시작하고요."
        );
    }

    #[test]
    fn output_name_has_no_korean_suffix() {
        assert!(OUT.ends_with("docs/dead-end-job.mp4"));
        let name = OUT.rsplit('/').next().unwrap();
        assert!(!name.contains("-korean"));
    }

    #[test]
    fn card_walks_vocab_then_grammar_then_meaning() {
        let lesson = &LESSONS[0];
        let beats = lesson_beats(lesson);
        assert_eq!(beats.first().unwrap().kind, 's');
        assert_eq!(beats.first().unwrap().lang, "ko");
        assert_eq!(beats.first().unwrap().spoken, "나 이번 생은 망한 것 같지");
        assert!(beats.iter().any(|b| b.kind == 'g'));
        assert_eq!(beats.last().unwrap().kind, 'm');
        let grammar_at = beats.iter().position(|b| b.kind == 'g').unwrap();
        let meaning_at = beats.iter().position(|b| b.kind == 'm').unwrap();
        assert!(grammar_at < meaning_at);
        let company = &LESSONS[2];
        let company_beats = lesson_beats(company);
        assert!(company_beats.iter().all(|b| b.kind != 'g'));
        assert_eq!(company.say, "거미인력.");
        assert!(company.words.iter().any(|w| w.ko == "인력" && w.hanja == "人力"));
        assert_eq!(company_beats.first().unwrap().kind, 's');
        assert_eq!(company_beats.first().unwrap().spoken, "거미인력");
    }

    #[test]
    fn pause_reads_the_full_sentence_before_vocab() {
        let lesson = &LESSONS[17];
        let beats = lesson_beats(lesson);
        assert_eq!(lesson.say, "꿀알바 시작하겠습니다.");
        assert_eq!(beats[0].kind, 's');
        assert_eq!(beats[0].spoken, "꿀알바 시작하겠습니다");
        assert!(beats[0].spoken.contains("꿀알바"));
        assert!(beats[0].spoken.contains("시작하겠습니다"));
        assert_eq!(beats[1].kind, 'v');
        assert_eq!(beats[1].spoken, "꿀알바");
        assert!(beats[0].spoken.chars().count() > beats[1].spoken.chars().count());
        let grammar_at = beats.iter().position(|b| b.kind == 'g').unwrap();
        let meaning_at = beats.iter().position(|b| b.kind == 'm').unwrap();
        assert!(grammar_at < meaning_at);
    }

    #[test]
    fn quiet_tail_and_end_card_have_no_watching_or_trailer_line() {
        let src = include_str!("dead_end.rs");
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
    fn hanja_follows_the_word_not_the_card_edge() {
        let hx = hanja_x(PX, "인력", 18);
        assert!(hx > PX);
        assert!(hx < PX + 120, "hanja was pinned to the right edge: {hx}");
        assert_ne!(hx, 1200);
    }

    #[test]
    fn screen_five_verb_ends_in_da() {
        let lesson = &LESSONS[4];
        assert_eq!(lesson.say, "최저 시급의 기본 10배부터 시작하고요.");
        assert!(lesson
            .words
            .iter()
            .any(|w| w.ko == "시작하다" && w.en == "to start"));
        assert!(lesson.words.iter().all(|w| w.ko != "시작하고요"));
        assert!(lesson.words.iter().all(|w| w.en != "and it starts"));
        assert!(lesson
            .grammar
            .iter()
            .any(|g| g.ko == "시작하고요" && g.en.starts_with("시작하다 +")));
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
        let spoken = speakable("시작하다 + 고 + 요 (starts, and)");
        assert_eq!(spoken, "시작하다 고 요 starts, and");
        assert!(!spoken.contains('/'));
    }

    #[test]
    fn pause_stops_the_trailer_audio() {
        let src = include_str!("dead_end.rs");
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
            "a short pause drops every frame if PTS stays on the source clock"
        );
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
