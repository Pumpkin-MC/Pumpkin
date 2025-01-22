use pumpkin_data::packet::clientbound::LOGIN_LOGIN_DISCONNECT;
use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet(LOGIN_LOGIN_DISCONNECT)]
pub struct CLoginDisconnect<'a> {
    json_reason: &'a str,
}

impl<'a> CLoginDisconnect<'a> {
    // input json!
    pub fn new(json_reason: &'a str) -> Self {
        Self { json_reason }
    }
}
