use std::io::{Error, Read};

use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketRead};

/// Sent by a Bedrock client (including split-screen sub-clients) when cleanly leaving the server.
#[packet(5)]
pub struct SDisconnect {
    /// Disconnect reason code reported by the client.
    pub reason: VarInt,
    /// Whether the message fields are omitted on the wire.
    pub skip_message: bool,
    /// Optional disconnect explanation message.
    pub message: String,
    /// Optional profanity-filtered disconnect message.
    pub filtered_message: String,
}

impl PacketRead for SDisconnect {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let reason = VarInt::read(reader)?;
        let skip_message = bool::read(reader)?;
        let (message, filtered_message) = if skip_message {
            (String::new(), String::new())
        } else {
            (String::read(reader)?, String::read(reader)?)
        };

        Ok(Self {
            reason,
            skip_message,
            message,
            filtered_message,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{
        bedrock::client::disconnect::CDisconnect,
        serial::{PacketRead, PacketWrite},
    };

    use super::*;

    #[test]
    fn roundtrips_disconnect_with_and_without_message() {
        for packet in [
            CDisconnect::new(0, String::new()),
            CDisconnect {
                reason: VarInt(5),
                skip_message: false,
                message: "Leaving".to_string(),
                filtered_message: "Leaving".to_string(),
            },
        ] {
            let mut buf = Vec::new();
            packet.write(&mut buf).expect("write CDisconnect");

            let decoded = SDisconnect::read(&mut Cursor::new(buf)).expect("read SDisconnect");
            assert_eq!(decoded.reason.0, packet.reason.0);
            assert_eq!(decoded.skip_message, packet.skip_message);
            assert_eq!(decoded.message, packet.message);
            assert_eq!(decoded.filtered_message, packet.filtered_message);
        }
    }
}
