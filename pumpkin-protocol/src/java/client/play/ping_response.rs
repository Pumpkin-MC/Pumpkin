use pumpkin_data::packet::clientbound::PLAY_PONG_RESPONSE;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

/// Responds to a client-initiated ping request to synchronize game state.
#[derive(Serialize, Deserialize)]
#[java_packet(PLAY_PONG_RESPONSE)]
pub struct CPingResponse {
    /// The unique identifier sent by the client in the initial Ping packet.
    /// The server must return this exact value.
    pub payload: i64,
}

impl CPingResponse {
    #[must_use]
    pub const fn new(payload: i64) -> Self {
        Self { payload }
    }
}
