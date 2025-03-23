use pumpkin_data::packet::serverbound::LOGIN_LOGIN_ACKNOWLEDGED;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

/// Acknowledgement to the `CLoginSuccess` packet sent by the server.
#[derive(Serialize, Deserialize)]
#[packet(LOGIN_LOGIN_ACKNOWLEDGED)]
pub struct SLoginAcknowledged;
