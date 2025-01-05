use pumpkin_core::text::{TextComponent, TextComponentNbt};

use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:disconnect")]
pub struct CPlayDisconnect {
    reason: TextComponentNbt,
}

impl<'a> CPlayDisconnect {
    pub fn new(reason: &TextComponent) -> Self {
        Self { reason: reason.to_nbt() }
    }
}
