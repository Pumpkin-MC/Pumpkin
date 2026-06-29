use pumpkin_data::dimension::Dimension;
use pumpkin_util::{
    math::position::BlockPos, resource_location::ResourceLocation, version::JavaMinecraftVersion,
};

use crate::{
    codec::var_int::VarInt,
    ser::{NetworkWriteExt, WritingError},
};

pub struct PlayerSpawnData {
    /// The Dimension for the current dimension's properties (lighting, sky color).
    pub dimension: Dimension,
    /// Used by the client to seed local biome noise and decoration algorithms.
    pub hashed_seed: i64,
    pub game_mode: u8,
    /// The previous gamemode (used for the F3+F4 toggle UI). -1 if none.
    pub previous_gamemode: i8,
    /// If true, the world is a debug world (all blocks shown in a grid).
    pub debug: bool,
    /// If true, the world is a flat world (affects the horizon rendering).
    pub is_flat: bool,
    /// The location where the player last died (used for the recovery compass).
    pub death_dimension_name: Option<(ResourceLocation, BlockPos)>,
    pub portal_cooldown: VarInt,
    /// The height of the ocean level (usually 63).
    pub sealevel: VarInt,
}

impl PlayerSpawnData {
    #[expect(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        dimension: Dimension,
        hashed_seed: i64,
        game_mode: u8,
        previous_gamemode: i8,
        debug: bool,
        is_flat: bool,
        death_dimension_name: Option<(ResourceLocation, BlockPos)>,
        portal_cooldown: VarInt,
        sealevel: VarInt,
    ) -> Self {
        Self {
            dimension,
            hashed_seed,
            game_mode,
            previous_gamemode,
            debug,
            is_flat,
            death_dimension_name,
            portal_cooldown,
            sealevel,
        }
    }

    pub fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if version >= &JavaMinecraftVersion::V_1_21_2 {
            write.write_var_int(&VarInt(self.dimension.id as i32))?;
        } else {
            write.write_string("")?;
        }
        write.write_string(self.dimension.minecraft_name)?;
        write.write_i64_be(self.hashed_seed)?;
        write.write_u8(self.game_mode)?;
        write.write_i8(self.previous_gamemode)?;
        write.write_bool(self.debug)?;
        write.write_bool(self.is_flat)?;
        write.write_option(&self.death_dimension_name, |write, (dim, pos)| {
            write.write_string(dim)?;
            write.write_block_pos(pos)?;
            Ok(())
        })?;
        write.write_var_int(&self.portal_cooldown)?;
        if version >= &JavaMinecraftVersion::V_1_21_2 {
            write.write_var_int(&self.sealevel)?;
        }
        Ok(())
    }
}
