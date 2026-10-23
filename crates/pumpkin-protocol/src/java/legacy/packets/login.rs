//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/login.rs`.

use crate::java::client::play::CLogin;
use crate::java::legacy::packets::player_spawn_data::write_game_modes;
use crate::java::legacy::{LegacyRegistryNbt, LegacyWriteExt, LegacyWriteWith};
use crate::{
    VarInt,
    ser::{NetworkWriteExt, WritingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

impl LegacyWriteWith<LegacyRegistryNbt<'_>> for CLogin<'_> {
    #[expect(clippy::too_many_lines)]
    fn write_legacy_with(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
        registry: &LegacyRegistryNbt<'_>,
    ) -> Result<(), WritingError> {
        write.write_i32_be(self.entity_id)?;

        let v1_20_2 = *version >= JavaMinecraftVersion::V_1_20_2;
        let v1_20_5 = *version >= JavaMinecraftVersion::V_1_20_5;
        let v1_21_2 = *version >= JavaMinecraftVersion::V_1_21_2;
        let v1_26_2 = *version >= JavaMinecraftVersion::V_26_2;
        let v1_19 = *version >= JavaMinecraftVersion::V_1_19;
        let v1_18 = *version >= JavaMinecraftVersion::V_1_18;
        let v1_16_2 = *version >= JavaMinecraftVersion::V_1_16_2;
        let v1_16 = *version >= JavaMinecraftVersion::V_1_16;
        let v1_15 = *version >= JavaMinecraftVersion::V_1_15;
        let v1_14 = *version >= JavaMinecraftVersion::V_1_14;
        let v1_8 = *version >= JavaMinecraftVersion::V_1_8;

        // Hardcore & GameMode
        if v1_16_2 {
            write.write_bool(self.is_hardcore)?;
            if !v1_20_2 {
                write.write_u8(self.spawn_data.game_mode)?;
            }
        } else {
            let mut game_mode_id = self.spawn_data.game_mode;
            if self.is_hardcore {
                game_mode_id |= 0x08;
            }
            write.write_u8(game_mode_id)?;
        }

        // Previous GameMode & Worlds & Dimension Codec / Dimension Type
        if v1_16 {
            if !v1_20_2 {
                write.write_i8(self.spawn_data.previous_gamemode)?;
            }
            write.write_list(self.dimension_names, |write, dim| write.write_string(dim))?;
            if !v1_20_2 {
                let dimension = self.spawn_data.dimension.minecraft_name;
                write.write_all(registry.login_codec.unwrap_or_default())?;
                // 1.16.2 - 1.18.2 send the dimension type itself, the others its name
                if let Some(dimension_type) = registry.dimension_type {
                    write.write_all(dimension_type)?;
                } else {
                    write.write_string(dimension)?;
                }
                write.write_string(dimension)?;
            }
        } else {
            let legacy_dim_id: i32 = match self.spawn_data.dimension.minecraft_name {
                "minecraft:the_nether" => -1,
                "minecraft:the_end" => 1,
                _ => 0,
            };
            if *version >= JavaMinecraftVersion::V_1_9_1 {
                write.write_i32_be(legacy_dim_id)?;
            } else {
                write.write_i8(legacy_dim_id as i8)?;
            }
            if !v1_14 {
                // Difficulty (0: peaceful, 1: easy, 2: normal, 3: hard) - default 2 (Normal)
                write.write_u8(2)?;
            }
        }

        // Hashed Seed (Added in 1.15, moved in 1.20.2)
        if v1_15 && !v1_20_2 {
            write.write_i64_be(self.spawn_data.hashed_seed)?;
        }

        // Max Players, View Distance, etc.
        if v1_16 {
            if v1_16_2 {
                write.write_var_int(&self.max_players)?;
            } else {
                write.write_u8(self.max_players.0 as u8)?;
            }
            write.write_var_int(&self.view_distance)?;
            if v1_18 {
                write.write_var_int(&self.simulated_distance)?;
            }
            write.write_bool(self.reduced_debug_info)?;
            write.write_bool(self.enabled_respawn_screen)?;
            if v1_20_2 {
                write.write_bool(self.limited_crafting)?;
                if v1_20_5 {
                    write.write_var_int(&VarInt(self.spawn_data.dimension.id as i32))?;
                } else {
                    write.write_string(self.spawn_data.dimension.minecraft_name)?;
                }
                write.write_string(self.spawn_data.dimension.minecraft_name)?;
                write.write_i64_be(self.spawn_data.hashed_seed)?;
                write_game_modes(
                    &mut write,
                    self.spawn_data.game_mode,
                    self.spawn_data.previous_gamemode,
                    *version,
                )?;
            }
            write.write_bool(self.spawn_data.debug)?;
            write.write_bool(self.spawn_data.is_flat)?;
        } else {
            write.write_u8(self.max_players.0 as u8)?;
            let level_type = if self.spawn_data.is_flat {
                "flat"
            } else if self.spawn_data.debug {
                "debug_all_block_states"
            } else {
                "default"
            };
            write.write_string(level_type)?;
            if v1_14 {
                write.write_var_int(&self.view_distance)?;
            }
            if v1_8 {
                write.write_bool(self.reduced_debug_info)?;
            }
            if v1_15 {
                write.write_bool(self.enabled_respawn_screen)?;
            }
        }

        // Last Death Position (Added in 1.19)
        if v1_19 {
            write.write_option(
                &self.spawn_data.death_dimension_name,
                |write, (dim, pos)| {
                    write.write_string(dim)?;
                    write.write_block_pos_legacy(pos, version)?;
                    Ok(())
                },
            )?;
        }

        // Portal Cooldown (Added in 1.20)
        if *version >= JavaMinecraftVersion::V_1_20 {
            write.write_var_int(&self.spawn_data.portal_cooldown)?;
        }

        // Sea Level (Added in 1.21.2)
        if v1_21_2 {
            write.write_var_int(&self.spawn_data.sealevel)?;
        }

        // Online Mode (Added in 26.2)
        if v1_26_2 {
            write.write_bool(self.online_mode)?;
        }

        // Enforces Secure Chat (Added in 1.20.5)
        if v1_20_5 {
            write.write_bool(self.enforce_secure_chat)?;
        }

        Ok(())
    }
}
