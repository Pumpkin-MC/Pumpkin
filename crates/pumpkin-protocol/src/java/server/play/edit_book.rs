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
    fn read(read: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        if *version >= JavaMinecraftVersion::V_1_17_1 {
            let slot = read.get_var_int()?;
            let count = read.get_var_int()?.0 as usize;
            let max_pages = if *version >= JavaMinecraftVersion::V_1_21_2 {
                100
            } else {
                200
            };
            let count = count.min(max_pages);
            let char_limit = if *version >= JavaMinecraftVersion::V_1_21_2 {
                1024
            } else {
                8192
            };
            let mut pages = Vec::with_capacity(count);
            for _ in 0..count {
                pages.push(read.get_str_bounded_borrowed(char_limit)?);
            }
            let has_title = read.get_bool()?;
            let title_limit = if *version >= JavaMinecraftVersion::V_1_21_2 {
                32
            } else {
                128
            };
            let title = if has_title {
                Some(read.get_str_bounded_borrowed(title_limit)?)
            } else {
                None
            };
            Ok(Self { slot, pages, title })
        } else {
            let _item = crate::codec::item_stack_seralizer::ItemStackSerializer::read_with_version(
                read, version,
            )?;
            let _signing = read.get_bool()?;
            let slot = read.get_var_int()?;
            Ok(Self {
                slot,
                pages: Vec::new(),
                title: None,
            })
        }
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
