use pumpkin_data::packet::clientbound::PLAY_LOGIN;
use pumpkin_util::{math::position::BlockPos, resource_location::ResourceLocation};

use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// The "Join Game" packet that transitions the client from the Configuration state
/// to the Play state.
///
/// This is one of the largest and most important packets in the protocol. It
/// initializes the player's world view, dimension settings, and local game
/// rules. Once received, the client begins rendering the world.
#[derive(Serialize)]
#[java_packet(PLAY_LOGIN)]
pub struct CLogin<'a> {
    /// The unique ID assigned to the player for the current session.
    pub entity_id: i32,
    pub is_hardcore: bool,
    /// A list of all dimensions present on the server (e.g., overworld, nether, end).
    pub dimension_names: &'a [ResourceLocation],
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
    /// The registry ID for the current dimension's properties (lighting, sky color).
    pub dimension_type: VarInt,
    /// The specific resource location of the current dimension.
    pub dimension_name: ResourceLocation,
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

impl<'a> CLogin<'a> {
    #[expect(clippy::too_many_arguments)]
    #[expect(clippy::fn_params_excessive_bools)]
    #[must_use]
    pub fn new(
        entity_id: i32,
        is_hardcore: bool,
        dimension_names: &'a [ResourceLocation],
        max_players: VarInt,
        view_distance: VarInt,
        simulated_distance: VarInt,
        reduced_debug_info: bool,
        enabled_respawn_screen: bool,
        limited_crafting: bool,
        dimension_type: VarInt,
        dimension_name: ResourceLocation,
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
            dimension_type,
            dimension_name,
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
