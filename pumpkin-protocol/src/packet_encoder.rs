use std::io::Write;

use aes::cipher::{BlockEncryptMut, BlockSizeUser, KeyIvInit, generic_array::GenericArray};
use async_compression::{Level, tokio::write::ZlibEncoder};
use bytes::{BufMut, BytesMut};
use thiserror::Error;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::{
    Aes128Cfb8Enc, ClientPacket, CompressionLevel, CompressionThreshold, MAX_PACKET_DATA_SIZE,
    MAX_PACKET_SIZE, StreamEncryptor, VarInt, codec::Codec, ser::NetworkRead,
};

// raw -> compress -> encrypt
pub enum CompressionWriter<W: AsyncWrite + Unpin> {
    Compress(ZlibEncoder<W>),
    None(W),
}

impl<W: AsyncWrite + Unpin> AsyncWrite for CompressionWriter<W> {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            Self::Compress(writer) => {
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
            Self::Compress(writer) => {
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
            Self::Compress(writer) => {
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
pub struct NetworkEncoder<W: AsyncWrite + Unpin> {
    writer: EncryptionWriter<W>,
    // compression and compression threshold
    compression: Option<(CompressionThreshold, CompressionLevel)>,
}

impl<W: AsyncWrite + Unpin> NetworkEncoder<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer: EncryptionWriter::None(writer),
            compression: None,
        }
    }

    pub fn set_compression(&mut self, threshold: Option<(CompressionThreshold, CompressionLevel)>) {
        self.compression = threshold;
    }

    /// NOTE: Encryption can only be set; a minecraft stream cannot go back to being unencrypted
    pub fn set_encryption(&mut self, key: &[u8; 16]) {
        if matches!(self.writer, EncryptionWriter::Encrypt(_)) {
            panic!("Cannot upgrade a stream that already has a cipher!");
        }
        let cipher = Aes128Cfb8Enc::new_from_slices(key, key).expect("invalid key");
        take_mut::take(&mut self.writer, |encoder| encoder.upgrade(cipher));
    }

