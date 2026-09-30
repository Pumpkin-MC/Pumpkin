// Last verified for v2169

use pumpkin_macros::packet;

use crate::{codec::var_uint::VarUInt, serial::PacketRead};

/// Sent by the Bedrock client when the player changes their active hotbar slot.
#[derive(PacketRead)]
#[packet(48)]
pub struct SPlayerHotbar {
    pub selected_slot: VarUInt,
    pub container_id: u8,
    pub should_select_slot: bool,
}
