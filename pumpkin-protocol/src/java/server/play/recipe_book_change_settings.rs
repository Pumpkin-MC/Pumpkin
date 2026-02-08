use pumpkin_data::packet::serverbound::PLAY_RECIPE_BOOK_CHANGE_SETTINGS;
use pumpkin_macros::java_packet;

use crate::VarInt;

/// Sent when the player changes recipe book settings (open/filter toggles).
#[derive(serde::Deserialize, serde::Serialize)]
#[java_packet(PLAY_RECIPE_BOOK_CHANGE_SETTINGS)]
pub struct SRecipeBookChangeSettings {
    /// 0 = crafting, 1 = furnace, 2 = blast furnace, 3 = smoker.
    pub book_type: VarInt,
    pub book_open: bool,
    pub filter_active: bool,
}
