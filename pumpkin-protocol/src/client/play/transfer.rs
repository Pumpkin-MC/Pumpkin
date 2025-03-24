use std::borrow::Cow;

use crate::VarInt;
use pumpkin_data::packet::clientbound::PLAY_TRANSFER;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(PLAY_TRANSFER)]
pub struct CTransfer<'a> {
    host: Cow<'a, str>,
    port: VarInt,
}

impl<'a> CTransfer<'a> {
    pub fn new(host: Cow<'a, str>, port: VarInt) -> Self {
        Self { host, port }
    }
}
