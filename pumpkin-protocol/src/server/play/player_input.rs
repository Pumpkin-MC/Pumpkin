use pumpkin_data::packet::serverbound::PLAY_PLAYER_INPUT;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[packet(PLAY_PLAYER_INPUT)]
pub struct SPlayerInput {
    // Yep, exactly how it looks like
    pub input: i8,
}

impl SPlayerInput {
    pub const FORWARD: i8 = 1;
    pub const BACKWARD: i8 = 2;
    pub const LEFT: i8 = 4;
    pub const RIGHT: i8 = 8;
    pub const JUMP: i8 = 16;
    pub const SNEAK: i8 = 32;
    pub const SPRINT: i8 = 64;
}
