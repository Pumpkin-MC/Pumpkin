use pumpkin_data::packet::clientbound::PLAY_CHANGE_DIFFICULTY;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_CHANGE_DIFFICULTY)]
pub struct CChangeDifficulty {
    difficulty: u8,
    locked: bool,
}

impl CChangeDifficulty {
    pub fn new(difficulty: u8, locked: bool) -> Self {
        Self { difficulty, locked }
    }
}
