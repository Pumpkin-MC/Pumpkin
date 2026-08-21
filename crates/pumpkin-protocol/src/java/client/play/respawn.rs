use pumpkin_data::packet::clientbound::play::RESPAWN;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ClientPacket,
    java::client::play::player_spawn_data::PlayerSpawnData,
    ser::{NetworkWriteExt, WritingError},
};

#[java_packet(RESPAWN)]
pub struct CRespawn {
    pub player_spawn_info: PlayerSpawnData,
    pub data_kept: u8,
}

impl CRespawn {
    pub const KEEP_ATTRIBUTE_MODIFIERS: u8 = 1;
    pub const KEEP_ENTITY_DATA: u8 = 2;
    pub const KEEP_ALL_DATA: u8 = 3;

    #[must_use]
    pub const fn new(player_spawn_info: PlayerSpawnData, data_kept: u8) -> Self {
        Self {
            player_spawn_info,
            data_kept,
        }
    }
}

impl ClientPacket for CRespawn {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if version < &JavaMinecraftVersion::V_1_20_2 {
            if *version >= JavaMinecraftVersion::V_1_16 {
                if *version >= JavaMinecraftVersion::V_1_16_2
                    && *version < JavaMinecraftVersion::V_1_19
                {
                    let dim_type_compound =
                        crate::java::client::play::login::get_dimension_type_nbt(
                            *version,
                            self.player_spawn_info.dimension.minecraft_name,
                        );
                    let dim_bytes = pumpkin_nbt::Nbt::new(String::new(), dim_type_compound).write();
                    write.write_all(&dim_bytes)?;
                } else {
                    write.write_string(self.player_spawn_info.dimension.minecraft_name)?;
                }
                write.write_string(self.player_spawn_info.dimension.minecraft_name)?;
                write.write_i64_be(self.player_spawn_info.hashed_seed)?;
                write.write_u8(self.player_spawn_info.game_mode)?;
                write.write_i8(self.player_spawn_info.previous_gamemode)?;
                write.write_bool(self.player_spawn_info.debug)?;
                write.write_bool(self.player_spawn_info.is_flat)?;
                write.write_bool(self.data_kept != 0)?;
                if *version >= JavaMinecraftVersion::V_1_19 {
                    write.write_option(
                        &self.player_spawn_info.death_dimension_name,
                        |write, (dim, pos)| {
                            write.write_string(dim)?;
                            write.write_block_pos(pos)?;
                            Ok(())
                        },
                    )?;
                }
                if *version >= JavaMinecraftVersion::V_1_20 {
                    write.write_var_int(&self.player_spawn_info.portal_cooldown)?;
                }
            } else {
                let legacy_dim_id: i32 = match self.player_spawn_info.dimension.minecraft_name {
                    "minecraft:the_nether" => -1,
                    "minecraft:the_end" => 1,
                    _ => 0,
                };
                if *version >= JavaMinecraftVersion::V_1_9_1 {
                    write.write_i32_be(legacy_dim_id)?;
                } else {
                    write.write_i8(legacy_dim_id as i8)?;
                }
                if *version < JavaMinecraftVersion::V_1_14 {
                    // Difficulty
                    write.write_u8(2)?;
                }
                if *version >= JavaMinecraftVersion::V_1_15 {
                    write.write_i64_be(self.player_spawn_info.hashed_seed)?;
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
            }
            return Ok(());
        }

        self.player_spawn_info
            .write_packet_data(&mut write, version)?;
        write.write_i8(self.data_kept as i8)?;
        Ok(())
    }
}
