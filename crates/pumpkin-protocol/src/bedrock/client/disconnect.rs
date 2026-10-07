// Last verified for v2169

use pumpkin_macros::packet;
use std::io::{Error, Write};

use crate::{codec::var_int::VarInt, serial::PacketWrite};

/// Sent by the server to cleanly disconnect a client with an optional error code and kick message.
#[packet(5)]
pub struct CDisconnect {
    /// Disconnect reason code determining the message shown on the client disconnect screen.
    pub reason: VarInt,
    /// Whether to skip showing the disconnect message dialog and return directly to the main menu.
    pub skip_message: bool,
    /// Disconnect explanation message displayed to the user.
    pub message: String,
    /// Profanity-filtered variant of the disconnect message.
    pub filtered_message: String,
}

impl CDisconnect {
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

impl PacketWrite for CDisconnect {
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
