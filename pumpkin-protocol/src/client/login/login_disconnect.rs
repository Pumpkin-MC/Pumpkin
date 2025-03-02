use pumpkin_data::packet::clientbound::LOGIN_LOGIN_DISCONNECT;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(LOGIN_LOGIN_DISCONNECT)]
pub struct CLoginDisconnect<'a> {
    pub json_reason: &'a str,
}

impl<'a> CLoginDisconnect<'a> {
    // input json!
    pub fn new(json_reason: &'a str) -> Self {
        Self { json_reason }
    }
}
