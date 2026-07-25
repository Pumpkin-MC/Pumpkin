use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_CLIENT_INFORMATION;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(PLAY_CLIENT_INFORMATION)]
pub struct SClientInformationPlay<'a> {
    pub locale: &'a str, // 16
    pub view_distance: i8,
    pub chat_mode: VarInt, // VarInt
    pub chat_colors: bool,
    pub skin_parts: u8,
    pub main_hand: VarInt,
    pub text_filtering: bool,
    pub server_listing: bool,
}

impl<'a> ServerPacket<'a> for SClientInformationPlay<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            locale: bytebuf.get_str_borrowed()?,
            view_distance: bytebuf.get_i8()?,
            chat_mode: bytebuf.get_var_int()?,
            chat_colors: bytebuf.get_bool()?,
            skin_parts: bytebuf.get_u8()?,
            main_hand: bytebuf.get_var_int()?,
            text_filtering: bytebuf.get_bool()?,
            server_listing: bytebuf.get_bool()?,
        })
    }
}
