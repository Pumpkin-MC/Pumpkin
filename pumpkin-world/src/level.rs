use std::{fs, path::PathBuf, sync::Arc};

use dashmap::{DashMap, Entry};
use log::trace;
use num_traits::Zero;
use pumpkin_config::{ADVANCED_CONFIG, chunk::ChunkFormat};
use pumpkin_util::math::vector2::Vector2;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tokio::sync::{RwLock, mpsc};

use crate::{
    chunk::{ChunkData, ChunkReadingError, anvil::AnvilChunkFile, linear::LinearFile},
    chunks_io::{ChunkFileManager, ChunkIO, LoadedData},
    generation::{Seed, WorldGenerator, get_world_gen},
    lock::{LevelLocker, anvil::AnvilLevelLocker},
    world_info::{
        LevelData, WorldInfoError, WorldInfoReader, WorldInfoWriter,
        anvil::{AnvilLevelInfo, LEVEL_DAT_BACKUP_FILE_NAME, LEVEL_DAT_FILE_NAME},
    },
};

/// The `Level` module provides functionality for working with chunks within or outside a Minecraft world.
///
/// Key features include:
///
/// - **Chunk Loading:** Efficiently loads chunks from disk.
/// - **Chunk Caching:** Stores accessed chunks in memory for faster access.
/// - **Chunk Generation:** Generates new chunks on-demand using a specified `WorldGenerator`.
///
/// For more details on world generation, refer to the `WorldGenerator` module.
pub struct Level {
    pub seed: Seed,
    pub level_info: LevelData,
    world_info_writer: Arc<dyn WorldInfoWriter>,
    level_folder: LevelFolder,
    loaded_chunks: Arc<DashMap<Vector2<i32>, Arc<RwLock<ChunkData>>>>,
    chunk_watchers: Arc<DashMap<Vector2<i32>, usize>>,
    chunk_saver: Arc<dyn ChunkIO<ChunkData>>,
    world_gen: Arc<dyn WorldGenerator>,
    // Gets unlocked when dropped
    // TODO: Make this a trait
    _locker: Arc<AnvilLevelLocker>,
}

#[derive(Clone)]
pub struct LevelFolder {
    pub root_folder: PathBuf,
    pub region_folder: PathBuf,
}

impl Level {
    pub fn from_root_folder(root_folder: PathBuf) -> Self {
        // If we are using an already existing world we want to read the seed from the level.dat, If not we want to check if there is a seed in the config, if not lets create a random one
        let region_folder = root_folder.join("region");
        if !region_folder.exists() {
            std::fs::create_dir_all(&region_folder).expect("Failed to create Region folder");
        }
        let level_folder = LevelFolder {
            root_folder,
            region_folder,
        };

        // if we fail to lock, lets crash ???. maybe not the best solution when we have a large server with many worlds and one is locked.
        // So TODO
        let locker = AnvilLevelLocker::look(&level_folder).expect("Failed to lock level");

        // TODO: Load info correctly based on world format type
        let level_info = AnvilLevelInfo.read_world_info(&level_folder);
        if let Err(error) = &level_info {
            match error {
                // If it doesn't exist, just make a new one
                WorldInfoError::InfoNotFound => (),
                WorldInfoError::UnsupportedVersion(version) => {
                    log::error!("Failed to load world info!, {version}");
                    log::error!("{}", error);
                    panic!("Unsupported world data! See the logs for more info.");
                }
                e => {
                    panic!("World Error {}", e);
                }
            }
        } else {
            let dat_path = level_folder.root_folder.join(LEVEL_DAT_FILE_NAME);
            if dat_path.exists() {
                let backup_path = level_folder.root_folder.join(LEVEL_DAT_BACKUP_FILE_NAME);
                fs::copy(dat_path, backup_path).unwrap();
            }
        }

        let level_info = level_info.unwrap_or_default(); // TODO: Improve error handling
        log::info!(
            "Loading world with seed: {}",
            level_info.world_gen_settings.seed
        );

        let seed = Seed(level_info.world_gen_settings.seed as u64);
        let world_gen = get_world_gen(seed).into();

        let chunk_saver: Arc<dyn ChunkIO<ChunkData>> = match ADVANCED_CONFIG.chunk.format {
            //ChunkFormat::Anvil => (Arc::new(AnvilChunkFormat), Arc::new(AnvilChunkFormat)),
            ChunkFormat::Linear => Arc::new(ChunkFileManager::<LinearFile>::default()),
            ChunkFormat::Anvil => Arc::new(ChunkFileManager::<AnvilChunkFile>::default()),
        };

        Self {
            seed,
            world_gen,
            world_info_writer: Arc::new(AnvilLevelInfo),
            level_folder,
            chunk_saver,
            loaded_chunks: Arc::new(DashMap::new()),
            chunk_watchers: Arc::new(DashMap::new()),
            level_info,
            _locker: Arc::new(locker),
        }
    }

