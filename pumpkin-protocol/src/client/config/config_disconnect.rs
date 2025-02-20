use pumpkin_macros::client_packet;

use pumpkin_data::packet::clientbound::CONFIG_DISCONNECT;

#[derive(serde::Serialize)]
#[client_packet(CONFIG_DISCONNECT)]
pub struct CConfigDisconnect<'a> {
    reason: &'a str,
}

impl<'a> CConfigDisconnect<'a> {
    pub fn new(reason: &'a str) -> Self {
        Self { reason }
    }
}
