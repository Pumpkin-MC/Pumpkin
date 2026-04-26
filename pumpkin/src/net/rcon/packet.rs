use std::string::FromUtf8Error;

use bytes::{BufMut, BytesMut};
use thiserror::Error;

/// Client -> Server
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundPacket {
    /// Typically, the first packet sent by the client, which is used to authenticate the connection with the server.
    Auth = 2,
    /// This packet type represents a command issued by a client to the server. This can be a `ConCommand` such as /kill <player> or /weather clear.
    /// The response will vary depending on the command issued.
    ExecCommand = 3,
}

impl ServerboundPacket {
    pub const fn from_i32(n: i32) -> Self {
        match n {
            //  3 => Self::Auth,
            2 => Self::ExecCommand,
            _ => Self::Auth,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Server -> Client
pub enum ClientboundPacket {
    /// This packet is a notification of the connection's current auth status. When the server receives an auth request, it will respond with an empty `SERVERDATA_RESPONSE_VALUE`,
    /// followed immediately by a `SERVERDATA_AUTH_RESPONSE` indicating whether authentication succeeded or failed. Note that the status code is returned in the packet id field, so when pairing the response with the original auth request, you may need to look at the packet id of the `SERVERDATA_RESPONSE_VALUE`.
    AuthResponse = 2,
    /// A `SERVERDATA_RESPONSE` packet is the response to a `SERVERDATA_EXECCOMMAND` request.
    Output = 0,
}

impl ClientboundPacket {
    pub fn write_buf(self, id: i32, body: &str) -> BytesMut {
        // let len = outgoing.len() as u64;
        let mut buf = BytesMut::new();
        // 10 is for 4 bytes ty, 4 bytes id, and 2 terminating nul bytes.
        buf.put_i32_le(10 + body.len() as i32);
        buf.put_i32_le(id);
        buf.put_i32_le(self as i32);
        let bytes = body.as_bytes();
        buf.put_slice(bytes);
        buf.put_u8(0);
        buf.put_u8(0);
        buf
    }
}

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("Invalid length")]
    InvalidLength,
    #[error("Dailed to send packet")]
    FailedSend(std::io::Error),
    #[error("Missing packet lull terminator")]
    MissingNullTerminator,
    #[error("Invalid packet string body")]
    InvalidBody(FromUtf8Error),
}

#[derive(Debug)]
/// Serverbound packet
pub struct Packet {
    id: i32,
    ptype: ServerboundPacket,
    body: String,
}

impl Packet {
    pub fn deserialize(incoming: &mut Vec<u8>) -> Result<Option<Self>, PacketError> {
        // We need at least 4 bytes to read the packet length header
        if incoming.len() < 4 {
            return Ok(None);
        }

        // Read the 4-byte length header (little-endian)
        let len_bytes: [u8; 4] = incoming[0..4].try_into().unwrap();
        let size = i32::from_le_bytes(len_bytes);

        // An RCON packet size includes: ID(4) + Type(4) + Body(n) + Null(1) + EmptyStringNull(1).
        // Minimum size for an empty body is 10. Maximum MTU size is typically 1460.
        if !(10..=1460).contains(&size) {
            return Err(PacketError::InvalidLength);
        }

        // Total bytes we need in the buffer (length header + the payload size)
        let total_packet_len = (size as usize) + 4;

        // If the network hasn't delivered the full packet yet, return Ok(None)
        if incoming.len() < total_packet_len {
            return Ok(None);
        }

        // We have the full packet. Parse it synchronously using fixed slices.
        let id = i32::from_le_bytes(incoming[4..8].try_into().unwrap());
        let ty = i32::from_le_bytes(incoming[8..12].try_into().unwrap());

        // Calculate body boundaries (starts after len, id, and ty -> 4+4+4 = 12)
        // Ends 2 bytes before the total length (excluding the two trailing null bytes)
        let body_start = 12;
        let body_end = total_packet_len - 2;

        if incoming[body_end] != 0 || incoming[body_end + 1] != 0 {
            return Err(PacketError::MissingNullTerminator);
        }

        let payload = &incoming[body_start..body_end];
        let body = String::from_utf8(payload.to_vec()).map_err(PacketError::InvalidBody)?;

        incoming.drain(0..total_packet_len);

        Ok(Some(Self {
            id,
            ptype: ServerboundPacket::from_i32(ty),
            body,
        }))
    }

    pub fn get_body(&self) -> &str {
        &self.body
    }

    pub const fn get_type(&self) -> ServerboundPacket {
        self.ptype
    }

    pub const fn get_id(&self) -> i32 {
        self.id
    }
}
