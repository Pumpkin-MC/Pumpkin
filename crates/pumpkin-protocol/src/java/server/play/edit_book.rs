use pumpkin_data::packet::serverbound::play::EDIT_BOOK;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};

#[derive(Debug)]
#[java_packet(EDIT_BOOK)]
pub struct SEditBook<'a> {
    pub slot: VarInt,
    pub pages: Vec<&'a str>,
    pub title: Option<&'a str>,
}

impl<'a> ServerPacket<'a> for SEditBook<'a> {
    fn read(read: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let slot = read.get_var_int()?;
        let count = read.get_var_int()?.0 as usize;
        let count = count.min(100);
        let mut pages = Vec::with_capacity(count);
        for _ in 0..count {
            pages.push(read.get_str_bounded_borrowed(1024)?);
        }
        let has_title = read.get_bool()?;
        let title = if has_title {
            Some(read.get_str_bounded_borrowed(128)?)
        } else {
            None
        };
        Ok(Self { slot, pages, title })
    }
}

impl crate::ClientPacket for SEditBook<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.slot)?;
        write.write_var_int(&VarInt(self.pages.len() as i32))?;
        for page in &self.pages {
            write.write_string_bounded(page, 1024)?;
        }
        if let Some(title) = self.title {
            write.write_bool(true)?;
            write.write_string_bounded(title, 128)?;
        } else {
            write.write_bool(false)?;
        }
        Ok(())
    }
}
