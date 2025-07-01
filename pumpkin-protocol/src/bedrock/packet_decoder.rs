use std::io::Cursor;

use async_compression::tokio::bufread::ZlibDecoder;
use bytes::Bytes;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

use crate::{
    Aes128Cfb8Dec, CompressionThreshold, MAX_PACKET_SIZE, PacketDecodeError, RawPacket,
    StreamDecryptor,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, ReadingError},
};

// decrypt -> decompress -> raw
pub enum DecompressionReader<R: AsyncRead + Unpin> {
    Decompress(ZlibDecoder<BufReader<R>>),
    None(R),
}

impl<R: AsyncRead + Unpin> AsyncRead for DecompressionReader<R> {
    #[inline]
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Decompress(reader) => {
                let reader = std::pin::Pin::new(reader);
                reader.poll_read(cx, buf)
            }
            Self::None(reader) => {
                let reader = std::pin::Pin::new(reader);
                reader.poll_read(cx, buf)
            }
        }
    }
}

pub enum DecryptionReader<R: AsyncRead + Unpin> {
    Decrypt(Box<StreamDecryptor<R>>),
    None(R),
}

impl<R: AsyncRead + Unpin> DecryptionReader<R> {
    pub fn upgrade(self, cipher: Aes128Cfb8Dec) -> Self {
        match self {
            Self::None(stream) => Self::Decrypt(Box::new(StreamDecryptor::new(cipher, stream))),
            _ => panic!("Cannot upgrade a stream that already has a cipher!"),
        }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for DecryptionReader<R> {
    #[inline]
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Decrypt(reader) => {
                let reader = std::pin::Pin::new(reader);
                reader.poll_read(cx, buf)
            }
            Self::None(reader) => {
                let reader = std::pin::Pin::new(reader);
                reader.poll_read(cx, buf)
            }
        }
    }
}

/// Decoder: Client -> Server
/// Supports ZLib decoding/decompression
/// Supports Aes128 Encryption
pub struct UDPNetworkDecoder {
    compression: Option<CompressionThreshold>,
}

impl Default for UDPNetworkDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl UDPNetworkDecoder {
    pub fn new() -> Self {
        Self { compression: None }
    }

    pub fn set_compression(&mut self, threshold: CompressionThreshold) {
        self.compression = Some(threshold);
    }

    /// NOTE: Encryption can only be set; a minecraft stream cannot go back to being unencrypted
    pub fn set_encryption(&mut self, _key: &[u8; 16]) {
        // if matches!(self.reader, DecryptionReader::Decrypt(_)) {
        //     panic!("Cannot upgrade a stream that already has a cipher!");
        // }
        // let cipher = Aes128Cfb8Dec::new_from_slices(key, key).expect("invalid key");
        // take_mut::take(&mut self.reader, |decoder| decoder.upgrade(cipher));
    }

    pub async fn get_packet_payload(
        &mut self,
        mut reader: Cursor<Vec<u8>>,
    ) -> Result<Bytes, PacketDecodeError> {
        let mut payload = Vec::new();
        reader
            .read_to_end(&mut payload)
            .await
            .map_err(|err| PacketDecodeError::FailedDecompression(err.to_string()))?;

        Ok(payload.into())
    }

    pub async fn get_game_packet(
        &mut self,
        mut reader: Cursor<Vec<u8>>,
    ) -> Result<RawPacket, PacketDecodeError> {
        let compression = reader.get_u8_be()?;
        dbg!(compression);

        // TODO: compression & encryption
        let packet_len = VarInt::decode_async(&mut reader)
            .await
            .map_err(|err| match err {
                ReadingError::CleanEOF(_) => PacketDecodeError::ConnectionClosed,
                err => PacketDecodeError::MalformedLength(err.to_string()),
            })?;

        let packet_len = packet_len.0 as u64;
        dbg!(packet_len);

        if !(0..=MAX_PACKET_SIZE).contains(&packet_len) {
            Err(PacketDecodeError::OutOfBounds)?
        }

        let header = VarInt::decode_async(&mut reader).await?;

        let header_value = header.0;

        // Extract components from GamePacket Header (14 bits)
        // Gamepacket ID (10 bits)
        // SubClient Sender ID (2 bits)
        // SubClient Target ID (2 bits)

        // The header is 14 bits. Ensure we only consider these bits.
        // A varint u32 could be larger, so we mask to the relevant bits.
        let fourteen_bit_header = header_value & 0x3FFF; // Mask to get the lower 14 bits (2^14 - 1)

        // SubClient Target ID: Lowest 2 bits
        let _sub_client_target_id = (fourteen_bit_header & 0b11) as u8;

        // SubClient Sender ID: Next 2 bits (bits 2 and 3)
        let _sub_client_sender_id = ((fourteen_bit_header >> 2) & 0b11) as u8;

        // Gamepacket ID: Remaining 10 bits (bits 4 to 13)
        let gamepacket_id = ((fourteen_bit_header >> 4) & 0x3FF) as u16; // 0x3FF is 10 bits set to 1

        let payload = reader
            .read_boxed_slice(packet_len as usize)
            .map_err(|err| PacketDecodeError::FailedDecompression(err.to_string()))?;

        Ok(RawPacket {
            id: gamepacket_id as i32,
            payload: payload.into(),
        })
    }
}
