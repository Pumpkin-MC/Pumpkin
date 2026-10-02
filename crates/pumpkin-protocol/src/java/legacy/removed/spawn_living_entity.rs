use crate::java::legacy::ids::clientbound;
use crate::java::legacy::{LegacyPacket, LegacyRead, LegacyWrite, legacy_ids};
use std::io::{Read, Write};

use crate::{
    VarInt,
    codec::lp_vector_3d::LpVector3d,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

const ROTATION_FACTOR: f32 = 256.0 / 360.0;
const VELOCITY_FACTOR: f64 = 8000.0;

#[derive(Clone, Debug, PartialEq)]
/// Spawns a mob for versions before 1.19; `type` is already the client's entity id.
pub struct CSpawnLivingEntity {
    pub entity_id: VarInt,
    pub entity_uuid: uuid::Uuid,
    pub r#type: VarInt,
    pub position: Vector3<f64>,
    pub yaw: u8,
    pub pitch: u8,
    pub head_yaw: u8,
    pub velocity: LpVector3d,
    pub metadata: Option<Box<[u8]>>,
}

legacy_ids! { CSpawnLivingEntity => clientbound::play::SPAWN_LIVING_ENTITY }

impl CSpawnLivingEntity {
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
        velocity: Vector3<f64>,
        metadata: Option<Box<[u8]>>,
    ) -> Self {
        Self {
            entity_id,
            entity_uuid,
            r#type,
            position,
            pitch: (pitch * ROTATION_FACTOR).floor() as u8,
            yaw: (yaw.rem_euclid(360.0) * ROTATION_FACTOR).floor() as u8,
            head_yaw: (head_yaw.rem_euclid(360.0) * ROTATION_FACTOR).floor() as u8,
            velocity: LpVector3d(velocity),
            metadata,
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

    pub fn read_packet_data(
        mut read: impl Read,
        version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        let v1_9 = *version >= JavaMinecraftVersion::V_1_9;
        let v1_11 = *version >= JavaMinecraftVersion::V_1_11;
        let v1_15 = *version >= JavaMinecraftVersion::V_1_15;

        let entity_id = read.get_var_int()?;

        let (entity_uuid, r#type, position) = if v1_9 {
            let entity_uuid = read.get_uuid()?;
            let type_id = if v1_11 {
                read.get_var_int()?
            } else {
                VarInt(i32::from(read.get_u8()?))
            };
            let position = Vector3::new(read.get_f64_be()?, read.get_f64_be()?, read.get_f64_be()?);
            (entity_uuid, type_id, position)
        } else {
            let entity_uuid = uuid::Uuid::nil();
            let type_id = VarInt(i32::from(read.get_u8()?));
            let position = Vector3::new(
                f64::from(read.get_i32_be()?) / 32.0,
                f64::from(read.get_i32_be()?) / 32.0,
                f64::from(read.get_i32_be()?) / 32.0,
            );
            (entity_uuid, type_id, position)
        };

        let yaw = read.get_u8()?;
        let pitch = read.get_u8()?;
        let head_yaw = read.get_u8()?;

        let vel_x = f64::from(read.get_i16_be()?) / VELOCITY_FACTOR;
        let vel_y = f64::from(read.get_i16_be()?) / VELOCITY_FACTOR;
        let vel_z = f64::from(read.get_i16_be()?) / VELOCITY_FACTOR;
        let velocity = LpVector3d(Vector3::new(vel_x, vel_y, vel_z));

        let metadata = if v1_15 {
            None
        } else {
            let mut meta_bytes = Vec::new();
            read.read_to_end(&mut meta_bytes)
                .map_err(|e| ReadingError::Message(e.to_string()))?;
            if meta_bytes.is_empty() {
                None
            } else {
                Some(meta_bytes.into_boxed_slice())
            }
        };

        Ok(Self {
            entity_id,
            entity_uuid,
            r#type,
            position,
            yaw,
            pitch,
            head_yaw,
            velocity,
            metadata,
        })
    }
}

impl<'a> LegacyRead<'a> for CSpawnLivingEntity {
    fn read_legacy(
        bytebuf: &mut &'a [u8],
        version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        Self::read_packet_data(bytebuf, version)
    }
}

impl LegacyWrite for CSpawnLivingEntity {
    fn write_legacy(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let v1_9 = *version >= JavaMinecraftVersion::V_1_9;
        let v1_11 = *version >= JavaMinecraftVersion::V_1_11;
        let v1_15 = *version >= JavaMinecraftVersion::V_1_15;

        write.write_var_int(&self.entity_id)?;

        if v1_9 {
            write.write_uuid(&self.entity_uuid)?;
            if v1_11 {
                write.write_var_int(&self.r#type)?;
            } else {
                write.write_u8(self.r#type.0 as u8)?;
            }
            write.write_f64_be(self.position.x)?;
            write.write_f64_be(self.position.y)?;
            write.write_f64_be(self.position.z)?;
        } else {
            write.write_u8(self.r#type.0 as u8)?;
            write.write_i32_be((self.position.x * 32.0).floor() as i32)?;
            write.write_i32_be((self.position.y * 32.0).floor() as i32)?;
            write.write_i32_be((self.position.z * 32.0).floor() as i32)?;
        }

        write.write_u8(self.yaw)?;
        write.write_u8(self.pitch)?;
        write.write_u8(self.head_yaw)?;

        crate::java::legacy::write_legacy_velocity(&mut write, &self.velocity.0)?;

        if !v1_15 {
            if v1_9 {
                if let Some(metadata) = &self.metadata {
                    write.write_slice(metadata)?;
                } else {
                    write.write_u8(0xFF)?;
                }
            } else if let Some(metadata) = &self.metadata
                && metadata.last().copied() == Some(127)
            {
                write.write_slice(metadata)?;
            } else {
                // In <= 1.8, write the 127 (0x7F) terminator byte for the DataWatcher list,
                // matching vanilla and PacketEvents.
                write.write_u8(127)?;
            }
        }

        Ok(())
    }
}
