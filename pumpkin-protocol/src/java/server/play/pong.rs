use pumpkin_data::packet::serverbound::PLAY_PONG;
use pumpkin_macros::java_packet;

/// Response to a Ping packet from the server.
#[derive(serde::Deserialize, serde::Serialize)]
#[java_packet(PLAY_PONG)]
pub struct SPong {
    pub id: i32,
}
