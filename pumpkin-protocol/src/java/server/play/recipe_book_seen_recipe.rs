use pumpkin_data::packet::serverbound::PLAY_RECIPE_BOOK_SEEN_RECIPE;
use pumpkin_macros::java_packet;
use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
#[java_packet(PLAY_RECIPE_BOOK_SEEN_RECIPE)]
pub struct SRecipeBookSeenRecipe {
    pub recipe_display_id: VarInt,
}
