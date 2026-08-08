use aes::cipher::KeyIvInit;
use bytes::Bytes;
use flate2::{Compress, Compression, FlushCompress, Status};
use thiserror::Error;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::{
    Aes128Cfb8Enc, CompressionLevel, CompressionThreshold, MAX_PACKET_DATA_SIZE, MAX_PACKET_SIZE,
    PacketEncodeError, StreamEncryptor, VarInt,
};

// raw -> compress -> encrypt

/// Upper bound on zlib (deflate) output size for `data_len` input bytes.
///
/// `compress_vec` never grows the caller's buffer, so the destination must be
/// pre-sized. Deflate expands incompressible input by at most a few bytes per
/// 64 KiB stored block plus a fixed header/trailer; `data_len / 16` is a
/// generous multiple of that worst case and keeps the output buffer always
/// large enough to reach `StreamEnd`.
const fn deflate_output_capacity(data_len: usize) -> usize {
    data_len.saturating_add(data_len / 16).saturating_add(64)
}
/// A validated Java packet payload consisting of a packet ID `VarInt` followed by packet fields.
///
/// This type deliberately excludes the outer packet-length prefix, compression framing, and
/// connection-local encryption.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SerializedPacket {
    bytes: Bytes,
}

impl SerializedPacket {
    /// Validates bytes supplied by raw/plugin packet APIs.
    pub fn try_from_bytes(bytes: Bytes) -> Result<Self, PacketEncodeError> {
        if bytes.is_empty() {
            return Err(PacketEncodeError::Message(
                "Serialized packet must contain a packet ID".into(),
            ));
        }
        if bytes.len() > MAX_PACKET_DATA_SIZE {
            return Err(PacketEncodeError::TooLong(bytes.len()));
        }

        let mut packet_id = 0u32;
        let mut complete = false;
        for (index, byte) in bytes.iter().copied().take(5).enumerate() {
            packet_id |= u32::from(byte & 0x7f) << (index * 7);
            if byte & 0x80 == 0 {
                complete = true;
                break;
            }
        }
        if !complete || packet_id > i32::MAX as u32 {
            return Err(PacketEncodeError::Message(
                "Serialized packet has an invalid packet ID VarInt".into(),
            ));
        }

        Ok(Self { bytes })
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &Bytes {
        &self.bytes
    }

    #[must_use]
    pub fn into_bytes(self) -> Bytes {
        self.bytes
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.bytes.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CompressionProfile {
    pub threshold: CompressionThreshold,
    pub level: CompressionLevel,
}

impl From<(CompressionThreshold, CompressionLevel)> for CompressionProfile {
    fn from((threshold, level): (CompressionThreshold, CompressionLevel)) -> Self {
        Self { threshold, level }
    }
}

/// Complete Java packet framing before connection-local encryption.
#[derive(Clone, Debug)]
pub struct PreparedPacket {
    bytes: Bytes,
    compression: Option<CompressionProfile>,
}

impl PreparedPacket {
    #[must_use]
    pub const fn as_bytes(&self) -> &Bytes {
        &self.bytes
    }

    #[must_use]
    pub const fn compression(&self) -> Option<CompressionProfile> {
        self.compression
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.bytes.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

pub fn prepare_packet(
    packet: &SerializedPacket,
    compression: Option<CompressionProfile>,
) -> Result<PreparedPacket, PacketEncodeError> {
    let packet_data = packet.as_bytes();
    let data_len = packet_data.len();
    let data_len_var_int = VarInt::try_from(data_len)
        .map_err(|_| PacketEncodeError::Message("Packet data length exceeds VarInt".into()))?;
    let mut output = Vec::with_capacity(data_len.saturating_add(10));

    if let Some(profile) = compression {
        if data_len >= profile.threshold {
            let mut compressor = Compress::new(Compression::new(profile.level), true);
            let mut compressed = Vec::with_capacity(deflate_output_capacity(data_len));
            let status = compressor
                .compress_vec(packet_data, &mut compressed, FlushCompress::Finish)
                .map_err(|err| PacketEncodeError::CompressionFailed(err.to_string()))?;
            if status != Status::StreamEnd {
                return Err(PacketEncodeError::CompressionFailed(format!(
                    "Unexpected compressor status: {status:?}"
                )));
            }
            let packet_len = VarInt::try_from(data_len_var_int.written_size() + compressed.len())
                .map_err(|_| PacketEncodeError::TooLong(data_len))?;
            packet_len
                .encode(&mut output)
                .map_err(|err| PacketEncodeError::Message(err.to_string()))?;
            data_len_var_int
                .encode(&mut output)
                .map_err(|err| PacketEncodeError::Message(err.to_string()))?;
            output.extend_from_slice(&compressed);
        } else {
            let packet_len = VarInt::try_from(VarInt(0).written_size() + data_len)
                .map_err(|_| PacketEncodeError::TooLong(data_len))?;
            packet_len
                .encode(&mut output)
                .map_err(|err| PacketEncodeError::Message(err.to_string()))?;
            VarInt(0)
                .encode(&mut output)
                .map_err(|err| PacketEncodeError::Message(err.to_string()))?;
            output.extend_from_slice(packet_data);
        }
    } else {
        data_len_var_int
            .encode(&mut output)
            .map_err(|err| PacketEncodeError::Message(err.to_string()))?;
        output.extend_from_slice(packet_data);
    }

    if output.len() > MAX_PACKET_SIZE as usize {
        return Err(PacketEncodeError::TooLong(output.len()));
    }
    Ok(PreparedPacket {
        bytes: output.into(),
        compression,
    })
}

pub enum EncryptionWriter<W: AsyncWrite + Unpin> {
    Encrypt(Box<StreamEncryptor<W>>),
    None(W),
}

impl<W: AsyncWrite + Unpin> EncryptionWriter<W> {
    #[must_use]
    pub fn upgrade(self, cipher: Aes128Cfb8Enc) -> Self {
        match self {
            Self::None(stream) => Self::Encrypt(Box::new(StreamEncryptor::new(cipher, stream))),
            Self::Encrypt(_) => self,
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
/// Supports `ZLib` endecoding/compression
/// Supports Aes128 Encryption
pub struct TCPNetworkEncoder<W: AsyncWrite + Unpin> {
    writer: Option<EncryptionWriter<W>>,
    // compression and compression threshold
    compression: Option<CompressionProfile>,
    // Reused compressor to avoid constructing zlib state per packet.
    compressor: Option<(CompressionLevel, Compress)>,
    // Reused compression buffer to avoid allocating a new Vec for each packet.
    compression_scratch: Vec<u8>,
}

impl<W: AsyncWrite + Unpin> TCPNetworkEncoder<W> {
    pub const fn new(writer: W) -> Self {
        Self {
            writer: Some(EncryptionWriter::None(writer)),
            compression: None,
            compressor: None,
            compression_scratch: Vec::new(),
        }
    }

    pub const fn set_compression(
        &mut self,
        compression_info: (CompressionThreshold, CompressionLevel),
    ) {
        self.compression = Some(CompressionProfile {
            threshold: compression_info.0,
            level: compression_info.1,
        });
    }

    /// NOTE: Encryption can only be set; a minecraft stream cannot go back to being unencrypted
    pub fn set_encryption(&mut self, key: &[u8; 16]) -> Result<(), PacketEncodeError> {
        if matches!(self.writer, Some(EncryptionWriter::Encrypt(_))) {
            return Err(PacketEncodeError::Message(
                "Encryption already enabled".into(),
            ));
        }
        let cipher = Aes128Cfb8Enc::new_from_slices(key, key)
            .map_err(|_| PacketEncodeError::Message("Invalid key".into()))?;

        if let Some(writer) = self.writer.take() {
            self.writer = Some(writer.upgrade(cipher));
        }
        Ok(())
    }

    fn compress_packet_data(
        &mut self,
        packet_data: &[u8],
        compression_level: CompressionLevel,
    ) -> Result<(), PacketEncodeError> {
        self.compression_scratch.clear();
        let reserve_hint = deflate_output_capacity(packet_data.len());
        let current_capacity = self.compression_scratch.capacity();
        if reserve_hint > current_capacity {
            self.compression_scratch
                .reserve(reserve_hint.saturating_sub(current_capacity));
        }

        let needs_new_compressor = match self.compressor.as_ref() {
            Some((level, _)) => *level != compression_level,
            None => true,
        };
        if needs_new_compressor {
            self.compressor = Some((
                compression_level,
                Compress::new(Compression::new(compression_level), true),
            ));
        }

        let (_, compressor) = self.compressor.as_mut().ok_or_else(|| {
            PacketEncodeError::Message("compressor must be present after initialization".into())
        })?;
        compressor.reset();
        let status = compressor
            .compress_vec(
                packet_data,
                &mut self.compression_scratch,
                FlushCompress::Finish,
            )
            .map_err(|err| PacketEncodeError::CompressionFailed(err.to_string()))?;

        if !matches!(status, Status::StreamEnd) {
            return Err(PacketEncodeError::CompressionFailed(format!(
                "Unexpected compressor status: {status:?}"
            )));
        }
        Ok(())
    }

    /// Appends a Clientbound `ClientPacket` to the internal buffer and applies compression when needed.
    ///
    /// If compression is enabled and the packet size exceeds the threshold, the packet is compressed.
    /// The packet is prefixed with its length and, if compressed, the uncompressed data length.
    /// The packet format is as follows:
    ///
    /// **Uncompressed:**
    /// |-----------------------|
    /// | Packet Length (`VarInt`)|
    /// |-----------------------|
    /// | Packet ID (`VarInt`)    |
    /// |-----------------------|
    /// | Data (Byte Array)     |
    /// |-----------------------|
    ///
    /// **Compressed:**
    /// |------------------------|
    /// | Packet Length (`VarInt`) |
    /// |------------------------|
    /// | Data Length (`VarInt`)   |
    /// |------------------------|
    /// | Packet ID (`VarInt`)     |
    /// |------------------------|
    /// | Data (Byte Array)      |
    /// |------------------------|
    ///
    /// -   `Packet Length`: The total length of the packet *excluding* the `Packet Length` field itself.
    /// -   `Data Length`: (Only present in compressed packets) The length of the uncompressed `Packet ID` and `Data`.
    /// -   `Packet ID`: The ID of the packet.
    /// -   `Data`: The packet's data.
    ///
    /// NOTE: This method does not flush. Call [`Self::flush`] to flush buffered data.
    #[allow(clippy::too_many_lines)]
    pub async fn write_packet(
        &mut self,
        packet: SerializedPacket,
    ) -> Result<(), PacketEncodeError> {
        let packet_data = packet.as_bytes();
        let data_len = packet_data.len();
        if data_len > MAX_PACKET_DATA_SIZE {
            return Err(PacketEncodeError::TooLong(data_len));
        }

        let data_len_var_int: VarInt = data_len.try_into().map_err(|_| {
            PacketEncodeError::Message(format!(
                "Packet data length is too large to fit in VarInt! ({data_len})"
            ))
        })?;

        let mut header_buf = [0u8; 10];
        let mut header_cursor = std::io::Cursor::new(&mut header_buf[..]);

        let payload_to_write: &[u8] = if let Some(compression) = self.compression {
            if data_len >= compression.threshold {
                self.compress_packet_data(packet_data.as_ref(), compression.level)?;
                debug_assert!(!self.compression_scratch.is_empty());

                let full_packet_len_var_int: VarInt = (data_len_var_int.written_size()
                    + self.compression_scratch.len())
                .try_into()
                .map_err(|_| {
                    PacketEncodeError::Message(format!(
                        "Full packet length is too large to fit in VarInt! ({data_len})"
                    ))
                })?;

                let complete_serialization_length =
                    full_packet_len_var_int.written_size() + full_packet_len_var_int.0 as usize;
                if complete_serialization_length > MAX_PACKET_SIZE as usize {
                    return Err(PacketEncodeError::TooLong(complete_serialization_length));
                }

                full_packet_len_var_int
                    .encode(&mut header_cursor)
                    .map_err(|err| PacketEncodeError::Message(err.to_string()))?;
                data_len_var_int
                    .encode(&mut header_cursor)
                    .map_err(|err| PacketEncodeError::Message(err.to_string()))?;

                self.compression_scratch.as_slice()
            } else {
                let data_len_var_int: VarInt = 0.into();
                let full_packet_len_var_int: VarInt = (data_len_var_int.written_size() + data_len)
                    .try_into()
                    .map_err(|_| {
                        PacketEncodeError::Message(format!(
                            "Full packet length is too large to fit in VarInt! ({data_len})"
                        ))
                    })?;

                let complete_serialization_length =
                    full_packet_len_var_int.written_size() + full_packet_len_var_int.0 as usize;
                if complete_serialization_length > MAX_PACKET_SIZE as usize {
                    return Err(PacketEncodeError::TooLong(complete_serialization_length));
                }

                full_packet_len_var_int
                    .encode(&mut header_cursor)
                    .map_err(|err| PacketEncodeError::Message(err.to_string()))?;
                data_len_var_int
                    .encode(&mut header_cursor)
                    .map_err(|err| PacketEncodeError::Message(err.to_string()))?;

                packet_data.as_ref()
            }
        } else {
            let full_packet_len_var_int: VarInt = data_len_var_int;

            let complete_serialization_length =
                full_packet_len_var_int.written_size() + full_packet_len_var_int.0 as usize;
            if complete_serialization_length > MAX_PACKET_SIZE as usize {
                return Err(PacketEncodeError::TooLong(complete_serialization_length));
            }

            full_packet_len_var_int
                .encode(&mut header_cursor)
                .map_err(|err| PacketEncodeError::Message(err.to_string()))?;

            packet_data.as_ref()
        };

        let header_len = header_cursor.position() as usize;
        let header_bytes = &header_buf[..header_len];
        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| PacketEncodeError::Message("Writer missing".into()))?;

        writer
            .write_all(header_bytes)
            .await
            .map_err(|err| PacketEncodeError::Message(err.to_string()))?;
        writer
            .write_all(payload_to_write)
            .await
            .map_err(|err| PacketEncodeError::Message(err.to_string()))?;

        Ok(())
    }

    pub async fn write_prepared_packet(
        &mut self,
        packet: &PreparedPacket,
    ) -> Result<(), PacketEncodeError> {
        if packet.compression != self.compression {
            return Err(PacketEncodeError::Message(
                "Prepared packet compression does not match connection".into(),
            ));
        }
        self.writer
            .as_mut()
            .ok_or_else(|| PacketEncodeError::Message("Writer missing".into()))?
            .write_all(&packet.bytes)
            .await
            .map_err(|err| PacketEncodeError::Message(err.to_string()))
    }

    pub async fn flush(&mut self) -> Result<(), PacketEncodeError> {
        self.writer
            .as_mut()
            .ok_or_else(|| PacketEncodeError::Message("Writer missing".into()))?
            .flush()
            .await
            .map_err(|err| PacketEncodeError::Message(err.to_string()))
    }
}

#[derive(Error, Debug)]
#[error("Invalid compression Level")]
pub struct CompressionLevelError;

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;
    use crate::java::client::status::CStatusResponse;
    use crate::packet::MultiVersionJavaPacket;
    use crate::ser::{NetworkReadExt, NetworkWriteExt};
    use crate::{ClientPacket, ReadingError};
    use aes::Aes128;
    use cfb8::Decryptor as Cfb8Decryptor;
    use flate2::read::ZlibDecoder;
    use pumpkin_data::packet::clientbound::STATUS_STATUS_RESPONSE;
    use pumpkin_macros::java_packet;
    use pumpkin_util::version::JavaMinecraftVersion;

    /// Define a custom packet for testing maximum packet size
    #[java_packet(STATUS_STATUS_RESPONSE)]
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

    impl ClientPacket for MaxSizePacket {
        fn write_packet_data(
            &self,
            mut write: impl std::io::Write,
            _version: &JavaMinecraftVersion,
        ) -> Result<(), crate::WritingError> {
            write
                .write_all(&self.data)
                .map_err(crate::WritingError::IoError)?;
            Ok(())
        }
    }

    /// Helper function to decode a `VarInt` from bytes
    fn decode_varint(buffer: &mut &[u8]) -> Result<i32, ReadingError> {
        Ok(buffer.get_var_int()?.0)
    }

    /// Helper function to decompress data using libdeflater's Zlib decompressor
    fn decompress_zlib(data: &[u8], expected_size: usize) -> Result<Vec<u8>, std::io::Error> {
        assert!(!data.is_empty());
        let mut decompressed = vec![0u8; expected_size];
        ZlibDecoder::new(data).read_exact(&mut decompressed)?;
        Ok(decompressed)
    }

    /// Helper function to decrypt data using AES-128 CFB-8 mode
    fn decrypt_aes128(
        encrypted_data: &mut [u8],
        key: &[u8; 16],
        iv: &[u8; 16],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut decryptor =
            Cfb8Decryptor::<Aes128>::new_from_slices(key, iv).map_err(|_| "Invalid key/iv")?;
        decryptor.decrypt(encrypted_data);
        Ok(())
    }

    /// Helper function to build a packet with optional compression and encryption
    async fn build_packet_with_encoder<T: ClientPacket>(
        packet: &T,
        compression_info: Option<(CompressionThreshold, CompressionLevel)>,
        key: Option<&[u8; 16]>,
    ) -> Result<Box<[u8]>, Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        let mut encoder = TCPNetworkEncoder::new(&mut buf);
        if let Some(compression_info) = compression_info {
            encoder.set_compression(compression_info);
        }

        if let Some(key) = key {
            encoder.set_encryption(key).map_err(|e| e.to_string())?;
        }

        let mut packet_buf = Vec::new();
        let writer = &mut packet_buf;
        writer.write_var_int(&VarInt(T::to_id(JavaMinecraftVersion::V_1_21_11)))?;
        packet.write_packet_data(writer, &JavaMinecraftVersion::V_1_21_11)?;

        encoder
            .write_packet(SerializedPacket::try_from_bytes(packet_buf.into())?)
            .await
            .map_err(|e| e.to_string())?;

        Ok(buf.into_boxed_slice())
    }

    /// Test encoding without compression and encryption
    #[tokio::test]
    async fn encode_without_compression_and_encryption() -> Result<(), Box<dyn std::error::Error>> {
        // Create a CStatusResponse packet
        let packet =
            CStatusResponse::new(String::from("{\"description\": \"A Minecraft Server\"}"));

        // Build the packet without compression and encryption
        let packet_bytes = build_packet_with_encoder(&packet, None, None).await?;

        // Decode the packet manually to verify correctness
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).map_err(|e| e.to_string())?;
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read packet ID VarInt
        let decoded_packet_id = decode_varint(&mut buffer).map_err(|e| e.to_string())?;
        assert_eq!(
            decoded_packet_id,
            CStatusResponse::to_id(JavaMinecraftVersion::V_1_21_11)
        );

        // Remaining buffer is the payload
        // We need to obtain the expected payload
        let mut expected_payload = Vec::new();
        packet.write_packet_data(&mut expected_payload, &JavaMinecraftVersion::V_1_21_11)?;

        assert_eq!(buffer, expected_payload);
        Ok(())
    }

    /// Test encoding with compression
    #[tokio::test]
    async fn encode_with_compression() -> Result<(), Box<dyn std::error::Error>> {
        // Create a CStatusResponse packet
        let packet = CStatusResponse::new("{\"description\": \"A Minecraft Server\"}".to_string());

        // Build the packet with compression enabled
        let packet_bytes = build_packet_with_encoder(&packet, Some((0, 6)), None).await?;

        // Decode the packet manually to verify correctness
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).map_err(|e| e.to_string())?;
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read data length VarInt (uncompressed data length)
        let data_length = decode_varint(&mut buffer).map_err(|e| e.to_string())?;
        let mut expected_payload = Vec::new();
        packet.write_packet_data(&mut expected_payload, &JavaMinecraftVersion::V_1_21_11)?;
        let uncompressed_data_length =
            VarInt(CStatusResponse::to_id(JavaMinecraftVersion::V_1_21_11)).written_size()
                + expected_payload.len();
        assert_eq!(data_length as usize, uncompressed_data_length);

        // Remaining buffer is the compressed data
        let compressed_data = buffer;

        // Decompress the data
        let decompressed_data = decompress_zlib(compressed_data, data_length as usize)?;

        // Verify packet ID and payload
        let mut decompressed_buffer = &decompressed_data[..];

        // Read packet ID VarInt
        let decoded_packet_id =
            decode_varint(&mut decompressed_buffer).map_err(|e| e.to_string())?;
        assert_eq!(
            decoded_packet_id,
            CStatusResponse::to_id(JavaMinecraftVersion::V_1_21_11)
        );

        // Remaining buffer is the payload
        assert_eq!(decompressed_buffer, expected_payload);
        Ok(())
    }

    /// Test encoding with encryption
    #[tokio::test]
    async fn encode_with_encryption() -> Result<(), Box<dyn std::error::Error>> {
        // Create a CStatusResponse packet
        let packet = CStatusResponse::new("{\"description\": \"A Minecraft Server\"}".to_string());

        // Encryption key and IV (IV is the same as key in this case)
        let key = [0x00u8; 16]; // Example key

        // Build the packet with encryption enabled (no compression)
        let mut packet_bytes = build_packet_with_encoder(&packet, None, Some(&key)).await?;

        // Decrypt the packet
        decrypt_aes128(&mut packet_bytes, &key, &key)?;

        // Decode the packet manually to verify correctness
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).map_err(|e| e.to_string())?;
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read packet ID VarInt
        let decoded_packet_id = decode_varint(&mut buffer).map_err(|e| e.to_string())?;
        assert_eq!(
            decoded_packet_id,
            CStatusResponse::to_id(JavaMinecraftVersion::V_1_21_11)
        );

        // Remaining buffer is the payload
        let mut expected_payload = Vec::new();
        packet.write_packet_data(&mut expected_payload, &JavaMinecraftVersion::V_1_21_11)?;
        assert_eq!(buffer, expected_payload);
        Ok(())
    }

    /// Test encoding with both compression and encryption
    #[tokio::test]
    async fn encode_with_compression_and_encryption() -> Result<(), Box<dyn std::error::Error>> {
        // Create a CStatusResponse packet
        let packet = CStatusResponse::new("{\"description\": \"A Minecraft Server\"}".to_string());

        // Encryption key and IV (IV is the same as key in this case)
        let key = [0x01u8; 16]; // Example key

        // Build the packet with both compression and encryption enabled
        // Compression threshold is set to 0 to force compression
        let mut packet_bytes = build_packet_with_encoder(&packet, Some((0, 6)), Some(&key)).await?;

        // Decrypt the packet
        decrypt_aes128(&mut packet_bytes, &key, &key)?;

        // Decode the packet manually to verify correctness
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).map_err(|e| e.to_string())?;
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read data length VarInt (uncompressed data length)
        let data_length = decode_varint(&mut buffer).map_err(|e| e.to_string())?;
        let mut expected_payload = Vec::new();
        packet.write_packet_data(&mut expected_payload, &JavaMinecraftVersion::V_1_21_11)?;
        let uncompressed_data_length =
            VarInt(CStatusResponse::to_id(JavaMinecraftVersion::V_1_21_11)).written_size()
                + expected_payload.len();
        assert_eq!(data_length as usize, uncompressed_data_length);

        // Remaining buffer is the compressed data
        let compressed_data = buffer;

        // Decompress the data
        let decompressed_data = decompress_zlib(compressed_data, data_length as usize)?;

        // Verify packet ID and payload
        let mut decompressed_buffer = &decompressed_data[..];

        // Read packet ID VarInt
        let decoded_packet_id =
            decode_varint(&mut decompressed_buffer).map_err(|e| e.to_string())?;
        assert_eq!(
            decoded_packet_id,
            CStatusResponse::to_id(JavaMinecraftVersion::V_1_21_11)
        );

        // Remaining buffer is the payload
        assert_eq!(decompressed_buffer, expected_payload);
        Ok(())
    }

    /// Test encoding with zero-length payload
    #[tokio::test]
    async fn encode_with_zero_length_payload() -> Result<(), Box<dyn std::error::Error>> {
        // Create a CStatusResponse packet with empty payload
        let packet = CStatusResponse::new(String::new());

        // Build the packet without compression and encryption
        let packet_bytes = build_packet_with_encoder(&packet, None, None).await?;

        // Decode the packet manually to verify correctness
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).map_err(|e| e.to_string())?;
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read packet ID VarInt
        let decoded_packet_id = decode_varint(&mut buffer).map_err(|e| e.to_string())?;
        assert_eq!(
            decoded_packet_id,
            CStatusResponse::to_id(JavaMinecraftVersion::V_1_21_11)
        );

        // Remaining buffer is the payload (empty)
        let mut expected_payload = Vec::new();
        packet.write_packet_data(&mut expected_payload, &JavaMinecraftVersion::V_1_21_11)?;

        assert_eq!(
            buffer.len(),
            expected_payload.len(),
            "Payload length mismatch"
        );
        assert_eq!(buffer, expected_payload);
        Ok(())
    }

