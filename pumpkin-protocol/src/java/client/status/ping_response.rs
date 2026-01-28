use pumpkin_data::packet::clientbound::STATUS_PONG_RESPONSE;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

/// Sent by the server to complete a latency check initiated by a `SStatusPingRequest`.
///
/// This is the final packet in the Server List Ping (SLP) sequence. It allows the
/// client to calculate the round-trip time (ping) to the server.
#[derive(Serialize, Deserialize)]
#[java_packet(STATUS_PONG_RESPONSE)]
pub struct CPingResponse {
    /// The exact 64-bit integer received from the client's ping request.
    ///
    /// The client uses this value to ensure the response matches the specific
    /// request it sent and to measure elapsed time.
    pub payload: i64,
}

impl CPingResponse {
    #[must_use]
    pub fn new(payload: i64) -> Self {
        Self { payload }
    }
}
