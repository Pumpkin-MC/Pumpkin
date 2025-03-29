use pumpkin_data::packet::clientbound::PLAY_CHANGE_DIFFICULTY;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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
