use pumpkin_macros::packet;
use pumpkin_nbt::compound::NbtCompound;

use crate::{
    codec::{var_int::VarInt, var_long::VarLong},
    serial::PacketWrite,
};

/// Bedrock container type of horse and llama screens.
pub const HORSE_CONTAINER_TYPE: u8 = 12;

/// Opens a mount equipment screen (horse, llama) on the client.
#[derive(PacketWrite)]
#[packet(81)]
pub struct CUpdateEquip {
    pub container_id: u8,
    pub container_type: u8,
    pub size: VarInt,
    pub entity_unique_id: VarLong,
    /// Network NBT with a `slots` list describing the equipment slots.
    pub data: NbtCompound,
}
