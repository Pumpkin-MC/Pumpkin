use pumpkin_data::packet::serverbound::PLAY_PADDLE_BOAT;
use pumpkin_macros::java_packet;

/// Sent when the player paddles a boat.
#[derive(serde::Deserialize, serde::Serialize)]
#[java_packet(PLAY_PADDLE_BOAT)]
pub struct SPaddleBoat {
    pub left_paddle_turning: bool,
    pub right_paddle_turning: bool,
}
