use pumpkin_macros::packet;

use crate::serial::PacketRead;

#[derive(PacketRead)]
#[packet(144)]
pub struct SPlayerAuthInput {
    // TODO https://mojang.github.io/bedrock-protocol-docs/html/PlayerAuthInputPacket.html
    pub pitch: f32,
    pub yaw: f32,
}
