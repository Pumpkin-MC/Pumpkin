use std::io::{Read, Write};

use pumpkin_data::packet::clientbound::play::ADD_ENTITY;
use pumpkin_macros::java_packet;
use pumpkin_util::{
    math::{pack_degrees, vector3::Vector3},
    version::JavaMinecraftVersion,
};

use crate::{
    ClientPacket, VarInt,
    codec::lp_vector_3d::LpVector3d,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};

// TODO: `unpack_degrees` helper next to `pumpkin_util::math::pack_degrees`.
const ROTATION_FACTOR: f32 = 256.0 / 360.0;

#[java_packet(ADD_ENTITY)]
pub struct CSpawnEntity {
    pub entity_id: VarInt,
    pub entity_uuid: uuid::Uuid,
    pub r#type: VarInt,
    pub position: Vector3<f64>,
    pub velocity: LpVector3d,
    pub pitch: u8,    // angle
    pub yaw: u8,      // angle
    pub head_yaw: u8, // angle
    pub data: VarInt,
}

impl CSpawnEntity {
    #[expect(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        entity_id: VarInt,
        entity_uuid: uuid::Uuid,
        r#type: VarInt,
        position: Vector3<f64>,
        pitch: f32,
        yaw: f32,
        head_yaw: f32,
        data: VarInt,
        velocity: Vector3<f64>,
    ) -> Self {
        Self::new_packed(
            entity_id,
            entity_uuid,
            r#type,
            position,
            pack_degrees(pitch),
            pack_degrees(yaw),
            pack_degrees(head_yaw),
            data,
            velocity,
        )
    }

    /// Already packed with vanilla `Mth.packDegrees` (tracker last-sent bytes).
    #[expect(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new_packed(
        entity_id: VarInt,
        entity_uuid: uuid::Uuid,
        r#type: VarInt,
        position: Vector3<f64>,
        pitch: u8,
        yaw: u8,
        head_yaw: u8,
        data: VarInt,
        velocity: Vector3<f64>,
    ) -> Self {
        Self {
            entity_id,
            entity_uuid,
            r#type,
            position,
            pitch,
            yaw,
            head_yaw,
            data,
            velocity: LpVector3d(velocity),
        }
    }

    #[must_use]
    pub fn pitch_degrees(&self) -> f32 {
        (self.pitch as i8 as f32) / ROTATION_FACTOR
    }

    #[must_use]
    pub fn yaw_degrees(&self) -> f32 {
        (self.yaw as i8 as f32) / ROTATION_FACTOR
    }

    #[must_use]
    pub fn head_yaw_degrees(&self) -> f32 {
        (self.head_yaw as i8 as f32) / ROTATION_FACTOR
    }

    pub fn read_packet_data(mut read: impl Read) -> Result<Self, ReadingError> {
        let entity_id = read.get_var_int()?;

        let entity_uuid = read.get_uuid()?;

        let r#type = read.get_var_int()?;

        let position = Vector3::new(read.get_f64_be()?, read.get_f64_be()?, read.get_f64_be()?);

        let velocity = LpVector3d::read(&mut read)?;

        let pitch = read.get_u8()?;
        let yaw = read.get_u8()?;

        let head_yaw = read.get_u8()?;

        let data = read.get_var_int()?;

        Ok(Self {
            entity_id,
            entity_uuid,
            r#type,
            position,
            velocity,
            pitch,
            yaw,
            head_yaw,
            data,
        })
    }
}

impl ClientPacket for CSpawnEntity {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_var_int(&self.entity_id)?;

        write.write_uuid(&self.entity_uuid)?;

        write.write_var_int(&self.r#type)?;

        write.write_f64_be(self.position.x)?;
        write.write_f64_be(self.position.y)?;
        write.write_f64_be(self.position.z)?;

        self.velocity.write(&mut write)?;

        write.write_u8(self.pitch)?;
        write.write_u8(self.yaw)?;

        write.write_u8(self.head_yaw)?;

        write.write_var_int(&self.data)?;

        Ok(())
    }
}
