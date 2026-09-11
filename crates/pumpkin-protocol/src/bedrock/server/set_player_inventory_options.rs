// Last verified for v2169

use crate::{codec::var_int::VarInt, serial::PacketRead};
use pumpkin_macros::packet;

#[derive(PacketRead)]
#[packet(307)]
pub struct SSetPlayerInventoryOptions {
    // TODO: enum InventoryLeftTabIndex
    pub left_inventory_tab: VarInt,
    // TODO: enum InventoryRightTabIndex
    pub right_inventory_tab: VarInt,

    pub filtering: bool,

    // TODO: enum InventoryLayout
    pub layout_inv: VarInt,
    // TODO: enum InventoryLayout
    pub layout_craft: VarInt,
}
