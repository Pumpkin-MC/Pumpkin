use std::borrow::Cow;

use crate::VarInt;
use pumpkin_data::packet::clientbound::CONFIG_TRANSFER;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(CONFIG_TRANSFER)]
pub struct CTransfer<'a> {
    pub host: Cow<'a, str>,
    pub port: &'a VarInt,
}

impl<'a> CTransfer<'a> {
    pub fn new(host: Cow<'a, str>, port: &'a VarInt) -> Self {
        Self { host, port }
    }
}
