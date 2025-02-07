use pumpkin_data::packet::serverbound::STATUS_PING_REQUEST;
use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet(STATUS_PING_REQUEST)]
pub struct SStatusPingRequest {
    pub payload: i64,
}
