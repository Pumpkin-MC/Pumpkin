use pumpkin_data::packet::clientbound::PLAY_OPEN_SCREEN;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_OPEN_SCREEN)]
pub struct COpenScreen<'a> {
    window_id: VarInt,
    window_type: VarInt,
    window_title: &'a TextComponent,
}

impl<'a> COpenScreen<'a> {
    pub fn new(window_id: VarInt, window_type: VarInt, window_title: &'a TextComponent) -> Self {
        Self {
            window_id,
            window_type,
            window_title,
        }
    }
}
