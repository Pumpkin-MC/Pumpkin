use pumpkin_data::packet::serverbound::play::CONTAINER_SLOT_STATE_CHANGED;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(CONTAINER_SLOT_STATE_CHANGED)]
pub struct SContainerSlotStateChanged {
    pub slot_id: VarInt,
    pub container_id: VarInt,
    pub new_state: bool,
}

impl<'a> ServerPacket<'a> for SContainerSlotStateChanged {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        let slot_id = bytebuf.get_var_int()?;
        let container_id = bytebuf.get_container_id()?;
        let new_state = bytebuf.get_bool()?;

        Ok(Self {
            slot_id,
            container_id,
            new_state,
        })
    }
}

impl crate::ClientPacket for SContainerSlotStateChanged {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.slot_id)?;
        write.write_container_id(&self.container_id)?;
        write.write_bool(self.new_state)?;
        Ok(())
    }
}
