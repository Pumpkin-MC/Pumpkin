use pumpkin_macros::packet;
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};
use serde::Serialize;

use crate::codec::{var_int::VarInt, var_long::VarLong};

#[derive(Serialize)]
#[packet(11)]
pub struct CStartGame {
    pub entity_id: VarLong,
    pub runtime_entity_id: VarLong,
    pub game_type_index: i32,
    pub position: Vector3<f32>,
    pub rotation: Vector2<f32>,

    // Level Settings
    pub seed: VarInt,
    // TODO: LE
    pub spawn_biome_type: u16,
    pub custom_biome_name: String,
    pub dimension_id: VarInt,
    pub generator_id: VarInt,
    pub level_game_type: VarInt,
    pub difficulty: VarInt,
    pub default_spawn: Vector3<i32>,
    pub achievements_disabled: bool,
    pub day_cycle_stop_time: VarInt,
    pub education_world: bool,
    pub education_features_enabled: bool,
    // TODO: LE
    pub rain_level: f32,
    // TODO: LE
    pub lightning_level: f32,
    pub multiplayer_game: bool,
    pub broadcast_to_lan: bool,
    pub xbl_broadcasting_to_lan: bool,
    pub commands_enabled: bool,
    pub texture_packs_required: bool,
    // Maybe VarInt ?
    pub game_rule_size: i32,
    pub bonus_chest_enabled: bool,
    pub starting_with_map: bool,
    pub trusting_players: bool,
    pub default_player_permission: VarInt,
    pub xbl_broadcast_mode: VarInt,
    // TODO: LE
    pub server_chunk_tick_range: i32,
    pub behavior_pack_locked: bool,
    pub resource_pack_locked: bool,
    pub from_locked_world_template: bool,
    pub using_msa_gamertags_only: bool,
    pub from_world_template: bool,
    pub world_template_option_locked: bool,
    pub only_spawning_v1_villagers: bool,
    pub disabling_personas: bool,
    pub disabling_custom_skins: bool,
    pub emote_chat_muted: bool,
    pub vanilla_version: String,
    // TODO: LE
    pub limited_world_width: i32,
    pub limited_world_height: i32,
    pub is_nether_type: bool,
    pub edu_button_name: String,
    pub edu_link_uri: String,
    pub force_experimental_gameplay: Option<bool>,
    pub chat_restriction_level: i8,
    pub disabling_player_interactions: bool,
    pub server_id: String,
    pub world_id: String,
    pub scenario_id: String,
    pub owner_id: String,

    pub level_id: String,
    pub level_name: String,
    pub premium_world_template_id: String,
    pub is_trial: bool,

    pub rewind_history_size: i32,
    pub server_authoritative_block_breaking: bool,

    pub current_tick: u16,
    pub enchantment_seed: i32,
    pub block_properties_size: u32,
    // TODO
    pub multiplayer_correlation_id: String,
}
