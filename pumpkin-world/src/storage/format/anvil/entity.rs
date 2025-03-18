use std::path::PathBuf;

use bytes::Bytes;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector2::Vector2;

use crate::storage::{
    ChunkData, ChunkReadingError, ChunkSerializingError, ChunkWritingError,
    format::{BytesToData, DataToBytes, EntityNbt, get_chunk_index},
    io::{ChunkSerializer, LoadedData},
};

use super::{AnvilFile, chunk::AnvilChunkFormat};

#[derive(Default)]
pub struct AnvilEntityFormat {
    anvil: AnvilFile,
}

#[async_trait]
impl ChunkSerializer for AnvilEntityFormat {
    type Data = EntityNbt;
    type WriteBackend = PathBuf;

    fn get_chunk_key(chunk: &Vector2<i32>) -> String {
        AnvilFile::get_chunk_key(chunk)
    }

    fn should_write(&self, is_watched: bool) -> bool {
        self.anvil.should_write(is_watched)
    }

    async fn write(&self, path: PathBuf) -> Result<(), std::io::Error> {
        self.anvil.write(path).await
    }

    fn read(bytes: Bytes) -> Result<Self, ChunkReadingError> {
        let anvil = AnvilFile::read(bytes)?;
        Ok(Self { anvil })
    }

    async fn update_chunk(&mut self, chunk: &Self::Data) -> Result<(), ChunkWritingError> {
        self.anvil
            .update_chunk::<Self>(Vector2::new(chunk.position[0], chunk.position[1]), chunk)
            .await
    }

    async fn get_chunks(
        &self,
        chunks: &[Vector2<i32>],
        stream: tokio::sync::mpsc::Sender<LoadedData<Self::Data, ChunkReadingError>>,
    ) {
        // Create an unbounded buffer so we don't block the rayon thread pool
        let (bridge_send, mut bridge_recv) = tokio::sync::mpsc::unbounded_channel();

        // Don't par iter here so we can prevent backpressure with the await in the async
        // runtime
        for chunk in chunks.iter().cloned() {
            let index = get_chunk_index(&chunk);
            match &self.anvil.chunks_data[index] {
                None => stream
                    .send(LoadedData::Missing(chunk))
                    .await
                    .expect("Failed to send chunk"),
                Some(chunk_metadata) => {
                    let send = bridge_send.clone();
                    let chunk_data = chunk_metadata.serialized_data.clone();
                    rayon::spawn(move || {
                        let result = match chunk_data.to_chunk::<Self>(chunk) {
                            Ok(chunk) => LoadedData::Loaded(chunk),
                            Err(err) => LoadedData::Error((chunk, err)),
                        };

                        send.send(result)
                            .expect("Failed to send anvil chunks from rayon thread");
                    });
                }
            }
        }
        // Drop the original so streams clean-up
        drop(bridge_send);

        // We don't want to waste work, so recv unbounded from the rayon thread pool, then re-send
        // to the channel

        while let Some(data) = bridge_recv.recv().await {
            stream
                .send(data)
                .await
                .expect("Failed to send anvil chunks from bridge");
        }
    }
}

impl DataToBytes for AnvilEntityFormat {
    type Data = EntityNbt;

    fn data_to_bytes(chunk_data: &Self::Data) -> Result<Vec<u8>, ChunkSerializingError> {}
}

impl BytesToData for AnvilEntityFormat {
    type Data = EntityNbt;

    fn bytes_to_data(
        chunk_data: &[u8],
        position: Vector2<i32>,
    ) -> Result<Self::Data, crate::storage::ChunkParsingError> {
        todo!()
    }
}
