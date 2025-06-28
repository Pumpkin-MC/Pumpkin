use std::net::SocketAddr;

use bytes::Bytes;
use thiserror::Error;
use tokio::{io::AsyncWrite, net::UdpSocket};

use crate::{
    Aes128Cfb8Enc, CompressionLevel, CompressionThreshold, PacketEncodeError, StreamEncryptor,
};

// raw -> compress -> encrypt

pub enum EncryptionWriter<W: AsyncWrite + Unpin> {
    Encrypt(Box<StreamEncryptor<W>>),
    None(W),
}

impl<W: AsyncWrite + Unpin> EncryptionWriter<W> {
    pub fn upgrade(self, cipher: Aes128Cfb8Enc) -> Self {
        match self {
            Self::None(stream) => Self::Encrypt(Box::new(StreamEncryptor::new(cipher, stream))),
            _ => panic!("Cannot upgrade a stream that already has a cipher!"),
        }
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for EncryptionWriter<W> {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            Self::Encrypt(writer) => {
                let writer = std::pin::Pin::new(writer);
                writer.poll_write(cx, buf)
            }
            Self::None(writer) => {
                let writer = std::pin::Pin::new(writer);
                writer.poll_write(cx, buf)
            }
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            Self::Encrypt(writer) => {
                let writer = std::pin::Pin::new(writer);
                writer.poll_flush(cx)
            }
            Self::None(writer) => {
                let writer = std::pin::Pin::new(writer);
                writer.poll_flush(cx)
            }
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            Self::Encrypt(writer) => {
                let writer = std::pin::Pin::new(writer);
                writer.poll_shutdown(cx)
            }
            Self::None(writer) => {
                let writer = std::pin::Pin::new(writer);
                writer.poll_shutdown(cx)
            }
        }
    }
}

/// Encoder: Server -> Client
/// Supports ZLib endecoding/compression
/// Supports Aes128 Encryption
pub struct UDPNetworkEncoder {
    // compression and compression threshold
    compression: Option<(CompressionThreshold, CompressionLevel)>,
}

impl Default for UDPNetworkEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl UDPNetworkEncoder {
    pub fn new() -> Self {
        Self { compression: None }
    }

    pub fn set_compression(&mut self, compression_info: (CompressionThreshold, CompressionLevel)) {
        self.compression = Some(compression_info);
    }

    /// NOTE: Encryption can only be set; a minecraft stream cannot go back to being unencrypted
    pub fn set_encryption(&mut self, _key: &[u8; 16]) {
        // if matches!(self.writer, EncryptionWriter::Encrypt(_)) {
        //     panic!("Cannot upgrade a stream that already has a cipher!");
        // }
        // let cipher = Aes128Cfb8Enc::new_from_slices(key, key).expect("invalid key");
        // take_mut::take(&mut self.writer, |encoder| encoder.upgrade(cipher));
    }

    pub async fn write_packet(
        &mut self,
        packet_data: Bytes,
        addr: SocketAddr,
        socket: &UdpSocket,
    ) -> Result<(), PacketEncodeError> {
        socket.send_to(&packet_data, addr).await.unwrap();

        Ok(())
    }
}

#[derive(Error, Debug)]
#[error("Invalid compression Level")]
pub struct CompressionLevelError;
