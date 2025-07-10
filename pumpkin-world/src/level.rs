use dashmap::{DashMap, Entry};
use log::trace;
use num_cpus;
use num_traits::Zero;
use pumpkin_config::{advanced_config, chunk::ChunkFormat};
use pumpkin_data::{Block, block_properties::has_random_ticks};
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::{
    select,
    sync::{
        Mutex, Notify, RwLock, Semaphore,
        mpsc::{self, UnboundedReceiver},
    },
    task::JoinHandle,
};
use tokio_util::task::TaskTracker;

use crate::{
    BlockStateId,
    block::{RawBlockState, entities::BlockEntity},
    chunk::{
        ChunkData, ChunkEntityData, ChunkParsingError, ChunkReadingError, ScheduledTick,
        TickPriority,
        format::{anvil::AnvilChunkFile, linear::LinearFile},
        io::{Dirtiable, FileIO, LoadedData, file_manager::ChunkFileManager},
    },
    dimension::Dimension,
    generation::{Seed, get_world_gen, implementation::WorldGenerator},
    world::BlockRegistryExt,
};

pub type SyncChunk = Arc<RwLock<ChunkData>>;
pub type SyncEntityChunk = Arc<RwLock<ChunkEntityData>>;

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
    block_registry: Arc<dyn BlockRegistryExt>,
    level_folder: LevelFolder,

    // Holds this level's spawn chunks, which are always loaded
    spawn_chunks: Arc<DashMap<Vector2<i32>, SyncChunk>>,

    // Chunks that are paired with chunk watchers. When a chunk is no longer watched, it is removed
    // from the loaded chunks map and sent to the underlying ChunkIO
    loaded_chunks: Arc<DashMap<Vector2<i32>, SyncChunk>>,
    loaded_entity_chunks: Arc<DashMap<Vector2<i32>, SyncEntityChunk>>,

    chunk_watchers: Arc<DashMap<Vector2<i32>, usize>>,

    chunk_saver: Arc<dyn FileIO<Data = SyncChunk>>,
    entity_saver: Arc<dyn FileIO<Data = SyncEntityChunk>>,

    world_gen: Arc<dyn WorldGenerator>,

    /// Semaphore to limit concurrent chunk generation tasks
    chunk_generation_semaphore: Arc<Semaphore>,
    /// Map to deduplicate chunk generation and avoid DashMap write lock
    chunk_generation_locks: Arc<Mutex<HashMap<Vector2<i32>, Arc<Notify>>>>,
    /// Tracks tasks associated with this world instance
    tasks: TaskTracker,
    /// Notification that interrupts tasks for shutdown
    pub shutdown_notifier: Notify,
}

pub struct TickData {
    pub block_ticks: Vec<ScheduledTick>,
    pub fluid_ticks: Vec<ScheduledTick>,
    pub random_ticks: Vec<ScheduledTick>,
    pub block_entities: Vec<Arc<dyn BlockEntity>>,
}

#[derive(Clone)]
pub struct LevelFolder {
    pub root_folder: PathBuf,
    pub region_folder: PathBuf,
    pub entities_folder: PathBuf,
}

