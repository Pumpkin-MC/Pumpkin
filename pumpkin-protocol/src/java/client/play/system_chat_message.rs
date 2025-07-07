use pumpkin_data::packet::clientbound::PLAY_SYSTEM_CHAT;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_SYSTEM_CHAT)]
pub struct CSystemChatMessage<'a> {
    content: &'a TextComponent,
    overlay: bool,
}

impl<'a> CSystemChatMessage<'a> {
    pub fn new(content: &'a TextComponent, overlay: bool) -> Self {
        Self { content, overlay }
    }
}
