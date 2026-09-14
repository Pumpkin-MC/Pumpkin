use pumpkin_data::packet::clientbound::play::OPEN_BOOK;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::ser::NetworkWriteExt;
use crate::{ClientPacket, VarInt};

#[derive(Clone, Copy, Debug)]
#[java_packet(OPEN_BOOK)]
pub struct COpenBook {
    pub hand: VarInt,
}

impl COpenBook {
    #[must_use]
    pub const fn new(hand: VarInt) -> Self {
        Self { hand }
    }
}

impl ClientPacket for COpenBook {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.hand)?;
        Ok(())
    }
}
