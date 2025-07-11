use pumpkin_macros::packet;

use crate::{ClientPacket, codec::var_uint::VarUInt, ser::NetworkWriteExt};

#[packet(145)]
pub struct CreativeContent<'a> {
    // https://mojang.github.io/bedrock-protocol-docs/html/CreativeContentPacket.html
    pub groups: &'a [u8],
    pub entries: &'a [u8],
}

impl ClientPacket for CreativeContent<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_uint(&VarUInt(self.groups.len() as _))?;
        write.write_slice(self.groups)?;

        write.write_var_uint(&VarUInt(self.entries.len() as _))?;
        write.write_slice(self.entries)?;
        Ok(())
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
