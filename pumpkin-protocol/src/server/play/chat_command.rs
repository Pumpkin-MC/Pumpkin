use std::io::Read;

use pumpkin_data::packet::serverbound::PLAY_CHAT_COMMAND;
use pumpkin_macros::packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

#[packet(PLAY_CHAT_COMMAND)]
pub struct SChatCommand {
    pub command: String, // 32767
}

const MAX_COMMAND_LENGTH: usize = 32767;

impl ServerPacket for SChatCommand {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;

        Ok(Self {
            command: read.get_string_bounded(MAX_COMMAND_LENGTH)?
        })
    }
}