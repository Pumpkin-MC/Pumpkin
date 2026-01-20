use pumpkin_data::packet::clientbound::PLAY_DISGUISED_CHAT;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_DISGUISED_CHAT)]
pub struct CDisguisedChatMessage<'a> {
    pub message: &'a TextComponent,
    pub chat_type: VarInt,
    pub sender_name: &'a TextComponent,
    pub target_name: Option<&'a TextComponent>,
}

impl<'a> CDisguisedChatMessage<'a> {
    #[must_use] 
    pub const fn new(
        message: &'a TextComponent,
        chat_type: VarInt,
        sender_name: &'a TextComponent,
        target_name: Option<&'a TextComponent>,
    ) -> Self {
        Self {
            message,
            chat_type,
            sender_name,
            target_name,
        }
    }
}
