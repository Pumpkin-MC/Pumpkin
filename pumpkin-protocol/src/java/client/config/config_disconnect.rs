use pumpkin_data::packet::clientbound::CONFIG_DISCONNECT;
use pumpkin_macros::java_packet;
use serde::Deserialize;

#[derive(serde::Serialize, Deserialize)]
#[java_packet(CONFIG_DISCONNECT)]
pub struct CConfigDisconnect<'a> {
    pub reason: &'a str,
}

impl<'a> CConfigDisconnect<'a> {
    #[must_use]
    pub const fn new(reason: &'a str) -> Self {
        Self { reason }
    }
}
