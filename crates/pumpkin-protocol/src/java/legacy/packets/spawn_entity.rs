//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/spawn_entity.rs`.

use crate::java::client::play::CSpawnEntity;
use crate::java::legacy::LegacyWrite;
use crate::{
    VarInt,
    ser::{NetworkWriteExt, WritingError},
};
use pumpkin_data::entity::EntityType;
use pumpkin_util::version::JavaMinecraftVersion;
use std::io::Write;

impl LegacyWrite for CSpawnEntity {
    fn write_legacy(
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
            crate::java::legacy::write_legacy_velocity(&mut write, &self.velocity.0)?;
        }

        Ok(())
    }
}
