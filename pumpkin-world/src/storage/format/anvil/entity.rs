use std::path::PathBuf;

use async_trait::async_trait;
use bytes::Bytes;
use pumpkin_nbt::{
    compound::NbtCompound, deserializer::ReadAdaptor, serializer::WriteAdaptor, tag::NbtTag,
};
use pumpkin_util::math::vector2::Vector2;

use crate::{
    level::LevelFolder,
    storage::{
        ChunkReadingError, ChunkSerializingError, ChunkWritingError,
        format::{BytesToData, DataToBytes, EntityNbt, get_chunk_index},
        io::{DataSerializer, LoadedData},
    },
};

use super::AnvilFile;

#[derive(Default)]
pub struct AnvilEntityFormat {
    anvil: AnvilFile,
}

#[async_trait]
impl DataSerializer for AnvilEntityFormat {
    type Data = EntityNbt;
    type WriteBackend = PathBuf;

    fn get_chunk_key(chunk: &Vector2<i32>) -> String {
        AnvilFile::get_chunk_key(chunk)
    }

    fn get_folder(folder: &LevelFolder, file_name: &str) -> PathBuf {
        folder.entities_folder.join(file_name)
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

    // TODO: Should this be called something more generic now that we're working with other data?
    async fn update_chunk(&mut self, chunk: &Self::Data) -> Result<(), ChunkWritingError> {
        self.anvil.update_chunk::<Self>(chunk.position, chunk).await
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

    fn data_to_bytes(data: &Self::Data) -> Result<Vec<u8>, ChunkSerializingError> {
        let mut content = NbtCompound::new();
        content.put_int("DataVersion", data.data_version);
        content.put(
            "Position",
            NbtTag::IntArray(vec![data.position.x, data.position.z].into_boxed_slice()),
        );
        let mut entities = Vec::new();
        for entity in &data.entities {
            entities.push(NbtTag::Compound(entity.clone()));
        }
        content.put_list("Entities", entities.into_boxed_slice());

        let mut result = Vec::new();
        let mut writer = WriteAdaptor::new(&mut result);
        content
            .serialize_content(&mut writer)
            .map_err(ChunkSerializingError::ErrorSerializingChunk)?;
        Ok(result)
    }
}

impl BytesToData for AnvilEntityFormat {
    type Data = EntityNbt;

    fn bytes_to_data(
        chunk_data: &[u8],
        _position: Vector2<i32>,
    ) -> Result<Self::Data, crate::storage::ChunkParsingError> {
        let content = NbtCompound::deserialize_content(&mut ReadAdaptor::new(chunk_data)).unwrap();
        let data_version = content.get_int("DataVersion").unwrap();
        let position = content.get_int_array("Position").unwrap();
        let entities = content.get_list("Entities").unwrap();
        let mut entity_components = Vec::new();
        for entity in entities {
            entity_components.push(entity.extract_compound().unwrap().clone());
        }
        Ok(EntityNbt {
            data_version,
            position: Vector2::new(position[0], position[1]),
            entities: entity_components,
        })
    }
}
