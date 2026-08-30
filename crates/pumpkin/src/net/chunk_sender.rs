use bytes::Bytes;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::{Arc, Weak};

use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{
    CChunkBatchEnd, CChunkBatchStart, CChunkData, CLightUpdate, CUnloadChunk,
};
use pumpkin_protocol::ser::NetworkWriteExt;
use pumpkin_protocol::{ClientPacket, MultiVersionJavaPacket};
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::level::{Level, SyncChunk};

use crate::net::ClientPlatform;

const MIN_CHUNKS_PER_TICK: f32 = 0.1;
const MAX_CHUNKS_PER_TICK: f32 = 500.0;
const INITIAL_CHUNKS_PER_TICK: f32 = 9.0;
const MAX_CONCURRENT_BATCHES: u16 = 10;

pub struct PreparedChunk {
    pub position: Vector2<i32>,
    pub chunk: SyncChunk,
}

pub struct PreparedBatch {
    pub chunks: Vec<PreparedChunk>,
    pub epoch_snapshot: u32,
    pub target_version: JavaMinecraftVersion,
}

#[derive(Clone)]
pub struct EncodedChunk {
    pub position: Vector2<i32>,
    pub payload: Bytes,
    pub light_payload: Option<Bytes>,
    pub chunk_ref: Weak<ChunkData>,
}

impl EncodedChunk {
    #[must_use]
    pub fn is_fresh_for(&self, candidate: &PreparedChunk) -> bool {
        let Some(held) = self.chunk_ref.upgrade() else {
            return false;
        };

        self.position == candidate.position && Arc::ptr_eq(&held, &candidate.chunk)
    }
}

#[derive(Debug)]
pub struct ChunkSender {
    pub pending_chunks: FxHashSet<Vector2<i32>>,
    sent_chunks: FxHashSet<Vector2<i32>>,
    pub in_flight_batches: u16,
    pub desired_rate: f32,
    pub send_quota: f32,
    pub max_in_flight: u16,
}

