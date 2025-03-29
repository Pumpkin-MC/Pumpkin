use std::{borrow::Cow, io::Read};

use crate::ser::NetworkReadExt;
use pumpkin_data::packet::serverbound::LOGIN_HELLO;
use pumpkin_macros::packet;

use crate::{ServerPacket, ser::ReadingError};

#[packet(LOGIN_HELLO)]
pub struct SLoginStart<'a> {
    pub name: Cow<'a, str>, // 16
    pub uuid: uuid::Uuid,
}

impl ServerPacket for SLoginStart<'_> {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;

        Ok(Self {
            name: Cow::Owned(read.get_string_bounded(16)?),
            uuid: read.get_uuid()?,
        })
    }
}
