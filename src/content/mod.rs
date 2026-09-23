pub mod toesa;

#[derive(Clone, Copy, Debug)]
pub struct Item {
    pub word: &'static str,
    pub meaning: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct Lesson {
    pub day: u16,
    pub series: &'static str,
    pub korean: &'static str,
    pub meaning: &'static str,
    pub vocab: &'static [Item],
    pub grammar: &'static [Item],
}

pub const SERIES: &[(&str, &[Lesson])] = &[("퇴사할게여", toesa::LESSONS)];

pub fn all() -> impl Iterator<Item = &'static Lesson> {
    SERIES.iter().flat_map(|(_, lessons)| lessons.iter())
}

pub fn get(day: u16) -> Option<&'static Lesson> {
    all().find(|l| l.day == day)
}

pub fn days() -> Vec<u16> {
    all().map(|l| l.day).collect()
}

pub fn next_day(day: u16) -> Option<u16> {
    let list = days();
    let i = list.iter().position(|&n| n == day)?;
    list.get(i + 1).copied().or_else(|| list.first().copied())
}

pub const fn v(word: &'static str, meaning: &'static str) -> Item {
    Item { word, meaning }
}
