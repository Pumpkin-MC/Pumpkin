use pumpkin_data::packet::clientbound::PLAY_DISCONNECT;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet(PLAY_DISCONNECT)]
pub struct CPlayDisconnect<'a> {
    reason: &'a TextComponent,
}

impl<'a> CPlayDisconnect<'a> {
    pub fn new(reason: &'a TextComponent) -> Self {
        Self { reason }
    }
}
