use pumpkin_data::packet::serverbound::LOGIN_LOGIN_ACKNOWLEDGED;
use pumpkin_macros::server_packet;

// Acknowledgement to the Login Success packet sent to the server.
#[derive(serde::Deserialize)]
#[server_packet(LOGIN_LOGIN_ACKNOWLEDGED)]
pub struct SLoginAcknowledged {}
