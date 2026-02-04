use std::io::Read;

use pumpkin_data::packet::serverbound::PLAY_EDIT_BOOK;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, ReadingError},
};

const MAX_PAGE_COUNT: usize = 100;
const MAX_PAGE_LENGTH: usize = 8192;
const MAX_TITLE_LENGTH: usize = 128;

#[java_packet(PLAY_EDIT_BOOK)]
pub struct SEditBook {
    pub slot: VarInt,
    pub pages: Vec<String>,
    pub title: Option<String>,
}

impl ServerPacket for SEditBook {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;
        let slot = read.get_var_int()?;
        let page_count = read.get_var_int()?;
        let count = usize::try_from(page_count.0).map_err(|_| {
            ReadingError::Message("Invalid edit book page count".to_string())
        })?;
        if count > MAX_PAGE_COUNT {
            return Err(ReadingError::Message("Too many book pages".to_string()));
        }

        let mut pages = Vec::with_capacity(count);
        for _ in 0..count {
            pages.push(read.get_string_bounded(MAX_PAGE_LENGTH)?);
        }

        let title = read.get_option(|value| value.get_string_bounded(MAX_TITLE_LENGTH))?;

        Ok(Self { slot, pages, title })
    }
}
