use pumpkin_data::{
    packet::clientbound::PLAY_SET_DISPLAY_OBJECTIVE, scoreboard::ScoreboardDisplaySlot,
};
use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_SET_DISPLAY_OBJECTIVE)]
pub struct CDisplayObjective {
    position: VarInt,
    score_name: String,
}

impl CDisplayObjective {
    pub fn new(position: ScoreboardDisplaySlot, score_name: String) -> Self {
        Self {
            position: VarInt(position as i32),
            score_name,
        }
    }
}
