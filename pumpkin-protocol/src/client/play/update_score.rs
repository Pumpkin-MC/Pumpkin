use pumpkin_data::packet::clientbound::PLAY_SET_SCORE;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::packet;
use serde::Serialize;

use crate::{NumberFormat, VarInt};

#[derive(Serialize)]
#[packet(PLAY_SET_SCORE)]
pub struct CUpdateScore<'a> {
    entity_name: &'a str,
    objective_name: &'a str,
    value: VarInt,
    display_name: Option<TextComponent>,
    number_format: Option<NumberFormat>,
}

impl<'a> CUpdateScore<'a> {
    pub fn new(
        entity_name: &'a str,
        objective_name: &'a str,
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
