use std::borrow::Cow;

use pumpkin_data::packet::clientbound::PLAY_DISCONNECT;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(PLAY_DISCONNECT)]
pub struct CPlayDisconnect<'a> {
    pub reason: Cow<'a, TextComponent>,
}

impl<'a> CPlayDisconnect<'a> {
    pub fn new(reason: Cow<'a, TextComponent>) -> Self {
        Self { reason }
    }
}
