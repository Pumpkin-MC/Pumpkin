// Last verified for v2169

use crate::serial::PacketWrite;
use pumpkin_macros::packet;

/// Removes a scoreboard objective, hiding it from the player's screen.
#[derive(PacketWrite)]
#[packet(106)]
pub struct CRemoveObjective {
    /// Identifier of the active objective to remove.
    pub objective_name: String,
}
