use crate::{
    bedrock::{client::GameType, network_item::NetworkItemStackDescriptor},
    codec::var_ulong::VarULong,
    serial::PacketWrite,
};
use pumpkin_macros::packet;
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};
use uuid::Uuid;

use super::{
    common::{ActorLink, BuildPlatform, SerializedAbilitiesData},
    set_actor_data::PropertySyncData,
    set_actor_data::SyncedActorDataList,
};

/// Sent by the server to spawn another player client-side into the world.
#[derive(PacketWrite)]
#[packet(12)]
pub struct CAddPlayer {
    /// UUID of the player being spawned.
    pub uuid: Uuid,
    /// Player display username.
    pub player_name: String,
    /// Session-unique runtime entity ID for the player.
    pub target_runtime_id: VarULong,
    /// Platform-specific chat identifier string.
    pub platform_chat_id: String,
    /// Spawn coordinates in world space.
    pub position: Vector3<f32>,
    /// Initial movement velocity.
    pub velocity: Vector3<f32>,
    /// Pitch (x) and yaw (y) rotation angles.
    pub rotation: Vector2<f32>,
    /// Horizontal head yaw rotation angle.
    pub y_head_rotation: f32,

    // TODO: update inventory
    /// Item stack currently held in the main hand.
    pub carried_item: NetworkItemStackDescriptor,

    /// Game mode of the player.
    pub player_game_type: GameType,
    /// Entity metadata flags and properties.
    pub entity_data: SyncedActorDataList,
    /// Dynamic Molang property synchronization.
    pub synced_properties: PropertySyncData,
    /// Permissions and ability layers for this player.
    pub abilities_data: SerializedAbilitiesData,
    /// Vehicle and attachment links.
    pub actor_links: Vec<ActorLink>,
    /// Unique device ID of the client device.
    pub device_id: String,
    /// Operating system platform of the player.
    pub build_platform: BuildPlatform,
}
