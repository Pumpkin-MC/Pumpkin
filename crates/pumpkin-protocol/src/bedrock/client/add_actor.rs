// Last verified for v2169

use crate::{
    codec::{var_long::VarLong, var_ulong::VarULong},
    serial::PacketWrite,
};
use pumpkin_macros::packet;
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

use super::{
    common::ActorLink,
    set_actor_data::{PropertySyncData, SyncedActorDataList},
};

#[derive(PacketWrite)]
#[packet(13)]
pub struct CAddActor {
    pub target_actor_id: VarLong,
    pub target_runtime_id: VarULong,
    pub actor_type: String,
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub rotation: Vector2<f32>,
    pub y_head_rotation: f32,
    pub y_body_rotation: f32,
    pub attributes_list: Vec<SyncedAttribute>,
    pub actor_data: SyncedActorDataList,
    pub synced_properties: PropertySyncData,
    pub actor_links: Vec<ActorLink>,
}

#[derive(PacketWrite)]
pub struct SyncedAttribute {
    pub attribute_name: String,
    pub min_value: f32,
    pub current_value: f32,
    pub max_value: f32,
}
