use crate::{
    bedrock::network_item::NetworkItemDescriptor,
    codec::{var_int::VarInt, var_uint::VarUInt, var_ulong::VarULong},
    serial::PacketWrite,
};
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;
use std::io::{Error, Write};
use uuid::Uuid;

use super::{
    common::{AbilityLayer, EntityLink},
    set_actor_data::EntityMetadata,
};

#[packet(12)]
pub struct CAddPlayer {
    pub uuid: Uuid,
    pub username: String,
    pub entity_runtime_id: VarULong,
    pub platform_chat_id: String,
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub pitch: f32,
    pub yaw: f32,
    pub head_yaw: f32,
    pub held_item: NetworkItemDescriptor,
    pub game_mode: VarInt,
    pub metadata: EntityMetadata,
    pub properties: EntityProperties,
    pub ability_data: AbilityData,
    pub links: Vec<EntityLink>,
    pub device_id: String,
    pub build_platform: i32,
}

impl PacketWrite for CAddPlayer {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.uuid.write(writer)?;
        self.username.write(writer)?;
        self.entity_runtime_id.write(writer)?;
        self.platform_chat_id.write(writer)?;
        self.position.write(writer)?;
        self.velocity.write(writer)?;
        self.pitch.write(writer)?;
        self.yaw.write(writer)?;
        self.head_yaw.write(writer)?;
        self.held_item.write(writer)?;
        self.game_mode.write(writer)?;
        self.metadata.write(writer)?;
        self.properties.write(writer)?;
        self.ability_data.write(writer)?;
        VarUInt(self.links.len() as u32).write(writer)?;
        for link in &self.links {
            link.write(writer)?;
        }
        self.device_id.write(writer)?;
        self.build_platform.write(writer)
    }
}

impl CAddPlayer {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        uuid: Uuid,
        username: String,
        entity_runtime_id: VarULong,
        platform_chat_id: String,
        position: Vector3<f32>,
        velocity: Vector3<f32>,
        pitch: f32,
        yaw: f32,
        head_yaw: f32,
        held_item: NetworkItemDescriptor,
        game_mode: VarInt,
        metadata: EntityMetadata,
        properties: EntityProperties,
        ability_data: AbilityData,
        links: Vec<EntityLink>,
        device_id: String,
        build_platform: i32,
    ) -> Self {
        Self {
            uuid,
            username,
            entity_runtime_id,
            platform_chat_id,
            position,
            velocity,
            pitch,
            yaw,
            head_yaw,
            held_item,
            game_mode,
            metadata,
            properties,
            ability_data,
            links,
            device_id,
            build_platform,
        }
    }
}

#[derive(Default, Clone)]
pub struct EntityProperties {
    pub ints: Vec<(VarUInt, VarInt)>,
    pub floats: Vec<(VarUInt, f32)>,
}

impl PacketWrite for EntityProperties {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarUInt(self.ints.len() as u32).write(writer)?;
        for (id, val) in &self.ints {
            id.write(writer)?;
            val.write(writer)?;
        }
        VarUInt(self.floats.len() as u32).write(writer)?;
        for (id, val) in &self.floats {
            id.write(writer)?;
            val.write(writer)?;
        }
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct AbilityData {
    pub entity_unique_id: i64,
    pub player_permissions: u8,
    pub command_permissions: u8,
    pub layers: Vec<AbilityLayer>,
}

impl PacketWrite for AbilityData {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.entity_unique_id.write(writer)?;
        self.player_permissions.write(writer)?;
        self.command_permissions.write(writer)?;
        (self.layers.len() as u8).write(writer)?;
        for layer in &self.layers {
            layer.write(writer)?;
        }
        Ok(())
    }
}