    pub async fn save(&self) {
        log::info!("Saving level...");

        // save all stragling chunks
        let chunks_to_write = self
            .loaded_chunks
            .iter()
            .map(|chunk| (*chunk.key(), chunk.value().clone()))
            .collect::<Vec<_>>();
        self.loaded_chunks.clear();

        self.write_chunks(chunks_to_write).await;

        // wait for chunks currently saving in other threads
        self.chunk_saver.await_tasks().await;

        // then lets save the world info
        let result = self
            .world_info_writer
            .write_world_info(self.level_info.clone(), &self.level_folder);

        // Lets not stop the overall save for this
        if let Err(err) = result {
            log::error!("Failed to save level.dat: {}", err);
        }
    }

    pub fn get_block() {}

    pub fn loaded_chunk_count(&self) -> usize {
        self.loaded_chunks.len()
    }

    pub async fn clean_up_log(&self) {
        self.chunk_saver.clean_up_log().await;
    }

    pub fn list_cached(&self) {
        for entry in self.loaded_chunks.iter() {
            log::debug!("In map: {:?}", entry.key());
        }
    }

    /// Marks chunks as "watched" by a unique player. When no players are watching a chunk,
    /// it is removed from memory. Should only be called on chunks the player was not watching
    /// before
    pub fn mark_chunks_as_newly_watched(&self, chunks: &[Vector2<i32>]) {
        chunks.iter().for_each(|chunk| {
            self.mark_chunk_as_newly_watched(*chunk);
        });
    }

    pub fn mark_chunk_as_newly_watched(&self, chunk: Vector2<i32>) {
        log::trace!("{:?} marked as newly watched", chunk);
        match self.chunk_watchers.entry(chunk) {
            Entry::Occupied(mut occupied) => {
                let value = occupied.get_mut();
                if let Some(new_value) = value.checked_add(1) {
                    *value = new_value;
                    //log::debug!("Watch value for {:?}: {}", chunk, value);
                } else {
                    log::error!("Watching overflow on chunk {:?}", chunk);
                }
            }
            Entry::Vacant(vacant) => {
                vacant.insert(1);
            }
        }
    }

    /// Marks chunks no longer "watched" by a unique player. When no players are watching a chunk,
    /// it is removed from memory. Should only be called on chunks the player was watching before
    pub fn mark_chunks_as_not_watched(&self, chunks: &[Vector2<i32>]) -> Vec<Vector2<i32>> {
        chunks
            .iter()
            .filter(|chunk| self.mark_chunk_as_not_watched(**chunk))
            .copied()
            .collect()
    }

    /// Returns whether the chunk should be removed from memory
    pub fn mark_chunk_as_not_watched(&self, chunk: Vector2<i32>) -> bool {
        log::trace!("{:?} marked as no longer watched", chunk);
        match self.chunk_watchers.entry(chunk) {
            Entry::Occupied(mut occupied) => {
                let value = occupied.get_mut();
                *value = value.saturating_sub(1);

                if *value == 0 {
                    occupied.remove_entry();
                    true
                } else {
                    false
                }
            }
            Entry::Vacant(_) => {
                // This can be:
                // - Player disconnecting before all packets have been sent
                // - Player moving so fast that the chunk leaves the render distance before it
                // is loaded into memory
                true
            }
        }
    }

    pub async fn clean_chunks(self: &Arc<Self>, chunks: &[Vector2<i32>]) {
        let mut chunks_tasks = tokio::task::JoinSet::new();

        for &at in chunks {
            let loaded_chunks = self.loaded_chunks.clone();
            let chunk_watchers = self.chunk_watchers.clone();

            chunks_tasks.spawn(async move {
                let removed_chunk = loaded_chunks.remove_if(&at, |at, _| {
                    if let Some(value) = &chunk_watchers.get(at) {
                        return value.is_zero();
                    }
                    true
                });

                if let Some((at, chunk)) = removed_chunk {
                    log::trace!("{:?} is being cleaned", at);
                    return Some((at, chunk));
                }

                if let Some(chunk_guard) = &loaded_chunks.get(&at) {
                    log::trace!("{:?} is not being cleaned but saved", at);
                    return Some((at, chunk_guard.value().clone()));
                }

                None
            });
        }

        let chunks_to_write = chunks_tasks
            .join_all()
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        self.write_chunks(chunks_to_write).await;
    }

    pub async fn clean_chunk(self: &Arc<Self>, chunk: &Vector2<i32>) {
        self.clean_chunks(&[*chunk]).await;
    }

    pub fn is_chunk_watched(&self, chunk: &Vector2<i32>) -> bool {
        self.chunk_watchers.get(chunk).is_some()
    }

