//! Read page vs record mode (9:16 short).

mod player;
mod script;
mod ui;

pub use player::Player;
pub use script::{compile, Timing};
pub use ui::{KaraokeLyrics, KaraokePlay, KaraokeRead, KaraokePlate};
