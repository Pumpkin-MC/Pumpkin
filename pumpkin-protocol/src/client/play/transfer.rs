use crate::VarInt;
use pumpkin_data::packet::clientbound::PLAY_TRANSFER;
use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet(PLAY_TRANSFER)]
pub struct CTransfer<'a> {
    host: &'a str,
    port: &'a VarInt,
}

impl<'a> CTransfer<'a> {
    pub fn new(host: &'a str, port: &'a VarInt) -> Self {
        Self { host, port }
    }
}
