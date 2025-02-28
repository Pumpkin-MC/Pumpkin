use pumpkin_data::packet::serverbound::PLAY_MOVE_PLAYER_POS_ROT;
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

#[derive(serde::Deserialize)]
#[packet(PLAY_MOVE_PLAYER_POS_ROT)]
pub struct SPlayerPositionRotation {
    pub position: Vector3<f64>,
    pub yaw: f32,
    pub pitch: f32,
    pub ground: bool,
}
