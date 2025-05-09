use pumpkin_data::packet::clientbound::PLAY_SET_ENTITY_DATA;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::{VarInt, ser::network_serialize_no_prefix};

#[derive(Serialize)]
#[packet(PLAY_SET_ENTITY_DATA)]
pub struct CSetEntityMetadata {
    entity_id: VarInt,
    // TODO: We should migrate the serialization of this into this file
    #[serde(serialize_with = "network_serialize_no_prefix")]
    metadata: Box<[u8]>,
}

impl CSetEntityMetadata {
    pub fn new(entity_id: VarInt, metadata: Box<[u8]>) -> Self {
        Self {
            entity_id,
            metadata,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct Metadata<T> {
    /// https://minecraft.wiki/w/Java_Edition_protocol/Entity_metadata
    index: u8,
    r#type: VarInt,
    value: T,
}

impl<T> Metadata<T> {
    pub fn new(index: u8, r#type: MetaDataType, value: T) -> Self {
        Self {
            index,
            r#type: VarInt(r#type as i32),
            value,
        }
    }
}

pub enum MetaDataType {
    Byte,
    Integer,
    Long,
    Float,
    String,
    TextComponent,
    OptionalTextComponent,
    ItemStack,
    Boolean,
    Rotation,
    BlockPos,
    OptionalBlockPos,
    Facing,
    OptionalUuid,
    BlockState,
    OptionalBlockState,
    NbtCompound,
    Particle,
    ParticleList,
    VillagerData,
    OptionalInt,
    EntityPose,
    CatVariant,
    WolfVariant,
    FrogVariant,
    OptionalGlobalPos,
    PaintingVariant,
    SnifferState,
    ArmadilloState,
    Vector3f,
    QuaternionF,
}
