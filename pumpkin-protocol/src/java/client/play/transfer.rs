use crate::VarInt;
use pumpkin_data::packet::clientbound::PLAY_TRANSFER;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[java_packet(PLAY_TRANSFER)]
pub struct CTransfer<'a> {
    pub host: &'a str,
    pub port: VarInt,
}

impl<'a> CTransfer<'a> {
    pub fn new(host: &'a str, port: VarInt) -> Self {
        Self { host, port }
    }
}