    /// Appends a Clientbound `ClientPacket` to the internal buffer and applies compression when needed.
    ///
    /// If compression is enabled and the packet size exceeds the threshold, the packet is compressed.
    /// The packet is prefixed with its length and, if compressed, the uncompressed data length.
    /// The packet format is as follows:
    ///
    /// **Uncompressed:**
    /// |-----------------------|
    /// | Packet Length (VarInt)|
    /// |-----------------------|
    /// | Packet ID (VarInt)    |
    /// |-----------------------|
    /// | Data (Byte Array)     |
    /// |-----------------------|
    ///
    /// **Compressed:**
    /// |------------------------|
    /// | Packet Length (VarInt) |
    /// |------------------------|
    /// | Data Length (VarInt)   |
    /// |------------------------|
    /// | Packet ID (VarInt)     |
    /// |------------------------|
    /// | Data (Byte Array)      |
    /// |------------------------|
    ///
    /// -   `Packet Length`: The total length of the packet *excluding* the `Packet Length` field itself.
    /// -   `Data Length`: (Only present in compressed packets) The length of the uncompressed `Packet ID` and `Data`.
    /// -   `Packet ID`: The ID of the packet.
    /// -   `Data`: The packet's data.
    pub async fn write_packet<P: ClientPacket>(
        &mut self,
        packet: &P,
    ) -> Result<(), PacketEncodeError> {
        // We need to know the length of the compressed buffer and serde is not async :(
        // We need to write to a buffer here 😔

        // TODO: We only need a length here, otherwise we could stream the deserialization (into a
        // buffer). Add a "serialized_size" or something to packets that gets the serialized length
        let mut packet_buf = Vec::new();
        packet.write(&mut packet_buf).map_err(|err| {
            // TODO: Remove this when we are confident with all of our networking

            panic!("Failed to serialize packet to the network: {}", err);
            //PacketEncodeError::Message(err.to_string())
        })?;
        let data_len = packet_buf.len();

        let packet_id_var_int: VarInt = P::PACKET_ID.into();
        let full_data_len = data_len + packet_id_var_int.written_size();
        if full_data_len > MAX_PACKET_DATA_SIZE {
            return Err(PacketEncodeError::TooLong(full_data_len));
        }
        let uncompressed_len_var_int: VarInt = full_data_len.into();

        if let Some((compression_threshold, compression_level)) = self.compression {
            if full_data_len > compression_threshold {
                // Pushed before data:
                // Length of (Data Length) + length of compressed (Packet ID + Data)
                // Length of uncompressed (Packet ID + Data)

                // TODO: We need the compressed length at the beginning of the packet so we need to write to
                // buf here :( Is there a magic way to find a compressed length?
                let mut compressed_buf = Vec::new();
                let mut compressor = CompressionWriter::Compress(ZlibEncoder::with_quality(
                    &mut compressed_buf,
                    Level::Precise(compression_level as i32),
                ));

                packet_id_var_int
                    .encode_async(&mut compressor)
                    .await
                    .map_err(|err| PacketEncodeError::Message(err.to_string()))?;

                compressor
                    .write_all(&packet_buf)
                    .await
                    .map_err(|err| PacketEncodeError::Message(err.to_string()))?;

                let full_packet_len_var_int: VarInt =
                    (uncompressed_len_var_int.written_size() + compressed_buf.len()).into();

                let full_packet_len =
                    full_packet_len_var_int.written_size() + full_packet_len_var_int.0 as usize;
                if full_packet_len > MAX_PACKET_SIZE as usize {
                    return Err(PacketEncodeError::TooLong(full_packet_len));
                }

                full_packet_len_var_int
                    .encode_async(&mut self.writer)
                    .await
                    .map_err(|err| PacketEncodeError::Message(err.to_string()))?;
                uncompressed_len_var_int
                    .encode_async(&mut self.writer)
                    .await
                    .map_err(|err| PacketEncodeError::Message(err.to_string()))?;
                self.writer
                    .write_all(&compressed_buf)
                    .await
                    .map_err(|err| PacketEncodeError::Message(err.to_string()))?;
            } else {
                // Pushed before data:
                // Length of (Data Length) + length of compressed (Packet ID + Data)
                // 0 to indicate uncompressed

                let zero_var_int: VarInt = 0.into();
                let full_packet_len_var_int: VarInt =
                    (full_data_len + zero_var_int.written_size()).into();

                let full_packet_len =
                    full_packet_len_var_int.written_size() + full_packet_len_var_int.0 as usize;
                if full_packet_len > MAX_PACKET_SIZE as usize {
                    return Err(PacketEncodeError::TooLong(full_packet_len));
                }

                full_packet_len_var_int
                    .encode_async(&mut self.writer)
                    .await
                    .map_err(|err| PacketEncodeError::Message(err.to_string()))?;
                zero_var_int
                    .encode_async(&mut self.writer)
                    .await
                    .map_err(|err| PacketEncodeError::Message(err.to_string()))?;
                self.writer
                    .write_all(&packet_buf)
                    .await
                    .map_err(|err| PacketEncodeError::Message(err.to_string()))?;
            }
        } else {
            // Pushed before data:
            // Length of Packet ID + Data

            let full_packet_len_var_int: VarInt = uncompressed_len_var_int;

            let full_packet_len =
                full_packet_len_var_int.written_size() + full_packet_len_var_int.0 as usize;
            if full_packet_len > MAX_PACKET_SIZE as usize {
                return Err(PacketEncodeError::TooLong(full_packet_len));
            }

            full_packet_len_var_int
                .encode_async(&mut self.writer)
                .await
                .map_err(|err| PacketEncodeError::Message(err.to_string()))?;
            self.writer
                .write_all(&packet_buf)
                .await
                .map_err(|err| PacketEncodeError::Message(err.to_string()))?;
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
#[error("Invalid compression Level")]
pub struct CompressionLevelError;

/// Errors that can occur during packet encoding.
#[derive(Error, Debug)]
pub enum PacketEncodeError {
    #[error("Packet exceeds maximum length: {0}")]
    TooLong(usize),
    #[error("Compression failed {0}")]
    CompressionFailed(String),
    #[error("Writing packet failed: {0}")]
    Message(String),
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;
    use crate::client::status::CStatusResponse;
    use crate::ser::packet::Packet;
    use crate::ser::{NetworkRead, ReadingError};
    use aes::Aes128;
    use cfb8::Decryptor as Cfb8Decryptor;
    use cfb8::cipher::AsyncStreamCipher;
    use pumpkin_data::packet::clientbound::STATUS_STATUS_RESPONSE;
    use pumpkin_macros::packet;
    use serde::Serialize;

    /// Define a custom packet for testing maximum packet size
    #[derive(Serialize)]
    #[packet(STATUS_STATUS_RESPONSE)]
    pub struct MaxSizePacket {
        data: Vec<u8>,
    }

    impl MaxSizePacket {
        pub fn new(size: usize) -> Self {
            Self {
                data: vec![0xAB; size], // Fill with arbitrary data
            }
        }
    }

    /// Helper function to decode a VarInt from bytes
    fn decode_varint(buffer: &mut &[u8]) -> Result<i32, ReadingError> {
        Ok(buffer.get_var_int()?.0)
    }

    /// Helper function to decompress data using libdeflater's Zlib decompressor
    fn decompress_zlib(data: &[u8], expected_size: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut decompressed = vec![0u8; expected_size];
        todo!();
        //ZlibDecoder::new(data).read_exact(&mut decompressed)?;
        Ok(decompressed)
    }

    /// Helper function to decrypt data using AES-128 CFB-8 mode
    fn decrypt_aes128(encrypted_data: &mut [u8], key: &[u8; 16], iv: &[u8; 16]) {
        let decryptor = Cfb8Decryptor::<Aes128>::new_from_slices(key, iv).expect("Invalid key/iv");
        decryptor.decrypt(encrypted_data);
    }

    /// Helper function to build a packet with optional compression and encryption
    fn build_packet_with_encoder<T: ClientPacket>(
        packet: &T,
        compression_info: Option<(CompressionThreshold, CompressionLevel)>,
        key: Option<&[u8; 16]>,
    ) -> BytesMut {
        let mut encoder = PacketEncoder::default();

        if let Some(compression) = compression_info {
            encoder.set_compression(Some(compression));
        } else {
            encoder.set_compression(None);
        }

        if let Some(key) = key {
            encoder.set_encryption(Some(key));
        }

        encoder
            .append_packet(packet)
            .expect("Failed to append packet");

        encoder.take()
    }

    /// Test encoding without compression and encryption
    #[test]
    fn test_encode_without_compression_and_encryption() {
        // Create a CStatusResponse packet
        let packet = CStatusResponse::new("{\"description\": \"A Minecraft Server\"}");

        // Build the packet without compression and encryption
        let packet_bytes = build_packet_with_encoder(&packet, None, None);

        // Decode the packet manually to verify correctness
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).expect("Failed to decode packet length");
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read packet ID VarInt
        let decoded_packet_id = decode_varint(&mut buffer).expect("Failed to decode packet ID");
        assert_eq!(decoded_packet_id, CStatusResponse::PACKET_ID);

        // Remaining buffer is the payload
        // We need to obtain the expected payload
        let mut expected_payload = BytesMut::new();
        packet.write(&mut expected_payload);

        assert_eq!(buffer, expected_payload);
    }

    /// Test encoding with compression
    #[test]
    fn test_encode_with_compression() {
        // Create a CStatusResponse packet
        let packet = CStatusResponse::new("{\"description\": \"A Minecraft Server\"}");

        // Build the packet with compression enabled
        let packet_bytes = build_packet_with_encoder(&packet, Some((0, 6)), None);

        // Decode the packet manually to verify correctness
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).expect("Failed to decode packet length");
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read data length VarInt (uncompressed data length)
        let data_length = decode_varint(&mut buffer).expect("Failed to decode data length");
        let mut expected_payload = BytesMut::new();
        packet.write(&mut expected_payload);
        let uncompressed_data_length =
            VarInt(CStatusResponse::PACKET_ID).written_size() + expected_payload.len();
        assert_eq!(data_length as usize, uncompressed_data_length);

        // Remaining buffer is the compressed data
        let compressed_data = buffer;

        // Decompress the data
        let decompressed_data = decompress_zlib(compressed_data, data_length as usize)
            .expect("Failed to decompress data");

        // Verify packet ID and payload
        let mut decompressed_buffer = &decompressed_data[..];

        // Read packet ID VarInt
        let decoded_packet_id =
            decode_varint(&mut decompressed_buffer).expect("Failed to decode packet ID");
        assert_eq!(decoded_packet_id, CStatusResponse::PACKET_ID);

        // Remaining buffer is the payload
        assert_eq!(decompressed_buffer, expected_payload);
    }

    /// Test encoding with encryption
    #[test]
    fn test_encode_with_encryption() {
        // Create a CStatusResponse packet
        let packet = CStatusResponse::new("{\"description\": \"A Minecraft Server\"}");

        // Encryption key and IV (IV is the same as key in this case)
        let key = [0x00u8; 16]; // Example key

        // Build the packet with encryption enabled (no compression)
        let mut packet_bytes = build_packet_with_encoder(&packet, None, Some(&key));

        // Decrypt the packet
        decrypt_aes128(&mut packet_bytes, &key, &key);

        // Decode the packet manually to verify correctness
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).expect("Failed to decode packet length");
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read packet ID VarInt
        let decoded_packet_id = decode_varint(&mut buffer).expect("Failed to decode packet ID");
        assert_eq!(decoded_packet_id, CStatusResponse::PACKET_ID);

        // Remaining buffer is the payload
        let mut expected_payload = BytesMut::new();
        packet.write(&mut expected_payload);

        assert_eq!(buffer, expected_payload);
    }

