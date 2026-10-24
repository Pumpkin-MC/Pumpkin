// Last verified for v2169

use pumpkin_macros::packet;

use crate::{codec::var_uint::VarUInt, serial::PacketRead};

/// Sent by the Bedrock client when the player changes their active hotbar slot.
#[derive(PacketRead)]
#[packet(48)]
pub struct SPlayerHotbar {
    /// Zero-based slot index selected in the hotbar.
    pub selected_slot: VarUInt,
    /// Container identifier (usually player inventory).
    pub container_id: u8,
    /// Whether the server should select this slot immediately.
    pub should_select_slot: bool,
}
