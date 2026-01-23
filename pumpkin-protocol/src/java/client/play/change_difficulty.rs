use pumpkin_data::packet::clientbound::PLAY_CHANGE_DIFFICULTY;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_CHANGE_DIFFICULTY)]
pub struct CChangeDifficulty {
    pub difficulty: u8,
    pub locked: bool,
}

impl CChangeDifficulty {
    #[must_use]
    pub const fn new(difficulty: u8, locked: bool) -> Self {
        Self { difficulty, locked }
    }
}
