use pumpkin_data::packets::serverbound::STATUS_STATUS_REQUEST;
use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet(STATUS_STATUS_REQUEST)]
pub struct SStatusRequest {
    // empty
}
