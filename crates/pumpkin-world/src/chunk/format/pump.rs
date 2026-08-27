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
    D: SingleChunkDataSerializer + Send + Sync + Sized + 'static,
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
        let mut root = pumpkin_nbt::compound::NbtCompound::new();
        root.put_int("x", self.data.x);
        root.put_int("z", self.data.z);
        let mut chunks_comp = pumpkin_nbt::compound::NbtCompound::new();
        for (k, v) in &self.data.chunks {
            let i8_vec: Vec<i8> = v.iter().map(|&b| b as i8).collect();
            chunks_comp.put(k, pumpkin_nbt::tag::NbtTag::ByteArray(i8_vec.into()));
        }
        root.put_compound("chunks", chunks_comp);

        let bytes = pumpkin_nbt::Nbt::from(root).write_unnamed();
        tokio::fs::write(backend, bytes).await
    }

    fn read(r: Bytes) -> Result<Self, ChunkReadingError> {
        let mut cursor = std::io::Cursor::new(r);
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
            pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
        );
        let nbt = pumpkin_nbt::Nbt::read_unnamed(&mut reader).map_err(|e| {
            ChunkReadingError::ParsingError(
                crate::chunk::ChunkParsingError::ErrorDeserializingChunk(e.to_string()),
            )
        })?;

        let x = nbt.get_int("x").unwrap_or(0);
        let z = nbt.get_int("z").unwrap_or(0);
        let mut chunks = BTreeMap::new();
        if let Some(chunks_tag) = nbt.get_compound("chunks") {
            for (k, v) in &chunks_tag.child_tags {
                if let pumpkin_nbt::tag::NbtTag::ByteArray(arr) = v {
                    let u8_vec: Vec<u8> = arr.iter().map(|&b| b as u8).collect();
                    chunks.insert(k.to_string(), u8_vec);
                }
            }
        }

        Ok(Self {
            data: PumpData { x, z, chunks },
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
            .map_err(|e| ChunkWritingError::ChunkSerializingError(e.to_string()))?;

        let compressed = compress_to_vec(&bytes[..], CompressionLevel::Fastest);

        self.data.chunks.insert(index.to_string(), compressed);

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
                let res = (|| {
                    let mut decoder = StreamingDecoder::new(&chunk_bytes[..]).map_err(|e| {
                        ChunkReadingError::IoError(std::io::Error::other(e.to_string()))
                    })?;
                    let mut decompressed = Vec::new();
                    std::io::Read::read_to_end(&mut decoder, &mut decompressed)
                        .map_err(ChunkReadingError::IoError)?;
                    let bytes = Bytes::from(decompressed);
                    D::from_bytes(&bytes, pos)
                })();

                let data_res = match res {
                    Ok(data) => LoadedData::Loaded(data),
                    Err(e) => LoadedData::Error((pos, e)),
                };

                if stream.send(data_res).await.is_err() {
                    return;
                }
            } else {
                let _ = stream.send(LoadedData::Missing(pos)).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::ChunkReadingError;
    use crate::chunk::ChunkSerializingError;
    use crate::chunk::format::anvil::SingleChunkDataSerializer;
    use crate::chunk::io::Dirtiable;
    use crate::chunk::io::{ChunkSerializer, LoadedData};
    use bytes::Bytes;
    use pumpkin_util::math::vector2::Vector2;
    use tempfile::TempDir;

    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct MockChunk {
        x: i32,
        z: i32,
        data: Vec<u8>,
    }

    impl Dirtiable for MockChunk {
        fn is_dirty(&self) -> bool {
            true
        }
        fn mark_dirty(&self, _: bool) {}
    }

    impl SingleChunkDataSerializer for MockChunk {
        fn to_bytes(&self) -> Result<Bytes, ChunkSerializingError> {
            let mut root = pumpkin_nbt::compound::NbtCompound::new();
            root.put_int("x", self.x);
            root.put_int("z", self.z);
            let i8_vec: Vec<i8> = self.data.iter().map(|&b| b as i8).collect();
            root.put("data", pumpkin_nbt::tag::NbtTag::ByteArray(i8_vec.into()));
            let bytes = pumpkin_nbt::Nbt::from(root).write_unnamed();
            Ok(bytes)
        }
        fn from_bytes(bytes: &Bytes, pos: Vector2<i32>) -> Result<Self, ChunkReadingError> {
            let mut cursor = std::io::Cursor::new(bytes);
            let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
                pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
            );
            let nbt = pumpkin_nbt::Nbt::read_unnamed(&mut reader).map_err(|e| {
                ChunkReadingError::ParsingError(
                    crate::chunk::ChunkParsingError::ErrorDeserializingChunk(e.to_string()),
                )
            })?;
            let data = match nbt.get("data") {
                Some(pumpkin_nbt::tag::NbtTag::ByteArray(arr)) => {
                    arr.iter().map(|&b| b as u8).collect()
                }
                _ => Vec::new(),
            };
            Ok(Self {
                x: pos.x,
                z: pos.y,
                data,
            })
        }
        fn position(&self) -> (i32, i32) {
            (self.x, self.z)
        }
    }

    #[tokio::test]
    async fn pump_file_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("r.0.0.pump");

        let mut pump_file: PumpFile<MockChunk> = PumpFile::default();
        let chunk = MockChunk {
            x: 0,
            z: 0,
            data: vec![1, 2, 3],
        };

        pump_file.update_chunk(&chunk, &()).await.unwrap();
        pump_file.write(&file_path).await.unwrap();

        let bytes = tokio::fs::read(&file_path).await.unwrap();
        let read_file = PumpFile::<MockChunk>::read(Bytes::from(bytes)).unwrap();

        assert_eq!(read_file.data.chunks.len(), 1);
        let (stream_tx, mut stream_rx) = tokio::sync::mpsc::channel(1);
        read_file
            .get_chunks(vec![Vector2::new(0, 0)], stream_tx)
            .await;

        let loaded = stream_rx.recv().await.unwrap();
        match loaded {
            LoadedData::Loaded(c) => {
                assert_eq!(c.data, vec![1, 2, 3]);
            }
            _ => panic!("Expected LoadedData::Loaded"),
        }
    }
}
