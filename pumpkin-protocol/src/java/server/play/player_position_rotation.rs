use pumpkin_data::packet::serverbound::PLAY_MOVE_PLAYER_POS_ROT;
use pumpkin_macros::java_packet;
use pumpkin_util::math::vector3::Vector3;

pub const FLAG_ON_GROUND: u8 = 0x01;
pub const FLAG_IN_WALL: u8 = 0x02;

#[derive(serde::Deserialize)]
#[java_packet(PLAY_MOVE_PLAYER_POS_ROT)]
pub struct SPlayerPositionRotation {
    pub position: Vector3<f64>,
    pub yaw: f32,
    pub pitch: f32,
    /// bit 0: [FLAG_ON_GROUND], bit 1: [FLAG_IN_WALL]
    pub collision: u8,
}
