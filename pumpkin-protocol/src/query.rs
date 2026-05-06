use std::{ffi::CString, fmt, io::Cursor};

use tokio::io::AsyncReadExt;

// Padding bytes are fixed by the vanilla server implementation.
// Some query checkers (incorrectly) depend on these exact bytes.
const PADDING_START: [u8; 11] = [
    0x73, 0x70, 0x6C, 0x69, 0x74, 0x6E, 0x75, 0x6D, 0x00, 0x80, 0x00,
];

// 11 bytes: 10 padding bytes + 1 null terminator for the key-value section.
const PADDING_END: [u8; 11] = [
    0x00, 0x01, 0x70, 0x6C, 0x61, 0x79, 0x65, 0x72, 0x5F, 0x00, 0x00,
];

/// The magic number that all Query protocol packets start with.
const MAGIC: u16 = 0xFEFD;

#[derive(Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// The byte stream ended before a complete packet could be read.
    UnexpectedEof,
    /// The first two bytes were not the expected magic value (0xFEFD).
    BadMagic,
    /// The packet type byte does not correspond to a known type.
    UnknownPacketType(u8),
    /// The packet was structurally valid but contained an unexpected payload length.
    MalformedPayload,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "unexpected end of packet"),
            Self::BadMagic => write!(f, "bad magic bytes (expected 0xFEFD)"),
            Self::UnknownPacketType(t) => write!(f, "unknown packet type: {t:#04x}"),
            Self::MalformedPayload => write!(f, "malformed packet payload"),
        }
    }
}

impl std::error::Error for DecodeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    Handshake = 9,
    Status = 0,
}

impl TryFrom<u8> for PacketType {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            9 => Ok(Self::Handshake),
            0 => Ok(Self::Status),
            other => Err(DecodeError::UnknownPacketType(other)),
        }
    }
}

/// A partially-decoded packet: magic verified, type extracted, payload buffered.
#[derive(Debug, PartialEq, Eq)]
pub struct RawQueryPacket {
    pub packet_type: PacketType,
    reader: Cursor<Vec<u8>>,
}

