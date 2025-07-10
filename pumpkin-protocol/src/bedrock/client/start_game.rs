use crate::{
    bedrock::client::gamerules_changed::GameRules,
    codec::{
        bedrock_block_pos::BedrockPos, little_endian::Le32, var_int::VarInt, var_long::VarLong,
        var_uint::VarUInt, var_ulong::VarULong,
    },
};
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
#[packet(11)]
pub struct CStartGame {
    // https://mojang.github.io/bedrock-protocol-docs/html/StartGamePacket.html
    pub entity_id: VarLong,
    pub runtime_entity_id: VarULong,
    pub player_gamemode: VarInt,
    pub position: Vector3<f32>,
    pub pitch: f32,
    pub yaw: f32,
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

    pub multiplayer_correlation_id: String,
    pub enable_itemstack_net_manager: bool,
    pub server_version: String,

    //pub player_property_data: NbtCompound
    pub compound_id: i8,
    pub compound_len: VarUInt,
    pub compound_end: i8,

    pub block_registry_checksum: u64,
    pub world_template_id: Uuid,

    pub enable_clientside_generation: bool,
    pub blocknetwork_ids_are_hashed: bool,
    pub server_auth_sounds: bool,
}

#[derive(Serialize)]
pub struct LevelSettings {
    // https://mojang.github.io/bedrock-protocol-docs/html/LevelSettings.html
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
    pub education_edition_offer: VarInt,
    pub has_education_features_enabled: bool,
    pub education_product_id: String,
    pub rain_level: f32,
    pub lightning_level: f32,
    pub has_confirmed_platform_locked_content: bool,
    pub was_multiplayer_intended: bool,
    pub was_lan_broadcasting_intended: bool,
    pub xbox_live_broadcast_setting: VarInt,
    pub platform_broadcast_setting: VarInt,
    pub commands_enabled: bool,
    pub is_texture_packs_required: bool,

    pub rule_data: GameRules,
    pub experiments: Experiments,

    pub bonus_chest: bool,
    pub has_start_with_map_enabled: bool,
    pub permission_level: VarInt,
    pub server_chunk_tick_range: Le32,
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
    pub new_nether: bool,
    pub edu_shared_uri_button_name: String,
    pub edu_shared_uri_link_uri: String,
    pub override_force_experimental_gameplay_has_value: bool,
    pub chat_restriction_level: i8,
    pub disable_player_interactions: bool,
    pub server_id: String,
    pub world_id: String,
    pub scenario_id: String,
    pub owner_id: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Experiments {
    //TODO! https://mojang.github.io/bedrock-protocol-docs/html/Experiments.html
    pub names_size: u32,
    pub experiments_ever_toggled: bool,
}

#[repr(i32)]
pub enum GamePublishSetting {
    NoMultiPlay = 0,
    InviteOnly = 1,
    FriendsOnly = 2,
    FriendsOfFriends = 3,
    Public = 4,
}
