use pumpkin_data::packet::clientbound::PLAY_SET_TITLE_TEXT;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(PLAY_SET_TITLE_TEXT)]
pub struct CTitleText<'a> {
    pub title: &'a TextComponent,
}

impl<'a> CTitleText<'a> {
    #[must_use]
    pub fn new(title: &'a TextComponent) -> Self {
        Self { title }
    }
}
