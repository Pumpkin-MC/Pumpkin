use aes::cipher::KeyIvInit;
use async_compression::tokio::bufread::ZlibDecoder;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

use crate::{
    Aes128Cfb8Dec, CompressionThreshold, MAX_PACKET_DATA_SIZE, MAX_PACKET_SIZE, RawPacket,
    StreamDecryptor, VarInt, ser::ReadingError,
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
pub struct NetworkDecoder<R: AsyncRead + Unpin> {
    reader: DecryptionReader<R>,
    compression: Option<CompressionThreshold>,
}

impl<R: AsyncRead + Unpin> NetworkDecoder<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: DecryptionReader::None(reader),
            compression: None,
        }
    }

    pub fn set_compression(&mut self, threshold: CompressionThreshold) {
        self.compression = Some(threshold);
    }

    /// NOTE: Encryption can only be set; a minecraft stream cannot go back to being unencrypted
    pub fn set_encryption(&mut self, key: &[u8; 16]) {
        if matches!(self.reader, DecryptionReader::Decrypt(_)) {
            panic!("Cannot upgrade a stream that already has a cipher!");
        }
        let cipher = Aes128Cfb8Dec::new_from_slices(key, key).expect("invalid key");
        take_mut::take(&mut self.reader, |decoder| decoder.upgrade(cipher));
    }

    pub async fn get_raw_packet(&mut self) -> Result<RawPacket, PacketDecodeError> {
        let packet_len = VarInt::decode_async(&mut self.reader)
            .await
            .map_err(|err| match err {
                ReadingError::CleanEOF(_) => PacketDecodeError::ConnectionClosed,
                err => PacketDecodeError::MalformedLength(err.to_string()),
            })?;

        let packet_len = packet_len.0 as u64;

        if !(0..=MAX_PACKET_SIZE).contains(&packet_len) {
            Err(PacketDecodeError::OutOfBounds)?
        }

        let mut bounded_reader = (&mut self.reader).take(packet_len);

        let mut reader = if let Some(threshold) = self.compression {
            let decompressed_length = VarInt::decode_async(&mut bounded_reader).await?;
            let raw_packet_length = packet_len as usize - decompressed_length.written_size();
            let decompressed_length = decompressed_length.0 as usize;

            if !(0..=MAX_PACKET_DATA_SIZE).contains(&decompressed_length) {
                Err(PacketDecodeError::TooLong)?
            }

            if decompressed_length > 0 {
                DecompressionReader::Decompress(ZlibDecoder::new(BufReader::new(bounded_reader)))
            } else {
                // Validate that we are not less than the compression threshold
                if raw_packet_length > threshold {
                    Err(PacketDecodeError::NotCompressed)?
                }

                DecompressionReader::None(bounded_reader)
            }
        } else {
            DecompressionReader::None(bounded_reader)
        };

        // TODO: Serde is sync so we need to write to a buffer here :(
        // Is there a way to deserialize in an asynchronous manner?

        let packet_id = VarInt::decode_async(&mut reader)
            .await
            .map_err(|_| PacketDecodeError::DecodeID)?
            .0;

        let mut payload = Vec::new();
        reader
            .read_to_end(&mut payload)
            .await
            .map_err(|err| PacketDecodeError::FailedDecompression(err.to_string()))?;

        Ok(RawPacket {
            id: packet_id,
            payload: payload.into(),
        })
    }
}

#[derive(Error, Debug)]
pub enum PacketDecodeError {
    #[error("failed to decode packet ID")]
    DecodeID,
    #[error("packet exceeds maximum length")]
    TooLong,
    #[error("packet length is out of bounds")]
    OutOfBounds,
    #[error("malformed packet length VarInt: {0}")]
    MalformedLength(String),
    #[error("failed to decompress packet: {0}")]
    FailedDecompression(String), // Updated to include error details
    #[error("packet is uncompressed but greater than the threshold")]
    NotCompressed,
    #[error("the connection has closed")]
    ConnectionClosed,
}

impl From<ReadingError> for PacketDecodeError {
    fn from(value: ReadingError) -> Self {
        Self::FailedDecompression(value.to_string())
    }
}

#[cfg(test)]
mod tests {

    use std::io::Write;

    use crate::ser::NetworkWriteExt;

    use super::*;
    use aes::Aes128;
    use cfb8::Encryptor as Cfb8Encryptor;
    use cfb8::cipher::AsyncStreamCipher;
    use flate2::Compression;
    use flate2::write::ZlibEncoder;

