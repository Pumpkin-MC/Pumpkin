use crate::VarInt;
use pumpkin_data::packet::serverbound::PLAY_CONTAINER_BUTTON_CLICK;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[derive(Debug)]
#[java_packet(PLAY_CONTAINER_BUTTON_CLICK)]
pub struct SContainerButtonClick {
    pub window_id: VarInt,
    pub button_id: VarInt,
}

impl<'a> ServerPacket<'a> for SContainerButtonClick {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            window_id: bytebuf.get_var_int()?,
            button_id: bytebuf.get_var_int()?,
        })
    }
}
