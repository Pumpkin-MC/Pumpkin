use crate::{
    codec::{var_long::VarLong, var_ulong::VarULong},
    serial::PacketWrite,
};
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use super::{
    common::EntityLink,
    set_actor_data::{EntityMetadata, PropertySyncData},
};

#[derive(PacketWrite)]
#[packet(13)]
pub struct CAddActor {
    pub entity_unique_id: VarLong,
    pub entity_runtime_id: VarULong,
    pub entity_type: String,
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub pitch: f32,
    pub yaw: f32,
    pub head_yaw: f32,
    pub body_yaw: f32,
    pub attributes: Vec<AttributeValue>,
    pub metadata: EntityMetadata,
    pub synced_properties: PropertySyncData,
    pub links: Vec<EntityLink>,
}

impl CAddActor {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        entity_unique_id: VarLong,
        entity_runtime_id: VarULong,
        entity_type: String,
        position: Vector3<f32>,
        velocity: Vector3<f32>,
        pitch: f32,
        yaw: f32,
        head_yaw: f32,
        body_yaw: f32,
        attributes: Vec<AttributeValue>,
        metadata: EntityMetadata,
        synced_properties: PropertySyncData,
        links: Vec<EntityLink>,
    ) -> Self {
        Self {
            entity_unique_id,
            entity_runtime_id,
            entity_type,
            position,
            velocity,
            pitch,
            yaw,
            head_yaw,
            body_yaw,
            attributes,
            metadata,
            synced_properties,
            links,
        }
    }
}

#[derive(PacketWrite)]
pub struct AttributeValue {
    pub name: String,
    pub min: f32,
    pub value: f32,
    pub max: f32,
}
