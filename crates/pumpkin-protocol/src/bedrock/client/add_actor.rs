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

/// Sent by the server to spawn non-player entities (mobs, projectiles, vehicles, items) on the client.
#[derive(PacketWrite)]
#[packet(13)]
pub struct CAddActor {
    /// Unique entity ID across sessions.
    pub target_actor_id: VarLong,
    /// Runtime entity ID for the current session.
    pub target_runtime_id: VarULong,
    /// Entity type identifier (e.g. `minecraft:zombie` or `minecraft:arrow`).
    pub actor_type: String,
    /// World spawn coordinates.
    pub position: Vector3<f32>,
    /// Initial movement velocity vector.
    pub velocity: Vector3<f32>,
    /// Pitch (x) and yaw (y) rotation angles.
    pub rotation: Vector2<f32>,
    /// Horizontal head yaw rotation angle.
    pub y_head_rotation: f32,
    /// Body yaw rotation angle.
    pub y_body_rotation: f32,
    /// Entity attributes (health, movement speed, follow range).
    pub attributes_list: Vec<SyncedAttribute>,
    /// Metadata dictionary (flags, name tags, scale).
    pub actor_data: SyncedActorDataList,
    /// Dynamic Molang property synchronization data.
    pub synced_properties: PropertySyncData,
    /// Entity attachment links (e.g. passenger or leash connections).
    pub actor_links: Vec<ActorLink>,
}

/// Synchronized numeric entity attribute with min, current, and max bounds.
#[derive(PacketWrite)]
pub struct SyncedAttribute {
    /// Identifier string for the attribute (e.g. `minecraft:health`).
    pub attribute_name: String,
    /// Minimum allowed attribute value.
    pub min_value: f32,
    /// Current active attribute value.
    pub current_value: f32,
    /// Maximum allowed attribute value.
    pub max_value: f32,
}
