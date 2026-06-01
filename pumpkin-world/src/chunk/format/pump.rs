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

        tokio::fs::write(backend, bytes).await
    }

    fn read(r: Bytes) -> Result<Self, ChunkReadingError> {
        let data: PumpData =
            pumpkin_nbt::from_bytes_unnamed(std::io::Cursor::new(r)).map_err(|e| {
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
                let mut decoder = match StreamingDecoder::new(&chunk_bytes[..]) {
                    Ok(d) => d,
                    Err(e) => {
                        let _ = stream
                            .send(LoadedData::Error((
                                pos,
                                ChunkReadingError::IoError(std::io::Error::other(e.to_string())),
                            )))
                            .await;
                        continue;
                    }
                };
                let mut decompressed = Vec::new();
                if let Err(e) = std::io::Read::read_to_end(&mut decoder, &mut decompressed) {
                    let _ = stream
                        .send(LoadedData::Error((pos, ChunkReadingError::IoError(e))))
                        .await;
                    continue;
                }

                let bytes = Bytes::from(decompressed);
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
    use serde::{Deserialize, Serialize};
    use std::future::Future;
    use std::pin::Pin;
    use temp_dir::TempDir;

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
        fn to_bytes(
            &self,
        ) -> Pin<Box<dyn Future<Output = Result<Bytes, ChunkSerializingError>> + Send + '_>>
        {
            let mut buf = Vec::new();
            pumpkin_nbt::to_bytes_unnamed(self, &mut buf).unwrap();
            let bytes = Bytes::from(buf);
            Box::pin(async move { Ok(bytes) })
        }
        fn from_bytes(bytes: &Bytes, pos: Vector2<i32>) -> Result<Self, ChunkReadingError> {
            let mut mock: MockChunk = pumpkin_nbt::from_bytes_unnamed(std::io::Cursor::new(bytes))
                .map_err(|e| {
                    ChunkReadingError::ParsingError(
                        crate::chunk::ChunkParsingError::ErrorDeserializingChunk(e.to_string()),
                    )
                })?;
            mock.x = pos.x;
            mock.z = pos.y;
            Ok(mock)
        }
        fn position(&self) -> (i32, i32) {
            (self.x, self.z)
        }
    }

    #[tokio::test]
    async fn test_pump_file_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.child("r.0.0.pump");

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
