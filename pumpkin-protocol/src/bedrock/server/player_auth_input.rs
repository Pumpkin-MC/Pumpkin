use pumpkin_macros::packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[packet(144)]
pub struct SPlayerAuthInput {
    // TODO https://mojang.github.io/bedrock-protocol-docs/html/PlayerAuthInputPacket.html
    pub pitch: f32,
    pub yaw: f32,
}
