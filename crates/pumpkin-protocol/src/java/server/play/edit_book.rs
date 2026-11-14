use pumpkin_data::packet::serverbound::play::EDIT_BOOK;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};

/// `WritableBookContent.MAX_PAGES`
const MAX_PAGES: usize = 100;
/// `WritableBookContent.PAGE_EDIT_LENGTH`
const PAGE_EDIT_LENGTH: usize = 1024;
/// `WrittenBookContent.TITLE_MAX_LENGTH`
const TITLE_MAX_LENGTH: usize = 32;

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
            let count = read.get_var_int()?.0;
            let max_pages = if *version >= JavaMinecraftVersion::V_1_21_2 {
                MAX_PAGES
            } else {
                200
            };
            // Vanilla's list codec rejects the packet instead of truncating it.
            let count = usize::try_from(count)
                .ok()
                .filter(|&count| count <= max_pages)
                .ok_or_else(|| {
                    ReadingError::TooLarge(format!(
                        "{count} book pages exceeded max size of {max_pages}"
                    ))
                })?;
            let char_limit = if *version >= JavaMinecraftVersion::V_1_21_2 {
                PAGE_EDIT_LENGTH
            } else {
                8192
            };
            let mut pages = Vec::with_capacity(count);
            for _ in 0..count {
                pages.push(read.get_str_bounded_borrowed(char_limit)?);
            }
            let has_title = read.get_bool()?;
            let title_limit = if *version >= JavaMinecraftVersion::V_1_21_2 {
                TITLE_MAX_LENGTH
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Slot 0, `declared` page count, `sent` one-byte pages, no title.
    fn payload(declared: u8, sent: usize) -> Vec<u8> {
        let mut bytes = vec![0x00, declared];
        for _ in 0..sent {
            bytes.extend_from_slice(&[0x01, b'a']);
        }
        bytes.push(0x00);
        bytes
    }

    #[test]
    fn accepts_the_maximum_page_count() -> Result<(), ReadingError> {
        let bytes = payload(100, 100);
        let packet = SEditBook::read(&mut bytes.as_slice(), &JavaMinecraftVersion::V_26_3)?;
        assert_eq!(packet.pages.len(), 100);
        assert_eq!(packet.title, None);
        Ok(())
    }

    #[test]
    fn round_trips_a_signed_book_at_the_limits() -> Result<(), Box<dyn std::error::Error>> {
        let page = "a".repeat(PAGE_EDIT_LENGTH);
        let title = "t".repeat(TITLE_MAX_LENGTH);
        let packet = SEditBook {
            slot: VarInt(40),
            pages: vec![page.as_str(); MAX_PAGES],
            title: Some(&title),
        };
        let mut encoded = Vec::new();
        crate::ClientPacket::write_packet_data(
            &packet,
            &mut encoded,
            &JavaMinecraftVersion::V_26_3,
        )?;

        let mut payload = encoded.as_slice();
        let decoded = SEditBook::read(&mut payload, &JavaMinecraftVersion::V_26_3)?;
        assert_eq!(decoded.slot.0, 40);
        assert_eq!(decoded.pages, packet.pages);
        assert_eq!(decoded.title, Some(title.as_str()));
        assert!(payload.is_empty());
        Ok(())
    }

    #[test]
    fn rejects_too_many_pages_instead_of_truncating() {
        let bytes = payload(101, 100);
        let result = SEditBook::read(&mut bytes.as_slice(), &JavaMinecraftVersion::V_26_3);
        assert!(matches!(result, Err(ReadingError::TooLarge(_))));
    }

    #[test]
    fn rejects_a_negative_page_count() {
        let bytes = [0x00, 0xff, 0xff, 0xff, 0xff, 0x0f, 0x00];
        let result = SEditBook::read(&mut bytes.as_slice(), &JavaMinecraftVersion::V_26_3);
        assert!(matches!(result, Err(ReadingError::TooLarge(_))));
    }
}