    /// Helper function to compress data using libdeflater's Zlib compressor
    fn compress_zlib(data: &[u8]) -> Vec<u8> {
        let mut compressed = Vec::new();
        ZlibEncoder::new(&mut compressed, Compression::default())
            .write_all(data)
            .unwrap();
        compressed
    }

    /// Helper function to encrypt data using AES-128 CFB-8 mode
    fn encrypt_aes128(data: &mut [u8], key: &[u8; 16], iv: &[u8; 16]) {
        let encryptor = Cfb8Encryptor::<Aes128>::new_from_slices(key, iv).expect("Invalid key/iv");
        encryptor.encrypt(data);
    }

    /// Helper function to build a packet with optional compression and encryption
    fn build_packet(
        packet_id: i32,
        payload: &[u8],
        compress: bool,
        key: Option<&[u8; 16]>,
        iv: Option<&[u8; 16]>,
    ) -> Vec<u8> {
        let mut buffer = Vec::new();

        if compress {
            // Create a buffer that includes `packet_id_varint` and payload
            let mut data_to_compress = Vec::new();
            let packet_id_varint = VarInt(packet_id);
            data_to_compress.write_var_int(&packet_id_varint).unwrap();
            data_to_compress.write_slice(payload).unwrap();

            // Compress the combined data
            let compressed_payload = compress_zlib(&data_to_compress);
            let data_len = data_to_compress.len() as i32; // 1 + payload.len()
            let data_len_varint = VarInt(data_len);
            buffer.write_var_int(&data_len_varint).unwrap();
            buffer.write_slice(&compressed_payload).unwrap();
        } else {
            // No compression; `data_len` is payload length
            let packet_id_varint = VarInt(packet_id);
            buffer.write_var_int(&packet_id_varint).unwrap();
            buffer.write_slice(payload).unwrap();
        }

        // Calculate packet length: length of buffer
        let packet_len = buffer.len() as i32;
        let packet_len_varint = VarInt(packet_len);
        let mut packet_length_encoded = Vec::new();
        {
            packet_len_varint
                .encode(&mut packet_length_encoded)
                .unwrap();
        }

        // Create a new buffer for the entire packet
        let mut packet = Vec::new();
        packet.extend_from_slice(&packet_length_encoded);
        packet.extend_from_slice(&buffer);

        // Encrypt if key and IV are provided.
        if let (Some(k), Some(v)) = (key, iv) {
            encrypt_aes128(&mut packet, k, v);
            packet
        } else {
            packet
        }
    }

    /// Test decoding without compression and encryption
    #[tokio::test]
    async fn test_decode_without_compression_and_encryption() {
        // Sample packet data: packet_id = 1, payload = "Hello"
        let packet_id = 1;
        let payload = b"Hello";

        // Build the packet without compression and encryption
        let packet = build_packet(packet_id, payload, false, None, None);

        // Initialize the decoder without compression and encryption
        let mut decoder = NetworkDecoder::new(packet.as_slice());

        // Attempt to decode
        let raw_packet = decoder.get_raw_packet().await.expect("Decoding failed");

        assert_eq!(raw_packet.id, packet_id);
        assert_eq!(raw_packet.payload.as_ref(), payload);
    }

    /// Test decoding with compression
    #[tokio::test]
    async fn test_decode_with_compression() {
        // Sample packet data: packet_id = 2, payload = "Hello, compressed world!"
        let packet_id = 2;
        let payload = b"Hello, compressed world!";

        // Build the packet with compression enabled
        let packet = build_packet(packet_id, payload, true, None, None);

        // Initialize the decoder with compression enabled
        let mut decoder = NetworkDecoder::new(packet.as_slice());
        // Larger than payload
        decoder.set_compression(1000);

        // Attempt to decode
        let raw_packet = decoder.get_raw_packet().await.expect("Decoding failed");

        assert_eq!(raw_packet.id, packet_id);
        assert_eq!(raw_packet.payload.as_ref(), payload);
    }

    /// Test decoding with encryption
    #[tokio::test]
    async fn test_decode_with_encryption() {
        // Sample packet data: packet_id = 3, payload = "Hello, encrypted world!"
        let packet_id = 3;
        let payload = b"Hello, encrypted world!";

        // Define encryption key and IV
        let key = [0x00u8; 16]; // Example key

        // Build the packet with encryption enabled (no compression)
        let packet = build_packet(packet_id, payload, false, Some(&key), Some(&key));

        // Initialize the decoder with encryption enabled
        let mut decoder = NetworkDecoder::new(packet.as_slice());
        decoder.set_encryption(&key);

        // Attempt to decode
        let raw_packet = decoder.get_raw_packet().await.expect("Decoding failed");

        assert_eq!(raw_packet.id, packet_id);
        assert_eq!(raw_packet.payload.as_ref(), payload);
    }

