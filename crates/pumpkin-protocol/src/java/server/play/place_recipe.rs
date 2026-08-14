use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_PLACE_RECIPE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(PLAY_PLACE_RECIPE)]
pub struct SPlaceRecipe {
    pub container_id: i8,
    pub recipe_display_id: VarInt,
    pub use_max_items: bool,
}

impl<'a> ServerPacket<'a> for SPlaceRecipe {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            container_id: bytebuf.get_i8()?,
            recipe_display_id: bytebuf.get_var_int()?,
            use_max_items: bytebuf.get_bool()?,
        })
    }
}
