use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_ACCEPT_TELEPORTATION;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(PLAY_ACCEPT_TELEPORTATION)]
pub struct SConfirmTeleport {
    pub teleport_id: VarInt,
}

impl<'a> ServerPacket<'a> for SConfirmTeleport {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            teleport_id: bytebuf.get_var_int()?,
        })
    }
}
