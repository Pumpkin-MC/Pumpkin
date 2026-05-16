use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::path::PathBuf;

use crate::chunk::format::anvil::SingleChunkDataSerializer;
use crate::chunk::io::{ChunkSerializer, LoadedData};
use crate::chunk::{ChunkReadingError, ChunkWritingError};
use bytes::Bytes;
use pumpkin_util::math::vector2::Vector2;
use ruzstd::decoding::StreamingDecoder;
use ruzstd::encoding::{CompressionLevel, compress_to_vec};
use serde::{Deserialize, Serialize};

pub struct PumpFile<D> {
    pub data: PumpData,
    _phantom: PhantomData<D>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PumpData {
    pub x: i32,
    pub z: i32,
    pub chunks: BTreeMap<String, Vec<u8>>,
}

impl<D> Default for PumpFile<D> {
    fn default() -> Self {
        Self {
            data: PumpData::default(),
            _phantom: PhantomData,
        }
    }
}

impl<D> ChunkSerializer for PumpFile<D>
where
    D: SingleChunkDataSerializer + Send + Sync + Sized,
{
    type Data = D;
    type WriteBackend = PathBuf;
    type ChunkConfig = ();

    fn get_chunk_key(chunk: &Vector2<i32>) -> String {
        let region_x = chunk.x >> 5;
        let region_z = chunk.y >> 5;
        format!("r.{region_x}.{region_z}.pump")
    }

    fn should_write(&self, _is_watched: bool) -> bool {
        true
    }

    async fn write(&self, backend: &Self::WriteBackend) -> Result<(), std::io::Error> {
        let mut bytes = Vec::new();
        pumpkin_nbt::to_bytes_unnamed(&self.data, &mut bytes)
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let compressed = compress_to_vec(&bytes[..], CompressionLevel::Fastest);

        tokio::fs::write(backend, compressed).await
    }

    fn read(r: Bytes) -> Result<Self, ChunkReadingError> {
        let mut decoder = StreamingDecoder::new(&r[..])
            .map_err(|e| ChunkReadingError::IoError(std::io::Error::other(e.to_string())))?;
        let mut decompressed = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut decompressed)
            .map_err(ChunkReadingError::IoError)?;

        let data: PumpData = pumpkin_nbt::from_bytes_unnamed(std::io::Cursor::new(decompressed))
            .map_err(|e| {
                ChunkReadingError::ParsingError(
                    crate::chunk::ChunkParsingError::ErrorDeserializingChunk(e.to_string()),
                )
            })?;

        Ok(Self {
            data,
            _phantom: PhantomData,
        })
    }

    async fn update_chunk(
        &mut self,
        chunk_data: &Self::Data,
        _chunk_config: &Self::ChunkConfig,
    ) -> Result<(), ChunkWritingError> {
        let (x, z) = chunk_data.position();
        self.data.x = x >> 5;
        self.data.z = z >> 5;
        let rel_x = x.rem_euclid(32);
        let rel_z = z.rem_euclid(32);
        let index = (rel_x + rel_z * 32) as usize;

        let bytes = chunk_data
            .to_bytes()
            .await
            .map_err(|e| ChunkWritingError::ChunkSerializingError(e.to_string()))?;

        self.data.chunks.insert(index.to_string(), bytes.to_vec());

        Ok(())
    }

    async fn get_chunks(
        &self,
        chunks: Vec<Vector2<i32>>,
        stream: tokio::sync::mpsc::Sender<LoadedData<Self::Data, ChunkReadingError>>,
    ) {
        for pos in chunks {
            let rel_x = pos.x.rem_euclid(32);
            let rel_z = pos.y.rem_euclid(32);
            let index = (rel_x + rel_z * 32) as usize;

            if let Some(chunk_bytes) = self.data.chunks.get(&index.to_string()) {
                let bytes = Bytes::copy_from_slice(chunk_bytes);
                match D::from_bytes(&bytes, pos) {
                    Ok(data) => {
                        let _ = stream.send(LoadedData::Loaded(data)).await;
                    }
                    Err(e) => {
                        let _ = stream.send(LoadedData::Error((pos, e))).await;
                    }
                }
            } else {
                let _ = stream.send(LoadedData::Missing(pos)).await;
            }
        }
    }
}
