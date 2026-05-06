use std::{
    io::{self, Error, Write},
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

use flate2::{Compression, write::DeflateEncoder};
use tokio::{io::AsyncWrite, net::UdpSocket};

use crate::{
    Aes128Cfb8Enc, CompressionLevel, CompressionThreshold, StreamEncryptor, bedrock::SubClient,
    codec::var_uint::VarUInt, ser::NetworkWriteExt,
};

// raw -> compress -> encrypt

pub enum EncryptionWriter<W: AsyncWrite + Unpin> {
    Encrypt(Box<StreamEncryptor<W>>),
    None(W),
}

impl<W: AsyncWrite + Unpin> EncryptionWriter<W> {
    #[must_use]
    pub fn upgrade(self, cipher: Aes128Cfb8Enc) -> Self {
        match self {
            Self::None(stream) => Self::Encrypt(Box::new(StreamEncryptor::new(cipher, stream))),
            Self::Encrypt(_) => panic!("Cannot upgrade a stream that already has a cipher!"),
        }
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for EncryptionWriter<W> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self.get_mut() {
            Self::Encrypt(writer) => {
                let writer = Pin::new(writer);
                writer.poll_write(cx, buf)
            }
            Self::None(writer) => {
                let writer = Pin::new(writer);
                writer.poll_write(cx, buf)
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.get_mut() {
            Self::Encrypt(writer) => {
                let writer = Pin::new(writer);
                writer.poll_flush(cx)
            }
            Self::None(writer) => {
                let writer = Pin::new(writer);
                writer.poll_flush(cx)
            }
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.get_mut() {
            Self::Encrypt(writer) => {
                let writer = Pin::new(writer);
                writer.poll_shutdown(cx)
            }
            Self::None(writer) => {
                let writer = Pin::new(writer);
                writer.poll_shutdown(cx)
            }
        }
    }
}

/// Encoder: Server -> Client
/// Supports `ZLib` endecoding/compression
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
    #[must_use]
    pub const fn new() -> Self {
        Self { compression: None }
    }

    pub const fn set_compression(
        &mut self,
        compression_info: (CompressionThreshold, CompressionLevel),
    ) {
        self.compression = Some(compression_info);
    }

    /// NOTE: Encryption can only be set; a minecraft stream cannot go back to being unencrypted
    pub const fn set_encryption(
        &mut self,
        _key: &[u8; 16],
    ) -> Result<(), crate::bedrock::packet_decoder::EncryptionAlreadyEnabledError> {
        // if matches!(self.writer, EncryptionWriter::Encrypt(_)) {
        //     return Err(crate::bedrock::packet_decoder::EncryptionAlreadyEnabledError);
        // }
        // let cipher = Aes128Cfb8Enc::new_from_slices(key, key).map_err(|_| ());
        // take_mut::take(&mut self.writer, |encoder| encoder.upgrade(cipher));
        Ok(())
    }

    pub fn write_game_packet(
        &mut self,
        packet_id: u16,
        sub_client_sender: SubClient,
        sub_client_target: SubClient,
        packet_payload: &[u8],
        mut writer: impl Write,
    ) -> Result<(), Error> {
        let mut inner_buffer = Vec::new();

        // Gamepacket ID Header (14 bits)
        let header_value: u32 = u32::from(packet_id)
            | ((sub_client_sender as u32) << 10)
            | ((sub_client_target as u32) << 12);
        let fourteen_bit_header = header_value & 0x3FFF;

        let header_varint = VarUInt(fourteen_bit_header);
        let total_content_length = (header_varint.written_size() + packet_payload.len()) as u32;

        inner_buffer
            .write_var_uint(&VarUInt(total_content_length))
            .map_err(|_| Error::other("Failed to write total content length"))?;
        inner_buffer
            .write_var_uint(&header_varint)
            .map_err(|_| Error::other("Failed to write header varint"))?;
        inner_buffer.write_all(packet_payload)?;

        // Handle Outer Container
        writer
            .write_u8(0xfe)
            .map_err(|e| Error::other(e.to_string()))?; // Bedrock Game Packet Header

        if let Some((_threshold, level)) = self.compression {
            // Write Compression Method (0x00 for Zlib)
            writer
                .write_u8(0x00)
                .map_err(|e| Error::other(e.to_string()))?;

            let mut encoder = DeflateEncoder::new(Vec::new(), Compression::new(level));
            encoder.write_all(&inner_buffer)?;
            let compressed_data = encoder.finish()?;

            writer.write_all(&compressed_data)?;
        } else {
            writer.write_all(&inner_buffer)?;
        }

        Ok(())
    }

    pub async fn write_packet(
        &mut self,
        packet_data: &[u8],
        addr: SocketAddr,
        socket: &UdpSocket,
    ) -> Result<(), Error> {
        socket.send_to(packet_data, addr).await.map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bedrock::packet_decoder::UDPNetworkDecoder;
    use std::io::Cursor;

    #[tokio::test]
    async fn bedrock_compression_cycle() -> Result<(), Box<dyn std::error::Error>> {
        let mut encoder = UDPNetworkEncoder::new();
        encoder.set_compression((256, 6));

        let packet_id = 1;
        let payload = b"Hello Bedrock Compression!";
        let mut encoded_buf = Vec::new();

        encoder.write_game_packet(
            packet_id,
            SubClient::Main,
            SubClient::Main,
            payload,
            &mut encoded_buf,
        )?;

        let mut decoder = UDPNetworkDecoder::new();
        decoder.set_compression(256);

        let decompressed_payload = decoder.get_packet_payload(encoded_buf).await?;
        let mut cursor = Cursor::new(decompressed_payload);
        let raw_packet = decoder.get_game_packet(&mut cursor)?;

        assert_eq!(raw_packet.id, packet_id as i32);
        assert_eq!(raw_packet.payload.as_ref(), payload);
        Ok(())
    }
}
