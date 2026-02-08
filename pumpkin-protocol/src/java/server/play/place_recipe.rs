use pumpkin_data::packet::serverbound::PLAY_PLACE_RECIPE;
use pumpkin_macros::java_packet;

use crate::VarInt;

/// Sent when the player clicks a recipe in the recipe book to auto-fill
/// the crafting grid.
#[derive(serde::Deserialize, serde::Serialize)]
#[java_packet(PLAY_PLACE_RECIPE)]
pub struct SPlaceRecipe {
    pub container_id: VarInt,
    pub recipe: VarInt,
    pub shift_down: bool,
}
