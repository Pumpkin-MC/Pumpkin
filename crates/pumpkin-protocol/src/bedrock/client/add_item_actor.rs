use crate::{
    codec::{var_long::VarLong, var_ulong::VarULong},
    serial::PacketWrite,
};
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use super::set_actor_data::EntityMetadata;
use crate::bedrock::network_item::ItemStackWrapper;

#[derive(PacketWrite)]
#[packet(15)]
pub struct CAddItemActor {
    pub entity_unique_id: VarLong,
    pub entity_runtime_id: VarULong,
    pub item: ItemStackWrapper,
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub metadata: EntityMetadata,
    pub from_fishing: bool,
}
