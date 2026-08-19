use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_SEEN_ADVANCEMENTS;
use pumpkin_macros::java_packet;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::version::JavaMinecraftVersion;

#[derive(Debug)]
#[java_packet(PLAY_SEEN_ADVANCEMENTS)]
pub enum SSeenAdvancement {
    OpenTab(Identifier),
    CloseTab,
}

impl<'a> ServerPacket<'a> for SSeenAdvancement {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let action = bytebuf.get_var_int()?;
        match action.0 {
            0 => {
                let id_str = bytebuf.get_str_borrowed()?;
                let id =
                    Identifier::parse(id_str).map_err(|e| ReadingError::Message(e.to_string()))?;
                Ok(Self::OpenTab(id))
            }
            1 => Ok(Self::CloseTab),
            _ => Err(ReadingError::Message(format!(
                "Invalid SeenAdvancement action: {}",
                action.0
            ))),
        }
    }
}

impl crate::ClientPacket for SSeenAdvancement {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::{VarInt, ser::NetworkWriteExt};
        match self {
            Self::OpenTab(id) => {
                write.write_var_int(&VarInt(0))?;
                write.write_string(&id.to_string())?;
            }
            Self::CloseTab => {
                write.write_var_int(&VarInt(1))?;
            }
        }
        Ok(())
    }
}
