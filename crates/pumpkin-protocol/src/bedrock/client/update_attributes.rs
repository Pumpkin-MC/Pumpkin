use pumpkin_macros::packet;

use crate::{
    codec::{var_uint::VarUInt, var_ulong::VarULong},
    serial::PacketWrite,
};

#[derive(PacketWrite)]
#[packet(29)]
pub struct CUpdateAttributes {
    pub runtime_id: VarULong,
    pub attributes: Vec<Attribute>,
    pub player_tick: VarULong,
}

#[derive(PacketWrite)]
pub struct Attribute {
    pub min_value: f32,
    pub max_value: f32,
    pub current_value: f32,
    pub default_min_value: f32,
    pub default_max_value: f32,
    pub default_value: f32,
    pub name: String,
    pub modifiers_list_size: VarUInt,
}
