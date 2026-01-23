use pumpkin_data::packet::clientbound::LOGIN_LOGIN_COMPRESSION;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Serialize, Deserialize)]
#[packet(LOGIN_LOGIN_COMPRESSION)]
pub struct CSetCompression {
    pub threshold: VarInt,
}

impl CSetCompression {
    #[must_use]
    pub const fn new(threshold: VarInt) -> Self {
        Self { threshold }
    }
}
