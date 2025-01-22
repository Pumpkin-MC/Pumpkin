use pumpkin_data::packet::clientbound::LOGIN_LOGIN_COMPRESSION;
use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet(LOGIN_LOGIN_COMPRESSION)]
pub struct CSetCompression {
    threshold: VarInt,
}

impl CSetCompression {
    pub fn new(threshold: VarInt) -> Self {
        Self { threshold }
    }
}
