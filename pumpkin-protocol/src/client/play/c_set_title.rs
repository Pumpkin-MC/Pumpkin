use pumpkin_util::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:set_title_text")]
pub struct CTitleText<'a> {
    title: &'a TextComponent,
}

impl<'a> CTitleText<'a> {
    pub fn new(title: &'a TextComponent) -> Self {
        Self { title }
    }
}
