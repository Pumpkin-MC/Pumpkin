use crate::VarInt;
use pumpkin_data::packet::clientbound::CONFIG_TRANSFER;
use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(CONFIG_TRANSFER)]
pub struct CTransfer<'a> {
    pub host: &'a str,
    pub port: &'a VarInt,
}

impl<'a> CTransfer<'a> {
    #[must_use]
    pub const fn new(host: &'a str, port: &'a VarInt) -> Self {
        Self { host, port }
    }
}
