//! Shared in-memory state for a loaded SlimeWorld.
//!
//! A `.slime` file holds an entire world, but Pumpkin wires up *two* separate
//! [`FileIO`](crate::chunk::io::FileIO) backends per level (one for block data,
//! one for entities). Both share a single [`SlimeWorldStore`] so that a save
//! from either side can re-serialize the complete world back to the one file.

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use bytes::Bytes;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector2::Vector2;
use rustc_hash::FxHashMap;
use tracing::error;

use crate::chunk::format::anvil::WORLD_DATA_VERSION;
use crate::level::{SyncChunk, SyncEntityChunk};

use super::writer::serialize_world;
use super::{chunk_to_chunk_data, chunk_to_entity_data, read_slime_world};

/// Filename used when a world is persisted but had no source `.slime` file.
const DEFAULT_SLIME_NAME: &str = "world.slime";

/// The whole loaded SlimeWorld, shared between the chunk and entity backends.
pub(crate) struct SlimeWorldStore {
    /// Where the world is read from / written back to.
    path: PathBuf,
    /// Vanilla `DataVersion`, preserved across save.
    world_version: i32,
    /// The world "extra" compound (carries `properties`), preserved verbatim.
    extra: Bytes,
    chunks: RwLock<FxHashMap<Vector2<i32>, SyncChunk>>,
    entities: RwLock<FxHashMap<Vector2<i32>, SyncEntityChunk>>,
    /// Serializes whole-file writes so concurrent saves can't interleave.
    write_lock: tokio::sync::Mutex<()>,
}

impl SlimeWorldStore {
    /// Load and convert the first `*.slime` file under `root` (if any).
    ///
    /// `min_section_y` is the bottom-most section index of the target dimension.
    pub(crate) fn load(root: &Path, min_section_y: i32) -> Arc<Self> {
        let existing = find_slime_path(root);
        let path = existing
            .clone()
            .unwrap_or_else(|| root.join(DEFAULT_SLIME_NAME));

        let (world_version, extra, chunks, entities) = existing
            .and_then(|file| read_world(&file, min_section_y))
            .unwrap_or_else(|| {
                (
                    WORLD_DATA_VERSION,
                    Bytes::new(),
                    FxHashMap::default(),
                    FxHashMap::default(),
                )
            });

        Arc::new(Self {
            path,
            world_version,
            extra,
            chunks: RwLock::new(chunks),
            entities: RwLock::new(entities),
            write_lock: tokio::sync::Mutex::new(()),
        })
    }

    pub(crate) fn get_chunk(&self, coord: &Vector2<i32>) -> Option<SyncChunk> {
        self.chunks.read().expect("chunks lock").get(coord).cloned()
    }

    pub(crate) fn get_entity(&self, coord: &Vector2<i32>) -> Option<SyncEntityChunk> {
        self.entities
            .read()
            .expect("entities lock")
            .get(coord)
            .cloned()
    }

    pub(crate) fn store_chunks(&self, data: Vec<(Vector2<i32>, SyncChunk)>) {
        let mut guard = self.chunks.write().expect("chunks lock");
        for (coord, chunk) in data {
            guard.insert(coord, chunk);
        }
    }

    pub(crate) fn store_entities(&self, data: Vec<(Vector2<i32>, SyncEntityChunk)>) {
        let mut guard = self.entities.write().expect("entities lock");
        for (coord, chunk) in data {
            guard.insert(coord, chunk);
        }
    }

    /// Re-serialize the whole world and write it back to disk.
    ///
    /// # Errors
    ///
    /// Returns the underlying [`std::io::Error`] if writing the file fails.
    pub(crate) async fn persist(&self) -> std::io::Result<()> {
        let _write = self.write_lock.lock().await;

        // Snapshot the chunk Arcs (cheap clones); the lock is released before any await.
        let chunks = self.chunks.read().expect("chunks lock").clone();
        let entity_handles: Vec<(Vector2<i32>, SyncEntityChunk)> = self
            .entities
            .read()
            .expect("entities lock")
            .iter()
            .map(|(coord, chunk)| (*coord, chunk.clone()))
            .collect();

        // Entity NBT lives behind a tokio mutex; collect owned copies up front.
        let mut entity_compounds: FxHashMap<Vector2<i32>, Vec<NbtCompound>> = FxHashMap::default();
        for (coord, chunk) in &entity_handles {
            let guard = chunk.data.lock().await;
            entity_compounds.insert(*coord, guard.values().cloned().collect());
        }

        // Serializing (NBT + zstd) and writing the file are CPU/IO heavy; run them
        // off the async runtime so the tokio worker is never blocked.
        let world_version = self.world_version;
        let extra = self.extra.clone();
        let path = self.path.clone();
        tokio::task::spawn_blocking(move || {
            let bytes = serialize_world(world_version, &extra, &chunks, &entity_compounds);
            std::fs::write(path, bytes)
        })
        .await
        .map_err(std::io::Error::other)?
    }
}

/// Find the first `*.slime` file directly under `root`.
fn find_slime_path(root: &Path) -> Option<PathBuf> {
    std::fs::read_dir(root)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("slime"))
        })
}

type LoadedWorld = (
    i32,
    Bytes,
    FxHashMap<Vector2<i32>, SyncChunk>,
    FxHashMap<Vector2<i32>, SyncEntityChunk>,
);

/// Read a `.slime` file and convert every chunk into Pumpkin's representation.
fn read_world(path: &Path, min_section_y: i32) -> Option<LoadedWorld> {
    let bytes = std::fs::read(path).ok()?;
    let world = match read_slime_world(Bytes::from(bytes)) {
        Ok(world) => world,
        Err(error) => {
            error!("Failed to read slime world at {path:?}: {error}");
            return None;
        }
    };

    let mut chunks = FxHashMap::default();
    let mut entities = FxHashMap::default();
    for chunk in &world.chunks {
        let coord = Vector2::new(chunk.x, chunk.z);
        match chunk_to_chunk_data(chunk, min_section_y) {
            Ok(data) => {
                chunks.insert(coord, Arc::new(data));
            }
            Err(error) => error!(
                "Failed to convert slime chunk {},{}: {error}",
                chunk.x, chunk.z
            ),
        }
        match chunk_to_entity_data(chunk) {
            Ok(data) => {
                entities.insert(coord, Arc::new(data));
            }
            Err(error) => error!(
                "Failed to convert slime entities for chunk {},{}: {error}",
                chunk.x, chunk.z
            ),
        }
    }

    Some((world.world_version, world.extra, chunks, entities))
}