    /// Test decoding with both compression and encryption
    #[tokio::test]
    async fn test_decode_with_compression_and_encryption() {
        // Sample packet data: packet_id = 4, payload = "Hello, compressed and encrypted world!"
        let packet_id = 4;
        let payload = b"Hello, compressed and encrypted world!";

        // Define encryption key and IV
        let key = [0x01u8; 16]; // Example key
        let iv = [0x01u8; 16]; // Example IV

        // Build the packet with both compression and encryption enabled
        let packet = build_packet(packet_id, payload, true, Some(&key), Some(&iv));

        // Initialize the decoder with both compression and encryption enabled
        let mut decoder = NetworkDecoder::new(packet.as_slice());
        decoder.set_compression(1000);
        decoder.set_encryption(&key);

        // Attempt to decode
        let raw_packet = decoder.get_raw_packet().await.expect("Decoding failed");

        assert_eq!(raw_packet.id, packet_id);
        assert_eq!(raw_packet.payload.as_ref(), payload);
    }

    /// Test decoding with invalid compressed data
    #[tokio::test]
    async fn test_decode_with_invalid_compressed_data() {
        // Sample packet data: packet_id = 5, payload_len = 10, but compressed data is invalid
        let data_len = 10; // Expected decompressed size
        let invalid_compressed_data = vec![0xFF, 0xFF, 0xFF]; // Invalid Zlib data

        // Build the packet with compression enabled but invalid compressed data
        let mut buffer = Vec::new();
        let data_len_varint = VarInt(data_len);
        buffer.write_var_int(&data_len_varint).unwrap();
        buffer.write_slice(&invalid_compressed_data).unwrap();

        // Calculate packet length: VarInt(data_len) + invalid compressed data
        let packet_len = buffer.len() as i32;
        let packet_len_varint = VarInt(packet_len);

        // Create a new buffer for the entire packet
        let mut packet_buffer = Vec::new();
        packet_buffer.write_var_int(&packet_len_varint).unwrap();
        packet_buffer.write_slice(&buffer).unwrap();

        let packet_bytes = packet_buffer;

        // Initialize the decoder with compression enabled
        let mut decoder = NetworkDecoder::new(&packet_bytes[..]);
        decoder.set_compression(1000);

        // Attempt to decode and expect a decompression error
        let result = decoder.get_raw_packet().await;

        if result.is_ok() {
            panic!("This should have errored!");
        }
    }

    /// Test decoding with a zero-length packet
    #[tokio::test]
    async fn test_decode_with_zero_length_packet() {
        // Sample packet data: packet_id = 7, payload = "" (empty)
        let packet_id = 7;
        let payload = b"";

        // Build the packet without compression and encryption
        let packet = build_packet(packet_id, payload, false, None, None);

        // Initialize the decoder without compression and encryption
        let mut decoder = NetworkDecoder::new(packet.as_slice());

        // Attempt to decode and expect a read error
        let raw_packet = decoder.get_raw_packet().await.unwrap();
        assert_eq!(raw_packet.id, packet_id);
        assert_eq!(raw_packet.payload.as_ref(), payload);
    }

    /// Test decoding with maximum length packet
    #[tokio::test]
    async fn test_decode_with_maximum_length_packet() {
        // Sample packet data: packet_id = 8, payload = "A" repeated MAX_PACKET_SIZE times
        // Sample packet data: packet_id = 8, payload = "A" repeated (MAX_PACKET_SIZE - 1) times
        let packet_id = 8;
        let payload = vec![0x41u8; MAX_PACKET_SIZE as usize - 1]; // "A" repeated

        // Build the packet with compression enabled
        let packet = build_packet(packet_id, &payload, true, None, None);
        println!(
            "Built packet (with compression, maximum length): {:?}",
            packet
        );

        // Initialize the decoder with compression enabled
        let mut decoder = NetworkDecoder::new(packet.as_slice());
        decoder.set_compression(MAX_PACKET_SIZE as usize + 1);

        // Attempt to decode
        let result = decoder.get_raw_packet().await;

        let raw_packet = result.unwrap();
        assert_eq!(raw_packet.id, packet_id);
        assert_eq!(raw_packet.payload.as_ref(), payload);
    }
}
