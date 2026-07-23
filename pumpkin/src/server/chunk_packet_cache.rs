use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
};

use dashmap::{DashMap, mapref::entry::Entry};
use pumpkin_protocol::{
    ClientPacket, CompressionLevel, CompressionThreshold, MultiVersionJavaPacket,
    codec::var_int::VarInt,
    java::{
        client::play::CChunkData,
        packet_encoder::{PreparedPacket, prepare_packet},
    },
    ser::NetworkWriteExt,
};
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::chunk::ChunkData;
use tokio::sync::{Mutex, OnceCell, Semaphore};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct CacheKey {
    instance_id: u64,
    revision: u64,
    protocol_version: i32,
    compression_enabled: bool,
    compression_threshold: usize,
    compression_level: u32,
}

type CacheValue = Result<Arc<PreparedPacket>, String>;

pub struct ChunkPacketCache {
    capacity: usize,
    bytes: AtomicUsize,
    entries: DashMap<CacheKey, Arc<OnceCell<CacheValue>>>,
    insertion_order: Mutex<VecDeque<CacheKey>>,
    preparation_permits: Semaphore,
    hits: AtomicU64,
    misses: AtomicU64,
    unstable_snapshots: AtomicU64,
    preparation_workers: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct ChunkPacketCacheStats {
    pub entries: usize,
    pub bytes: usize,
    pub hits: u64,
    pub misses: u64,
    pub unstable_snapshots: u64,
    pub active_preparations: usize,
}

impl ChunkPacketCache {
    #[must_use]
    pub fn new(capacity_mib: usize) -> Self {
        let preparation_workers = std::thread::available_parallelism()
            .map_or(1, std::num::NonZero::get)
            .div_ceil(2)
            .clamp(1, 4);
        Self {
            capacity: capacity_mib.saturating_mul(1024 * 1024),
            bytes: AtomicUsize::new(0),
            entries: DashMap::new(),
            insertion_order: Mutex::new(VecDeque::new()),
            preparation_permits: Semaphore::new(preparation_workers),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            unstable_snapshots: AtomicU64::new(0),
            preparation_workers,
        }
    }

    #[must_use]
    pub fn stats(&self) -> ChunkPacketCacheStats {
        ChunkPacketCacheStats {
            entries: self.entries.len(),
            bytes: self.bytes.load(Ordering::Relaxed),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            unstable_snapshots: self.unstable_snapshots.load(Ordering::Relaxed),
            active_preparations: self
                .preparation_workers
                .saturating_sub(self.preparation_permits.available_permits()),
        }
    }

    pub async fn get_or_prepare(
        &self,
        chunk: Arc<ChunkData>,
        version: JavaMinecraftVersion,
        compression: Option<(CompressionThreshold, CompressionLevel)>,
    ) -> Result<Arc<PreparedPacket>, String> {
        if self.capacity == 0 {
            let _permit = self
                .preparation_permits
                .acquire()
                .await
                .map_err(|error| error.to_string())?;
            return prepare(chunk, version, compression).await;
        }

        let revision = chunk.network_revision();
        let key = CacheKey {
            instance_id: chunk.instance_id(),
            revision,
            protocol_version: version.protocol_version(),
            compression_enabled: compression.is_some(),
            compression_threshold: compression.map_or(0, |value| value.0),
            compression_level: compression.map_or(0, |value| value.1),
        };
        let (cell, inserted) = match self.entries.entry(key) {
            Entry::Occupied(entry) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                (entry.get().clone(), false)
            }
            Entry::Vacant(entry) => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                let cell = Arc::new(OnceCell::new());
                entry.insert(cell.clone());
                (cell, true)
            }
        };

        let value = cell
            .get_or_init(|| async {
                let _permit = self
                    .preparation_permits
                    .acquire()
                    .await
                    .map_err(|error| error.to_string())?;
                prepare(chunk.clone(), version, compression).await
            })
            .await
            .clone();

        match value {
            Ok(packet) => {
                if chunk.network_revision() == revision {
                    if inserted {
                        self.record_insert(key, packet.len()).await;
                    }
                } else {
                    self.unstable_snapshots.fetch_add(1, Ordering::Relaxed);
                    self.entries.remove(&key);
                    // Do not retry recursively. Hot chunks can change continuously, and retrying
                    // here turns normal mutation into unbounded compression work. These bytes are
                    // a valid point-in-time snapshot, equivalent to the uncached send path, but
                    // are not retained under a stale revision.
                }
                Ok(packet)
            }
            Err(error) => {
                self.entries.remove(&key);
                Err(error)
            }
        }
    }

    async fn record_insert(&self, key: CacheKey, packet_size: usize) {
        self.bytes.fetch_add(packet_size, Ordering::Relaxed);
        let mut order = self.insertion_order.lock().await;
        order.push_back(key);
        while self.bytes.load(Ordering::Relaxed) > self.capacity {
            let Some(oldest) = order.pop_front() else {
                break;
            };
            if let Some((_, cell)) = self.entries.remove(&oldest)
                && let Some(Ok(packet)) = cell.get()
            {
                self.bytes.fetch_sub(packet.len(), Ordering::Relaxed);
            }
        }
    }
}

async fn prepare(
    chunk: Arc<ChunkData>,
    version: JavaMinecraftVersion,
    compression: Option<(CompressionThreshold, CompressionLevel)>,
) -> CacheValue {
    tokio::task::spawn_blocking(move || {
        let mut data = Vec::new();
        data.write_var_int(&VarInt(CChunkData::to_id(version)))
            .map_err(|error| error.to_string())?;
        CChunkData(&chunk)
            .write_packet_data(&mut data, &version)
            .map_err(|error| error.to_string())?;
        prepare_packet(&data, compression)
            .map(Arc::new)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}
