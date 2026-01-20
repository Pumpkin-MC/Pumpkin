use pumpkin_macros::packet;
use std::io::{Error, Write};

use crate::{codec::var_int::VarInt, serial::PacketWrite};

#[packet(5)]
pub struct CDisconnectPlayer {
    // https://mojang.github.io/bedrock-protocol-docs/html/DisconnectPacket.html
    pub reason: VarInt,
    pub skip_message: bool,
    pub message: String,
    pub filtered_message: String,
}

impl CDisconnectPlayer {
    #[must_use] 
    pub const fn new(reason: i32, message: String) -> Self {
        Self {
            reason: VarInt(reason),
            skip_message: message.is_empty(),
            message,
            filtered_message: String::new(),
        }
    }
}

impl PacketWrite for CDisconnectPlayer {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.reason.write(writer)?;
        self.skip_message.write(writer)?;
        if !self.skip_message {
            self.message.write(writer)?;
            self.filtered_message.write(writer)?;
        }
        Ok(())
    }
}
