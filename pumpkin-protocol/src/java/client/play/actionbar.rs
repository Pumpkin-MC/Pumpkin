use pumpkin_data::packet::clientbound::PLAY_SET_ACTION_BAR_TEXT;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_SET_ACTION_BAR_TEXT)]
pub struct CActionBar<'a> {
    pub action_bar: &'a TextComponent,
}

impl<'a> CActionBar<'a> {
    #[must_use] 
    pub const fn new(action_bar: &'a TextComponent) -> Self {
        Self { action_bar }
    }
}
