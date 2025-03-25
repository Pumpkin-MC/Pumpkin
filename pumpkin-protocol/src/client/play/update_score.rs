use std::borrow::Cow;

use pumpkin_data::packet::clientbound::PLAY_SET_SCORE;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::{NumberFormat, VarInt};

#[derive(Serialize, Deserialize)]
#[packet(PLAY_SET_SCORE)]
pub struct CUpdateScore<'a> {
    entity_name: Cow<'a, str>,
    objective_name: Cow<'a, str>,
    value: VarInt,
    display_name: Option<TextComponent>,
    number_format: Option<NumberFormat>,
}

impl<'a> CUpdateScore<'a> {
    pub fn new(
        entity_name: Cow<'a, str>,
        objective_name: Cow<'a, str>,
        value: VarInt,
        display_name: Option<TextComponent>,
        number_format: Option<NumberFormat>,
    ) -> Self {
        Self {
            entity_name,
            objective_name,
            value,
            display_name,
            number_format,
        }
    }
}
