use crate::VarInt;
use pumpkin_data::packet::serverbound::play::CONTAINER_BUTTON_CLICK;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

#[derive(Debug)]
#[java_packet(CONTAINER_BUTTON_CLICK)]
pub struct SContainerButtonClick {
    pub window_id: VarInt,
    pub button_id: VarInt,
}

impl<'a> ServerPacket<'a> for SContainerButtonClick {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        let window_id = bytebuf.get_container_id()?;
        let button_id = bytebuf.get_var_int()?;
        Ok(Self {
            window_id,
            button_id,
        })
    }
}

impl crate::ClientPacket for SContainerButtonClick {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_container_id(&self.window_id)?;
        write.write_var_int(&self.button_id)?;
        Ok(())
    }
}
