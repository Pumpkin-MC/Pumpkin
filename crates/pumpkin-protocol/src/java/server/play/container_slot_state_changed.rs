use pumpkin_data::packet::serverbound::PLAY_CONTAINER_SLOT_STATE_CHANGED;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_CONTAINER_SLOT_STATE_CHANGED)]
pub struct SContainerSlotStateChanged {
    pub slot_id: VarInt,
    pub container_id: VarInt,
    pub new_state: bool,
}

impl<'a> ServerPacket<'a> for SContainerSlotStateChanged {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            slot_id: bytebuf.get_var_int()?,
            container_id: bytebuf.get_var_int()?,
            new_state: bytebuf.get_bool()?,
        })
    }
}
