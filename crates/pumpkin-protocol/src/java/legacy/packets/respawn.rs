//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/respawn.rs`.

use crate::java::client::play::CRespawn;
use crate::java::legacy::{LegacyRegistryNbt, LegacyWrite, LegacyWriteExt, LegacyWriteWith};
use crate::ser::{NetworkWriteExt, WritingError};
use pumpkin_util::version::JavaMinecraftVersion;

impl LegacyWriteWith<LegacyRegistryNbt<'_>> for CRespawn {
    fn write_legacy_with(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
        registry: &LegacyRegistryNbt<'_>,
    ) -> Result<(), WritingError> {
        let v1_14 = *version >= JavaMinecraftVersion::V_1_14;
        let v1_15 = *version >= JavaMinecraftVersion::V_1_15;
        let v1_16 = *version >= JavaMinecraftVersion::V_1_16;
        let v1_19 = *version >= JavaMinecraftVersion::V_1_19;
        let v1_19_3 = *version >= JavaMinecraftVersion::V_1_19_3;
        let v1_20 = *version >= JavaMinecraftVersion::V_1_20;
        let v1_20_2 = *version >= JavaMinecraftVersion::V_1_20_2;

        if !v1_16 {
            let legacy_dim_id: i32 = match self.player_spawn_info.dimension.minecraft_name {
                "minecraft:the_nether" => -1,
                "minecraft:the_end" => 1,
                _ => 0,
            };
            write.write_i32_be(legacy_dim_id)?;
            if v1_15 {
                write.write_i64_be(self.player_spawn_info.hashed_seed)?;
            } else if !v1_14 {
                // Difficulty: 0: peaceful, 1: easy, 2: normal, 3: hard (default: 2 Normal)
                write.write_u8(2)?;
            }
            write.write_u8(self.player_spawn_info.game_mode)?;
            let level_type = if self.player_spawn_info.is_flat {
                "flat"
            } else if self.player_spawn_info.debug {
                "debug_all_block_states"
            } else {
                "default"
            };
            write.write_string(level_type)?;
            return Ok(());
        }

        if !v1_20_2 {
            let dimension = self.player_spawn_info.dimension.minecraft_name;
            // 1.16.2 - 1.18.2 send the dimension type itself, the others its name
            if let Some(dimension_type) = registry.dimension_type {
                write.write_all(dimension_type)?;
            } else {
                write.write_string(dimension)?;
            }
            write.write_string(self.player_spawn_info.dimension.minecraft_name)?;
            write.write_i64_be(self.player_spawn_info.hashed_seed)?;
            write.write_u8(self.player_spawn_info.game_mode)?;
            write.write_i8(self.player_spawn_info.previous_gamemode)?;
            write.write_bool(self.player_spawn_info.debug)?;
            write.write_bool(self.player_spawn_info.is_flat)?;
            if v1_19_3 {
                write.write_u8(self.data_kept)?;
            } else {
                write.write_bool((self.data_kept & Self::KEEP_ATTRIBUTES) != 0)?;
            }
            if v1_19 {
                write.write_option(
                    &self.player_spawn_info.death_dimension_name,
                    |write, (dim, pos)| {
                        write.write_string(dim)?;
                        write.write_block_pos_legacy(pos, version)?;
                        Ok(())
                    },
                )?;
            }
            if v1_20 {
                write.write_var_int(&self.player_spawn_info.portal_cooldown)?;
            }
            return Ok(());
        }

        self.player_spawn_info.write_legacy(&mut write, version)?;
        write.write_u8(self.data_kept)?;
        Ok(())
    }
}
