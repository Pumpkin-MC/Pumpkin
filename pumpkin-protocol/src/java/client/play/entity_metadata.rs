use pumpkin_data::{meta_data_type::MetaDataType, packet::clientbound::PLAY_SET_ENTITY_DATA};
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
    #[must_use] 
    pub const fn new(entity_id: VarInt, metadata: Box<[u8]>) -> Self {
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
    pub const fn new(index: u8, r#type: MetaDataType, value: T) -> Self {
        Self {
            index,
            r#type: VarInt(r#type as i32),
            value,
        }
    }
}
