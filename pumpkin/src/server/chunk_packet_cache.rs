use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
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
use tokio::sync::{Mutex, OnceCell};

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
}

impl ChunkPacketCache {
    #[must_use]
    pub fn new(capacity_mib: usize) -> Self {
        Self {
            capacity: capacity_mib.saturating_mul(1024 * 1024),
            bytes: AtomicUsize::new(0),
            entries: DashMap::new(),
            insertion_order: Mutex::new(VecDeque::new()),
        }
    }

    pub async fn get_or_prepare(
        &self,
        chunk: Arc<ChunkData>,
        version: JavaMinecraftVersion,
        compression: Option<(CompressionThreshold, CompressionLevel)>,
    ) -> Result<Arc<PreparedPacket>, String> {
        if self.capacity == 0 {
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
            Entry::Occupied(entry) => (entry.get().clone(), false),
            Entry::Vacant(entry) => {
                let cell = Arc::new(OnceCell::new());
                entry.insert(cell.clone());
                (cell, true)
            }
        };

        let value = cell
            .get_or_init(|| prepare(chunk.clone(), version, compression))
            .await
            .clone();

        match value {
            Ok(packet) if chunk.network_revision() == revision => {
                if inserted {
                    self.record_insert(key, packet.len()).await;
                }
                Ok(packet)
            }
            Ok(_) => {
                self.entries.remove(&key);
                Box::pin(self.get_or_prepare(chunk, version, compression)).await
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