impl ChunkSender {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending_chunks: FxHashSet::default(),
            sent_chunks: FxHashSet::default(),
            in_flight_batches: 0,
            desired_rate: INITIAL_CHUNKS_PER_TICK,
            send_quota: 0.0,
            max_in_flight: 1,
        }
    }

    pub fn reset(&mut self) {
        self.pending_chunks.clear();
        self.sent_chunks.clear();
        self.in_flight_batches = 0;
        self.send_quota = 0.0;
    }

    #[must_use]
    pub fn is_chunk_sent(&self, pos: &Vector2<i32>) -> bool {
        self.sent_chunks.contains(pos)
    }

    #[must_use]
    pub fn sent_chunks_count(&self) -> usize {
        self.sent_chunks.len()
    }

    pub const fn on_batch_acknowledged(&mut self, client_requested_rate: f32) -> bool {
        if self.in_flight_batches == 0 {
            return false;
        }

        self.in_flight_batches = self.in_flight_batches.saturating_sub(1);
        self.desired_rate = if client_requested_rate.is_nan() {
            MIN_CHUNKS_PER_TICK
        } else {
            client_requested_rate.clamp(MIN_CHUNKS_PER_TICK, MAX_CHUNKS_PER_TICK)
        };

        if self.in_flight_batches == 0 {
            self.send_quota = 1.0;
        }

        self.max_in_flight = MAX_CONCURRENT_BATCHES;
        true
    }

    pub fn enqueue_chunk(&mut self, pos: Vector2<i32>) {
        self.sent_chunks.remove(&pos);
        self.pending_chunks.insert(pos);
    }

    pub fn unload_chunk(&mut self, client: &ClientPlatform, pos: Vector2<i32>) {
        self.pending_chunks.remove(&pos);
        if self.sent_chunks.remove(&pos)
            && let ClientPlatform::Java(java_client) = client
            && !java_client.is_closed()
        {
            java_client.try_send_packet(&CUnloadChunk::new(pos.x, pos.y));
        }
    }

    fn collect_sorted_candidates(&self, level: &Level, center: Vector2<i32>) -> Vec<PreparedChunk> {
        let quota_limit = self.send_quota.floor() as usize;
        let mut sorted: Vec<Vector2<i32>> = self.pending_chunks.iter().copied().collect();

        sorted.sort_by_key(|pos| {
            let dx = (pos.x - center.x).unsigned_abs() as u64;
            let dz = (pos.y - center.y).unsigned_abs() as u64;
            dx * dx + dz * dz
        });

        let mut ready = Vec::with_capacity(quota_limit);
        for pos in sorted {
            if ready.len() >= quota_limit {
                break;
            }

            if let Some(chunk) = level.loaded_chunks.get(&pos) {
                ready.push(PreparedChunk {
                    position: pos,
                    chunk: chunk.value().clone(),
                });
            }
        }

        ready
    }

    pub fn prepare_batch(
        &mut self,
        level: &Level,
        player_chunk: Vector2<i32>,
        epoch: u32,
        version: JavaMinecraftVersion,
    ) -> Option<PreparedBatch> {
        if self.in_flight_batches >= self.max_in_flight {
            return None;
        }

        let max_batch = self.desired_rate.max(1.0);
        self.send_quota = (self.send_quota + self.desired_rate).min(max_batch);

        if self.send_quota < 1.0 || self.pending_chunks.is_empty() {
            return None;
        }

        let candidates = self.collect_sorted_candidates(level, player_chunk);
        if candidates.is_empty() {
            return None;
        }

        Some(PreparedBatch {
            chunks: candidates,
            epoch_snapshot: epoch,
            target_version: version,
        })
    }

    pub fn encode_batch(
        batch: &PreparedBatch,
        cache: &mut FxHashMap<Vector2<i32>, EncodedChunk>,
    ) -> Vec<EncodedChunk> {
        let version = batch.target_version;
        let cached_map = &*cache;

        let encoded_results: Vec<Option<EncodedChunk>> = batch
            .chunks
            .par_iter()
            .map(|candidate| {
                let pos = candidate.position;
                if let Some(cached) = cached_map.get(&pos)
                    && cached.is_fresh_for(candidate)
                {
                    return Some(cached.clone());
                }

                let chunk = &candidate.chunk;
                let mut chunk_buf = Vec::with_capacity(32 * 1024);
                if chunk_buf
                    .write_var_int(&VarInt(CChunkData::to_id(version)))
                    .is_err()
                {
                    return None;
                }
                if CChunkData(chunk)
                    .write_packet_data(&mut chunk_buf, &version)
                    .is_err()
                {
                    return None;
                }

                let light_payload = if version >= JavaMinecraftVersion::V_1_14
                    && version < JavaMinecraftVersion::V_1_18
                {
                    CLightUpdate::from_chunk(chunk, version)
                        .ok()
                        .and_then(|light_packet| {
                            let mut light_buf = Vec::new();
                            (light_buf
                                .write_var_int(&VarInt(CLightUpdate::to_id(version)))
                                .is_ok()
                                && light_packet
                                    .write_packet_data(&mut light_buf, &version)
                                    .is_ok())
                            .then(|| Bytes::from(light_buf))
                        })
                } else {
                    None
                };

                Some(EncodedChunk {
                    position: pos,
                    payload: Bytes::from(chunk_buf),
                    light_payload,
                    chunk_ref: Arc::downgrade(chunk),
                })
            })
            .collect();

        let mut output = Vec::with_capacity(encoded_results.len());
        for encoded in encoded_results.into_iter().flatten() {
            cache.insert(encoded.position, encoded.clone());
            output.push(encoded);
        }

        output
    }

    pub fn commit_batch(
        &mut self,
        batch: &PreparedBatch,
        encoded_chunks: &[EncodedChunk],
        client: &ClientPlatform,
        current_epoch: u32,
    ) -> Vec<Vector2<i32>> {
        if current_epoch != batch.epoch_snapshot || encoded_chunks.is_empty() {
            return Vec::new();
        }

        let mut dispatched_positions = Vec::with_capacity(encoded_chunks.len());
        let version = batch.target_version;

        if version >= JavaMinecraftVersion::V_1_20_2
            && let ClientPlatform::Java(java_client) = client
        {
            java_client.try_send_packet(&CChunkBatchStart);
        }

        for chunk in encoded_chunks {
            if !self.pending_chunks.contains(&chunk.position) {
                continue;
            }

            client.try_enqueue_packet(chunk.payload.clone());
            if let Some(ref light) = chunk.light_payload {
                client.try_enqueue_packet(light.clone());
            }

            self.pending_chunks.remove(&chunk.position);
            self.sent_chunks.insert(chunk.position);
            dispatched_positions.push(chunk.position);
        }

        let sent_count = dispatched_positions.len();
        if sent_count > 0 {
            if version >= JavaMinecraftVersion::V_1_20_2
                && let ClientPlatform::Java(java_client) = client
            {
                java_client.try_send_packet(&CChunkBatchEnd::new(sent_count as u16));
            }

            self.in_flight_batches = self.in_flight_batches.saturating_add(1);
            self.send_quota -= sent_count as f32;
        }

        dispatched_positions
    }
}

impl Default for ChunkSender {
    fn default() -> Self {
        Self::new()
    }
}
