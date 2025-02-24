use pumpkin_data::packet::serverbound::PLAY_MOVE_PLAYER_POS;
use pumpkin_macros::packet;
use pumpkin_util::math::vec3::Vec3;

#[derive(serde::Deserialize)]
#[packet(PLAY_MOVE_PLAYER_POS)]
pub struct SPlayerPosition {
    pub position: Vec3<f64>,
    pub ground: bool,
}
