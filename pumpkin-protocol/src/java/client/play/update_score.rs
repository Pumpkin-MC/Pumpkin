use pumpkin_data::packet::clientbound::PLAY_SET_SCORE;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::packet;
use serde::Serialize;

use crate::{NumberFormat, VarInt};

#[derive(Serialize)]
#[packet(PLAY_SET_SCORE)]
pub struct CUpdateScore {
    pub entity_name: String,
    pub objective_name: String,
    pub value: VarInt,
    pub display_name: Option<TextComponent>,
    pub number_format: Option<NumberFormat>,
}

impl CUpdateScore {
    #[must_use]
    pub const fn new(
        entity_name: String,
        objective_name: String,
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
