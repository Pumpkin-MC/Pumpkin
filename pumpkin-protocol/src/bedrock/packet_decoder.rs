use std::io::Cursor;

use async_compression::tokio::bufread::ZlibDecoder;
use bytes::Buf;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

use crate::{Aes128Cfb8Dec, CompressionThreshold, PacketDecodeError, RawPacket, StreamDecryptor};

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

    pub async fn get_raw_packet(
        &mut self,
        mut reader: Cursor<Vec<u8>>,
    ) -> Result<RawPacket, PacketDecodeError> {
        // TODO: Serde is sync so we need to write to a buffer here :(
        // Is there a way to deserialize in an asynchronous manner?

        let packet_id = reader
            .try_get_u8()
            .map_err(|_| PacketDecodeError::DecodeID)?;

        let mut payload = Vec::new();
        reader
            .read_to_end(&mut payload)
            .await
            .map_err(|err| PacketDecodeError::FailedDecompression(err.to_string()))?;

        Ok(RawPacket {
            id: packet_id as i32,
            payload: payload.into(),
        })
    }
}
