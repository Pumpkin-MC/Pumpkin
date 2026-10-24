// Last verified for v2169

use pumpkin_macros::packet;

use crate::{codec::var_uint::VarUInt, serial::PacketWrite};

/// Sent by the server to update the active selected hotbar slot of the player.
#[derive(PacketWrite)]
#[packet(48)]
pub struct CPlayerHotbar {
    /// Zero-based slot index selected in the hotbar.
    pub selected_slot: VarUInt,
    /// Container identifier (usually player inventory).
    pub container_id: u8,
    /// Whether the client should immediately focus the slot.
    pub should_select_slot: bool,
}
