use pumpkin_data::packet::serverbound::LOGIN_LOGIN_ACKNOWLEDGED;
use pumpkin_macros::java_packet;
use serde::Serialize;

/// Acknowledgement to the `CLoginSuccess` packet sent by the server.
#[derive(Serialize)]
#[java_packet(LOGIN_LOGIN_ACKNOWLEDGED)]
pub struct SLoginAcknowledged;
