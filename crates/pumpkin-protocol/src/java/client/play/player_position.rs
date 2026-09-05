use std::io::Write;

use pumpkin_data::packet::clientbound::play::PLAYER_POSITION;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

use crate::{
    ClientPacket, PositionFlag, ServerPacket, VarInt, WritingError, ser::NetworkReadExt,
    ser::NetworkWriteExt,
};

/// Updates the player's position and rotation on the client.
///
/// Commonly known as the "Teleport Packet," this is sent by the server to
/// force a change in the player's location. The client must respond with a
/// `Teleport Confirm` packet matching the `teleport_id`.
#[java_packet(PLAYER_POSITION)]
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
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if version >= &JavaMinecraftVersion::V_1_21_2 {
            // Reordered and added delta/int flags in 1.21.2
            write.write_var_int(&self.teleport_id)?;
            write.write_f64_be(self.position.x)?;
            write.write_f64_be(self.position.y)?;
            write.write_f64_be(self.position.z)?;
            write.write_f64_be(self.delta.x)?;
            write.write_f64_be(self.delta.y)?;
            write.write_f64_be(self.delta.z)?;
            write.write_f32_be(self.yaw)?;
            write.write_f32_be(self.pitch)?;
            write.write_i32_be(PositionFlag::get_bitfield(self.relatives.as_slice()))?;
        } else {
            write.write_f64_be(self.position.x)?;
            write.write_f64_be(self.position.y)?;
            write.write_f64_be(self.position.z)?;
            write.write_f32_be(self.yaw)?;
            write.write_f32_be(self.pitch)?;
            if version >= &JavaMinecraftVersion::V_1_8 {
                // Relative flags added in 1.8
                write.write_u8(PositionFlag::get_bitfield(self.relatives.as_slice()) as u8)?;
            } else {
                // 1.7.x: on_ground boolean
                write.write_bool(false)?;
            }
            if version >= &JavaMinecraftVersion::V_1_9 {
                // Teleport confirmation ID added in 1.9
                write.write_var_int(&self.teleport_id)?;
            }
            if *version >= JavaMinecraftVersion::V_1_17
                && *version <= JavaMinecraftVersion::V_1_19_3
            {
                write.write_bool(false)?;
            }
        }
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CPlayerPosition {
    fn read(
        read: &mut &'a [u8],
        version: &JavaMinecraftVersion,
    ) -> Result<Self, crate::ser::ReadingError> {
        if version >= &JavaMinecraftVersion::V_1_21_2 {
            let teleport_id = read.get_var_int()?;
            let x = read.get_f64_be()?;
            let y = read.get_f64_be()?;
            let z = read.get_f64_be()?;
            let dx = read.get_f64_be()?;
            let dy = read.get_f64_be()?;
            let dz = read.get_f64_be()?;
            let yaw = read.get_f32_be()?;
            let pitch = read.get_f32_be()?;
            let relatives_bits = read.get_i32_be()?;
            Ok(Self {
                teleport_id,
                position: Vector3::new(x, y, z),
                delta: Vector3::new(dx, dy, dz),
                yaw,
                pitch,
                relatives: PositionFlag::from_bitfield(relatives_bits),
            })
        } else {
            let x = read.get_f64_be()?;
            let y = read.get_f64_be()?;
            let z = read.get_f64_be()?;
            let yaw = read.get_f32_be()?;
            let pitch = read.get_f32_be()?;
            let relatives = if version >= &JavaMinecraftVersion::V_1_8 {
                let relatives_bits = i32::from(read.get_u8()?);
                PositionFlag::from_bitfield(relatives_bits)
            } else {
                let _on_ground = read.get_bool()?;
                Vec::new()
            };
            let teleport_id = if version >= &JavaMinecraftVersion::V_1_9 {
                read.get_var_int()?
            } else {
                VarInt(0)
            };
            if version >= &JavaMinecraftVersion::V_1_20_2 {
                let _ = read.get_bool()?;
            }
            Ok(Self {
                teleport_id,
                position: Vector3::new(x, y, z),
                delta: Vector3::new(0.0, 0.0, 0.0),
                yaw,
                pitch,
                relatives,
            })
        }
    }
}
