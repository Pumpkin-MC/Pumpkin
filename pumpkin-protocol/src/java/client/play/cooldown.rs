use pumpkin_data::packet::clientbound::PLAY_COOLDOWN;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// Applies a cooldown period to an item group.
///
/// Used for ender pearls, chorus fruit, shields, etc.
/// Set `cooldown_ticks` to 0 to clear the cooldown.
#[derive(Serialize)]
#[java_packet(PLAY_COOLDOWN)]
pub struct CCooldown {
    pub item_id: VarInt,
    pub cooldown_ticks: VarInt,
}

impl CCooldown {
    #[must_use]
    pub const fn new(item_id: VarInt, cooldown_ticks: VarInt) -> Self {
        Self {
            item_id,
            cooldown_ticks,
        }
    }
}
