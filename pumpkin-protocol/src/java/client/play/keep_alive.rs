use pumpkin_data::packet::clientbound::PLAY_KEEP_ALIVE;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

/// Maintains the connection and measures latency (ping) between client and server.
///
/// The server sends this packet at regular intervals (typically every 15–20 seconds).
/// The client must respond with the exact same ID. If the server does not receive
/// a response within a timeout period (usually 30 seconds), it will disconnect
/// the player with a "Timed Out" message.
#[derive(Serialize, Deserialize)]
#[java_packet(PLAY_KEEP_ALIVE)]
pub struct CKeepAlive {
    /// A unique random identifier for this specific keep-alive request.
    /// Used to match the server's request with the client's response.
    pub keep_alive_id: i64,
}

impl CKeepAlive {
    pub fn new(keep_alive_id: i64) -> Self {
        Self { keep_alive_id }
    }
}
