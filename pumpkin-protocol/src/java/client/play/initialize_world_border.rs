use pumpkin_data::packet::clientbound::PLAY_INITIALIZE_BORDER;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::{VarInt, codec::var_long::VarLong};

/// Fully initializes the world border for the client.
///
/// This packet is sent when a player joins the world or changes dimensions.
/// It synchronizes the current position, size, and all warning parameters
/// to ensure the client-side visual barrier matches the server's authority.
#[derive(Serialize)]
#[java_packet(PLAY_INITIALIZE_BORDER)]
pub struct CInitializeWorldBorder {
    /// The X coordinate of the center of the world border.
    pub x: f64,
    /// The Z coordinate of the center of the world border.
    pub z: f64,
    /// The diameter the border is moving from.
    pub old_diameter: f64,
    /// The diameter the border is moving toward.
    pub new_diameter: f64,
    /// The time (in milliseconds) it will take to reach `new_diameter`.
    pub speed: VarLong,
    /// The maximum distance a player can be teleported by a portal
    /// before the border prevents the teleport.
    pub portal_teleport_boundary: VarInt,
    /// Distance in blocks from the border where the screen starts to tint red.
    pub warning_blocks: VarInt,
    /// Time in seconds that a player must be on a collision course with
    /// the border before the warning tint appears.
    pub warning_time: VarInt,
}

impl CInitializeWorldBorder {
    #[expect(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        x: f64,
        z: f64,
        old_diameter: f64,
        new_diameter: f64,
        speed: VarLong,
        portal_teleport_boundary: VarInt,
        warning_blocks: VarInt,
        warning_time: VarInt,
    ) -> Self {
        Self {
            x,
            z,
            old_diameter,
            new_diameter,
            speed,
            portal_teleport_boundary,
            warning_blocks,
            warning_time,
        }
    }
}
