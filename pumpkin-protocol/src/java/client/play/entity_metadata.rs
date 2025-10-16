use pumpkin_data::packet::clientbound::PLAY_SET_ENTITY_DATA;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::{VarInt, ser::network_serialize_no_prefix};

#[derive(Serialize)]
#[packet(PLAY_SET_ENTITY_DATA)]
pub struct CSetEntityMetadata {
    pub entity_id: VarInt,
    // TODO: We should migrate the serialization of this into this file
    #[serde(serialize_with = "network_serialize_no_prefix")]
    pub metadata: Box<[u8]>,
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
    Byte = 0,
    Integer = 1,
    Long = 2,
    Float = 3,
    String = 4,
    TextComponent = 5,
    OptionalTextComponent = 6,
    ItemStack = 7,
    Boolean = 8,
    Rotation = 9,
    BlockPos = 10,
    OptionalBlockPos = 11,
    Facing = 12,
    LazyEntityReference = 13,
    BlockState = 14,
    OptionalBlockState = 15,
    Particle = 16,
    ParticleList = 17,
    VillagerData = 18,
    OptionalInt = 19,
    EntityPose = 20,
    CatVariant = 21,
    ChickenVariant = 22,
    CowVariant = 23,
    WolfVariant = 24,
    WolfSoundVariant = 25,
    FrogVariant = 26,
    PigVariant = 27,
    OptionalGlobalPos = 28,
    PaintingVariant = 29,
    SnifferState = 30,
    ArmadilloState = 31,
    CopperGolemState = 32,
    OxidationLevel = 33,
    Vector3f = 34,
    QuaternionF = 35,
    Profile = 36,
}
