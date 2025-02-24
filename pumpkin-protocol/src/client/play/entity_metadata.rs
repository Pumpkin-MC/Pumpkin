use pumpkin_data::packet::clientbound::PLAY_SET_ENTITY_DATA;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_SET_ENTITY_DATA)]
pub struct CSetEntityMetadata<T> {
    entity_id: VarInt,
    metadata: Metadata<T>,
    end: u8,
}

impl<T> CSetEntityMetadata<T> {
    pub fn new(entity_id: VarInt, metadata: Metadata<T>) -> Self {
        Self {
            entity_id,
            metadata,
            end: 255,
        }
    }
}

#[derive(Serialize)]
pub struct Metadata<T> {
    index: u8,
    typ: VarInt,
    value: T,
}

impl<T> Metadata<T> {
    pub fn new(index: u8, typ: MetaDataType, value: T) -> Self {
        Self {
            index,
            typ: VarInt(typ as i32),
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