    pub fn clean_memory(&self, chunks_to_check: &[Vector2<i32>]) {
        let deleted_chunks = chunks_to_check
            .par_iter()
            .filter_map(|chunk| {
                self.chunk_watchers
                    .remove_if(chunk, |_, watcher| watcher.is_zero());
                self.loaded_chunks
                    .remove_if(chunk, |at, _| self.chunk_watchers.get(at).is_none())
            })
            .count();

        if deleted_chunks > 0 {
            trace!("Cleaned {} chunks from memory", deleted_chunks);

            self.loaded_chunks.shrink_to_fit();
            self.chunk_watchers.shrink_to_fit();
        }
    }
    pub async fn write_chunks(&self, chunks_to_write: Vec<(Vector2<i32>, Arc<RwLock<ChunkData>>)>) {
        if chunks_to_write.is_empty() {
            return;
        }

        let chunk_saver = self.chunk_saver.clone();
        let level_folder = self.level_folder.clone();

        trace!("Writing chunks to disk {:}", chunks_to_write.len());

        if let Err(error) = chunk_saver
            .save_chunks(&level_folder, chunks_to_write)
            .await
        {
            log::error!("Failed writing Chunk to disk {}", error.to_string());
        }
    }

    async fn load_chunks_from_save(
        &self,
        chunks_pos: &[Vector2<i32>],
        channel: mpsc::Sender<(Vector2<i32>, Option<ChunkData>)>,
    ) {
        trace!("Loading chunks from disk {:}", chunks_pos.len());

        let (send, recv) =
            mpsc::channel::<LoadedData<ChunkData, ChunkReadingError>>(chunks_pos.len());
        let converter_task = async move {
            let mut recv = recv;
            while let Some(data) = recv.recv().await {
                let converted = match data {
                    LoadedData::Loaded(chunk) => (chunk.position, Some(chunk)),
                    LoadedData::Missing(pos) => (pos, None),
                    LoadedData::Error((position, error)) => {
                        log::error!(
                            "Failed to load chunk at {:?}: {} (regenerating)",
                            position,
                            error
                        );
                        (position, None)
                    }
                };

                channel
                    .send(converted)
                    .await
                    .expect("Failed to stream chunks from converter!");
            }
        };

        let stream_task = self
            .chunk_saver
            .stream_chunks(&self.level_folder, chunks_pos, send);

        let _ = tokio::join!(converter_task, stream_task);
    }

    /// Reads/Generates many chunks in a world
    /// Note: The order of the output chunks will almost never be in the same order as the order of input chunks
    pub async fn fetch_chunks(
        self: &Arc<Self>,
        chunks: &[Vector2<i32>],
        channel: mpsc::Sender<(Arc<RwLock<ChunkData>>, bool)>,
    ) {
        if chunks.is_empty() {
            return;
        }

        let gen_channel = channel.clone();
        let send_chunk = async move |is_new: bool, chunk: Arc<RwLock<ChunkData>>| {
            let _ = channel
                .send((chunk, is_new))
                .await
                .inspect_err(|err| log::error!("unable to send chunk to channel: {}", err));
        };

        // First send all chunks that we have cached
        let mut remaining_chunks = Vec::new();
        for chunk in chunks {
            if let Some(chunk) = self.loaded_chunks.get(chunk) {
                send_chunk(false, chunk.value().clone()).await;
            } else {
                remaining_chunks.push(*chunk);
            }
        }

        if remaining_chunks.is_empty() {
            return;
        }

        // Then attempt to get chunks from disk, generating them if they do not exist
        let (send, recv) = mpsc::channel(remaining_chunks.len());

        let disk_read_task = self.load_chunks_from_save(&remaining_chunks, send);

        let loaded_chunks = self.loaded_chunks.clone();
        let world_gen = self.world_gen.clone();
        let disk_handle_task = async move {
            let mut recv = recv;
            while let Some((pos, data)) = recv.recv().await {
                if let Some(data) = data {
                    let entry = loaded_chunks
                        .entry(pos)
                        .or_insert_with(|| Arc::new(RwLock::new(data)));
                    let value = entry.clone();
                    send_chunk(false, value).await;
                } else {
                    let loaded_chunks = loaded_chunks.clone();
                    let world_gen = world_gen.clone();
                    let gen_channel = gen_channel.clone();
                    rayon::spawn(move || {
                        let data = world_gen.generate_chunk(pos);
                        let entry = loaded_chunks
                            .entry(pos)
                            .or_insert_with(|| Arc::new(RwLock::new(data)));
                        let value = entry.clone();
                        gen_channel
                            .blocking_send((value, true))
                            .expect("Failed to send chunk from generation thread!");
                    });
                }
            }
        };

        let _ = tokio::join!(disk_read_task, disk_handle_task);
    }
}
