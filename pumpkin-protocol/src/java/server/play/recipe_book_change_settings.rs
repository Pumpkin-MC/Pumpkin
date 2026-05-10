use pumpkin_data::packet::serverbound::PLAY_RECIPE_BOOK_CHANGE_SETTINGS;
use pumpkin_macros::java_packet;
use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
#[java_packet(PLAY_RECIPE_BOOK_CHANGE_SETTINGS)]
pub struct SRecipeBookChangeSettings {
    pub book_type: VarInt,
    pub is_open: bool,
    pub is_filtering: bool,
}
