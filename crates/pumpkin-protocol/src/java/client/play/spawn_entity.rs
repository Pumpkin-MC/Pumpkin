use std::io::{Read, Write};

use pumpkin_data::entity::EntityType;
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
const VELOCITY_FACTOR: f64 = 8000.0;

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

    pub fn read_packet_data(
        mut read: impl Read,
        version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        let v1_9 = *version >= JavaMinecraftVersion::V_1_9;
        let v1_14 = *version >= JavaMinecraftVersion::V_1_14;
        let v1_19 = *version >= JavaMinecraftVersion::V_1_19;
        let v1_21_9 = *version >= JavaMinecraftVersion::V_1_21_9;

        let entity_id = read.get_var_int()?;

        let entity_uuid = if v1_9 {
            read.get_uuid()?
        } else {
            uuid::Uuid::nil()
        };

        let r#type = if v1_14 {
            read.get_var_int()?
        } else {
            VarInt(i32::from(read.get_u8()?))
        };

        let position = if v1_9 {
            Vector3::new(read.get_f64_be()?, read.get_f64_be()?, read.get_f64_be()?)
        } else {
            Vector3::new(
                f64::from(read.get_i32_be()?) / 32.0,
                f64::from(read.get_i32_be()?) / 32.0,
                f64::from(read.get_i32_be()?) / 32.0,
            )
        };

        let mut velocity = if v1_21_9 {
            LpVector3d::read(&mut read)?
        } else {
            LpVector3d(Vector3::new(0.0, 0.0, 0.0))
        };

        let pitch = read.get_u8()?;
        let yaw = read.get_u8()?;

        let head_yaw = if v1_19 { read.get_u8()? } else { 0 };

        let data = if v1_19 {
            read.get_var_int()?
        } else {
            VarInt(read.get_i32_be()?)
        };

        if !v1_21_9 && (v1_9 || data.0 > 0) {
            let vel_x = f64::from(read.get_i16_be()?) / VELOCITY_FACTOR;
            let vel_y = f64::from(read.get_i16_be()?) / VELOCITY_FACTOR;
            let vel_z = f64::from(read.get_i16_be()?) / VELOCITY_FACTOR;
            velocity = LpVector3d(Vector3::new(vel_x, vel_y, vel_z));
        }

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
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let v1_9 = *version >= JavaMinecraftVersion::V_1_9;
        let v1_14 = *version >= JavaMinecraftVersion::V_1_14;
        let v1_19 = *version >= JavaMinecraftVersion::V_1_19;
        let v1_21_9 = *version >= JavaMinecraftVersion::V_1_21_9;

        write.write_var_int(&self.entity_id)?;

        if v1_9 {
            write.write_uuid(&self.entity_uuid)?;
        }

        if v1_14 {
            write.write_var_int(&self.r#type)?;
        } else {
            write.write_u8(self.r#type.0 as u8)?;
        }

        if v1_9 {
            write.write_f64_be(self.position.x)?;
            write.write_f64_be(self.position.y)?;
            write.write_f64_be(self.position.z)?;
        } else {
            write.write_i32_be((self.position.x * 32.0).floor() as i32)?;
            write.write_i32_be((self.position.y * 32.0).floor() as i32)?;
            write.write_i32_be((self.position.z * 32.0).floor() as i32)?;
        }

        if v1_21_9 {
            self.velocity.write(&mut write)?;
        }

        write.write_u8(self.pitch)?;
        write.write_u8(self.yaw)?;

        if v1_19 {
            write.write_u8(self.head_yaw)?;
        }

        let mut data = self.data;

        if !v1_14 && data.0 == 0 {
            if self.r#type.0 == i32::from(EntityType::CHEST_MINECART.id) {
                data = VarInt(1);
            } else if self.r#type.0 == i32::from(EntityType::FURNACE_MINECART.id) {
                data = VarInt(2);
            } else if self.r#type.0 == i32::from(EntityType::TNT_MINECART.id) {
                data = VarInt(3);
            } else if self.r#type.0 == i32::from(EntityType::SPAWNER_MINECART.id) {
                data = VarInt(4);
            } else if self.r#type.0 == i32::from(EntityType::HOPPER_MINECART.id) {
                data = VarInt(5);
            } else if self.r#type.0 == i32::from(EntityType::COMMAND_BLOCK_MINECART.id) {
                data = VarInt(6);
            } else if self.r#type.0 == i32::from(EntityType::ITEM.id) {
                data = VarInt(1);
            }
        }

        if v1_19 {
            write.write_var_int(&data)?;
        } else {
            write.write_i32_be(data.0)?;
        }

        if !v1_21_9 && (v1_9 || data.0 > 0) {
            self.velocity.write_legacy(&mut write)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::CSpawnEntity;
    use crate::{ClientPacket, VarInt, codec::lp_vector_3d::encode_legacy_velocity_component};
    use pumpkin_util::version::JavaMinecraftVersion;

    fn legacy_tail(velocity: pumpkin_util::math::vector3::Vector3<f64>) -> [u8; 6] {
        let x = encode_legacy_velocity_component(velocity.x);
        let y = encode_legacy_velocity_component(velocity.y);
        let z = encode_legacy_velocity_component(velocity.z);
        let xb = x.to_be_bytes();
        let yb = y.to_be_bytes();
        let zb = z.to_be_bytes();
        [xb[0], xb[1], yb[0], yb[1], zb[0], zb[1]]
    }

    fn encode_spawn(version: JavaMinecraftVersion) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let velocity = pumpkin_util::math::vector3::Vector3::new(0.5, -0.5, 0.25);
        let packet = CSpawnEntity::new(
            VarInt(1),
            uuid::Uuid::nil(),
            VarInt(1),
            pumpkin_util::math::vector3::Vector3::new(1.0, 2.0, 3.0),
            0.0,
            90.0,
            90.0,
            VarInt(42),
            velocity,
        );
        let mut out = Vec::new();
        packet.write_packet_data(&mut out, &version)?;
        Ok(out)
    }

    #[test]
    fn spawn_entity_uses_legacy_velocity_tail_for_1_21_8() -> Result<(), Box<dyn std::error::Error>>
    {
        // V_1_21_7 enum variant represents protocol 772 (used by 1.21.7 and 1.21.8).
        let velocity = pumpkin_util::math::vector3::Vector3::new(0.5, -0.5, 0.25);
        let expected_tail = legacy_tail(velocity);
        let encoded = encode_spawn(JavaMinecraftVersion::V_1_21_7)?;

        assert!(encoded.ends_with(&expected_tail));
        Ok(())
    }

    #[test]
    fn spawn_entity_does_not_use_legacy_velocity_tail_for_1_21_9()
    -> Result<(), Box<dyn std::error::Error>> {
        let velocity = pumpkin_util::math::vector3::Vector3::new(0.5, -0.5, 0.25);
        let expected_tail = legacy_tail(velocity);
        let encoded = encode_spawn(JavaMinecraftVersion::V_1_21_9)?;

        assert!(!encoded.ends_with(&expected_tail));
        Ok(())
    }
}
