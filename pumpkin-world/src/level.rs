use std::{fs, ops::Deref, path::PathBuf, sync::Arc};

use dashmap::{DashMap, Entry};
use log::trace;
use num_traits::Zero;
use pumpkin_config::{ADVANCED_CONFIG, chunk::ChunkFormat};
use pumpkin_util::math::vector2::Vector2;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use tokio::{
    runtime::Handle,
    sync::{RwLock, mpsc},
};

use crate::{
    chunk::{ChunkData, anvil::AnvilChunkFile, linear::LinearFile},
    chunks_io::{ChunkFileManager, ChunkIO, LoadedData},
    generation::{ Seed, WorldGenerator, get_world_gen},
    lock::{LevelLocker,anvil::AnvilLevelLocker },
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
        // chunks are automatically saved when all players get removed
        // TODO: Await chunks that have been called by this ^

        // save all stragling chunks
        let chunks_to_write = self
            .loaded_chunks
            .iter()
            .map(|chunk| chunk.value().clone())
            .collect::<Vec<_>>();
        self.write_chunks(chunks_to_write).await;

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
                    return Some(chunk);
                } else if let Some(chunk_guard) = &loaded_chunks.get(&at) {
                    log::trace!("{:?} is not being cleaned but saved", at);
                    return Some(chunk_guard.value().clone());
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
    pub async fn write_chunks(&self, chunks_to_write: Vec<Arc<RwLock<ChunkData>>>) {
        if chunks_to_write.is_empty() {
            return;
        }

        let chunk_saver = self.chunk_saver.clone();
        let level_folder = self.level_folder.clone();

        // TODO: Save the join handles to await them when stopping the server
        tokio::spawn(async move {
            let futures = chunks_to_write
                .iter()
                .map(|chunk| chunk.read())
                .collect::<Vec<_>>();

            let mut chunks_guards = Vec::new();
            for guard in futures {
                let chunk = guard.await;
                chunks_guards.push(chunk);
            }

            let chunks = chunks_guards
                .iter()
                .map(|chunk| (chunk.position, chunk.deref()))
                .collect::<Vec<_>>();

            trace!("Writing chunks to disk {:}", chunks_guards.len());

            if let Err(error) = chunk_saver.save_chunks(&level_folder, chunks.as_slice()) {
                log::error!("Failed writing Chunk to disk {}", error.to_string());
            }
        });
    }

    fn load_chunks_from_save(
        &self,
        chunks_pos: &[Vector2<i32>],
    ) -> Vec<(Vector2<i32>, Option<ChunkData>)> {
        if chunks_pos.is_empty() {
            return vec![];
        }
        trace!("Loading chunks from disk {:}", chunks_pos.len());
        self.chunk_saver
            .load_chunks(&self.level_folder, chunks_pos)
            .into_par_iter()
            .filter_map(|chunk_data| match chunk_data {
                LoadedData::Loaded(chunk) => Some((chunk.position, Some(chunk))),
                LoadedData::Missing(pos) => Some((pos, None)),
                LoadedData::Error((position, error)) => {
                    log::error!("Failed to load chunk at {:?}: {}", position, error);
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    /// Reads/Generates many chunks in a world
    /// Note: The order of the output chunks will almost never be in the same order as the order of input chunks
    pub fn fetch_chunks(
        &self,
        chunks: &[Vector2<i32>],
        channel: mpsc::Sender<(Arc<RwLock<ChunkData>>, bool)>,
        rt: &Handle,
    ) {
        fn send_chunks(
            is_new: bool,
            chunk: Arc<RwLock<ChunkData>>,
            channel: &mpsc::Sender<(Arc<RwLock<ChunkData>>, bool)>,
            rt: &Handle,
        ) {
            let channel = channel.clone();
            rt.spawn(async move {
                let _ = channel
                    .send((chunk, is_new))
                    .await
                    .inspect_err(|err| log::error!("unable to send chunk to channel: {}", err));
            });
        }

        if chunks.is_empty() {
            return;
        }

        let chunks_to_load = chunks
            .par_iter()
            .filter(|pos| {
                if let Some(entry_bind) = &self.loaded_chunks.get(pos) {
                    send_chunks(false, entry_bind.value().clone(), &channel, rt);
                    false
                } else {
                    true
                }
            })
            .copied()
            .collect::<Vec<_>>();

        if chunks_to_load.is_empty() {
            return;
        }

        self.load_chunks_from_save(&chunks_to_load)
            .into_par_iter()
            .for_each(|(pos, chunk)| {
                let (is_new, entry_bind) = if let Some(entry_bind) = self.loaded_chunks.get(&pos) {
                    (false, entry_bind)
                } else {
                    let entry_bind = self
                        .loaded_chunks
                        .entry(pos)
                        .or_insert_with(|| {
                            Arc::new(RwLock::new(
                                chunk.unwrap_or_else(|| self.world_gen.generate_chunk(pos)),
                            ))
                        })
                        .downgrade();

                    (true, entry_bind)
                };
                send_chunks(is_new, entry_bind.value().clone(), &channel, rt);
                drop(entry_bind);
            });
    }
}
