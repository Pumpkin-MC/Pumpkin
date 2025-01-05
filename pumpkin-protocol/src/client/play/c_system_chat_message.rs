use pumpkin_core::text::Text;

use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:system_chat")]
pub struct CSystemChatMessage {
    content: Text,
    overlay: bool,
}

impl CSystemChatMessage {
    pub fn new(content: impl Into<Text>, overlay: bool) -> Self {
        Self {
            content: content.into(),
            overlay,
        }
    }
}
