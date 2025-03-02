use std::{fs, path::PathBuf, sync::Arc};

use dashmap::{DashMap, Entry};
use futures::future::join_all;
use log::trace;
use num_traits::Zero;
use pumpkin_config::{ADVANCED_CONFIG, chunk::ChunkFormat};
use pumpkin_util::math::vector2::Vector2;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tokio::sync::{RwLock, mpsc};

use crate::{
    chunk::{
        ChunkData, ChunkParsingError, ChunkReadingError,
        format::{anvil::AnvilChunkFile, linear::LinearFile},
        io::{ChunkIO, LoadedData, chunk_file_manager::ChunkFileManager},
    },
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

        // save all chunks currently in memory
        let chunks_to_write = self
            .loaded_chunks
            .iter()
            .map(|chunk| (*chunk.key(), chunk.value().clone()))
            .collect::<Vec<_>>();
        self.loaded_chunks.clear();

        self.write_chunks(chunks_to_write).await;

        // wait for chunks currently saving in other threads
        self.chunk_saver.close().await;

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
        let chunk_tasks = chunks.iter().map(async |&at| {
            let removed_chunk = self.loaded_chunks.remove_if(&at, |at, _| {
                if let Some(value) = &self.chunk_watchers.get(at) {
                    return value.is_zero();
                }
                true
            });

            if let Some((at, chunk)) = removed_chunk {
                log::trace!("{:?} is being cleaned", at);
                return Some((at, chunk));
            }

            if let Some(chunk_guard) = &self.loaded_chunks.get(&at) {
                log::trace!("{:?} is not being cleaned but saved", at);
                return Some((at, chunk_guard.value().clone()));
            }

            None
        });

        let chunks_to_write = join_all(chunk_tasks)
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

    pub fn clean_memory(&self) {
        self.chunk_watchers.retain(|_, watcher| !watcher.is_zero());
        self.loaded_chunks
            .retain(|at, _| self.chunk_watchers.get(at).is_some());

        // if the difference is too big, we can shrink the loaded chunks
        // (1024 chunks is the equivalent to a 32x32 chunks area)
        if self.chunk_watchers.capacity() - self.chunk_watchers.len() >= 4096 {
            self.chunk_watchers.shrink_to_fit();
        }

        // if the difference is too big, we can shrink the loaded chunks
        // (1024 chunks is the equivalent to a 32x32 chunks area)
        if self.loaded_chunks.capacity() - self.loaded_chunks.len() >= 4096 {
            self.loaded_chunks.shrink_to_fit();
        }
    }
  
    pub async fn write_chunks(&self, chunks_to_write: Vec<(Vector2<i32>, Arc<RwLock<ChunkData>>)>) {
        if chunks_to_write.is_empty() {
            return;
        }

        let chunk_saver = self.chunk_saver.clone();
        let level_folder = self.level_folder.clone();

        trace!("Writing chunks to disk {:}", chunks_to_write.len());
        tokio::spawn(async move {
            if let Err(error) = chunk_saver
                .save_chunks(&level_folder, chunks_to_write)
                .await
            {
                log::error!("Failed writing Chunk to disk {}", error.to_string());
            }
        });
    }

    async fn load_chunks_from_save(
        &self,
        chunks_pos: &[Vector2<i32>],
    ) -> (Vec<Vector2<i32>>, Vec<ChunkData>) {
        trace!("Loading chunks from disk {:}", chunks_pos.len());

        //we expect best case scenario to have all pre-generated
        let mut loaded_chunks = Vec::with_capacity(chunks_pos.len());
        let mut non_generated_chunks = Vec::new();

        let fetched_chunks = self
            .chunk_saver
            .fetch_chunks(&self.level_folder, chunks_pos)
            .await;

        for data in fetched_chunks {
            match data {
                LoadedData::Loaded(chunk) => loaded_chunks.push(chunk),
                LoadedData::Missing(pos) => non_generated_chunks.push(pos),
                LoadedData::Error((pos, error)) => match error {
                    // this is expected, and is not an error
                    ChunkReadingError::ChunkNotExist
                    | ChunkReadingError::ParsingError(ChunkParsingError::ChunkNotGenerated) => {
                        non_generated_chunks.push(pos);
                    }
                    // this is an error, and we should log it
                    error => {
                        log::error!(
                            "Failed to load chunk at {:?}: {} (regenerating)",
                            pos,
                            error
                        );
                        non_generated_chunks.push(pos);
                    }
                },
            }
        }

        (non_generated_chunks, loaded_chunks)
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

        let send_chunk = async move |is_new: bool, chunk: Arc<RwLock<ChunkData>>| {
            let _ = channel
                .send((chunk, is_new))
                .await
                .inspect_err(|err| log::error!("unable to send chunk to channel: {}", err));
        };

        // First send all chunks that we have cached
        // We expect best case scenario to have all cached
        let mut tasks = Vec::with_capacity(chunks.len());
        let mut remaining_chunks = Vec::new();
        for chunk in chunks {
            if let Some(chunk) = self.loaded_chunks.get(chunk) {
                tasks.push(send_chunk(false, chunk.value().clone()));
            } else {
                remaining_chunks.push(*chunk);
            }
        }

        futures::future::join_all(tasks).await;

        if remaining_chunks.is_empty() {
            return;
        }

        // Then attempt to get chunks from disk
        let (to_generate, to_send) = self.load_chunks_from_save(&remaining_chunks).await;

        // Send all chunks that were loaded from disk
        if !to_send.is_empty() {
            let tasks = to_send.into_iter().map(async |data| {
                let value = self
                    .loaded_chunks
                    .entry(data.position)
                    .or_insert_with(|| Arc::new(RwLock::new(data)))
                    .value()
                    .clone();
                send_chunk(false, value).await;
            });

            futures::future::join_all(tasks).await;
        }

        // Finally generate any chunks that are missing
        if !to_generate.is_empty() {
            let loaded_chunks = self.loaded_chunks.clone();
            let world_gen = self.world_gen.clone();

            let (gen_channel, mut gen_receiver) = mpsc::channel(to_generate.len());
            rayon::spawn(move || {
                to_generate.into_par_iter().for_each(|position| {
                    let generated_chunk = world_gen.generate_chunk(position);
                    let chunk = loaded_chunks
                        .entry(position)
                        .or_insert_with(|| Arc::new(RwLock::new(generated_chunk)))
                        .value()
                        .clone();

                    //this relay on a channel with the same size as the chunks to generate
                    gen_channel
                        .try_send(chunk)
                        .expect("Failed to send chunk from generation thread!");
                })
            });

            // As we are generating the chunks at the same time
            // we can send them sequentially and avoid
            // the overhead of joining the tasks and let the loop
            // as a CPU bound task
            while let Some(chunk) = gen_receiver.recv().await {
                send_chunk(true, chunk).await;
            }
        }
    }
}
