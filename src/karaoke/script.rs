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
        lines.push(card_line("Vocabulary", item, timing.card_ms));
    }
    for item in lesson.grammar {
        lines.push(card_line("Grammar", item, timing.card_ms));
    }
    Script { lines }
}

fn card_line(section: &'static str, item: &Item, dur: u32) -> Line {
    Line {
        section,
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
    use crate::content::get;

    #[test]
    fn day_one_vocab_then_grammar() {
        let lesson = get(1).unwrap();
        let script = compile(lesson, Timing::default());
        assert_eq!(
            script.lines.len(),
            lesson.vocab.len() + lesson.grammar.len()
        );
        assert_eq!(script.lines[0].section, "Vocabulary");
        assert!(script.lines.iter().any(|l| l.section == "Grammar"));
    }
}
