// Last verified for v2169

use crate::{
    codec::{var_int::VarInt, var_ulong::VarULong},
    serial::PacketWrite,
};
use pumpkin_macros::packet;

/// Displays or dismisses the end-game rolling credits screen.
#[derive(PacketWrite)]
#[packet(75)]
pub struct CShowCredits {
    /// Runtime entity identifier of the player viewing the credits.
    pub player_runtime_id: VarULong,
    /// State transition flag (0: start showing credits, 1: stop showing credits).
    pub credits_state: VarInt,
}
