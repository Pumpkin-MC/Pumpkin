use pumpkin_data::packet::clientbound::play::LOGIN;
use pumpkin_util::{resource_location::ResourceLocation, version::JavaMinecraftVersion};

use pumpkin_macros::java_packet;

use crate::{
    ClientPacket, VarInt,
    java::client::play::player_spawn_data::{PlayerSpawnData, write_game_modes},
    ser::{NetworkWriteExt, WritingError},
};

/// The "Join Game" packet that transitions the client from the Configuration state
/// to the Play state.
///
/// This is one of the largest and most important packets in the protocol. It
/// initializes the player's world view, dimension settings, and local game
/// rules. Once received, the client begins rendering the world.
#[java_packet(LOGIN)]
pub struct CLogin<'a> {
    /// The unique ID assigned to the player for the current session.
    pub entity_id: i32,
    pub is_hardcore: bool,
    /// A list of all dimensions present on the server (Added in 1.16).
    pub dimension_names: &'a [ResourceLocation],
    pub max_players: VarInt,
    /// The number of chunks the client will render in each direction (Added in 1.14).
    pub view_distance: VarInt,
    /// The distance at which entities and world ticks are processed (Added in 1.18).
    pub simulated_distance: VarInt,
    /// If true, hides coordinates and other info from the F3 screen (Added in 1.8).
    pub reduced_debug_info: bool,
    /// Added in 1.15.
    pub enabled_respawn_screen: bool,
    /// Added in 1.19.3.
    pub limited_crafting: bool,
    // Spawn info
    pub spawn_data: PlayerSpawnData,
    /// Added in 26.2.
    pub online_mode: bool,
    /// If true, the client will warn the player if they send unsigned chat messages (Added in 1.20.5).
    pub enforce_secure_chat: bool,
}

impl<'a> CLogin<'a> {
    #[expect(clippy::too_many_arguments)]
    #[expect(clippy::fn_params_excessive_bools)]
    #[must_use]
    pub const fn new(
        entity_id: i32,
        is_hardcore: bool,
        dimension_names: &'a [ResourceLocation],
        max_players: VarInt,
        view_distance: VarInt,
        simulated_distance: VarInt,
        reduced_debug_info: bool,
        enabled_respawn_screen: bool,
        limited_crafting: bool,
        spawn_data: PlayerSpawnData,
        online_mode: bool,
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
            spawn_data,
            online_mode,
            enforce_secure_chat,
        }
    }
}

impl ClientPacket for CLogin<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_i32_be(self.entity_id)?;

        // Hardcore & GameMode
        write.write_bool(self.is_hardcore)?;

        // Previous GameMode & Worlds & Dimension Codec / Dimension Type

        write.write_list(self.dimension_names, |write, dim| write.write_string(dim))?;

        // Hashed Seed (Added in 1.15, moved in 1.20.2)

        // Max Players, View Distance, etc.
        write.write_var_int(&self.max_players)?;
        write.write_var_int(&self.view_distance)?;
        write.write_var_int(&self.simulated_distance)?;
        write.write_bool(self.reduced_debug_info)?;
        write.write_bool(self.enabled_respawn_screen)?;
        write.write_bool(self.limited_crafting)?;
        write.write_var_int(&VarInt(self.spawn_data.dimension.id as i32))?;
        write.write_string(self.spawn_data.dimension.minecraft_name)?;
        write.write_i64_be(self.spawn_data.hashed_seed)?;
        write_game_modes(
            &mut write,
            self.spawn_data.game_mode,
            self.spawn_data.previous_gamemode,
        )?;
        write.write_bool(self.spawn_data.debug)?;
        write.write_bool(self.spawn_data.is_flat)?;

        // Last Death Position (Added in 1.19)
        write.write_option(
            &self.spawn_data.death_dimension_name,
            |write, (dim, pos)| {
                write.write_string(dim)?;
                write.write_block_pos(pos)?;
                Ok(())
            },
        )?;

        // Portal Cooldown (Added in 1.20)
        write.write_var_int(&self.spawn_data.portal_cooldown)?;

        // Sea Level (Added in 1.21.2)
        write.write_var_int(&self.spawn_data.sealevel)?;

        // Online Mode (Added in 26.2)
        write.write_bool(self.online_mode)?;

        // Enforces Secure Chat (Added in 1.20.5)
        write.write_bool(self.enforce_secure_chat)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
