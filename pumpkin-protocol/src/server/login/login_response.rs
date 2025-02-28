use pumpkin_data::packet::serverbound::LOGIN_LOGIN_ACKNOWLEDGED;
use pumpkin_macros::packet;
use serde::Serialize;

/// Acknowledgement to the Login Success packet sent to the server.
#[derive(Serialize)]
#[packet(LOGIN_LOGIN_ACKNOWLEDGED)]
pub struct SLoginAcknowledged;
