use pumpkin_macros::packet;

use crate::serial::PacketRead;

/// Sent by the Bedrock client when the player changes their active hotbar slot.
///
/// Packet ID: `48`
/// Ref: <https://mojang.github.io/bedrock-protocol-docs/html/PlayerHotbarPacket.html>
#[derive(PacketRead)]
#[packet(48)]
pub struct SPlayerHotbar {
    pub selected_slot: u32,
    pub container_id: u8,
    pub select_slot: bool,
}
