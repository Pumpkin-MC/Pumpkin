use pumpkin_data::packet::serverbound::STATUS_PING_REQUEST;
use pumpkin_macros::java_packet;
use serde::Serialize;

/// Sent by the client to measure the round-trip time (latency) to the server.
///
/// This is the second part of the Server List Ping (SLP) process
/// The server should respond with `CPingResponse`.
#[derive(serde::Deserialize, Serialize)]
#[java_packet(STATUS_PING_REQUEST)]
pub struct SStatusPingRequest {
    pub payload: i64,
}
