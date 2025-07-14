use std::io::{Error, Write};

use pumpkin_macros::packet;

use crate::{BClientPacket, codec::var_uint::VarUInt, serial::PacketWrite};

#[packet(145)]
pub struct CreativeContent<'a> {
    // https://mojang.github.io/bedrock-protocol-docs/html/CreativeContentPacket.html
    pub groups: &'a [u8],
    pub entries: &'a [u8],
}

impl BClientPacket for CreativeContent<'_> {
    fn write_packet(&self, mut writer: impl Write) -> Result<(), Error> {
        VarUInt(self.groups.len() as _).write(&mut writer)?;
        writer.write_all(self.groups)?;

        VarUInt(self.entries.len() as _).write(&mut writer)?;
        writer.write_all(self.entries)
    }
}

#[repr(i32)]
#[allow(unused)]
enum CreativeCategory {
    Construction = 1,
    Nature = 2,
    Equipment = 3,
    Items = 4,
    CommandOnly = 5,
    Undefined = 6,
}
