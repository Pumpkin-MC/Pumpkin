use pumpkin_data::packet::clientbound::PLAY_CONTAINER_CLOSE;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// Notifies the client that a container (inventory, chest, etc.) has been closed.
///
/// This is used by the server to force the player's UI to shut, for example,
/// if the player moves too far away from a chest or if an NPC's trade window
/// is invalidated.
#[derive(Serialize)]
#[java_packet(PLAY_CONTAINER_CLOSE)]
pub struct CCloseContainer {
    /// The ID of the container window to close.
    ///
    /// A value of 0 usually refers to the player's own inventory, while higher
    /// values refer to active windows opened via previous packets.
    pub sync_id: VarInt,
}
impl CCloseContainer {
    #[must_use]
    pub const fn new(window_id: VarInt) -> Self {
        Self { sync_id: window_id }
    }
}
