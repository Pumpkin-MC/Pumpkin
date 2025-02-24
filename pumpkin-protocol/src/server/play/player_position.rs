use pumpkin_data::packet::serverbound::PLAY_MOVE_PLAYER_POS;
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

#[derive(serde::Deserialize)]
#[packet(PLAY_MOVE_PLAYER_POS)]
pub struct SPlayerPosition {
    pub position: Vector3<f64>,
    pub ground: bool,
}