    /// Test encoding with maximum length payload
    #[tokio::test]
    async fn encode_with_maximum_string_length() -> Result<(), Box<dyn std::error::Error>> {
        // Maximum allowed string length is 32767 bytes
        let max_string_length = 32767;
        let payload_str = "A".repeat(max_string_length);
        let packet = CStatusResponse::new(payload_str);

        // Build the packet without compression and encryption
        let packet_bytes = build_packet_with_encoder(&packet, None, None).await?;

        // Verify that the packet size does not exceed MAX_PACKET_SIZE as usize
        assert!(
            packet_bytes.len() <= MAX_PACKET_SIZE as usize,
            "Packet size exceeds maximum allowed size"
        );

        // Decode the packet manually to verify correctness
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).map_err(|e| e.to_string())?;
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read packet ID VarInt
        let decoded_packet_id = decode_varint(&mut buffer).map_err(|e| e.to_string())?;
        // Assume packet ID is 0 for CStatusResponse
        assert_eq!(
            decoded_packet_id,
            CStatusResponse::to_id(JavaMinecraftVersion::V_1_21_11)
        );

        // Remaining buffer is the payload
        let mut expected_payload = Vec::new();
        packet.write_packet_data(&mut expected_payload, &JavaMinecraftVersion::V_1_21_11)?;

