// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

/// Transitions the player to a different dimension with a loading screen.
#[derive(PacketWrite)]
#[packet(61)]
pub struct CChangeDimension {
    /// Target dimension identifier (0: Overworld, 1: Nether, 2: The End).
    ///
    /// Must differ from the player's current dimension to prevent an infinite loading screen.
    pub dimension_id: VarInt,
    /// Coordinates where the player spawns in the destination dimension.
    pub position: Vector3<f32>,
    /// Whether this dimension change was triggered by player respawn after death.
    pub respawn: bool,
    /// Optional identifier for the loading screen session.
    pub loading_screen_id: Option<u32>,
}
