use pumpkin_data::packet::serverbound::PLAY_PADDLE_BOAT;
use pumpkin_macros::java_packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[java_packet(PLAY_PADDLE_BOAT)]
pub struct SPaddleBoat {
    pub left_paddle: bool,
    pub right_paddle: bool,
}
