use std::io::{Error, Write};

use crate::{
    bedrock::client::{GameType, gamerules_changed::GameRule},
    codec::{var_int::VarInt, var_long::VarLong, var_uint::VarUInt, var_ulong::VarULong},
    serial::PacketWrite,
};
use pumpkin_macros::packet;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use uuid::Uuid;

/// Sent by the server to initialize the client's world, spawn coordinates, gamerules, and entity runtime ID.
#[derive(PacketWrite)]
#[packet(11)]
pub struct CStartGame {
    /// Persistent unique ID of the player entity across sessions.
    pub entity_id: VarLong,
    /// Runtime entity ID uniquely identifying the player during the current session.
    pub runtime_entity_id: VarULong,
    /// Initial game mode assigned to the player.
    pub player_gamemode: GameType,
    /// World coordinates where the player spawns.
    pub position: Vector3<f32>,
    /// Initial pitch rotation angle.
    pub pitch: f32,
    /// Initial yaw rotation angle.
    pub yaw: f32,
    /// World and dimension settings (generator, seed, difficulty, gamerules).
    pub level_settings: LevelSettings,

    /// Unique level identifier string.
    pub level_id: String,
    /// Display name of the world shown in menus.
    pub level_name: String,
    /// Template ID for marketplace worlds.
    pub premium_world_template_id: String,
    /// True if the world is running in trial mode.
    pub is_trial: bool,

    /// Number of movement ticks kept in rewind history for server authoritative movement.
    pub rewind_history_size: VarInt,
    /// True if block breaking progress is checked and controlled by the server.
    pub server_authoritative_block_breaking: bool,

    /// Current in-game world time in ticks.
    pub current_level_time: u64,
    /// Player's enchanting seed for deterministic enchanting tables.
    pub enchantment_seed: VarInt,
    /// Custom block property definitions registered on the server.
    pub block_properties: Vec<BlockProperty>,

    /// Correlation ID string used for telemetry and matchmaking.
    pub multiplayer_correlation_id: String,
    /// Whether the modern item stack network manager is enabled.
    pub enable_itemstack_net_manager: bool,
    /// Version string of the server software.
    pub server_version: String,

    //pub player_property_data: NbtCompound
    pub compound_id: i8,
    pub compound_len: VarUInt,
    pub compound_end: i8,

    /// CRC64 checksum of the block state registry.
    pub block_registry_checksum: u64,
    /// UUID of the world template if created from a template.
    pub world_template_id: Uuid,

    /// Whether client-side chunk generation is enabled.
    pub enable_clientside_generation: bool,
    /// Whether block runtime IDs are cryptographic hashes instead of sequential IDs.
    pub blocknetwork_ids_are_hashed: bool,
    /// Whether entity and environment sounds are validated by the server.
    pub server_auth_sounds: bool,

    // 2 Optionals is what we need Mojang :cap:
    /// Server connection information.
    pub server_join_information: Option<ServerJoinInformation>,
    /// Telemetry and session tracking identifiers.
    pub telemetry: ServerTelemetryData,
}

#[derive(PacketWrite)]
pub struct ServerJoinInformation {
    gathering: Option<GatheringJoinInfo>,
    store_entry_point: Option<StoreEntryPointInfo>,
    presence: Option<PresenceInfo>,
}

#[derive(PacketWrite)]
pub struct GatheringJoinInfo {
    experience_id: Uuid,
    experience_name: String,
    experience_world_id: Uuid,
    experience_world_name: String,
    creator_id: String,
    unknown_uuid_1: Uuid,
    unknown_uuid_2: Uuid,
    server_id: String,
}

#[derive(PacketWrite)]
pub struct StoreEntryPointInfo {
    store_id: String,
    store_name: String,
}

#[derive(PacketWrite)]
pub struct PresenceInfo {
    experience_name: String,
    world_name: String,
}

#[derive(PacketWrite)]
pub struct ServerTelemetryData {
    pub server_id: String,
    pub scenario_id: String,
    pub world_id: String,
    pub owner_id: String,
}

#[derive(PacketWrite)]
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
    pub world_gamemode: GameType,
    pub hardcore: bool,
    pub difficulty: VarInt,
    pub spawn_position: BlockPos,
    pub has_achievements_disabled: bool,
    pub editor_world_type: VarInt,
    pub is_created_in_editor: bool,
    pub is_exported_from_editor: bool,
    pub day_cycle_stop_time: VarInt,
    pub education_edition_offer: VarUInt,
    pub has_education_features_enabled: bool,
    pub education_product_id: String,
    pub rain_level: f32,
    pub lightning_level: f32,
    pub has_confirmed_platform_locked_content: bool,
    pub was_multiplayer_intended: bool,
    pub was_lan_broadcasting_intended: bool,
    pub xbox_live_broadcast_setting: GamePublishSetting,
    pub platform_broadcast_setting: GamePublishSetting,
    pub commands_enabled: bool,
    pub is_texture_packs_required: bool,

    pub rule_data: Vec<GameRule>,
    pub experiments: Experiments,

    pub bonus_chest: bool,
    pub has_start_with_map_enabled: bool,
    pub permission_level: u8,
    pub server_simulation_distance: i32,
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
    pub server_editor_connection_policy: VarInt,
    pub allow_anonymous_block_drops_in_editor_worlds: bool,
}

#[derive(Default)]
pub struct Experiments {
    pub toggles: Vec<ExperimentToggle>,
    pub experiments_ever_toggled: bool,
}

impl PacketWrite for Experiments {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (self.toggles.len() as u32).write(writer)?;
        for toggle in &self.toggles {
            toggle.write(writer)?;
        }
        self.experiments_ever_toggled.write(writer)?;

        Ok(())
    }
}

#[derive(PacketWrite)]
pub struct ExperimentToggle {
    pub name: String,
    pub enabled: bool,
}

#[derive(Clone, Copy, PacketWrite)]
#[repr(i32)]
#[serial(varint)]
pub enum GamePublishSetting {
    NoMultiPlay = 0,
    InviteOnly = 1,
    FriendsOnly = 2,
    FriendsOfFriends = 3,
    Public = 4,
}

/// An entry for a custom block registered on the server.
///
/// The runtime ID of these custom block entries is based on their alphabetic index
/// within the registered block palette.
#[derive(Clone, Debug, PartialEq, PacketWrite)]
pub struct BlockProperty {
    /// Name or identifier of the custom block (e.g. `namespace:block`).
    pub name: String,
    /// NBT compound containing properties defining the unique block state.
    pub properties: NbtCompound,
}

/// Alias for [`BlockProperty`] matching the vanilla Bedrock protocol name.
pub type BlockEntry = BlockProperty;

impl BlockProperty {
    /// Constructs a new custom block property entry.
    #[must_use]
    pub fn new(name: impl Into<String>, properties: NbtCompound) -> Self {
        Self {
            name: name.into(),
            properties,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_property_serializes_with_bedrock_nbt() {
        let prop = BlockProperty::new("custom:block", NbtCompound::new());
        let mut encoded = Vec::new();
        prop.write(&mut encoded).expect("serialize block property");

        // Length of "custom:block" is 12 (VarUInt: 12)
        assert_eq!(encoded[0], 12);
        assert_eq!(&encoded[1..13], b"custom:block");
        // Root compound tag ID (10) followed by empty string length and END tag (0)
        assert_eq!(&encoded[13..], &[10, 0, 0]);
    }
}
