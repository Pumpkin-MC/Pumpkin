use pumpkin_data::packet::serverbound::PLAY_MOVE_PLAYER_ROT;
use pumpkin_macros::packet;

#[derive(serde::Deserialize)]
#[packet(PLAY_MOVE_PLAYER_ROT)]
pub struct SPlayerRotation {
    pub yaw: f32,
    pub pitch: f32,
    pub ground: bool,
}
