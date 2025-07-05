use crate::{
    codec::{bedrock_block_pos::BedrockPos, var_ulong::VarULong},
    ser::network_serialize_no_prefix,
};
use pumpkin_macros::packet;
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    bedrock::client::gamerules_changed::GameRules,
    codec::{var_int::VarInt, var_long::VarLong, var_uint::VarUInt, var_ulong::VarULong},
};
use crate::codec::{var_int::VarInt, var_long::VarLong, var_uint::VarUInt};

pub const GAME_PUBLISH_SETTING_NO_MULTI_PLAY: i32 = 0;
pub const GAME_PUBLISH_SETTING_INVITE_ONLY: i32 = 1;
pub const GAME_PUBLISH_SETTING_FRIENDS_ONLY: i32 = 2;
pub const GAME_PUBLISH_SETTING_FRIENDS_OF_FRIENDS: i32 = 3;
pub const GAME_PUBLISH_SETTING_PUBLIC: i32 = 4;

#[derive(Serialize)]
#[packet(11)]
pub struct CStartGame {
    pub entity_id: VarLong,
    pub runtime_entity_id: VarULong,
    pub player_gamemode: VarInt,
    pub position: Vector3<f32>,
    pub rotation: Vector2<f32>,
    pub level_settings: LevelSettings,

    pub level_id: String,
    pub level_name: String,
    pub premium_world_template_id: String,
    pub is_trial: bool,

    pub rewind_history_size: VarInt,
    pub server_authoritative_block_breaking: bool,

    pub current_level_time: u64,
    pub enchantment_seed: VarInt,
    pub block_properties_size: VarUInt,

    pub enable_itemstack_net_manager: bool,
    pub multiplayer_correlation_id: String,
    pub server_version: String,

    //pub player_property_data: nbt commpound
    pub block_registry_checksum: u64,
    pub world_template_id: uuid::Uuid,

    pub enable_clientside_generation: bool,
    pub blocknetwork_ids_are_hashed: bool,
    pub server_auth_sounds: bool,
}

#[derive(Serialize)]
// https://mojang.github.io/bedrock-protocol-docs/html/LevelSettings.html
pub struct LevelSettings {
    pub seed: u64,

    // Spawn Settings
    // https://mojang.github.io/bedrock-protocol-docs/html/SpawnSettings.html
    pub spawn_biome_type: i16,
    pub custom_biome_name: String,
    pub dimension: VarInt,

    // Level Settings
    pub generator_type: VarInt,
    pub world_gamemode: VarInt,
    pub hardcore: bool,
    pub difficulty: VarInt,
    pub spawn_position: BedrockPos,
    pub has_achievements_disabled: bool,
    pub editor_world_type: VarInt,
    pub is_created_in_editor: bool,
    pub is_exported_from_editor: bool,
    pub day_cycle_stop_time: VarInt,
    pub edu_edition_offer: VarInt,
    pub has_edu_features_enabled: bool,
    pub education_edition_product_id: String,
    pub rain_level: f32,
    pub lightning_level: f32,
    pub has_confirmed_platform_locked_content: bool,
    pub multiplayer_game: bool,
    pub broadcast_to_lan: bool,
    pub xbl_broadcast_intent: VarInt,
    pub platform_broadcast_intent: VarInt,
    pub commands_enabled: bool,
    pub is_texture_packs_required: bool,

    pub rule_data: GameRules,
    pub experiments: Experiments,

    pub bonus_chest_enabled: bool,
    pub start_with_map_enabled: bool,
    pub player_permissions: VarInt,
    // TODO: LE
    pub experiments_len: u32,
    pub exeriments_ever_toggeld: bool,
    pub bonus_chest: bool,
    pub has_start_with_map_enabled: bool,
    pub permission_level: VarInt,
    pub server_chunk_tick_range: i32,
    pub has_locked_behavior_pack: bool,
    pub has_locked_resource_pack: bool,
    pub is_from_locked_world_template: bool,
    pub is_using_msa_gamertags_only: bool,
    pub is_from_world_template: bool,
    pub is_world_template_option_locked: bool,
    pub is_only_spawning_v1_villagers: bool,
    pub is_disabling_personas: bool,
    pub is_disabling_custom_skins: bool,
    pub emote_chat_muted: bool,
    // TODE BaseGameVersion
    pub game_version: String,
    // TODO: LE
    pub limited_world_width: i32,
    pub limited_world_height: i32,
    pub is_nether_type: bool,
    pub edu_shared_uri_button_name: String,
    pub edu_shared_uri_link_uri: String,
    pub override_force_experimental_gameplay_has_value: bool,
    pub chat_restriction_level: i8,
    pub disable_player_interactions: bool,
    pub disable_player_interactions: bool,
    pub server_id: String,
    pub world_id: String,
    pub scenario_id: String,
    pub owner_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Experiments {
    pub names_size: u32,
    //TODO! https://mojang.github.io/bedrock-protocol-docs/html/Experiments.html
    pub experiments_ever_toggled: bool,
}

impl Default for Experiments {
    fn default() -> Self {
        Self {
            names_size: 0,
            experiments_ever_toggled: false,
        }
    }
}
