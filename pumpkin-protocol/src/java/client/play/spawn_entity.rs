use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_ADD_ENTITY;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::MinecraftVersion};

use crate::{
    ClientPacket, VarInt,
    codec::velocity::Velocity,
    ser::{NetworkWriteExt, WritingError},
};

#[java_packet(PLAY_ADD_ENTITY)]
pub struct CSpawnEntity {
    pub entity_id: VarInt,
    pub entity_uuid: uuid::Uuid,
    pub r#type: VarInt,
    pub position: Vector3<f64>,
    pub velocity: Velocity,
    pub pitch: u8,    // angle
    pub yaw: u8,      // angle
    pub head_yaw: u8, // angle
    pub data: VarInt,
}

impl CSpawnEntity {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        entity_id: VarInt,
        entity_uuid: uuid::Uuid,
        r#type: VarInt,
        position: Vector3<f64>,
        pitch: f32,    // angle
        yaw: f32,      // angle
        head_yaw: f32, // angle
        data: VarInt,
        velocity: Vector3<f64>,
    ) -> Self {
        Self {
            entity_id,
            entity_uuid,
            r#type,
            position,
            pitch: (pitch * 256.0 / 360.0).floor() as u8,
            yaw: (yaw.rem_euclid(360.0) * 256.0 / 360.0).floor() as u8,
            head_yaw: (head_yaw.rem_euclid(360.0) * 256.0 / 360.0).floor() as u8,
            data,
            velocity: Velocity(velocity),
        }
    }
}

impl ClientPacket for CSpawnEntity {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_var_int(&self.entity_id)?;
        write.write_uuid(&self.entity_uuid)?;
        write.write_var_int(&self.r#type)?;

        write.write_f64_be(self.position.x)?;
        write.write_f64_be(self.position.y)?;
        write.write_f64_be(self.position.z)?;

        // Angles
        if version >= &MinecraftVersion::V_1_21_9 {
            self.velocity.write(&mut write)?;
        }
        write.write_u8(self.pitch)?;
        write.write_u8(self.yaw)?;
        write.write_u8(self.head_yaw)?;

        write.write_var_int(&self.data)?;

        if version <= &MinecraftVersion::V_1_21_7 {
            write.write_i16_be(self.velocity.0.x as i16)?;
            write.write_i16_be(self.velocity.0.y as i16)?;
            write.write_i16_be(self.velocity.0.z as i16)?;
        }

        Ok(())
    }
}
