use pumpkin_data::packet::clientbound::PLAY_UPDATE_ATTRIBUTES;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

use crate::codec::var_int::VarInt;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[java_packet(PLAY_UPDATE_ATTRIBUTES)]
pub struct CUpdateAttributes {
    pub entity_id: VarInt,
    pub properties: Vec<Property>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Property {
    pub id: VarInt,
    pub value: f64,
    pub modifiers: Vec<AttributeModifier>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct AttributeModifier {
    pub id: String,
    pub amount: f64,
    pub operation: i8,
}

impl CUpdateAttributes {
    #[must_use]
    pub const fn new(entity_id: VarInt, properties: Vec<Property>) -> Self {
        Self {
            entity_id,
            properties,
        }
    }
}

impl Property {
    #[must_use]
    pub const fn new(id: VarInt, value: f64, modifiers: Vec<AttributeModifier>) -> Self {
        Self {
            id,
            value,
            modifiers,
        }
    }
}

impl AttributeModifier {
    #[must_use]
    pub const fn new(id: String, amount: f64, operation: i8) -> Self {
        Self {
            id,
            amount,
            operation,
        }
    }
}
