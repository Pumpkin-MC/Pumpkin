use pumpkin_data::packet::clientbound::PLAY_COOLDOWN;

use crate::codec::var_int::VarInt;
use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(PLAY_COOLDOWN)]
pub struct CItemCooldown {
    pub group: String,
    pub cooldown: VarInt,
}

impl CItemCooldown {
    #[must_use]
    pub const fn new(group: String, cooldown: VarInt) -> Self {
        Self { group, cooldown }
    }
}
