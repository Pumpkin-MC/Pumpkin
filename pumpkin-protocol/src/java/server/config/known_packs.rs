use pumpkin_data::packet::serverbound::CONFIG_SELECT_KNOWN_PACKS;
use pumpkin_macros::java_packet;

use crate::VarInt;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONFIG_SELECT_KNOWN_PACKS)]
pub struct SKnownPacks {
    pub known_pack_count: VarInt,
    // known_packs: &'a [KnownPack]
}

impl<'a> ServerPacket<'a> for SKnownPacks {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            known_pack_count: bytebuf.get_var_int()?,
        })
    }
}
