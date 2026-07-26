use bytes::{Buf, Bytes};
use pumpkin_config::chunk::AnvilChunkConfig;
use pumpkin_util::math::vector2::Vector2;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::chunk::{ChunkParsingError, ChunkReadingError, ChunkWritingError, CompressionError};

use super::{AnvilChunkData, Compression, SECTOR_BYTES, SingleChunkDataSerializer};

impl AnvilChunkData {
    /// Raw size of serialized chunk
    #[inline]
    const fn raw_write_size(&self) -> usize {
        // 4 bytes for the *length* and 1 byte for the *compression* method
        self.compressed_data.len() + 4 + 1
    }

    /// Size of serialized chunk with padding
    #[inline]
    const fn padded_size(&self) -> usize {
        let sector_count = self.sector_count() as usize;
        sector_count * SECTOR_BYTES
    }

    #[inline]
    pub(super) const fn sector_count(&self) -> u32 {
        let total_size = self.raw_write_size();
        total_size.div_ceil(SECTOR_BYTES) as u32
    }

    pub(super) fn from_bytes(bytes: Bytes) -> Result<Self, ChunkReadingError> {
        let mut bytes = bytes;
        if bytes.remaining() < 5 {
            return Err(ChunkReadingError::ParsingError(
                ChunkParsingError::ErrorDeserializingChunk(
                    "Chunk stream header is truncated".to_string(),
                ),
            ));
        }

        let stored_length = bytes.get_u32() as usize;
        if stored_length == 0 || stored_length > bytes.remaining() {
            return Err(ChunkReadingError::ParsingError(
                ChunkParsingError::ErrorDeserializingChunk(format!(
                    "Chunk length is invalid or greater than available bytes ({} vs {})",
                    stored_length,
                    bytes.len()
                )),
            ));
        }

        let compression_method = bytes.get_u8();
        let compression = Compression::from_byte(compression_method)
            .map_err(|()| ChunkReadingError::Compression(CompressionError::UnknownCompression))?;

        Ok(Self {
            compression,
            // If this has padding, we need to trim it
            compressed_data: bytes.slice(..stored_length - 1),
        })
    }

    pub(super) async fn write(
        &self,
        w: &mut (impl AsyncWrite + Unpin + Send),
    ) -> Result<(), std::io::Error> {
        let padded_size = self.padded_size();

        w.write_u32((self.compressed_data.remaining() + 1) as u32)
            .await?;
        w.write_u8(
            self.compression
                .map_or(Compression::NO_COMPRESSION_ID, |c| c as u8),
        )
        .await?;

        w.write_all(&self.compressed_data).await?;

        let padding_len = padded_size - self.raw_write_size();
        if padding_len > 0 {
            static PADDING: [u8; SECTOR_BYTES] = [0; SECTOR_BYTES];
            w.write_all(&PADDING[..padding_len]).await?;
        }

        Ok(())
    }

    pub(super) fn to_chunk<S>(&self, pos: Vector2<i32>) -> Result<S, ChunkReadingError>
    where
        S: SingleChunkDataSerializer,
    {
        if let Some(compression) = self.compression {
            let decompress_bytes = compression
                .decompress_data(&self.compressed_data)
                .map_err(ChunkReadingError::Compression)?;

            S::from_bytes(&decompress_bytes.into(), pos)
        } else {
            S::from_bytes(&self.compressed_data, pos)
        }
    }

    pub(super) async fn from_chunk<S>(
        chunk: &S,
        compression: Option<Compression>,
        chunk_config: &AnvilChunkConfig,
    ) -> Result<Self, ChunkWritingError>
    where
        S: SingleChunkDataSerializer,
    {
        let raw_bytes = chunk
            .to_bytes()
            .await
            .map_err(|err| ChunkWritingError::ChunkSerializingError(err.to_string()))?;

        let compression = compression.unwrap_or_else(|| chunk_config.compression.algorithm.into());

        // We need to buffer here anyway so there's no use in making an impl Write for this
        let compressed_data = compression
            .compress_data(&raw_bytes, chunk_config.compression.level)
            .map_err(ChunkWritingError::Compression)?;

        Ok(Self {
            compression: Some(compression),
            compressed_data: compressed_data.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{AnvilChunkData, Bytes, ChunkReadingError, Compression, SECTOR_BYTES};

    fn sample_chunk_stream(compression_byte: u8, payload: &[u8]) -> Bytes {
        let mut raw = Vec::new();
        raw.extend_from_slice(&((payload.len() + 1) as u32).to_be_bytes());
        raw.push(compression_byte);
        raw.extend_from_slice(payload);
        Bytes::from(raw)
    }

    #[test]
    fn compression_roundtrip_preserves_data() {
        let payload = b"pumpkin anvil chunk payload".repeat(64);
        for compression in [Compression::GZip, Compression::ZLib, Compression::LZ4] {
            let compressed = compression.compress_data(&payload, 6).unwrap();
            let decompressed = compression.decompress_data(&compressed).unwrap();
            assert_eq!(decompressed.as_ref(), payload.as_slice());
        }
    }

    #[test]
    fn from_bytes_parses_compression_and_payload() {
        let data = AnvilChunkData::from_bytes(sample_chunk_stream(2, &[0xAB, 0xCD])).unwrap();
        assert_eq!(data.compression, Some(Compression::ZLib));
        assert_eq!(data.compressed_data.as_ref(), &[0xAB, 0xCD]);
        assert_eq!(data.raw_write_size(), 2 + 4 + 1);
        assert_eq!(data.sector_count(), 1);
        assert_eq!(data.padded_size(), SECTOR_BYTES);
    }

    #[test]
    fn from_bytes_rejects_truncated_header() {
        assert!(matches!(
            AnvilChunkData::from_bytes(Bytes::from_static(&[0, 0, 0])),
            Err(ChunkReadingError::ParsingError(_))
        ));
    }

    #[test]
    fn from_bytes_rejects_invalid_length() {
        // The stored length is larger than the remaining bytes
        let mut raw = 16u32.to_be_bytes().to_vec();
        raw.push(2);
        assert!(matches!(
            AnvilChunkData::from_bytes(Bytes::from(raw)),
            Err(ChunkReadingError::ParsingError(_))
        ));
    }

    #[test]
    fn from_bytes_rejects_unknown_compression() {
        assert!(matches!(
            AnvilChunkData::from_bytes(sample_chunk_stream(99, &[0xAB])),
            Err(ChunkReadingError::Compression(_))
        ));
    }

    #[test]
    fn sector_count_covers_padding_boundary() {
        // 4091 payload bytes + 4-byte length + 1-byte compression tag = exactly one sector
        let payload = [0u8; 4091];
        let data = AnvilChunkData::from_bytes(sample_chunk_stream(2, &payload)).unwrap();
        assert_eq!(data.sector_count(), 1);

        // One more byte spills into a second sector
        let payload = [0u8; 4092];
        let data = AnvilChunkData::from_bytes(sample_chunk_stream(2, &payload)).unwrap();
        assert_eq!(data.sector_count(), 2);
    }
}
