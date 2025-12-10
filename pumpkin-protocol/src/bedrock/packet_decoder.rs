use std::{
    io::Cursor,
    pin::Pin,
    task::{Context, Poll},
};

use async_compression::tokio::bufread::ZlibDecoder;
use bytes::Bytes;
use tokio::io::{AsyncRead, BufReader, ReadBuf};

use crate::{
    Aes128Cfb8Dec, CompressionThreshold, PacketDecodeError, RawPacket, StreamDecryptor,
    codec::var_uint::VarUInt,
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
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Decompress(reader) => {
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
        reader: Cursor<Vec<u8>>,
    ) -> Result<Bytes, PacketDecodeError> {
        //let mut payload = Vec::new();
        //reader
        //    .read_to_end(&mut payload)
        //    .await
        //    .map_err(|err| PacketDecodeError::FailedDecompression(err.to_string()))?;

        Ok(reader.into_inner().into())
    }

    pub async fn get_game_packet(
        &mut self,
        mut reader: Cursor<Vec<u8>>,
    ) -> Result<RawPacket, PacketDecodeError> {
        if self.compression.is_some() {
            let _method = reader.get_u8().unwrap();
            // None Compression
        }

        //compression is only included after the network settings packet is sent
        // TODO: compression & encryption
        let packet_len = VarUInt::decode(&mut reader).map_err(|err| match err {
            ReadingError::CleanEOF(_) => PacketDecodeError::ConnectionClosed,
            err => PacketDecodeError::MalformedLength(err.to_string()),
        })?;

        let packet_len = packet_len.0 as usize;

        let var_header = VarUInt::decode(&mut reader)?;

        // The header is 14 bits. Ensure we only consider these bits.
        // A varint u32 could be larger, so we mask to the relevant bits.
        let header = var_header.0 & 0x3FFF; // Mask to get the lower 14 bits (2^14 - 1)

        // Extract components from GamePacket Header (14 bits)
        // Gamepacket ID (10 bits)
        // SubClient Sender ID (2 bits)
        // SubClient Target ID (2 bits)

        // SubClient Target ID: Lowest 2 bits
        let _sub_client_target = (header >> 10 & 0b11) as u8;

        // SubClient Sender ID: Next 2 bits (bits 2 and 3)
        let _sub_client_sender = (header >> 12 & 0b11) as u8;

        // Gamepacket ID: Remaining 10 bits (bits 4 to 13)
        let gamepacket_id = (header & 0x3FF) as u16; // 0x3FF is 10 bits set to 1

        let payload = reader
            .read_boxed_slice(packet_len - var_header.written_size())
            .map_err(|err| PacketDecodeError::FailedDecompression(err.to_string()))?;

        Ok(RawPacket {
            id: gamepacket_id as i32,
            payload: payload.into(),
        })
    }
}