    /// Test encoding with both compression and encryption
    #[test]
    fn test_encode_with_compression_and_encryption() {
        // Create a CStatusResponse packet
        let packet = CStatusResponse::new("{\"description\": \"A Minecraft Server\"}");

        // Encryption key and IV (IV is the same as key in this case)
        let key = [0x01u8; 16]; // Example key

        // Build the packet with both compression and encryption enabled
        // Compression threshold is set to 0 to force compression
        let mut packet_bytes = build_packet_with_encoder(&packet, Some((0, 6)), Some(&key));

        // Decrypt the packet
        decrypt_aes128(&mut packet_bytes, &key, &key);

        // Decode the packet manually to verify correctness
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).expect("Failed to decode packet length");
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read data length VarInt (uncompressed data length)
        let data_length = decode_varint(&mut buffer).expect("Failed to decode data length");
        let mut expected_payload = BytesMut::new();
        packet.write(&mut expected_payload);
        let uncompressed_data_length =
            VarInt(CStatusResponse::PACKET_ID).written_size() + expected_payload.len();
        assert_eq!(data_length as usize, uncompressed_data_length);

        // Remaining buffer is the compressed data
        let compressed_data = buffer;

        // Decompress the data
        let decompressed_data = decompress_zlib(compressed_data, data_length as usize)
            .expect("Failed to decompress data");

