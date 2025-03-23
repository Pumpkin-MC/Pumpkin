use std::borrow::Cow;

use pumpkin_data::packet::clientbound::PLAY_SET_ACTION_BAR_TEXT;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(PLAY_SET_ACTION_BAR_TEXT)]
pub struct CActionBar<'a> {
    action_bar: Cow<'a, TextComponent>,
}

impl<'a> CActionBar<'a> {
    pub fn new(action_bar: Cow<'a, TextComponent>) -> Self {
        Self { action_bar }
    }
}
