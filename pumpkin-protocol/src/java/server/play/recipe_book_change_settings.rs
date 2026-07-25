use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_RECIPE_BOOK_CHANGE_SETTINGS;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(PLAY_RECIPE_BOOK_CHANGE_SETTINGS)]
pub struct SRecipeBookChangeSettings {
    pub book_type: VarInt,
    pub is_open: bool,
    pub is_filtering: bool,
}

impl<'a> ServerPacket<'a> for SRecipeBookChangeSettings {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            book_type: bytebuf.get_var_int()?,
            is_open: bytebuf.get_bool()?,
            is_filtering: bytebuf.get_bool()?,
        })
    }
}