        // Verify packet ID and payload
        let mut decompressed_buffer = &decompressed_data[..];

        // Read packet ID VarInt
        let decoded_packet_id =
            decode_varint(&mut decompressed_buffer).expect("Failed to decode packet ID");
        assert_eq!(decoded_packet_id, CStatusResponse::PACKET_ID);

        // Remaining buffer is the payload
        assert_eq!(decompressed_buffer, expected_payload);
    }

    /// Test encoding with zero-length payload
    #[test]
    fn test_encode_with_zero_length_payload() {
        // Create a CStatusResponse packet with empty payload
        let packet = CStatusResponse::new("");

        // Build the packet without compression and encryption
        let packet_bytes = build_packet_with_encoder(&packet, None, None);

        // Decode the packet manually to verify correctness
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).expect("Failed to decode packet length");
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read packet ID VarInt
        let decoded_packet_id = decode_varint(&mut buffer).expect("Failed to decode packet ID");
        assert_eq!(decoded_packet_id, CStatusResponse::PACKET_ID);

        // Remaining buffer is the payload (empty)
        let mut expected_payload = BytesMut::new();
        packet.write(&mut expected_payload);

        assert_eq!(
            buffer.len(),
            expected_payload.len(),
            "Payload length mismatch"
        );
        assert_eq!(buffer, expected_payload);
    }

    /// Test encoding with maximum length payload
    #[test]
    fn test_encode_with_maximum_string_length() {
        // Maximum allowed string length is 32767 bytes
        let max_string_length = 32767;
        let payload_str = "A".repeat(max_string_length);
        let packet = CStatusResponse::new(&payload_str);

        // Build the packet without compression and encryption
        let packet_bytes = build_packet_with_encoder(&packet, None, None);

        // Verify that the packet size does not exceed MAX_PACKET_SIZE as usize
        assert!(
            packet_bytes.len() <= MAX_PACKET_SIZE as usize,
            "Packet size exceeds maximum allowed size"
        );

        // Decode the packet manually to verify correctness
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).expect("Failed to decode packet length");
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read packet ID VarInt
        let decoded_packet_id = decode_varint(&mut buffer).expect("Failed to decode packet ID");
        // Assume packet ID is 0 for CStatusResponse
        assert_eq!(decoded_packet_id, CStatusResponse::PACKET_ID);

        // Remaining buffer is the payload
        let mut expected_payload = BytesMut::new();
        packet.write(&mut expected_payload);

        assert_eq!(buffer, expected_payload);
    }

    /// Test encoding a packet that exceeds MAX_PACKET_SIZE as usize
    #[test]
    #[should_panic(expected = "TooLong")]
    fn test_encode_packet_exceeding_maximum_size() {
        // Create a custom packet with data exceeding MAX_PACKET_SIZE as usize
        let data_size = MAX_PACKET_SIZE as usize + 1; // Exceed by 1 byte
        let packet = MaxSizePacket::new(data_size);

        // Build the packet without compression and encryption
        // This should panic with PacketEncodeError::TooLong
        build_packet_with_encoder(&packet, None, None);
    }

    /// Test encoding with a small payload that should not be compressed
    #[test]
    fn test_encode_small_payload_no_compression() {
        // Create a CStatusResponse packet with small payload
        let packet = CStatusResponse::new("Hi");

        // Build the packet with compression enabled
        // Compression threshold is set to a value higher than payload length
        let packet_bytes = build_packet_with_encoder(&packet, Some((10, 6)), None);

        // Decode the packet manually to verify that it was not compressed
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).expect("Failed to decode packet length");
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read data length VarInt (should be 0 indicating no compression)
        let data_length = decode_varint(&mut buffer).expect("Failed to decode data length");
        assert_eq!(
            data_length, 0,
            "Data length should be 0 indicating no compression"
        );

        // Read packet ID VarInt
        let decoded_packet_id = decode_varint(&mut buffer).expect("Failed to decode packet ID");
        assert_eq!(decoded_packet_id, CStatusResponse::PACKET_ID);

        // Remaining buffer is the payload
        let mut expected_payload = BytesMut::new();
        packet.write(&mut expected_payload);

        assert_eq!(buffer, expected_payload);
    }
}
