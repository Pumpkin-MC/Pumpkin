use pumpkin_core::text::{TextComponent, TextComponentNbt};

use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:system_chat")]
pub struct CSystemChatMessage {
    content: TextComponentNbt,
    overlay: bool,
}

impl CSystemChatMessage {
    pub fn new(content: &TextComponent, overlay: bool) -> Self {
        Self {
            content: content.to_nbt(),
            overlay
        }
    }
}
