pub mod chamgyoyuk;
pub mod toesa;

#[derive(Clone, Copy, Debug)]
pub struct Item {
    pub word: &'static str,
    /// Hanja shown beside the Korean word. 하다 is never written in hanja.
    pub hanja: &'static str,
    pub meaning: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct Lesson {
    /// Series number. Hash `#/2` is series 2.
    pub series_n: u16,
    /// Episode within the series. Hash `#/2/0` is series 2, episode 0.
    pub episode: u16,
    pub series: &'static str,
    pub korean: &'static str,
    pub meaning: &'static str,
    pub vocab: &'static [Item],
    pub grammar: &'static [Item],
}

pub const SERIES: &[(&str, &[Lesson])] = &[
    ("퇴사할게여", toesa::LESSONS),
    ("참교육", chamgyoyuk::LESSONS),
];

pub fn all() -> impl Iterator<Item = &'static Lesson> {
    SERIES.iter().flat_map(|(_, lessons)| lessons.iter())
}

pub fn get(series_n: u16, episode: u16) -> Option<&'static Lesson> {
    all().find(|l| l.series_n == series_n && l.episode == episode)
}

pub fn series_name(series_n: u16) -> Option<&'static str> {
    SERIES
        .get(series_n.saturating_sub(1) as usize)
        .map(|(name, _)| *name)
}

pub fn episodes_in(series_n: u16) -> Vec<u16> {
    all()
        .filter(|l| l.series_n == series_n)
        .map(|l| l.episode)
        .collect()
}

pub fn next_episode(series_n: u16, episode: u16) -> Option<(u16, u16)> {
    let list = episodes_in(series_n);
    let i = list.iter().position(|&n| n == episode)?;
    let next = list.get(i + 1).copied().or_else(|| list.first().copied())?;
    Some((series_n, next))
}

pub const fn v(word: &'static str, meaning: &'static str) -> Item {
    Item {
        word,
        hanja: "",
        meaning,
    }
}

pub const fn vh(word: &'static str, hanja: &'static str, meaning: &'static str) -> Item {
    Item {
        word,
        hanja,
        meaning,
    }
}
