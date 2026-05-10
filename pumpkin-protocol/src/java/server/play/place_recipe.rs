use pumpkin_data::packet::serverbound::PLAY_PLACE_RECIPE;
use pumpkin_macros::java_packet;
use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
#[java_packet(PLAY_PLACE_RECIPE)]
pub struct SPlaceRecipe {
    pub container_id: i8,
    pub recipe_display_id: VarInt,
    pub use_max_items: bool,
}
