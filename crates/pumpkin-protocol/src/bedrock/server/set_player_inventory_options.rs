use crate::{codec::var_int::VarInt, serial::PacketRead};
use pumpkin_macros::packet;

#[derive(PacketRead)]
#[packet(307)]
pub struct SSetPlayerInventoryOptions {
    pub left_inventory_tab: VarInt,
    pub right_inventory_tab: VarInt,
    pub filtering: bool,
    pub inventory_layout: VarInt,
    pub crafting_layout: VarInt,
}
