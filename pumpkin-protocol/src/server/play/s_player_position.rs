use pumpkin_macros::server_packet;
use pumpkin_util::math::vector3::Vector3;

#[derive(serde::Deserialize)]
#[server_packet("play:move_player_pos")]
pub struct SPlayerPosition {
    pub position: Vector3<f64>,
    pub ground: bool,
}
