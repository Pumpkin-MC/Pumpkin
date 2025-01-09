use pumpkin_macros::server_packet;
use pumpkin_util::math::vector3::Vector3;

#[derive(serde::Deserialize)]
#[server_packet("play:move_player_pos_rot")]
pub struct SPlayerPositionRotation {
    pub position: Vector3<f64>,
    pub yaw: f32,
    pub pitch: f32,
    pub ground: bool,
}
