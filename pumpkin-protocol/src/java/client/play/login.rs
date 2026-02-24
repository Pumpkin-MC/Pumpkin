use pumpkin_data::{dimension::Dimension, packet::clientbound::PLAY_LOGIN};
use pumpkin_util::{
    math::position::BlockPos, resource_location::ResourceLocation, version::MinecraftVersion,
};

use pumpkin_macros::java_packet;

use crate::{
    ClientPacket, VarInt,
    ser::{NetworkWriteExt, WritingError},
};

/// The "Join Game" packet that transitions the client from the Configuration state
/// to the Play state.
///
/// This is one of the largest and most important packets in the protocol. It
/// initializes the player's world view, dimension settings, and local game
/// rules. Once received, the client begins rendering the world.
#[java_packet(PLAY_LOGIN)]
pub struct CLogin {
    /// The unique ID assigned to the player for the current session.
    pub entity_id: i32,
    pub is_hardcore: bool,
    /// A list of all dimensions present on the server (e.g., overworld, nether, end).
    pub dimension_names: Vec<ResourceLocation>,
    pub max_players: VarInt,
    /// The number of chunks the client will render in each direction.
    pub view_distance: VarInt,
    /// The distance at which entities and world ticks are processed.
    pub simulated_distance: VarInt,
    /// If true, hides coordinates and other info from the F3 screen.
    pub reduced_debug_info: bool,
    pub enabled_respawn_screen: bool,
    pub limited_crafting: bool,
    // Spawn info
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
    /// If true, the client will warn the player if they send unsigned chat messages.
    pub enforce_secure_chat: bool,
}

impl CLogin {
    #[expect(clippy::too_many_arguments)]
    #[expect(clippy::fn_params_excessive_bools)]
    #[must_use]
    pub const fn new(
        entity_id: i32,
        is_hardcore: bool,
        dimension_names: Vec<ResourceLocation>,
        max_players: VarInt,
        view_distance: VarInt,
        simulated_distance: VarInt,
        reduced_debug_info: bool,
        enabled_respawn_screen: bool,
        limited_crafting: bool,
        dimension: Dimension,
        hashed_seed: i64,
        game_mode: u8,
        previous_gamemode: i8,
        debug: bool,
        is_flat: bool,
        death_dimension_name: Option<(ResourceLocation, BlockPos)>,
        portal_cooldown: VarInt,
        sealevel: VarInt,
        enforce_secure_chat: bool,
    ) -> Self {
        Self {
            entity_id,
            is_hardcore,
            dimension_names,
            max_players,
            view_distance,
            simulated_distance,
            reduced_debug_info,
            enabled_respawn_screen,
            limited_crafting,
            dimension,
            hashed_seed,
            game_mode,
            previous_gamemode,
            debug,
            is_flat,
            death_dimension_name,
            portal_cooldown,
            sealevel,
            enforce_secure_chat,
        }
    }
}

impl ClientPacket for CLogin {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_i32_be(self.entity_id)?;
        write.write_bool(self.is_hardcore)?;
        write.write_list(&self.dimension_names, |write, dim| write.write_string(dim))?;
        write.write_var_int(&self.max_players)?;
        write.write_var_int(&self.view_distance)?;
        write.write_var_int(&self.simulated_distance)?;
        write.write_bool(self.reduced_debug_info)?;
        write.write_bool(self.enabled_respawn_screen)?;
        write.write_bool(self.limited_crafting)?;
        if version >= &MinecraftVersion::V_1_21_2 {
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
        if version >= &MinecraftVersion::V_1_21_2 {
            write.write_var_int(&self.sealevel)?;
        }
        write.write_bool(self.enforce_secure_chat)?;
        Ok(())
    }
}
