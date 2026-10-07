use crate::content::{Item, Lesson};

pub struct Timing {
    pub card_ms: u32,
}

impl Default for Timing {
    fn default() -> Self {
        Self { card_ms: 2600 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Word,
    Cite,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub text: String,
    pub kind: Kind,
    pub dur_ms: u32,
}

impl Token {
    pub fn playable(&self) -> bool {
        self.kind != Kind::Cite && self.dur_ms > 0
    }
}

#[derive(Clone, Debug)]
pub struct Line {
    pub section: &'static str,
    pub tokens: Vec<Token>,
    /// Inclusive byte range in the lesson sentence this card highlights.
    pub span: Option<(usize, usize)>,
}

impl Line {
    pub fn word(&self) -> Option<&str> {
        self.tokens
            .iter()
            .find(|t| t.kind == Kind::Word)
            .map(|t| t.text.as_str())
    }

    pub fn cite(&self) -> Option<&str> {
        self.tokens
            .iter()
            .find(|t| t.kind == Kind::Cite)
            .map(|t| t.text.as_str())
    }
}

#[derive(Clone, Debug)]
pub struct Script {
    pub lines: Vec<Line>,
}

impl Script {
    pub fn get(&self, n: usize) -> Option<(usize, usize, &Token)> {
        token_at(&self.lines, n)
    }

    pub fn token_count(&self) -> usize {
        self.lines
            .iter()
            .flat_map(|l| l.tokens.iter())
            .filter(|t| t.playable())
            .count()
    }
}

/// Vocabulary cards, then grammar. Sentence stays on the plate.
pub fn compile(lesson: &Lesson, timing: Timing) -> Script {
    let mut lines = Vec::new();
    let mut from = 0;
    for item in lesson.vocab {
        let (line, next) = card_line("Vocabulary", item, timing.card_ms, lesson.korean, from);
        from = next;
        lines.push(line);
    }
    from = 0;
    for item in lesson.grammar {
        let (line, next) = card_line("Grammar", item, timing.card_ms, lesson.korean, from);
        from = next;
        lines.push(line);
    }
    Script { lines }
}

fn card_line(
    section: &'static str,
    item: &Item,
    dur: u32,
    sentence: &str,
    from: usize,
) -> (Line, usize) {
    let span = span_in(sentence, item.word, from, section);
    // Step past a finished syllable so the next card is not this one.
    // A stem match (`매` inside `매를`) ends mid-syllable; jump to the next
    // character so `들다` is not hidden inside `를`.
    let next = span
        .map(|(_, b)| {
            let mut i = b.min(sentence.len());
            if i < sentence.len() {
                i += 1;
            }
            while i < sentence.len() && !sentence.is_char_boundary(i) {
                i += 1;
            }
            i
        })
        .unwrap_or(from);
    (
        Line {
            section,
            span,
            tokens: vec![
                Token {
                    text: shown_word(item),
                    kind: Kind::Word,
                    dur_ms: dur,
                },
                Token {
                    text: item.meaning.to_string(),
                    kind: Kind::Cite,
                    dur_ms: 0,
                },
            ],
        },
        next,
    )
}

/// Every Hangul/Latin/digit run in the card label, joined across spaces.
/// `통과 이후` covers both words; `작품은` beats a bare `작품`. Hanja glosses are skipped.
/// A dictionary verb (`만들다`) highlights its stem when the sentence uses a form (`만들어진`).
/// Search starts at `from` so a later `들` is not the plural in `교사들`.
fn span_in(sentence: &str, label: &str, from: usize, section: &str) -> Option<(usize, usize)> {
    let sentence = sentence.trim();
    let from = char_boundary_at_or_after(sentence, from.min(sentence.len()));
    let mut start: Option<usize> = None;
    let mut end = 0usize;
    let chars: Vec<(usize, char)> = label.char_indices().collect();
    let mut i = 0;
    while i < chars.len() {
        let (byte, ch) = chars[i];
        if !is_ko_or_latin(ch) {
            i += 1;
            continue;
        }
        let mut end_i = i + 1;
        let mut end_byte = byte + ch.len_utf8();
        while end_i < chars.len() && is_ko_or_latin(chars[end_i].1) {
            end_byte = chars[end_i].0 + chars[end_i].1.len_utf8();
            end_i += 1;
        }
        let needle = &label[byte..end_byte];
        let hit = if section == "Grammar" {
            find_from(sentence, needle, from).map(|at| (at, at + needle.len()))
        } else {
            find_from(sentence, needle, from)
                .map(|at| (at, at + needle.len()))
                .or_else(|| {
                    let stem = verb_stem(needle)?;
                    // A one-syllable stem (`놓`) may sit inside the previous card (`마음`).
                    let origin = if stem.chars().count() == 1 { 0 } else { from };
                    let at = find_from(sentence, stem, origin)?;
                    let to = at + stem.len();
                    if !stem_continues(sentence, to) {
                        return None;
                    }
                    Some((at, to))
                })
                .or_else(|| {
                    // `유리하다` → `유리하게`: the 하다 verb surfaces without a bare `하`.
                    let root = hada_root(needle)?;
                    let at = find_from(sentence, root, from)?;
                    let to = at + root.len();
                    if !stem_continues(sentence, to) {
                        return None;
                    }
                    Some((at, to))
                })
                .or_else(|| {
                    // `어렵다` → `어려워졌다`: ㅂ drops, so the spoken stem is `어려`.
                    let stem = verb_stem(needle)?;
                    let root = bieup_stem(stem)?;
                    let at = find_from(sentence, &root, from)?;
                    Some((at, at + root.len()))
                })
        };
        if let Some((at, to)) = hit {
            cover(
                at,
                char_boundary_at_or_before(sentence, to),
                &mut start,
                &mut end,
            );
        }
        i = end_i;
    }
    start.map(|s| (s, end))
}

fn find_from(sentence: &str, needle: &str, from: usize) -> Option<usize> {
    let from = char_boundary_at_or_after(sentence, from);
    sentence[from..].find(needle).map(|rel| from + rel)
}

fn char_boundary_at_or_after(s: &str, mut i: usize) -> usize {
    if i > s.len() {
        i = s.len();
    }
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

fn char_boundary_at_or_before(s: &str, mut i: usize) -> usize {
    if i > s.len() {
        i = s.len();
    }
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// A dictionary stem (`놓`) is only the verb when an ending follows (`놓고`).
/// `마음` is a different word, not `놓다`.
fn stem_continues(sentence: &str, to: usize) -> bool {
    sentence
        .get(to..)
        .and_then(|rest| rest.chars().next())
        .is_some_and(|c| !c.is_whitespace() && c != ',' && c != '.')
}

fn cover(at: usize, to: usize, start: &mut Option<usize>, end: &mut usize) {
    *start = Some(start.map_or(at, |s| s.min(at)));
    *end = (*end).max(to);
}

fn is_ko_or_latin(c: char) -> bool {
    c.is_ascii_alphanumeric() || ('가'..='힣').contains(&c)
}

/// `만들다` → `만들`, `하다` → `하`. Leaves stems that would be a single jamo alone.
fn verb_stem(word: &str) -> Option<&str> {
    let stem = word.strip_suffix('다')?;
    if stem.chars().count() >= 1 && stem.chars().all(|c| ('가'..='힣').contains(&c)) {
        Some(stem)
    } else {
        None
    }
}

/// `유리하다` highlights `유리하게`; `이용하다` highlights `이용할`.
/// Hanja after the dictionary form is not part of the sentence.
fn hada_root(word: &str) -> Option<&str> {
    word.strip_suffix("하다")
        .filter(|root| !root.is_empty() && root.chars().all(|c| ('가'..='힣').contains(&c)))
}

/// Korean keeps 하다. The Chinese gloss does not: `유리하다` + `有利` → `유리하다 有利`.
/// A trailing 하다 on the hanja field is dropped; the Chinese word itself stays.
fn shown_word(item: &crate::content::Item) -> String {
    if item.hanja.is_empty() {
        return item.word.to_string();
    }
    let gloss = if item.word.ends_with("하다") {
        item.hanja.strip_suffix("하다").unwrap_or(item.hanja)
    } else {
        item.hanja
    };
    if gloss.is_empty() {
        item.word.to_string()
    } else {
        format!("{} {gloss}", item.word)
    }
}

/// `어렵` → `어려`. ㅂ-irregular adjectives surface as `어려워`, not `어렵`.
fn bieup_stem(stem: &str) -> Option<String> {
    let (at, ch) = stem.char_indices().last()?;
    let n = ch as u32 - '가' as u32;
    if n > 11_172 || n % 28 != 17 {
        return None;
    }
    let bare = char::from_u32('가' as u32 + n - 17)?;
    let mut out = stem[..at].to_string();
    out.push(bare);
    Some(out)
}

fn token_at(lines: &[Line], mut n: usize) -> Option<(usize, usize, &Token)> {
    for (li, line) in lines.iter().enumerate() {
        for (ti, tok) in line.tokens.iter().enumerate() {
            if !tok.playable() {
                continue;
            }
            if n == 0 {
                return Some((li, ti, tok));
            }
            n -= 1;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_one_vocab_then_grammar() {
        let lesson = crate::content::get(1, 1).unwrap();
        let script = compile(lesson, Timing::default());
        assert_eq!(
            script.lines.len(),
            lesson.vocab.len() + lesson.grammar.len()
        );
        assert_eq!(script.lines[0].section, "Vocabulary");
        assert!(script.lines.iter().any(|l| l.section == "Grammar"));
    }

    #[test]
    fn sentence_span_prefers_inflected_form() {
        let lesson = crate::content::get(2, 0).unwrap();
        let script = compile(lesson, Timing::default());
        let grammar = script
            .lines
            .iter()
            .find(|l| l.word() == Some("작품은"))
            .unwrap();
        let (a, b) = grammar.span.unwrap();
        assert_eq!(&lesson.korean[a..b], "작품은");
        let made = script
            .lines
            .iter()
            .find(|l| l.word() == Some("만들어진"))
            .unwrap();
        let (a, b) = made.span.unwrap();
        assert_eq!(&lesson.korean[a..b], "만들어진");
        let bon = script
            .lines
            .iter()
            .find(|l| l.word() == Some("본 本"))
            .unwrap();
        let (a, b) = bon.span.unwrap();
        assert_eq!(&lesson.korean[a..b], "본");
        let make = script
            .lines
            .iter()
            .find(|l| l.word() == Some("만들다"))
            .unwrap();
        let (a, b) = make.span.unwrap();
        assert_eq!(&lesson.korean[a..b], "만들");
    }

    #[test]
    fn grammar_phrase_covers_every_word() {
        let lesson = crate::content::get(2, 1).unwrap();
        let script = compile(lesson, Timing::default());
        let last = script
            .lines
            .iter()
            .find(|l| l.word() == Some("통과 이후"))
            .unwrap();
        let (a, b) = last.span.unwrap();
        assert_eq!(&lesson.korean[a..b], "통과 이후");
    }

    #[test]
    fn day_two_vocab_follows_the_sentence() {
        let lesson = crate::content::get(2, 2).unwrap();
        let script = compile(lesson, Timing::default());
        let vocab: Vec<_> = script
            .lines
            .iter()
            .take_while(|l| l.section == "Vocabulary")
            .map(|l| l.word().unwrap_or("").to_string())
            .collect();
        assert_eq!(
            vocab,
            [
                "교사 敎師",
                "원칙 原則",
                "더 이상",
                "학생 學生",
                "매",
                "들다",
                "일",
                "금지 禁止"
            ]
            .map(str::to_string)
        );
        let ban = script
            .lines
            .iter()
            .find(|l| l.word() == Some("금지 禁止"))
            .unwrap();
        let (a, b) = ban.span.unwrap();
        assert_eq!(&lesson.korean[a..b], "금지");
    }

    #[test]
    fn day_two_grammar_hits_the_sentence() {
        let lesson = crate::content::get(2, 2).unwrap();
        let script = compile(lesson, Timing::default());
        for line in &script.lines {
            let shown = match line.span {
                Some((a, b)) => lesson.korean[a..b].to_string(),
                None => "NONE".into(),
            };
            eprintln!("SPAN {} {}", line.word().unwrap_or("?"), shown);
        }
        let expect = [
            ("교사들은", "교사들은"),
            ("원칙적으로", "원칙적으로"),
            ("더 이상", "더 이상"),
            ("학생에게", "학생에게"),
            ("매를", "매를"),
            ("드는", "드는"),
            ("일이", "일이"),
            ("금지되었다", "금지되었다"),
        ];
        for (word, span) in expect {
            let line = script
                .lines
                .iter()
                .find(|l| l.word() == Some(word))
                .unwrap();
            let (a, b) = line.span.unwrap();
            assert_eq!(&lesson.korean[a..b], span, "{word}");
        }
    }

    #[test]
    fn day_three_vocab_follows_the_sentence() {
        let lesson = crate::content::get(2, 3).unwrap();
        let script = compile(lesson, Timing::default());
        let vocab: Vec<_> = script
            .lines
            .iter()
            .take_while(|l| l.section == "Vocabulary")
            .map(|l| l.word().unwrap_or("").to_string())
            .collect();
        assert_eq!(
            vocab,
            [
                "체벌금지법 體罰禁止法",
                "도입 導入",
                "이후 以後",
                "교사 敎師",
                "학생 學生",
                "지도 指導",
                "대단히",
                "어렵다",
            ]
            .map(str::to_string)
        );
        let expect = [
            ("체벌금지법 體罰禁止法", "체벌금지법"),
            ("도입 導入", "도입"),
            ("이후 以後", "이후"),
            ("교사 敎師", "교사"),
            ("학생 學生", "학생"),
            ("지도 指導", "지도"),
            ("대단히", "대단히"),
            ("어렵다", "어려"),
            ("체벌금지법", "체벌금지법"),
            ("도입 이후", "도입 이후"),
            ("교사들은", "교사들은"),
            ("학생들을", "학생들을"),
            ("지도하기", "지도하기"),
            ("대단히", "대단히"),
            ("어려워졌다", "어려워졌다"),
        ];
        for (word, span) in expect {
            let line = script
                .lines
                .iter()
                .find(|l| l.word() == Some(word))
                .unwrap();
            let (a, b) = line.span.expect(word);
            assert_eq!(&lesson.korean[a..b], span, "{word}");
        }
    }

    #[test]
    fn day_four_vocab_follows_the_sentence() {
        let lesson = crate::content::get(2, 4).unwrap();
        let script = compile(lesson, Timing::default());
        let expect = [
            ("학생 學生", "학생"),
            ("마음", "마음"),
            ("놓다", "놓"),
            ("교권 敎權", "교권"),
            ("침해 侵害", "침해"),
            ("행동 行動", "행동"),
            ("제재 制裁", "제재"),
            ("수단 手段", "수단"),
            ("그다지", "그다지"),
            ("많다", "많"),
            ("학생들이", "학생들이"),
            ("마음놓고", "마음놓고"),
            ("교권을", "교권을"),
            ("침해하는", "침해하는"),
            ("행동을", "행동을"),
            ("하여도", "하여도"),
            ("이를", "이를"),
            ("제재할", "제재할"),
            ("수단이", "수단이"),
            ("그다지", "그다지"),
            ("많지 않았던", "많지 않았던"),
            ("것이다", "것이다"),
        ];
        for (word, span) in expect {
            let line = script
                .lines
                .iter()
                .find(|l| l.word() == Some(word))
                .unwrap();
            let shown = line.span.map(|(a, b)| &lesson.korean[a..b]).unwrap_or("NONE");
            assert_eq!(shown, span, "{word}");
        }
    }
}
