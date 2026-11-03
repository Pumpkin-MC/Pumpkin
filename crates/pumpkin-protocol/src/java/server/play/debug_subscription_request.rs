use crate::{
    ServerPacket, VarInt,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::DEBUG_SUBSCRIPTION_REQUEST;
use pumpkin_macros::java_packet;

#[java_packet(DEBUG_SUBSCRIPTION_REQUEST)]
pub struct SDebugSubscriptionRequest {
    pub sample_type: VarInt,
}

impl SDebugSubscriptionRequest {
    pub const TICK_TIME: i32 = 0;
}

impl<'a> ServerPacket<'a> for SDebugSubscriptionRequest {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        Ok(Self {
            sample_type: bytebuf.get_var_int()?,
        })
    }
}

impl crate::ClientPacket for SDebugSubscriptionRequest {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.sample_type)?;
        Ok(())
    }
}
