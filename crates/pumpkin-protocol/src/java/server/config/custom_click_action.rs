use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_data::packet::serverbound::CONFIG_CUSTOM_CLICK_ACTION;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONFIG_CUSTOM_CLICK_ACTION)]
pub struct SCustomClickAction<'a> {
    pub action_id: &'a str,
    pub payload: Option<&'a [u8]>,
}

impl<'a> ServerPacket<'a> for SCustomClickAction<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            action_id: bytebuf.get_str_borrowed()?,
            payload: bytebuf.get_option(|v| v.read_remaining_slice_borrowed(32767))?,
        })
    }
}

impl crate::ClientPacket for SCustomClickAction<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_string(self.action_id)?;
        if let Some(payload) = self.payload {
            write.write_bool(true)?;
            write.write_slice(payload)?;
        } else {
            write.write_bool(false)?;
        }
        Ok(())
    }
}
