use pumpkin_data::packet::clientbound::PLAY_SET_SUBTITLE_TEXT;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(PLAY_SET_SUBTITLE_TEXT)]
pub struct CSubtitle<'a> {
    subtitle: &'a TextComponent,
}

impl<'a> CSubtitle<'a> {
    pub fn new(subtitle: &'a TextComponent) -> Self {
        Self { subtitle }
    }
}
