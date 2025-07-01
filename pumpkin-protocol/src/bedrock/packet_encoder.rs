use std::{io::Write, net::SocketAddr};

use bytes::Bytes;
use thiserror::Error;
use tokio::{io::AsyncWrite, net::UdpSocket};

use crate::{
    Aes128Cfb8Enc, CompressionLevel, CompressionThreshold, PacketEncodeError, StreamEncryptor,
    codec::var_int::VarInt, ser::NetworkWriteExt,
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

    pub async fn write_game_packet(
        &mut self,
        packet_id: i32,
        sub_client_sender_id: i32,
        sub_client_target_id: i32,
        packet_payload: Bytes,
        mut writer: impl Write,
    ) -> Result<(), PacketEncodeError> {
        // Game Packet ID
        writer.write_u8(0xfe).unwrap();

        // TODO: compression & encryption

        // Gamepacket ID (10 bits) << 4 (offset by 2 bits for target + 2 bits for sender)
        // SubClient Sender ID (2 bits) << 2 (offset by 2 bits for target)
        // SubClient Target ID (2 bits)
        let header_value: u32 = ((packet_id as u32) << 4)
            | ((sub_client_sender_id as u32) << 2)
            | (sub_client_target_id as u32);

        // Ensure the combined header doesn't exceed 14 bits (just a sanity check, should be handled by above shifts)
        let fourteen_bit_header = header_value & 0x3FFF; // Mask to ensure it fits in 14 bits

        // 2. Calculate total packet_len
        // This is where `VarInt::encoded_len` is crucial.
        // We need to know the byte length of the header's VarInt *before* we write the packet_len.
        let header_byte_len = VarInt(fourteen_bit_header as i32).written_size();

        let packet_payload_len = packet_payload.len() as u32;
        // total_content_length is the length of the header VarInt bytes + payload bytes.
        let total_content_length = header_byte_len as u32 + packet_payload_len;

        // 3. Write packet_len as VarInt
        // Note: Your `VarInt` struct takes `i32`, but lengths are typically `u32`.
        // Ensure consistency in your actual `VarInt` definition.
        // For this example, I'll cast `total_content_length` to `i32`.
        writer
            .write_var_int(&VarInt(total_content_length as i32))
            .unwrap();

        // 4. Write the combined 14-bit header_value as VarInt
        writer
            .write_var_int(&VarInt(fourteen_bit_header as i32))
            .unwrap();

        // 5. Write the Packet ID + payload
        writer.write_u8(packet_id as u8).unwrap();
        writer.write_all(&packet_payload).unwrap();
        Ok(())
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
