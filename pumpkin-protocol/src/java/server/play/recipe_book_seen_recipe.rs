use pumpkin_data::packet::serverbound::PLAY_RECIPE_BOOK_SEEN_RECIPE;
use pumpkin_macros::java_packet;

use crate::VarInt;

/// Sent when the player views a newly unlocked recipe (clears the notification).
#[derive(serde::Deserialize, serde::Serialize)]
#[java_packet(PLAY_RECIPE_BOOK_SEEN_RECIPE)]
pub struct SRecipeBookSeenRecipe {
    pub recipe_id: VarInt,
}
