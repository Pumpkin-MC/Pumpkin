use crate::{
    codec::{var_long::VarLong, var_uint::VarUInt, var_ulong::VarULong},
    serial::PacketWrite,
};
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;
use std::io::{Error, Write};

use super::{
    common::EntityLink,
    set_actor_data::{EntityMetadata, PropertySyncData},
};

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

impl PacketWrite for CAddActor {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.entity_unique_id.write(writer)?;
        self.entity_runtime_id.write(writer)?;
        self.entity_type.write(writer)?;
        self.position.write(writer)?;
        self.velocity.write(writer)?;
        self.pitch.write(writer)?;
        self.yaw.write(writer)?;
        self.head_yaw.write(writer)?;
        self.body_yaw.write(writer)?;
        VarUInt(self.attributes.len() as u32).write(writer)?;
        for attr in &self.attributes {
            attr.write(writer)?;
        }
        self.metadata.write(writer)?;
        self.synced_properties.write(writer)?;
        VarUInt(self.links.len() as u32).write(writer)?;
        for link in &self.links {
            link.write(writer)?;
        }
        Ok(())
    }
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