        assert_eq!(buffer, expected_payload);
        Ok(())
    }

    /// Test encoding a packet that exceeds `MAX_PACKET_SIZE` as usize
    #[tokio::test]
    async fn encode_packet_exceeding_maximum_size() -> Result<(), Box<dyn std::error::Error>> {
        // Create a custom packet with data exceeding MAX_PACKET_SIZE as usize
        let data_size = MAX_PACKET_SIZE as usize + 1; // Exceed by 1 byte
        let packet = MaxSizePacket::new(data_size);

        // Build the packet without compression and encryption
        // This should return PacketEncodeError::TooLong
        let result = build_packet_with_encoder(&packet, None, None).await;
        assert!(result.is_err());
        // We can't easily check for TooLong since it's boxed, but we verified it returns an error
        Ok(())
    }

    /// Test encoding with a small payload that should not be compressed
    #[tokio::test]
    async fn encode_small_payload_no_compression() -> Result<(), Box<dyn std::error::Error>> {
        // Create a CStatusResponse packet with small payload
        let packet = CStatusResponse::new(String::from("Hi"));

        // Build the packet with compression enabled
        // Compression threshold is set to a value higher than payload length
        let packet_bytes = build_packet_with_encoder(&packet, Some((10, 6)), None).await?;

        // Decode the packet manually to verify that it was not compressed
        let mut buffer = &packet_bytes[..];

        // Read packet length VarInt
        let packet_length = decode_varint(&mut buffer).map_err(|e| e.to_string())?;
        assert_eq!(
            packet_length as usize,
            buffer.len(),
            "Packet length mismatch"
        );

        // Read data length VarInt (should be 0 indicating no compression)
        let data_length = decode_varint(&mut buffer).map_err(|e| e.to_string())?;
        assert_eq!(
            data_length, 0,
            "Data length should be 0 indicating no compression"
        );

        // Read packet ID VarInt
        let decoded_packet_id = decode_varint(&mut buffer).map_err(|e| e.to_string())?;
        assert_eq!(
            decoded_packet_id,
            CStatusResponse::to_id(JavaMinecraftVersion::V_1_21_11)
        );

        // Remaining buffer is the payload
        let mut expected_payload = Vec::new();
        packet.write_packet_data(&mut expected_payload, &JavaMinecraftVersion::V_1_21_11)?;

        assert_eq!(buffer, expected_payload);
        Ok(())
    }

    #[test]
    fn serialized_packet_validates_the_packet_id_boundary() {
        assert!(SerializedPacket::try_from_bytes(Bytes::new()).is_err());
        assert!(
            SerializedPacket::try_from_bytes(Bytes::from_static(&[
                0x80, 0x80, 0x80, 0x80, 0x80, 0x00
            ]))
            .is_err()
        );
        assert!(SerializedPacket::try_from_bytes(Bytes::from_static(&[0x00])).is_ok());
    }

    #[tokio::test]
    async fn prepared_packet_matches_regular_encoder() -> Result<(), Box<dyn std::error::Error>> {
        let compressions = [
            None,
            Some(CompressionProfile {
                threshold: usize::MAX,
                level: 4,
            }),
            Some(CompressionProfile {
                threshold: 64 * 1024,
                level: 4,
            }),
            Some(CompressionProfile {
                threshold: 0,
                level: 4,
            }),
        ];

        for compression in compressions {
            let packet = SerializedPacket::try_from_bytes(Bytes::from(vec![0x5a; 64 * 1024]))?;
            let prepared = prepare_packet(&packet, compression)?;

            let mut expected = Vec::new();
            let mut encoder = TCPNetworkEncoder::new(&mut expected);
            if let Some(profile) = compression {
                encoder.set_compression((profile.threshold, profile.level));
            }
            encoder.write_packet(packet).await?;
            encoder.flush().await?;

            assert_eq!(prepared.as_bytes().as_ref(), expected);
        }
        Ok(())
    }

    #[tokio::test]
    async fn prepared_packet_rejects_different_compression() {
        let packet = SerializedPacket::try_from_bytes(Bytes::from_static(b"\x00packet")).unwrap();
        let prepared = prepare_packet(
            &packet,
            Some(CompressionProfile {
                threshold: 0,
                level: 4,
            }),
        )
        .unwrap();
        let mut output = Vec::new();
        let mut encoder = TCPNetworkEncoder::new(&mut output);
        encoder.set_compression((256, 4));

        assert!(encoder.write_prepared_packet(&prepared).await.is_err());
        assert!(output.is_empty());
    }

    /// Deterministic high-entropy bytes so that tests are reproducible and the
    /// payload is not compressible by repetition.
    fn high_entropy_payload(size: usize) -> Vec<u8> {
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut payload = Vec::with_capacity(size);
        while payload.len() < size {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            payload.extend_from_slice(&state.to_le_bytes());
        }
        payload.truncate(size);
        payload
    }

    #[tokio::test]
    async fn prepared_packet_matches_encoder_for_high_entropy_payloads()
    -> Result<(), Box<dyn std::error::Error>> {
        for size in [64 * 1024, 256 * 1024, 1024 * 1024] {
            let profile = Some(CompressionProfile {
                threshold: 0,
                level: 4,
            });
            let packet = SerializedPacket::try_from_bytes(Bytes::from(high_entropy_payload(size)))?;
            let prepared = prepare_packet(&packet, profile)?;

            let mut expected = Vec::new();
            let mut encoder = TCPNetworkEncoder::new(&mut expected);
            encoder.set_compression((0, 4));
            encoder.write_packet(packet.clone()).await?;
            encoder.flush().await?;

            assert_eq!(prepared.as_bytes().as_ref(), expected, "size {size}");
        }
        Ok(())
    }

    #[tokio::test]
    async fn prepared_packet_handles_near_maximum_payload() -> Result<(), Box<dyn std::error::Error>>
    {
        let size = MAX_PACKET_SIZE as usize - 2048;
        let profile = Some(CompressionProfile {
            threshold: 0,
            level: 4,
        });
        let packet = SerializedPacket::try_from_bytes(Bytes::from(high_entropy_payload(size)))?;
        let prepared = prepare_packet(&packet, profile)?;
        assert!(prepared.len() <= MAX_PACKET_SIZE as usize);

        let mut expected = Vec::new();
        let mut encoder = TCPNetworkEncoder::new(&mut expected);
        encoder.set_compression((0, 4));
        encoder.write_packet(packet).await?;
        encoder.flush().await?;

        assert_eq!(prepared.as_bytes().as_ref(), expected);
        Ok(())
    }

    #[tokio::test]
    async fn prepared_packet_matches_encoder_at_every_compression_level()
    -> Result<(), Box<dyn std::error::Error>> {
        for level in 0..=9 {
            let profile = Some(CompressionProfile {
                threshold: 0,
                level,
            });
            let packet =
                SerializedPacket::try_from_bytes(Bytes::from(high_entropy_payload(256 * 1024)))?;
            let prepared = prepare_packet(&packet, profile)?;

            let mut expected = Vec::new();
            let mut encoder = TCPNetworkEncoder::new(&mut expected);
            encoder.set_compression((0, level));
            encoder.write_packet(packet.clone()).await?;
            encoder.flush().await?;

            assert_eq!(prepared.as_bytes().as_ref(), expected, "level {level}");
        }
        Ok(())
    }

    #[tokio::test]
    async fn prepared_packet_compressed_output_can_expand_beyond_source()
    -> Result<(), Box<dyn std::error::Error>> {
        let profile = Some(CompressionProfile {
            threshold: 0,
            level: 0,
        });
        let packet =
            SerializedPacket::try_from_bytes(Bytes::from(high_entropy_payload(256 * 1024)))?;
        let prepared = prepare_packet(&packet, profile)?;
        assert!(
            prepared.len() > packet.len(),
            "compressed output {} should exceed source {}",
            prepared.len(),
            packet.len()
        );
        Ok(())
    }
}
