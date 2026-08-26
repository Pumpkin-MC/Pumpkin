// Last verified for v2169

use pumpkin_macros::packet;

use crate::{codec::var_ulong::VarULong, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(29)]
pub struct CUpdateAttributes {
    pub target_runtime_id: VarULong,
    pub attribute_list: Vec<AttributeData>,
    pub tick: VarULong,
}

#[derive(PacketWrite)]
pub struct AttributeData {
    pub min_value: f32,
    pub max_value: f32,
    pub current_value: f32,
    pub default_min_value: f32,
    pub default_max_value: f32,
    pub default_value: f32,
    pub name: String,
    pub modifiers: Vec<AttributeModifier>,
}

#[derive(PacketWrite)]
pub struct AttributeModifier {
    pub id: String,
    pub name: String,
    pub amount: f32,
    pub operation: i32,
    pub operand: i32,
    pub is_serializable: bool,
}
