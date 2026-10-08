// TODO: update inventory

use crate::{
    bedrock::network_item::ItemStackWrapper,
    codec::{var_long::VarLong, var_ulong::VarULong},
    serial::PacketWrite,
};
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use super::set_actor_data::SyncedActorDataList;

/// Sent by the server to spawn a dropped item entity in the world.
#[derive(PacketWrite)]
#[packet(15)]
pub struct CAddItemActor {
    /// Persistent unique entity ID of the dropped item entity.
    pub target_actor_id: VarLong,
    /// Runtime entity ID for the current session.
    pub target_runtime_id: VarULong,
    /// Item stack representation of the dropped item.
    pub item: ItemStackWrapper,
    /// World spawn coordinates.
    pub position: Vector3<f32>,
    /// Movement velocity vector.
    pub velocity: Vector3<f32>,
    /// Metadata dictionary for the item entity.
    pub entity_data: SyncedActorDataList,
    /// Whether the item originated from fishing.
    pub is_from_fishing: bool,
}
