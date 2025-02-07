use pumpkin_data::packet::clientbound::CONFIG_CUSTOM_PAYLOAD;
use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet(CONFIG_CUSTOM_PAYLOAD)]
pub struct CPluginMessage<'a> {
    channel: &'a str,
    data: &'a [u8],
}

impl<'a> CPluginMessage<'a> {
    pub fn new(channel: &'a str, data: &'a [u8]) -> Self {
        Self { channel, data }
    }
}
