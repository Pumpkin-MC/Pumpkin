use pumpkin_macros::packet;

use crate::{codec::var_uint::VarUInt, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(48)]
pub struct CPlayerHotbar {
    pub selected_slot: VarUInt,
    pub container_id: u8,
    pub should_select_block: bool,
}
