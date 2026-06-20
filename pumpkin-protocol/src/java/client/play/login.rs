use pumpkin_data::packet::clientbound::PLAY_LOGIN;
use pumpkin_util::{resource_location::ResourceLocation, version::JavaMinecraftVersion};

use pumpkin_macros::java_packet;

use crate::{
    ClientPacket, VarInt,
    java::client::play::player_spawn_data::PlayerSpawnData,
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
    pub spawn_data: PlayerSpawnData,
    pub online_mode: bool,
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

impl ClientPacket for CLogin {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
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
        self.spawn_data.write_packet_data(&mut write, version)?;
        if version >= &JavaMinecraftVersion::V_26_2 {
            write.write_bool(self.online_mode)?;
        }
        write.write_bool(self.enforce_secure_chat)?;
        Ok(())
    }
}
