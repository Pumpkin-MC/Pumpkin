use pumpkin_data::dimension::Dimension;
use pumpkin_util::{
    math::position::BlockPos, resource_location::ResourceLocation, version::JavaMinecraftVersion,
};

use crate::{
    codec::var_int::VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};

#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// The location where the player last died (Added in 1.19, used for the recovery compass).
    pub death_dimension_name: Option<(ResourceLocation, BlockPos)>,
    /// Added in 1.20.
    pub portal_cooldown: VarInt,
    /// The height of the ocean level, usually 63 (Added in 1.21.2).
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
        write.write_u8(self.game_mode)?;
        write.write_i8(self.previous_gamemode)?;
        write.write_bool(self.debug)?;
        write.write_bool(self.is_flat)?;
        if version >= &JavaMinecraftVersion::V_1_19 {
            write.write_option(&self.death_dimension_name, |write, (dim, pos)| {
                write.write_string(dim)?;
                write.write_block_pos(pos, version)?;
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

    pub fn read(read: &mut &[u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let dimension = if version >= &JavaMinecraftVersion::V_1_20_5 {
            let id = read.get_var_int()?.0 as u8;
            match id {
                1 => Dimension::OVERWORLD_CAVES,
                2 => Dimension::THE_END,
                3 => Dimension::THE_NETHER,
                _ => Dimension::OVERWORLD,
            }
        } else if version >= &JavaMinecraftVersion::V_1_16 {
            let dim_name = read.get_str()?;
            Dimension::from_name(&dim_name)
                .cloned()
                .unwrap_or(Dimension::OVERWORLD)
        } else if version >= &JavaMinecraftVersion::V_1_9 {
            let legacy_id = read.get_i32_be()?;
            match legacy_id {
                -1 => Dimension::THE_NETHER,
                1 => Dimension::THE_END,
                _ => Dimension::OVERWORLD,
            }
        } else {
            let legacy_id = read.get_i8()?;
            match legacy_id {
                -1 => Dimension::THE_NETHER,
                1 => Dimension::THE_END,
                _ => Dimension::OVERWORLD,
            }
        };

        let _world_name = read.get_str()?;
        let hashed_seed = read.get_i64_be()?;
        let game_mode = read.get_u8()?;
        let previous_gamemode = read.get_i8()?;
        let debug = read.get_bool()?;
        let is_flat = read.get_bool()?;

        let death_dimension_name = if version >= &JavaMinecraftVersion::V_1_19 {
            if read.get_bool()? {
                let dim = read.get_str()?.into();
                let pos = read.get_block_pos(version)?;
                Some((dim, pos))
            } else {
                None
            }
        } else {
            None
        };

        let portal_cooldown = if version >= &JavaMinecraftVersion::V_1_20 {
            read.get_var_int()?
        } else {
            VarInt(0)
        };

        let sealevel = if version >= &JavaMinecraftVersion::V_1_21_2 {
            read.get_var_int()?
        } else {
            VarInt(63)
        };

        Ok(Self {
            dimension,
            hashed_seed,
            game_mode,
            previous_gamemode,
            debug,
            is_flat,
            death_dimension_name,
            portal_cooldown,
            sealevel,
        })
    }
}
