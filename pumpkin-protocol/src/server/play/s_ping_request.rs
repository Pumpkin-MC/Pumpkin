use pumpkin_data::packet::serverbound::PLAY_PING_REQUEST;
use pumpkin_macros::server_packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[server_packet(PLAY_PING_REQUEST)]
pub struct SPlayPingRequest {
    pub payload: i64,
}
