use pumpkin_data::packet::clientbound::LOGIN_LOGIN_DISCONNECT;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(LOGIN_LOGIN_DISCONNECT)]
pub struct CLoginDisconnect {
    pub json_reason: String,
}

impl CLoginDisconnect {
    // input json!
    #[must_use]
    pub const fn new(json_reason: String) -> Self {
        Self { json_reason }
    }
}
