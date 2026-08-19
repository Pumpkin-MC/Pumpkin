use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_TELEPORT_ENTITY;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

use crate::{ClientPacket, PositionFlag, VarInt, WritingError, ser::NetworkWriteExt};

/// Only used when teleporting a player's vehicle, this packet is sent to the player.
#[java_packet(PLAY_TELEPORT_ENTITY)]
pub struct CTeleportEntity<'a> {
    pub entity_id: VarInt,
    pub position: Vector3<f64>,
    pub delta: Vector3<f64>,
    pub yaw: f32,
    pub pitch: f32,
    pub relatives: &'a [PositionFlag],
    pub on_ground: bool,
}

impl<'a> CTeleportEntity<'a> {
    #[must_use]
    pub const fn new(
        entity_id: VarInt,
        position: Vector3<f64>,
        delta: Vector3<f64>,
        yaw: f32,
        pitch: f32,
        relatives: &'a [PositionFlag],
        on_ground: bool,
    ) -> Self {
        Self {
            entity_id,
            position,
            delta,
            yaw,
            pitch,
            relatives,
            on_ground,
        }
    }
}

// TODO: Do we need a custom impl?
impl ClientPacket for CTeleportEntity<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_var_int(&self.entity_id)?;
        write.write_f64_be(self.position.x)?;
        write.write_f64_be(self.position.y)?;
        write.write_f64_be(self.position.z)?;
        if version >= &JavaMinecraftVersion::V_1_21_2 {
            write.write_f64_be(self.delta.x)?;
            write.write_f64_be(self.delta.y)?;
            write.write_f64_be(self.delta.z)?;
            write.write_f32_be(self.yaw)?;
            write.write_f32_be(self.pitch)?;
            write.write_i32_be(PositionFlag::get_bitfield(self.relatives))?;
        } else {
            write.write_u8((self.yaw.rem_euclid(360.0) * 256.0 / 360.0).floor() as u8)?;
            write.write_u8((self.pitch.rem_euclid(360.0) * 256.0 / 360.0).floor() as u8)?;
        }
        write.write_bool(self.on_ground)
    }
}
