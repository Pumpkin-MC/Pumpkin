use crate::{
    ServerPacket, VarInt,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_DEBUG_SAMPLE_SUBSCRIPTION;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_DEBUG_SAMPLE_SUBSCRIPTION)]
pub struct SDebugSampleSubscription {
    pub sample_type: VarInt,
}

impl<'a> ServerPacket<'a> for SDebugSampleSubscription {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            sample_type: bytebuf.get_var_int()?,
        })
    }
}
