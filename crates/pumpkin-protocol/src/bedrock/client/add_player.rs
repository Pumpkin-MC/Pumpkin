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

#[derive(PacketWrite)]
#[packet(12)]
pub struct CAddPlayer {
    pub uuid: Uuid,
    pub player_name: String,
    pub target_runtime_id: VarULong,
    pub platform_chat_id: String,
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub rotation: Vector2<f32>,
    pub y_head_rotation: f32,

    // TODO: update inventory
    pub carried_item: NetworkItemStackDescriptor,

    pub player_game_type: GameType,
    pub entity_data: SyncedActorDataList,
    pub synced_properties: PropertySyncData,
    pub abilities_data: SerializedAbilitiesData,
    pub actor_links: Vec<ActorLink>,
    pub device_id: String,
    pub build_platform: BuildPlatform,
}
