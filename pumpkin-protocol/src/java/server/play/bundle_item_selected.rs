use pumpkin_data::packet::serverbound::PLAY_BUNDLE_ITEM_SELECTED;
use pumpkin_macros::java_packet;

use crate::VarInt;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_BUNDLE_ITEM_SELECTED)]
pub struct SBundleItemSelected {
    pub slot_id: VarInt,
    pub selected_item_index: VarInt,
}

impl<'a> ServerPacket<'a> for SBundleItemSelected {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            slot_id: bytebuf.get_var_int()?,
            selected_item_index: bytebuf.get_var_int()?,
        })
    }
}
