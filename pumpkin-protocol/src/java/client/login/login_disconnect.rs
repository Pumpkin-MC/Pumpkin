use pumpkin_data::packet::clientbound::LOGIN_LOGIN_DISCONNECT;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

/// Sent by the server to reject a login attempt or kick a player during the login phase
///
/// This is used for reasons such as the server being full, the player being banned,
/// or version mismatches. After this packet is sent, the connection is closed.
#[derive(Serialize, Deserialize)]
#[java_packet(LOGIN_LOGIN_DISCONNECT)]
pub struct CLoginDisconnect {
    /// A JSON-encoded chat component explaining why the player was disconnected.
    pub json_reason: String,
}

impl CLoginDisconnect {
    #[must_use]
    pub fn new(json_reason: String) -> Self {
        Self { json_reason }
    }
}
