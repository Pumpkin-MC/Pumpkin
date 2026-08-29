// TODO: update inventory

use crate::{
    bedrock::network_item::ItemStackWrapper,
    codec::{var_long::VarLong, var_ulong::VarULong},
    serial::PacketWrite,
};
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use super::set_actor_data::SyncedActorDataList;

#[derive(PacketWrite)]
#[packet(15)]
pub struct CAddItemActor {
    pub target_actor_id: VarLong,
    pub target_runtime_id: VarULong,
    pub item: ItemStackWrapper,
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub entity_data: SyncedActorDataList,
    pub is_from_fishing: bool,
}
