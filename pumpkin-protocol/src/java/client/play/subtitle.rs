use pumpkin_data::packet::clientbound::PLAY_SET_SUBTITLE_TEXT;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_SET_SUBTITLE_TEXT)]
pub struct CSubtitle<'a> {
    pub subtitle: &'a TextComponent,
}

impl<'a> CSubtitle<'a> {
    #[must_use] 
    pub const fn new(subtitle: &'a TextComponent) -> Self {
        Self { subtitle }
    }
}
