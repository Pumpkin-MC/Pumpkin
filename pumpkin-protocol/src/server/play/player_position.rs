use pumpkin_data::packet::serverbound::PLAY_MOVE_PLAYER_POS;
use pumpkin_macros::server_packet;
use pumpkin_util::math::vector3::Vec3;

#[derive(serde::Deserialize)]
#[server_packet(PLAY_MOVE_PLAYER_POS)]
pub struct SPlayerPosition {
    pub position: Vec3<f64>,
    pub ground: bool,
}