impl RawQueryPacket {
    pub async fn decode(bytes: Vec<u8>) -> Result<Self, DecodeError> {
        let mut reader = Cursor::new(bytes);

        let magic = reader
            .read_u16()
            .await
            .map_err(|_| DecodeError::UnexpectedEof)?;
        if magic != MAGIC {
            return Err(DecodeError::BadMagic);
        }

        let type_byte = reader
            .read_u8()
            .await
            .map_err(|_| DecodeError::UnexpectedEof)?;
        let packet_type = PacketType::try_from(type_byte)?;

        Ok(Self {
            packet_type,
            reader,
        })
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct SHandshake {
    pub session_id: i32,
}

impl SHandshake {
    pub async fn decode(packet: &mut RawQueryPacket) -> Result<Self, DecodeError> {
        let session_id = packet
            .reader
            .read_i32()
            .await
            .map_err(|_| DecodeError::UnexpectedEof)?;
        Ok(Self { session_id })
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct SStatusRequest {
    pub session_id: i32,
    pub challenge_token: i32,
    /// `true` for a full-status request (payload padded to 8 extra bytes),
    /// `false` for a basic-status request (no padding).
    pub is_full_request: bool,
}

impl SStatusRequest {
    pub async fn decode(packet: &mut RawQueryPacket) -> Result<Self, DecodeError> {
        let session_id = packet
            .reader
            .read_i32()
            .await
            .map_err(|_| DecodeError::UnexpectedEof)?;
        let challenge_token = packet
            .reader
            .read_i32()
            .await
            .map_err(|_| DecodeError::UnexpectedEof)?;

        let position = packet.reader.position();
        let total_len = packet.reader.get_ref().len() as u64;
        let remaining = total_len.saturating_sub(position);

        let is_full_request = match remaining {
            0 => false,
            4 => {
                packet.reader.set_position(position + 4);
                true
            }
            _ => return Err(DecodeError::MalformedPayload),
        };

        Ok(Self {
            session_id,
            challenge_token,
            is_full_request,
        })
    }
}

#[derive(Debug)]
pub struct CHandshake {
    pub session_id: i32,
    /// Encoded as a decimal string in the wire format.
    pub challenge_token: i32,
}

impl CHandshake {
    #[must_use]
    pub fn encode(&self) -> Option<Vec<u8>> {
        let token = CString::new(self.challenge_token.to_string()).ok()?;

        let mut buf = Vec::with_capacity(6 + token.as_bytes_with_nul().len());
        buf.push(PacketType::Handshake as u8);
        buf.extend_from_slice(&self.session_id.to_be_bytes());
        buf.extend_from_slice(token.as_bytes_with_nul());
        Some(buf)
    }
}

#[derive(Debug)]
pub struct CBasicStatus {
    pub session_id: i32,
    pub motd: CString,
    pub map: CString,
    pub num_players: usize,
    pub max_players: usize,
    /// Little-endian on the wire (Notchian quirk).
    pub host_port: u16,
    pub host_ip: CString,
}

impl CBasicStatus {
    #[must_use]
    pub fn encode(&self) -> Option<Vec<u8>> {
        let game_type = CString::new("SMP").ok()?;
        let num_players = CString::new(self.num_players.to_string()).ok()?;
        let max_players = CString::new(self.max_players.to_string()).ok()?;

        let mut buf = Vec::new();
        buf.push(PacketType::Status as u8);
        buf.extend_from_slice(&self.session_id.to_be_bytes());
        buf.extend_from_slice(self.motd.as_bytes_with_nul());
        buf.extend_from_slice(game_type.as_bytes_with_nul());
        buf.extend_from_slice(self.map.as_bytes_with_nul());
        buf.extend_from_slice(num_players.as_bytes_with_nul());
        buf.extend_from_slice(max_players.as_bytes_with_nul());
        // The port is written little-endian — this is a known Notchian quirk.
        buf.extend_from_slice(&self.host_port.to_le_bytes());
        buf.extend_from_slice(self.host_ip.as_bytes_with_nul());
        Some(buf)
    }
}

#[derive(Debug)]
pub struct CFullStatus {
    pub session_id: i32,
    pub hostname: CString,
    pub version: CString,
    pub plugins: CString,
    pub map: CString,
    pub num_players: usize,
    pub max_players: usize,
    /// Little-endian on the wire (Notchian quirk).
    pub host_port: u16,
    pub host_ip: CString,
    pub players: Vec<CString>,
}

impl CFullStatus {
    #[must_use]
    pub fn encode(&self) -> Option<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(PacketType::Status as u8);
        buf.extend_from_slice(&self.session_id.to_be_bytes());
        buf.extend_from_slice(&PADDING_START);

        let kv_pairs: &[(&str, &CString)] = &[
            ("hostname", &self.hostname),
            ("gametype", &CString::new("SMP").ok()?),
            ("game_id", &CString::new("MINECRAFT").ok()?),
            ("version", &self.version),
            ("plugins", &self.plugins),
            ("map", &self.map),
            (
                "numplayers",
                &CString::new(self.num_players.to_string()).ok()?,
            ),
            (
                "maxplayers",
                &CString::new(self.max_players.to_string()).ok()?,
            ),
            ("hostport", &CString::new(self.host_port.to_string()).ok()?),
            ("hostip", &self.host_ip),
        ];

        for (key, value) in kv_pairs {
            buf.extend_from_slice(CString::new(*key).ok()?.as_bytes_with_nul());
            buf.extend_from_slice(value.as_bytes_with_nul());
        }

        // End padding + null terminator for the key-value section.
        buf.extend_from_slice(&PADDING_END);

        // Player list, each null-terminated, followed by a final null.
        for player in &self.players {
            buf.extend_from_slice(player.as_bytes_with_nul());
        }
        buf.push(0x00);

        Some(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn handshake_request() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = vec![0xFE, 0xFD, 0x09, 0x00, 0x00, 0x00, 0x01];
        let mut raw = RawQueryPacket::decode(bytes).await?;
        assert_eq!(raw.packet_type, PacketType::Handshake);
        let pkt = SHandshake::decode(&mut raw).await?;
        assert_eq!(pkt, SHandshake { session_id: 1 });
        Ok(())
    }

    #[tokio::test]
    async fn handshake_response() -> Result<(), Box<dyn std::error::Error>> {
        let expected = vec![
            0x09, 0x00, 0x00, 0x00, 0x01, 0x39, 0x35, 0x31, 0x33, 0x33, 0x30, 0x37, 0x00,
        ];
        let pkt = CHandshake {
            session_id: 1,
            challenge_token: 9513307,
        };
        assert_eq!(pkt.encode().ok_or("Encoding failed")?, expected);
        Ok(())
    }

    #[tokio::test]
    async fn basic_stat_request() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = vec![
            0xFE, 0xFD, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x91, 0x29, 0x5B,
        ];
        let mut raw = RawQueryPacket::decode(bytes).await?;
        let pkt = SStatusRequest::decode(&mut raw).await?;
        assert_eq!(
            pkt,
            SStatusRequest {
                session_id: 1,
                challenge_token: 9513307,
                is_full_request: false
            }
        );
        Ok(())
    }

    #[tokio::test]
    async fn basic_stat_response() -> Result<(), Box<dyn std::error::Error>> {
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x01, 0x41, 0x20, 0x4D, 0x69, 0x6E, 0x65, 0x63, 0x72, 0x61,
            0x66, 0x74, 0x20, 0x53, 0x65, 0x72, 0x76, 0x65, 0x72, 0x00, 0x53, 0x4D, 0x50, 0x00,
            0x77, 0x6F, 0x72, 0x6C, 0x64, 0x00, 0x32, 0x00, 0x32, 0x30, 0x00, 0xDD, 0x63, 0x31,
            0x32, 0x37, 0x2E, 0x30, 0x2E, 0x30, 0x2E, 0x31, 0x00,
        ];
        let pkt = CBasicStatus {
            session_id: 1,
            motd: CString::new("A Minecraft Server")?,
            map: CString::new("world")?,
            num_players: 2,
            max_players: 20,
            host_port: 25565,
            host_ip: CString::new("127.0.0.1")?,
        };
        assert_eq!(pkt.encode().ok_or("Encoding failed")?, expected);
        Ok(())
    }

    #[tokio::test]
    async fn full_stat_request() -> Result<(), Box<dyn std::error::Error>> {
        let bytes = vec![
            0xFE, 0xFD, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x91, 0x29, 0x5B, 0x00, 0x00, 0x00,
            0x00,
        ];
        let mut raw = RawQueryPacket::decode(bytes).await?;
        let pkt = SStatusRequest::decode(&mut raw).await?;
        assert_eq!(
            pkt,
            SStatusRequest {
                session_id: 1,
                challenge_token: 9513307,
                is_full_request: true
            }
        );
        Ok(())
    }

    #[tokio::test]
    async fn full_stat_response() -> Result<(), Box<dyn std::error::Error>> {
        let expected = vec![
            0x00, 0x00, 0x00, 0x00, 0x01, 0x73, 0x70, 0x6C, 0x69, 0x74, 0x6E, 0x75, 0x6D, 0x00,
            0x80, 0x00, 0x68, 0x6F, 0x73, 0x74, 0x6E, 0x61, 0x6D, 0x65, 0x00, 0x41, 0x20, 0x4D,
            0x69, 0x6E, 0x65, 0x63, 0x72, 0x61, 0x66, 0x74, 0x20, 0x53, 0x65, 0x72, 0x76, 0x65,
            0x72, 0x00, 0x67, 0x61, 0x6D, 0x65, 0x74, 0x79, 0x70, 0x65, 0x00, 0x53, 0x4D, 0x50,
            0x00, 0x67, 0x61, 0x6D, 0x65, 0x5F, 0x69, 0x64, 0x00, 0x4D, 0x49, 0x4E, 0x45, 0x43,
            0x52, 0x41, 0x46, 0x54, 0x00, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x42,
            0x65, 0x74, 0x61, 0x20, 0x31, 0x2E, 0x39, 0x20, 0x50, 0x72, 0x65, 0x72, 0x65, 0x6C,
            0x65, 0x61, 0x73, 0x65, 0x20, 0x34, 0x00, 0x70, 0x6C, 0x75, 0x67, 0x69, 0x6E, 0x73,
            0x00, 0x00, 0x6D, 0x61, 0x70, 0x00, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x00, 0x6E, 0x75,
            0x6D, 0x70, 0x6C, 0x61, 0x79, 0x65, 0x72, 0x73, 0x00, 0x32, 0x00, 0x6D, 0x61, 0x78,
            0x70, 0x6C, 0x61, 0x79, 0x65, 0x72, 0x73, 0x00, 0x32, 0x30, 0x00, 0x68, 0x6F, 0x73,
            0x74, 0x70, 0x6F, 0x72, 0x74, 0x00, 0x32, 0x35, 0x35, 0x36, 0x35, 0x00, 0x68, 0x6F,
            0x73, 0x74, 0x69, 0x70, 0x00, 0x31, 0x32, 0x37, 0x2E, 0x30, 0x2E, 0x30, 0x2E, 0x31,
            0x00, 0x00, 0x01, 0x70, 0x6C, 0x61, 0x79, 0x65, 0x72, 0x5F, 0x00, 0x00, 0x62, 0x61,
            0x72, 0x6E, 0x65, 0x79, 0x67, 0x61, 0x6C, 0x65, 0x00, 0x56, 0x69, 0x76, 0x61, 0x6C,
            0x61, 0x68, 0x65, 0x6C, 0x76, 0x69, 0x67, 0x00, 0x00,
        ];
        let pkt = CFullStatus {
            session_id: 1,
            hostname: CString::new("A Minecraft Server")?,
            version: CString::new("Beta 1.9 Prerelease 4")?,
            plugins: CString::new("")?,
            map: CString::new("world")?,
            num_players: 2,
            max_players: 20,
            host_port: 25565,
            host_ip: CString::new("127.0.0.1")?,
            players: vec![CString::new("barneygale")?, CString::new("Vivalahelvig")?],
        };
        assert_eq!(pkt.encode().ok_or("Encoding failed")?, expected);
        Ok(())
    }

    #[tokio::test]
    async fn bad_magic_rejected() {
        let bytes = vec![0xDE, 0xAD, 0x09, 0x00, 0x00, 0x00, 0x01];
        assert_eq!(
            RawQueryPacket::decode(bytes).await,
            Err(DecodeError::BadMagic)
        );
    }

    #[tokio::test]
    async fn unknown_packet_type_rejected() {
        let bytes = vec![0xFE, 0xFD, 0xFF];
        assert_eq!(
            RawQueryPacket::decode(bytes).await,
            Err(DecodeError::UnknownPacketType(0xFF))
        );
    }

    #[tokio::test]
    async fn truncated_packet_rejected() {
        let bytes = vec![0xFE, 0xFD];
        assert_eq!(
            RawQueryPacket::decode(bytes).await,
            Err(DecodeError::UnexpectedEof)
        );
    }
}