impl Level {
    pub fn from_root_folder(
        root_folder: PathBuf,
        block_registry: Arc<dyn BlockRegistryExt>,
        seed: i64,
        dimension: Dimension,
    ) -> Self {
        // If we are using an already existing world we want to read the seed from the level.dat, If not we want to check if there is a seed in the config, if not lets create a random one
        let region_folder = root_folder.join("region");
        if !region_folder.exists() {
            std::fs::create_dir_all(&region_folder).expect("Failed to create Region folder");
        }
        let entities_folder = root_folder.join("entities");
        if !entities_folder.exists() {
            std::fs::create_dir_all(&region_folder).expect("Failed to create Entities folder");
        }
        let level_folder = LevelFolder {
            root_folder,
            region_folder,
            entities_folder,
        };

        // TODO: Load info correctly based on world format type

        let seed = Seed(seed as u64);
        let world_gen = get_world_gen(seed, dimension).into();

        let chunk_saver: Arc<dyn FileIO<Data = SyncChunk>> = match advanced_config().chunk.format {
            ChunkFormat::Linear => Arc::new(ChunkFileManager::<LinearFile<ChunkData>>::default()),
            ChunkFormat::Anvil => {
                Arc::new(ChunkFileManager::<AnvilChunkFile<ChunkData>>::default())
            }
        };
        let entity_saver: Arc<dyn FileIO<Data = SyncEntityChunk>> =
            match advanced_config().chunk.format {
                ChunkFormat::Linear => {
                    Arc::new(ChunkFileManager::<LinearFile<ChunkEntityData>>::default())
                }
                ChunkFormat::Anvil => {
                    Arc::new(ChunkFileManager::<AnvilChunkFile<ChunkEntityData>>::default())
                }
            };

        Self {
            seed,
            block_registry,
            world_gen,
            level_folder,
            chunk_saver,
            entity_saver,
            spawn_chunks: Arc::new(DashMap::new()),
            loaded_chunks: Arc::new(DashMap::new()),
            loaded_entity_chunks: Arc::new(DashMap::new()),
            chunk_watchers: Arc::new(DashMap::new()),
            tasks: TaskTracker::new(),
            shutdown_notifier: Notify::new(),
            // Limits concurrent chunk generation tasks to 2x the number of CPUs
            chunk_generation_semaphore: Arc::new(Semaphore::new(num_cpus::get())),
            chunk_generation_locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Spawns a task associated with this world. All tasks spawned with this method are awaited
    /// when the client. This means tasks should complete in a reasonable (no looping) amount of time.
    pub fn spawn_task<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tasks.spawn(task)
    }

    pub async fn shutdown(&self) {
        log::info!("Saving level...");

        self.shutdown_notifier.notify_waiters();
        self.tasks.close();
        log::debug!("Awaiting level tasks");
        self.tasks.wait().await;
        log::debug!("Done awaiting level chunk tasks");

        // wait for chunks currently saving in other threads
        self.chunk_saver.block_and_await_ongoing_tasks().await;

        // save all chunks currently in memory
        let chunks_to_write = self
            .loaded_chunks
            .iter()
            .map(|chunk| (*chunk.key(), chunk.value().clone()))
            .collect::<Vec<_>>();
        self.loaded_chunks.clear();

        // TODO: I think the chunk_saver should be at the server level
        self.chunk_saver.clear_watched_chunks().await;
        self.write_chunks(chunks_to_write).await;

        log::debug!("Done awaiting level entity tasks");

        // wait for chunks currently saving in other threads
        self.entity_saver.block_and_await_ongoing_tasks().await;

        // save all chunks currently in memory
        let chunks_to_write = self
            .loaded_entity_chunks
            .iter()
            .map(|chunk| (*chunk.key(), chunk.value().clone()))
            .collect::<Vec<_>>();
        self.loaded_entity_chunks.clear();

        // TODO: I think the chunk_saver should be at the server level
        self.entity_saver.clear_watched_chunks().await;
        self.write_entity_chunks(chunks_to_write).await;
    }

    pub fn loaded_chunk_count(&self) -> usize {
        self.loaded_chunks.len()
    }

    pub async fn clean_up_log(&self) {
        self.chunk_saver.clean_up_log().await;
        self.entity_saver.clean_up_log().await;
    }

    pub fn list_cached(&self) {
        for entry in self.loaded_chunks.iter() {
            log::debug!("In map: {:?}", entry.key());
        }
    }

    /// Marks chunks as "watched" by a unique player. When no players are watching a chunk,
    /// it is removed from memory. Should only be called on chunks the player was not watching
    /// before
    pub async fn mark_chunks_as_newly_watched(&self, chunks: &[Vector2<i32>]) {
        for chunk in chunks {
            log::trace!("{chunk:?} marked as newly watched");
            match self.chunk_watchers.entry(*chunk) {
                Entry::Occupied(mut occupied) => {
                    let value = occupied.get_mut();
                    if let Some(new_value) = value.checked_add(1) {
                        *value = new_value;
                        //log::debug!("Watch value for {:?}: {}", chunk, value);
                    } else {
                        log::error!("Watching overflow on chunk {chunk:?}");
                    }
                }
                Entry::Vacant(vacant) => {
                    vacant.insert(1);
                }
            }
        }

        self.chunk_saver
            .watch_chunks(&self.level_folder, chunks)
            .await;
        self.entity_saver
            .watch_chunks(&self.level_folder, chunks)
            .await;
    }

    #[inline]
    pub async fn mark_chunk_as_newly_watched(&self, chunk: Vector2<i32>) {
        self.mark_chunks_as_newly_watched(&[chunk]).await;
    }

    /// Marks chunks no longer "watched" by a unique player. When no players are watching a chunk,
    /// it is removed from memory. Should only be called on chunks the player was watching before
    pub async fn mark_chunks_as_not_watched(&self, chunks: &[Vector2<i32>]) -> Vec<Vector2<i32>> {
        let mut chunks_to_clean = Vec::new();

        for chunk in chunks {
            log::trace!("{chunk:?} marked as no longer watched");
            match self.chunk_watchers.entry(*chunk) {
                Entry::Occupied(mut occupied) => {
                    let value = occupied.get_mut();
                    *value = value.saturating_sub(1);

                    if *value == 0 {
                        occupied.remove_entry();
                        chunks_to_clean.push(*chunk);
                    }
                }
                Entry::Vacant(_) => {
                    // This can be:
                    // - Player disconnecting before all packets have been sent
                    // - Player moving so fast that the chunk leaves the render distance before it
                    // is loaded into memory
                }
            }
        }

        self.chunk_saver
            .unwatch_chunks(&self.level_folder, chunks)
            .await;
        self.entity_saver
            .unwatch_chunks(&self.level_folder, chunks)
            .await;
        chunks_to_clean
    }

    /// Returns whether the chunk should be removed from memory
    #[inline]
    pub async fn mark_chunk_as_not_watched(&self, chunk: Vector2<i32>) -> bool {
        !self.mark_chunks_as_not_watched(&[chunk]).await.is_empty()
    }

    pub async fn clean_chunks(self: &Arc<Self>, chunks: &[Vector2<i32>]) {
        // Care needs to be take here because of interweaving case:
        // 1) Remove chunk from cache
        // 2) Another player wants same chunk
        // 3) Load (old) chunk from serializer
        // 4) Write (new) chunk from serializer
        // Now outdated chunk data is cached and will be written later

        let chunks_with_no_watchers = chunks
            .iter()
            .filter_map(|pos| {
                // Only chunks that have no entry in the watcher map or have 0 watchers
                if self
                    .chunk_watchers
                    .get(pos)
                    .is_none_or(|count| count.is_zero())
                {
                    self.loaded_chunks.remove(pos).map(|chunk| (*pos, chunk.1))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let level = self.clone();
        self.spawn_task(async move {
            let chunks_to_remove = chunks_with_no_watchers.clone();

            level.write_chunks(chunks_with_no_watchers).await;
            // Only after we have written the chunks to the serializer do we remove them from the
            // cache
            for (pos, chunk) in chunks_to_remove {
                // Add them back if they have watchers
                if level.chunk_watchers.get(&pos).is_some() {
                    let entry = level.loaded_chunks.entry(pos);
                    if let Entry::Vacant(vacant) = entry {
                        vacant.insert(chunk);
                    }
                }
            }
        });
    }

    pub async fn clean_entity_chunks(self: &Arc<Self>, chunks: &[Vector2<i32>]) {
        // Care needs to be take here because of interweaving case:
        // 1) Remove chunk from cache
        // 2) Another player wants same chunk
        // 3) Load (old) chunk from serializer
        // 4) Write (new) chunk from serializer
        // Now outdated chunk data is cached and will be written later

        let chunks_with_no_watchers = chunks
            .iter()
            .filter_map(|pos| {
                // Only chunks that have no entry in the watcher map or have 0 watchers
                if self
                    .chunk_watchers
                    .get(pos)
                    .is_none_or(|count| count.is_zero())
                {
                    self.loaded_entity_chunks
                        .get(pos)
                        .map(|chunk| (*pos, chunk.value().clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let level = self.clone();
        self.spawn_task(async move {
            let chunks_to_remove = chunks_with_no_watchers.clone();
            level.write_entity_chunks(chunks_with_no_watchers).await;
            // Only after we have written the chunks to the serializer do we remove them from the
            // cache
            for (pos, _) in chunks_to_remove {
                let _ = level.loaded_entity_chunks.remove_if(&pos, |_, _| {
                    // Recheck that there is no one watching
                    level
                        .chunk_watchers
                        .get(&pos)
                        .is_none_or(|count| count.is_zero())
                });
            }
        });
    }

    // Gets random ticks, block ticks and fluid ticks
    pub async fn get_tick_data(&self) -> TickData {
        let mut ticks = TickData {
            block_ticks: Vec::new(),
            fluid_ticks: Vec::new(),
            random_ticks: Vec::with_capacity(self.loaded_chunks.len() * 3 * 16 * 16),
            block_entities: Vec::new(),
        };
        let mut rng = SmallRng::from_os_rng();
        for chunk in self.loaded_chunks.iter() {
            use tokio::time::{Duration, timeout};
            let mut chunk = match timeout(Duration::from_millis(1), chunk.write()).await {
                Ok(chunk_guard) => chunk_guard,
                Err(_) => {
                    log::info!("Chunk {:?} took too long to lock, skipping", chunk.key());
                    continue;
                }
            };
            ticks.block_ticks.extend(chunk.get_and_tick_block_ticks());
            ticks.fluid_ticks.extend(chunk.get_and_tick_fluid_ticks());
            let chunk = chunk.downgrade();

            let chunk_x_base = chunk.position.x * 16;
            let chunk_z_base = chunk.position.y * 16;

            let mut section_blocks = Vec::new();
            for i in 0..chunk.section.sections.len() {
                let mut section_block_data = Vec::new();

                //TODO use game rules to determine how many random ticks to perform
                for _ in 0..3 {
                    let r = rng.random::<u32>();
                    let x_offset = (r & 0xF) as i32;
                    let y_offset = ((r >> 4) & 0xF) as i32 - 32;
                    let z_offset = (r >> 8 & 0xF) as i32;

                    let random_pos = BlockPos::new(
                        chunk_x_base + x_offset,
                        i as i32 * 16 + y_offset,
                        chunk_z_base + z_offset,
                    );

                    let block_id = chunk
                        .section
                        .get_block_absolute_y(x_offset as usize, random_pos.0.y, z_offset as usize)
                        .unwrap_or(Block::AIR.default_state.id);

                    section_block_data.push((random_pos, block_id));
                }
                section_blocks.push(section_block_data);
            }

            for section_data in section_blocks {
                for (random_pos, block_id) in section_data {
                    if has_random_ticks(block_id) {
                        ticks.random_ticks.push(ScheduledTick {
                            block_pos: random_pos,
                            delay: 0,
                            priority: TickPriority::Normal,
                            target_block_id: 0,
                        });
                    }
                }
            }

            let cloned_entities = chunk.block_entities.values().cloned().collect::<Vec<_>>();
            ticks.block_entities.extend(cloned_entities);
        }

        ticks.block_ticks.sort_by_key(|tick| tick.priority);

        ticks
    }

    pub async fn clean_chunk(self: &Arc<Self>, chunk: &Vector2<i32>) {
        self.clean_chunks(&[*chunk]).await;
    }

    pub async fn clean_entity_chunk(self: &Arc<Self>, chunk: &Vector2<i32>) {
        self.clean_entity_chunks(&[*chunk]).await;
    }

    pub fn is_chunk_watched(&self, chunk: &Vector2<i32>) -> bool {
        self.chunk_watchers.get(chunk).is_some()
    }

    pub fn clean_memory(&self) {
        self.chunk_watchers.retain(|_, watcher| !watcher.is_zero());
        self.loaded_chunks
            .retain(|at, _| self.chunk_watchers.get(at).is_some());
        self.loaded_entity_chunks
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

        if self.loaded_entity_chunks.capacity() - self.loaded_entity_chunks.len() >= 4096 {
            self.loaded_entity_chunks.shrink_to_fit();
        }
    }

    // Stream the chunks (don't collect them and then do stuff with them)
    /// Spawns a tokio task to stream chunks.
    /// Important: must be called from an async function (or changed to accept a tokio runtime
    /// handle)
    pub fn receive_chunks(
        self: &Arc<Self>,
        chunks: Vec<Vector2<i32>>,
    ) -> UnboundedReceiver<(SyncChunk, bool)> {
        let (sender, receiver) = mpsc::unbounded_channel();
        // Put this in another thread so we aren't blocking on it
        let level = self.clone();
        self.spawn_task(async move {
            let cancel_notifier = level.shutdown_notifier.notified();
            let fetch_task = level.fetch_chunks(&chunks, sender);

            // Don't continue to handle chunks if we are shutting down
            select! {
                () = cancel_notifier => {},
                () = fetch_task => {}
            };
        });

        receiver
    }

    pub fn receive_entity_chunks(
        self: &Arc<Self>,
        chunks: Vec<Vector2<i32>>,
    ) -> UnboundedReceiver<(SyncEntityChunk, bool)> {
        let (sender, receiver) = mpsc::unbounded_channel();
        // Put this in another thread so we aren't blocking on it
        let level = self.clone();
        self.spawn_task(async move {
            let cancel_notifier = level.shutdown_notifier.notified();
            let fetch_task = level.fetch_entity_chunks(&chunks, sender);

            // Don't continue to handle chunks if we are shutting down
            select! {
                () = cancel_notifier => {},
                () = fetch_task => {}
            };
        });

        receiver
    }

    pub async fn get_chunk(
        self: &Arc<Self>,
        chunk_coordinate: Vector2<i32>,
    ) -> Arc<RwLock<ChunkData>> {
        match self.try_get_chunk(chunk_coordinate) {
            Some(chunk) => chunk.clone(),
            None => self.receive_chunk(chunk_coordinate).await.0,
        }
    }

    pub async fn get_entity_chunk(
        self: &Arc<Self>,
        chunk_coordinate: Vector2<i32>,
    ) -> Arc<RwLock<ChunkEntityData>> {
        match self.try_get_entity_chunk(chunk_coordinate) {
            Some(chunk) => chunk.clone(),
            None => self.receive_entity_chunk(chunk_coordinate).await.0,
        }
    }

    pub async fn receive_chunk(
        self: &Arc<Self>,
        chunk_pos: Vector2<i32>,
    ) -> (Arc<RwLock<ChunkData>>, bool) {
        let mut receiver = self.receive_chunks(vec![chunk_pos]);

        receiver
            .recv()
            .await
            .expect("Channel closed for unknown reason")
    }

    pub async fn receive_entity_chunk(
        self: &Arc<Self>,
        chunk_pos: Vector2<i32>,
    ) -> (Arc<RwLock<ChunkEntityData>>, bool) {
        let mut receiver = self.receive_entity_chunks(vec![chunk_pos]);

        receiver
            .recv()
            .await
            .expect("Channel closed for unknown reason")
    }

    pub async fn get_block_state(self: &Arc<Self>, position: &BlockPos) -> RawBlockState {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let chunk = self.get_chunk(chunk_coordinate).await;

        let chunk = chunk.read().await;
        let Some(id) = chunk.section.get_block_absolute_y(
            relative.x as usize,
            relative.y,
            relative.z as usize,
        ) else {
            return RawBlockState(Block::AIR.default_state.id);
        };

        RawBlockState(id)
    }

    pub async fn set_block_state(
        self: &Arc<Self>,
        position: &BlockPos,
        block_state_id: BlockStateId,
    ) -> BlockStateId {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let chunk = self.get_chunk(chunk_coordinate).await;
        let mut chunk = chunk.write().await;

        let replaced_block_state_id = chunk
            .section
            .get_block_absolute_y(relative.x as usize, relative.y, relative.z as usize)
            .unwrap();

        if replaced_block_state_id == block_state_id {
            return block_state_id;
        }

        chunk.mark_dirty(true);

        chunk.section.set_block_absolute_y(
            relative.x as usize,
            relative.y,
            relative.z as usize,
            block_state_id,
        );
        replaced_block_state_id
    }

    pub async fn write_chunks(&self, chunks_to_write: Vec<(Vector2<i32>, SyncChunk)>) {
        if chunks_to_write.is_empty() {
            return;
        }

        let chunk_saver = self.chunk_saver.clone();
        let level_folder = self.level_folder.clone();

        trace!("Sending chunks to ChunkIO {:}", chunks_to_write.len());
        if let Err(error) = chunk_saver
            .save_chunks(&level_folder, chunks_to_write)
            .await
        {
            log::error!("Failed writing Chunk to disk {error}");
        }
    }

    pub async fn write_entity_chunks(&self, chunks_to_write: Vec<(Vector2<i32>, SyncEntityChunk)>) {
        if chunks_to_write.is_empty() {
            return;
        }

        let chunk_saver = self.entity_saver.clone();
        let level_folder = self.level_folder.clone();

        trace!("Sending chunks to ChunkIO {:}", chunks_to_write.len());
        if let Err(error) = chunk_saver
            .save_chunks(&level_folder, chunks_to_write)
            .await
        {
            log::error!("Failed writing Chunk to disk {error}");
        }
    }

    /// Initializes the spawn chunks to these chunks
    pub async fn read_spawn_chunks(self: &Arc<Self>, chunks: &[Vector2<i32>]) {
        let (send, mut recv) = mpsc::unbounded_channel();

        let fetcher = self.fetch_chunks(chunks, send);
        let handler = async {
            while let Some((chunk, _)) = recv.recv().await {
                let pos = chunk.read().await.position;
                self.spawn_chunks.insert(pos, chunk);
            }
        };

        let _ = tokio::join!(fetcher, handler);
        log::debug!("Read {} chunks as spawn chunks", chunks.len());
    }

    /// Reads/Generates many chunks in a world
    /// Note: The order of the output chunks will almost never be in the same order as the order of input chunks
    pub async fn fetch_chunks(
        self: &Arc<Self>,
        chunks: &[Vector2<i32>],
        channel: mpsc::UnboundedSender<(SyncChunk, bool)>,
    ) {
        if chunks.is_empty() {
            return;
        }

        // If false, stop loading chunks because the channel has closed.
        let send_chunk =
            move |is_new: bool,
                  chunk: SyncChunk,
                  channel: &mpsc::UnboundedSender<(SyncChunk, bool)>| {
                channel.send((chunk, is_new)).is_ok()
            };

        // First send all chunks that we have cached
        // We expect best case scenario to have all cached
        let mut remaining_chunks = Vec::new();
        for chunk in chunks {
            let is_ok = if let Some(chunk) = self.loaded_chunks.get(chunk) {
                send_chunk(false, chunk.value().clone(), &channel)
            } else if let Some(spawn_chunk) = self.spawn_chunks.get(chunk) {
                // Also clone the arc into the loaded chunks
                self.loaded_chunks
                    .insert(*chunk, spawn_chunk.value().clone());
                send_chunk(false, spawn_chunk.value().clone(), &channel)
            } else {
                remaining_chunks.push(*chunk);
                true
            };

            if !is_ok {
                return;
            }
        }

        if remaining_chunks.is_empty() {
            return;
        }

        // These just pass data between async tasks, each of which do not block on anything, so
        // these do not need to hold a lot
        let (load_bridge_send, mut load_bridge_recv) =
            tokio::sync::mpsc::channel::<LoadedData<SyncChunk, ChunkReadingError>>(16);
        let (generate_bridge_send, mut generate_bridge_recv) = tokio::sync::mpsc::channel(16);

        let load_channel = channel.clone();
        let loaded_chunks = self.loaded_chunks.clone();
        let handle_load = async move {
            while let Some(data) = load_bridge_recv.recv().await {
                let is_ok = match data {
                    LoadedData::Loaded(chunk) => {
                        let position = chunk.read().await.position;

                        let value = loaded_chunks
                            .entry(position)
                            .or_insert(chunk)
                            .value()
                            .clone();
                        send_chunk(false, value, &load_channel)
                    }
                    LoadedData::Missing(pos) => generate_bridge_send.send(pos).await.is_ok(),
                    LoadedData::Error((pos, error)) => {
                        match error {
                            // this is expected, and is not an error
                            ChunkReadingError::ChunkNotExist
                            | ChunkReadingError::ParsingError(
                                ChunkParsingError::ChunkNotGenerated,
                            ) => {}
                            // this is an error, and we should log it
                            error => {
                                log::error!(
                                    "Failed to load chunk at {pos:?}: {error} (regenerating)"
                                );
                            }
                        };

                        generate_bridge_send.send(pos).await.is_ok()
                    }
                };

                if !is_ok {
                    // This isn't recoverable, so stop listening
                    return;
                }
            }
        };

        let loaded_chunks = self.loaded_chunks.clone();
        let world_gen = self.world_gen.clone();
        let block_registry = self.block_registry.clone();
        let self_clone = self.clone();
        let chunk_generation_semaphore = self.chunk_generation_semaphore.clone();
        let handle_generate = async move {
            let continue_to_generate = Arc::new(AtomicBool::new(true));
            while let Some(pos) = generate_bridge_recv.recv().await {
                if !continue_to_generate.load(Ordering::Relaxed) {
                    return;
                }

                let loaded_chunks = loaded_chunks.clone();
                let world_gen = world_gen.clone();
                let channel = channel.clone();
                let cloned_continue_to_generate = continue_to_generate.clone();
                let block_registry = block_registry.clone();
                let self_clone = self_clone.clone();
                let semaphore = chunk_generation_semaphore.clone();

                tokio::spawn(async move {
                    // Acquire a permit from the semaphore to limit concurrent generation
                    let _permit = semaphore.acquire().await.expect("Semaphore closed");

                    // Rayon tasks are queued, so also check it here
                    if !cloned_continue_to_generate.load(Ordering::Relaxed) {
                        return;
                    }

                    let result = {
                        // Deduplicate chunk generation using chunk_generation_locks
                        let notify = {
                            let mut locks = self_clone.chunk_generation_locks.lock().await;
                            if let Some(existing) = locks.get(&pos) {
                                Some(existing.clone())
                            } else {
                                let notify = Arc::new(Notify::new());
                                locks.insert(pos, notify.clone());
                                None
                            }
                        };
                        if let Some(notify) = notify {
                            // Wait for the chunk to be generated by another task
                            notify.notified().await;
                            // After being notified, the chunk should be in loaded_chunks
                            loaded_chunks.get(&pos).unwrap().clone()
                        } else {
                            // We are responsible for generating the chunk
                            let generated_chunk = world_gen
                                .generate_chunk(&self_clone, block_registry.as_ref(), &pos)
                                .await;
                            let arc_chunk = Arc::new(RwLock::new(generated_chunk));
                            loaded_chunks.insert(pos, arc_chunk.clone());
                            // Remove the notify and wake up any waiters
                            let notify = {
                                let mut locks = self_clone.chunk_generation_locks.lock().await;
                                locks.remove(&pos).unwrap()
                            };
                            notify.notify_waiters();
                            arc_chunk
                        }
                    };

                    if !send_chunk(true, result, &channel) {
                        // Stop any additional queued generations
                        cloned_continue_to_generate.store(false, Ordering::Relaxed);
                    }
                });
            }
        };

        let tracker = TaskTracker::new();
        tracker.spawn(handle_load);
        tracker.spawn(handle_generate);

        self.chunk_saver
            .fetch_chunks(&self.level_folder, &remaining_chunks, load_bridge_send)
            .await;

        tracker.close();
        tracker.wait().await;
    }

    pub async fn fetch_entity_chunks(
        self: &Arc<Self>,
        chunks: &[Vector2<i32>],
        channel: mpsc::UnboundedSender<(SyncEntityChunk, bool)>,
    ) {
        if chunks.is_empty() {
            return;
        }

        // If false, stop loading chunks because the channel has closed.
        let send_chunk =
            move |is_new: bool,
                  chunk: SyncEntityChunk,
                  channel: &mpsc::UnboundedSender<(SyncEntityChunk, bool)>| {
                channel.send((chunk, is_new)).is_ok()
            };

        // First send all chunks that we have cached
        // We expect best case scenario to have all cached
        let mut remaining_chunks = Vec::new();
        for chunk in chunks {
            let is_ok = if let Some(chunk) = self.loaded_entity_chunks.get(chunk) {
                send_chunk(false, chunk.value().clone(), &channel)
            } else {
                remaining_chunks.push(*chunk);
                true
            };

            if !is_ok {
                return;
            }
        }

        if remaining_chunks.is_empty() {
            return;
        }

        // These just pass data between async tasks, each of which do not block on anything, so
        // these do not need to hold a lot
        let (load_bridge_send, mut load_bridge_recv) =
            tokio::sync::mpsc::channel::<LoadedData<SyncEntityChunk, ChunkReadingError>>(16);
        let (generate_bridge_send, mut generate_bridge_recv) = tokio::sync::mpsc::channel(16);

        let load_channel = channel.clone();
        let loaded_chunks = self.loaded_entity_chunks.clone();
        let handle_load = async move {
            while let Some(data) = load_bridge_recv.recv().await {
                let is_ok = match data {
                    LoadedData::Loaded(chunk) => {
                        let position = chunk.read().await.chunk_position;

                        let value = loaded_chunks
                            .entry(position)
                            .or_insert(chunk)
                            .value()
                            .clone();
                        send_chunk(false, value, &load_channel)
                    }
                    LoadedData::Missing(pos) => generate_bridge_send.send(pos).await.is_ok(),
                    LoadedData::Error((pos, error)) => {
                        match error {
                            // this is expected, and is not an error
                            ChunkReadingError::ChunkNotExist
                            | ChunkReadingError::InvalidHeader
                            | ChunkReadingError::ParsingError(
                                ChunkParsingError::ChunkNotGenerated,
                            ) => {}
                            // this is an error, and we should log it
                            error => {
                                log::error!(
                                    "Failed to load a Entity chunk at {pos:?}: {error} (regenerating)"
                                );
                            }
                        };

                        generate_bridge_send.send(pos).await.is_ok()
                    }
                };

                if !is_ok {
                    // This isn't recoverable, so stop listening
                    return;
                }
            }
        };

        let loaded_chunks = self.loaded_entity_chunks.clone();
        let chunk_generation_semaphore = self.chunk_generation_semaphore.clone();
        let handle_generate = async move {
            let continue_to_generate = Arc::new(AtomicBool::new(true));
            while let Some(pos) = generate_bridge_recv.recv().await {
                if !continue_to_generate.load(Ordering::Relaxed) {
                    return;
                }

                let loaded_chunks = loaded_chunks.clone();
                let channel = channel.clone();
                let cloned_continue_to_generate = continue_to_generate.clone();
                let semaphore = chunk_generation_semaphore.clone();

                tokio::spawn(async move {
                    // Acquire a permit from the semaphore to limit concurrent generation
                    let _permit = semaphore.acquire().await.expect("Semaphore closed");

                    // Rayon tasks are queued, so also check it here
                    if !cloned_continue_to_generate.load(Ordering::Relaxed) {
                        return;
                    }

                    let result = {
                        let entry = loaded_chunks.entry(pos); // Get the entry for the position

                        // Check if the entry already exists.
                        // If not, generate the chunk asynchronously and insert it.
                        match entry {
                            Entry::Occupied(entry) => entry.into_ref(),
                            Entry::Vacant(entry) => {
                                let generated_chunk = ChunkEntityData {
                                    chunk_position: pos,
                                    data: HashMap::new(),
                                    dirty: true,
                                };
                                entry.insert(Arc::new(RwLock::new(generated_chunk)))
                            }
                        }
                        .value()
                        .clone()
                    };

                    if !send_chunk(true, result, &channel) {
                        // Stop any additional queued generations
                        cloned_continue_to_generate.store(false, Ordering::Relaxed);
                    }
                });
            }
        };

        let tracker = TaskTracker::new();
        tracker.spawn(handle_load);
        tracker.spawn(handle_generate);

        self.entity_saver
            .fetch_chunks(&self.level_folder, &remaining_chunks, load_bridge_send)
            .await;

        tracker.close();
        tracker.wait().await;
    }

    pub fn try_get_chunk(
        &self,
        coordinates: Vector2<i32>,
    ) -> Option<dashmap::mapref::one::Ref<'_, Vector2<i32>, Arc<RwLock<ChunkData>>>> {
        self.loaded_chunks.try_get(&coordinates).try_unwrap()
    }

    pub fn try_get_entity_chunk(
        &self,
        coordinates: Vector2<i32>,
    ) -> Option<dashmap::mapref::one::Ref<'_, Vector2<i32>, Arc<RwLock<ChunkEntityData>>>> {
        self.loaded_entity_chunks.try_get(&coordinates).try_unwrap()
    }
}
