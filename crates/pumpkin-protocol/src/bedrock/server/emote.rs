use crate::codec::{var_uint::VarUInt, var_ulong::VarULong};
use crate::serial::{PacketRead, PacketReadSlice, PacketWrite, read_str_slice};
use pumpkin_macros::packet;
use std::borrow::Cow;
use std::io::{Error, Write};

pub const EMOTE_FLAG_SERVER_SIDE: u8 = 1 << 0;
pub const EMOTE_FLAG_MUTE_CHAT: u8 = 1 << 1;

#[derive(Debug)]
#[packet(138)]
pub struct SEmote<'a> {
    pub runtime_entity_id: VarULong,
    pub emote_length: VarUInt,
    pub emote_id: Cow<'a, str>,
    pub xuid: Cow<'a, str>,
    pub platform_id: Cow<'a, str>,
    pub flags: u8,
}

impl<'a> PacketReadSlice<'a> for SEmote<'a> {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let runtime_entity_id = VarULong::read_slice(buf)?;
        let emote_length = VarUInt::read_slice(buf)?;
        let emote_id = Cow::Borrowed(read_str_slice(buf)?);
        let xuid = Cow::Borrowed(read_str_slice(buf)?);
        let platform_id = Cow::Borrowed(read_str_slice(buf)?);
        let flags = u8::read_slice(buf)?;
        Ok(Self {
            runtime_entity_id,
            emote_length,
            emote_id,
            xuid,
            platform_id,
            flags,
        })
    }
}

impl PacketRead for SEmote<'static> {
    fn read<R: std::io::Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(Self {
            runtime_entity_id: VarULong::read(reader)?,
            emote_length: VarUInt::read(reader)?,
            emote_id: Cow::Owned(String::read(reader)?),
            xuid: Cow::Owned(String::read(reader)?),
            platform_id: Cow::Owned(String::read(reader)?),
            flags: u8::read(reader)?,
        })
    }
}

impl PacketWrite for SEmote<'_> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.runtime_entity_id.write(writer)?;
        self.emote_length.write(writer)?;
        self.emote_id.as_ref().write(writer)?;
        self.xuid.as_ref().write(writer)?;
        self.platform_id.as_ref().write(writer)?;
        self.flags.write(writer)
    }
}
