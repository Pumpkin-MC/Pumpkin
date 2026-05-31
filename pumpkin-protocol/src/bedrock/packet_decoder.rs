use std::{
    io::{Cursor, Read},
    pin::Pin,
    task::{Context, Poll},
};

use async_compression::tokio::bufread::DeflateDecoder;
use tokio::io::{AsyncRead, BufReader, ReadBuf};

use crate::{
    Aes128Cfb8Dec, CompressionThreshold, MAX_PACKET_DATA_SIZE, PacketDecodeError, RawPacket,
    StreamDecryptor, codec::var_uint::VarUInt, ser::ReadingError,
};

pub enum DecryptionReader<R: AsyncRead + Unpin> {
    Decrypt(Box<StreamDecryptor<R>>),
    None(R),
}

impl<R: AsyncRead + Unpin> DecryptionReader<R> {
    #[must_use]
    pub fn upgrade(self, cipher: Aes128Cfb8Dec) -> Self {
        match self {
            Self::None(stream) => Self::Decrypt(Box::new(StreamDecryptor::new(cipher, stream))),
            Self::Decrypt(_) => self,
        }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for DecryptionReader<R> {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Decrypt(reader) => {
                let reader = Pin::new(reader);
                reader.poll_read(cx, buf)
            }
            Self::None(reader) => {
                let reader = Pin::new(reader);
                reader.poll_read(cx, buf)
            }
        }
    }
}

/// Decoder: Client -> Server
/// Supports `ZLib` decoding/decompression
/// Supports Aes128 Encryption
pub struct UDPNetworkDecoder {
    compression: Option<CompressionThreshold>,
}

impl Default for UDPNetworkDecoder {
    fn default() -> Self {
        Self::new()
    }
}

use thiserror::Error;

#[derive(Debug, Error)]
#[error("Encryption already enabled")]
pub struct EncryptionAlreadyEnabledError;

impl UDPNetworkDecoder {
    #[must_use]
    pub const fn new() -> Self {
        Self { compression: None }
    }

    pub const fn set_compression(&mut self, threshold: CompressionThreshold) {
        self.compression = Some(threshold);
    }

    /// NOTE: Encryption can only be set; a minecraft stream cannot go back to being unencrypted
    pub const fn set_encryption(
        &mut self,
        _key: &[u8; 16],
    ) -> Result<(), EncryptionAlreadyEnabledError> {
        Ok(())
    }

    pub async fn get_packet_payload(
        &mut self,
        full_packet: Vec<u8>,
    ) -> Result<Vec<u8>, PacketDecodeError> {
        if full_packet.is_empty() {
            return Err(PacketDecodeError::MalformedLength("Empty packet".into()));
        }

        // If the first byte isn't 0xfe, it's likely a RakNet control packet or encrypted.
        // Ensure your RakNet implementation is providing ONLY the payload here.
        if full_packet[0] != 0xfe {
            return Err(PacketDecodeError::MalformedLength(format!(
                "Missing 0xfe header (found 0x{:02x})",
                full_packet[0]
            )));
        }

        // If compression is NOT enabled yet, the payload starts at index 1.
        if self.compression.is_none() {
            let payload = &full_packet[1..];
            if payload.len() > MAX_PACKET_DATA_SIZE {
                return Err(PacketDecodeError::TooLong);
            }
            return Ok(payload.to_vec());
        }

        // If compression IS enabled, Bedrock expects a compression method byte at index 1.
        let compression_method = *full_packet.get(1).ok_or_else(|| {
            PacketDecodeError::MalformedLength("Missing Bedrock compression method".into())
        })?;
        let data_start = 2;

        match compression_method {
            0x00 => {
                use tokio::io::AsyncReadExt;
                let compressed_payload = &full_packet[data_start..];
                let mut decoder = DeflateDecoder::new(BufReader::new(compressed_payload))
                    .take(MAX_PACKET_DATA_SIZE as u64 + 1);
                let mut decompressed = Vec::new();
                decoder
                    .read_to_end(&mut decompressed)
                    .await
                    .map_err(|e| PacketDecodeError::FailedDecompression(e.to_string()))?;
                if decompressed.len() > MAX_PACKET_DATA_SIZE {
                    return Err(PacketDecodeError::TooLong);
                }
                Ok(decompressed)
            }
            0xff => {
                // None (Compression enabled but this specific packet is raw)
                let payload = &full_packet[data_start..];
                if payload.len() > MAX_PACKET_DATA_SIZE {
                    return Err(PacketDecodeError::TooLong);
                }
                Ok(payload.to_vec())
            }
            _ => Err(PacketDecodeError::FailedDecompression(format!(
                "Unsupported compression method: 0x{compression_method:02x}"
            ))),
        }
    }

    pub fn get_game_packet(
        &mut self,
        decompressed_reader: &mut Cursor<Vec<u8>>,
    ) -> Result<RawPacket, PacketDecodeError> {
        let packet_len = VarUInt::decode(decompressed_reader).map_err(|err| match err {
            ReadingError::CleanEOF(_) => PacketDecodeError::ConnectionClosed,
            err => PacketDecodeError::MalformedLength(err.to_string()),
        })?;
        let packet_len = packet_len.0 as usize;
        if packet_len == 0 {
            return Err(PacketDecodeError::MalformedLength(
                "Bedrock game packet length is zero".into(),
            ));
        }
        if packet_len > MAX_PACKET_DATA_SIZE {
            return Err(PacketDecodeError::TooLong);
        }

        let var_header = VarUInt::decode(decompressed_reader)?;
        let header = var_header.0 & 0x3FFF;
        let gamepacket_id = (header & 0x3FF) as u16;

        let header_size = var_header.written_size();
        if packet_len < header_size {
            return Err(PacketDecodeError::MalformedLength(format!(
                "Bedrock game packet length {packet_len} is smaller than header size {header_size}"
            )));
        }

        let payload_len = packet_len - header_size;
        let remaining = decompressed_reader
            .get_ref()
            .len()
            .saturating_sub(decompressed_reader.position() as usize);
        if payload_len > remaining {
            return Err(PacketDecodeError::MalformedLength(format!(
                "Bedrock game packet payload length {payload_len} exceeds remaining batch bytes {remaining}"
            )));
        }

        let mut payload = vec![0; payload_len].into_boxed_slice();
        decompressed_reader
            .read_exact(&mut payload)
            .map_err(|err| PacketDecodeError::FailedDecompression(err.to_string()))?;

        Ok(RawPacket {
            id: i32::from(gamepacket_id),
            payload: payload.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::bedrock::{SubClient, packet_encoder::UDPNetworkEncoder};

    use super::*;

    #[test]
    fn decodes_payload_larger_than_two_mib() {
        const PAYLOAD_LEN: usize = 2 * 1024 * 1024 + 1;
        let payload = vec![0x2a; PAYLOAD_LEN];
        let mut wire_buf = Vec::new();
        let mut network_encoder = UDPNetworkEncoder::new();
        network_encoder
            .write_game_packet(
                0x01,
                SubClient::Main,
                SubClient::Main,
                &payload,
                &mut wire_buf,
            )
            .expect("encode Bedrock game packet");

        let mut cursor = Cursor::new(wire_buf[1..].to_vec());
        let mut decoder = UDPNetworkDecoder::new();

        let packet = decoder
            .get_game_packet(&mut cursor)
            .expect("decode Bedrock game packet");

        assert_eq!(packet.id, 0x01);
        assert_eq!(packet.payload.len(), PAYLOAD_LEN);
        assert_eq!(packet.payload.as_ref(), payload.as_slice());
    }
}
