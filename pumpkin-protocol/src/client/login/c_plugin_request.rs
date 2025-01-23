use pumpkin_data::packet::clientbound::LOGIN_CUSTOM_QUERY;
use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet(LOGIN_CUSTOM_QUERY)]
pub struct CLoginPluginRequest<'a> {
    message_id: VarInt,
    channel: &'a str,
    data: &'a [u8],
}

impl<'a> CLoginPluginRequest<'a> {
    pub fn new(message_id: VarInt, channel: &'a str, data: &'a [u8]) -> Self {
        Self {
            message_id,
            channel,
            data,
        }
    }
}
