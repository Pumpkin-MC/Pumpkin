use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_RECIPE_BOOK_SEEN_RECIPE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(PLAY_RECIPE_BOOK_SEEN_RECIPE)]
pub struct SRecipeBookSeenRecipe {
    pub recipe_display_id: VarInt,
}

impl<'a> ServerPacket<'a> for SRecipeBookSeenRecipe {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            recipe_display_id: bytebuf.get_var_int()?,
        })
    }
}
