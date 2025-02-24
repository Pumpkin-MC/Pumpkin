use pumpkin_data::packet::clientbound::PLAY_SET_TITLE_TEXT;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(PLAY_SET_TITLE_TEXT)]
pub struct CTitleText<'a> {
    title: &'a TextComponent,
}

impl<'a> CTitleText<'a> {
    pub fn new(title: &'a TextComponent) -> Self {
        Self { title }
    }
}
