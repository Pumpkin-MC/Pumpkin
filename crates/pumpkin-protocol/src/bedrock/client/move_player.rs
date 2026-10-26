use crate::{codec::var_ulong::VarULong, serial::PacketWrite};
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;
use std::io::{Error, Write};

/// Sent by the server to update a player's movement, teleport, or view angle.
#[derive(Debug)]
#[packet(19)]
pub struct CMovePlayer {
    /// Runtime entity ID of the player being moved.
    pub player_runtime_id: VarULong,
    /// Absolute position coordinates in world space.
    pub position: Vector3<f32>,
    /// Pitch angle in degrees.
    pub pitch: f32,
    /// Yaw angle in degrees.
    pub yaw: f32,
    /// Horizontal head yaw rotation angle in degrees.
    pub head_yaw: f32,
    /// Movement mode (normal, reset, teleport, or rotation).
    pub mode: u8,
    /// Whether the player is resting on the ground.
    pub on_ground: bool,
    /// Runtime entity ID of the vehicle the player is riding, or zero.
    pub riding_runtime_id: VarULong,
    /// Cause of the teleportation (present when mode is `MODE_TELEPORT`).
    pub teleport_cause: i32,
    /// Entity type that triggered the teleportation.
    pub teleport_source_entity_type: i32,
    /// World simulation tick number.
    pub tick: VarULong,
}

impl CMovePlayer {
    /// Standard movement interpolation mode.
    pub const MODE_NORMAL: u8 = 0;
    /// Reset interpolation mode.
    pub const MODE_RESET: u8 = 1;
    /// Instantaneous teleportation mode.
    pub const MODE_TELEPORT: u8 = 2;
    /// Rotation-only update mode without changing position.
    pub const MODE_ROTATION: u8 = 3;

    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        player_runtime_id: VarULong,
        position: Vector3<f32>,
        pitch: f32,
        yaw: f32,
        head_yaw: f32,
        mode: u8,
        on_ground: bool,
        riding_runtime_id: VarULong,
        teleport_cause: i32,
        teleport_source_entity_type: i32,
        tick: VarULong,
    ) -> Self {
        Self {
            player_runtime_id,
            position,
            pitch,
            yaw,
            head_yaw,
            mode,
            on_ground,
            riding_runtime_id,
            teleport_cause,
            teleport_source_entity_type,
            tick,
        }
    }
}

impl PacketWrite for CMovePlayer {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.player_runtime_id.write(writer)?;
        self.position.write(writer)?;
        self.pitch.write(writer)?;
        self.yaw.write(writer)?;
        self.head_yaw.write(writer)?;
        self.mode.write(writer)?;
        self.on_ground.write(writer)?;
        self.riding_runtime_id.write(writer)?;
        (self.mode == Self::MODE_TELEPORT).write(writer)?;
        if self.mode == Self::MODE_TELEPORT {
            self.teleport_cause.write(writer)?;
            self.teleport_source_entity_type.write(writer)?;
        }
        self.tick.write(writer)?;
        Ok(())
    }
}
