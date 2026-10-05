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
    for item in lesson.vocab {
        lines.push(card_line("Vocabulary", item, timing.card_ms, lesson.korean));
    }
    for item in lesson.grammar {
        lines.push(card_line("Grammar", item, timing.card_ms, lesson.korean));
    }
    Script { lines }
}

fn card_line(section: &'static str, item: &Item, dur: u32, sentence: &str) -> Line {
    Line {
        section,
        span: span_in(sentence, item.word),
        tokens: vec![
            Token {
                text: item.word.to_string(),
                kind: Kind::Word,
                dur_ms: dur,
            },
            Token {
                text: item.meaning.to_string(),
                kind: Kind::Cite,
                dur_ms: 0,
            },
        ],
    }
}

/// Longest Hangul/Latin run in the card label that occurs in the sentence.
/// `작품은` wins over `작품`; Hanja glosses are skipped.
/// A dictionary verb (`만들다`) highlights its stem when the sentence uses a form (`만들어진`).
fn span_in(sentence: &str, label: &str) -> Option<(usize, usize)> {
    let sentence = sentence.trim();
    let mut best: Option<(usize, usize)> = None;
    let mut best_len = 0usize;
    for (start, ch) in label.char_indices() {
        if !is_ko_or_latin(ch) {
            continue;
        }
        let mut end = start + ch.len_utf8();
        for c in label[end..].chars() {
            if is_ko_or_latin(c) {
                end += c.len_utf8();
            } else {
                break;
            }
        }
        let needle = &label[start..end];
        consider(sentence, needle, &mut best, &mut best_len);
        if let Some(stem) = verb_stem(needle) {
            consider(sentence, stem, &mut best, &mut best_len);
        }
    }
    best
}

fn consider(sentence: &str, needle: &str, best: &mut Option<(usize, usize)>, best_len: &mut usize) {
    let n = needle.chars().count();
    if n == 0 || n < *best_len {
        return;
    }
    let Some(at) = sentence.find(needle) else {
        return;
    };
    if n > *best_len || best.is_none_or(|(b, _)| at < b) {
        *best = Some((at, at + needle.len()));
        *best_len = n;
    }
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
}
