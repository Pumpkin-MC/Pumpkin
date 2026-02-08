use pumpkin_data::packet::clientbound::PLAY_PLAYER_COMBAT_END;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// Notifies the client that combat has ended.
#[derive(Serialize)]
#[java_packet(PLAY_PLAYER_COMBAT_END)]
pub struct CPlayerCombatEnd {
    /// Duration of the combat in ticks.
    pub duration: VarInt,
}

impl CPlayerCombatEnd {
    #[must_use]
    pub const fn new(duration: VarInt) -> Self {
        Self { duration }
    }
}
