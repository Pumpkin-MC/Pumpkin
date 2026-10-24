// Last verified for v2169

use crate::{codec::var_ulong::VarULong, serial::PacketRead};
use pumpkin_macros::packet;

/// Sent by the client after receiving PlayStatus(PlayerSpawn) to signal that world initialization is complete.
#[derive(PacketRead)]
#[packet(113)]
pub struct SSetLocalPlayerAsInitialized {
    /// Runtime entity ID assigned to the player in `StartGame`.
    pub player_id: VarULong,
}
