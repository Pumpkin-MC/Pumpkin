//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/player_spawn_data.rs`.

use crate::{
    VarInt,
    java::client::play::PlayerSpawnData,
    ser::{NetworkWriteExt, WritingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

use crate::java::legacy::{LegacyWrite, LegacyWriteExt};

/// Game modes as var ints since 26.3, a byte and a signed byte before.
///
/// Writes the current and previous game mode. Since 26.3 both are var ints and the previous game
/// mode is optional, where 0 means none and any other value is the game mode id plus one.
pub fn write_game_modes(
    mut write: impl std::io::Write,
    game_mode: u8,
    previous_gamemode: i8,
    version: JavaMinecraftVersion,
) -> Result<(), WritingError> {
    if version >= JavaMinecraftVersion::V_26_3 {
        write.write_var_int(&VarInt(i32::from(game_mode)))?;
        let previous = if previous_gamemode < 0 {
            0
        } else {
            i32::from(previous_gamemode) + 1
        };
        write.write_var_int(&VarInt(previous))
    } else {
        write.write_u8(game_mode)?;
        write.write_i8(previous_gamemode)
    }
}

impl LegacyWrite for PlayerSpawnData {
    fn write_legacy(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if version >= &JavaMinecraftVersion::V_1_20_5 {
            write.write_var_int(&VarInt(self.dimension.id as i32))?;
        } else if version >= &JavaMinecraftVersion::V_1_16 {
            write.write_string(self.dimension.minecraft_name)?;
        } else if version >= &JavaMinecraftVersion::V_1_9 {
            write.write_i32_be(self.dimension.id as i32)?;
        } else {
            write.write_i8(self.dimension.id as i8)?;
        }
        write.write_string(self.dimension.minecraft_name)?;
        write.write_i64_be(self.hashed_seed)?;
        write_game_modes(&mut write, self.game_mode, self.previous_gamemode, *version)?;
        write.write_bool(self.debug)?;
        write.write_bool(self.is_flat)?;
        if version >= &JavaMinecraftVersion::V_1_19 {
            write.write_option(&self.death_dimension_name, |write, (dim, pos)| {
                write.write_string(dim)?;
                write.write_block_pos_legacy(pos, version)?;
                Ok(())
            })?;
        }
        if version >= &JavaMinecraftVersion::V_1_20 {
            write.write_var_int(&self.portal_cooldown)?;
        }
        if version >= &JavaMinecraftVersion::V_1_21_2 {
            write.write_var_int(&self.sealevel)?;
        }
        Ok(())
    }
}
