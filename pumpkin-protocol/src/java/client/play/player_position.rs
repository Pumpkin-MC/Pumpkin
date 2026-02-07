use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_PLAYER_POSITION;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::MinecraftVersion};

use crate::{
    ClientPacket, PositionFlag, ServerPacket, VarInt, WritingError, ser::NetworkReadExt,
    ser::NetworkWriteExt,
};

/// Updates the player's position and rotation on the client.
///
/// Commonly known as the "Teleport Packet," this is sent by the server to
/// force a change in the player's location. The client must respond with a
/// `Teleport Confirm` packet matching the `teleport_id`.
#[java_packet(PLAY_PLAYER_POSITION)]
pub struct CPlayerPosition {
    /// A unique ID for this teleport. The client must echo this back
    /// to confirm the teleport was processed.
    pub teleport_id: VarInt,
    /// The absolute or relative target position.
    pub position: Vector3<f64>,
    /// The intended velocity of the player after teleporting.
    pub delta: Vector3<f64>,
    /// The horizontal rotation (0-360 degrees).
    pub yaw: f32,
    /// The vertical rotation (-90 to 90 degrees).
    pub pitch: f32,
    /// A set of flags determining which of the above fields are relative (~).
    pub relatives: Vec<PositionFlag>,
}

impl CPlayerPosition {
    #[must_use]
    pub const fn new(
        teleport_id: VarInt,
        position: Vector3<f64>,
        delta: Vector3<f64>,
        yaw: f32,
        pitch: f32,
        relatives: Vec<PositionFlag>,
    ) -> Self {
        Self {
            teleport_id,
            position,
            delta,
            yaw,
            pitch,
            relatives,
        }
    }
}

// TODO: Do we need a custom impl?
impl ClientPacket for CPlayerPosition {
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_var_int(&self.teleport_id)?;
        write.write_f64_be(self.position.x)?;
        write.write_f64_be(self.position.y)?;
        write.write_f64_be(self.position.z)?;
        write.write_f64_be(self.delta.x)?;
        write.write_f64_be(self.delta.y)?;
        write.write_f64_be(self.delta.z)?;
        write.write_f32_be(self.yaw)?;
        write.write_f32_be(self.pitch)?;
        // not sure about that
        write.write_i32_be(PositionFlag::get_bitfield(self.relatives.as_slice()))
    }
}

impl ServerPacket for CPlayerPosition {
    fn read(mut read: impl std::io::Read) -> Result<Self, crate::ser::ReadingError> {
        let teleport_id = read.get_var_int()?;
        let position = Vector3::new(
            read.get_f64_be()?,
            read.get_f64_be()?,
            read.get_f64_be()?,
        );
        let delta = Vector3::new(
            read.get_f64_be()?,
            read.get_f64_be()?,
            read.get_f64_be()?,
        );
        let yaw = read.get_f32_be()?;
        let pitch = read.get_f32_be()?;
        let relatives = PositionFlag::from_bitfield(read.get_i32_be()?);

        Ok(Self {
            teleport_id,
            position,
            delta,
            yaw,
            pitch,
            relatives,
        })
    }
}
