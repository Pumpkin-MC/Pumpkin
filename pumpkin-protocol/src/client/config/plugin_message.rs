use std::borrow::Cow;

use pumpkin_data::packet::clientbound::CONFIG_CUSTOM_PAYLOAD;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(CONFIG_CUSTOM_PAYLOAD)]
pub struct CPluginMessage<'a> {
    pub channel: Cow<'a, str>,
    pub data: &'a [u8],
}

impl<'a> CPluginMessage<'a> {
    pub fn new(channel: Cow<'a, str>, data: &'a [u8]) -> Self {
        Self { channel, data }
    }
}
