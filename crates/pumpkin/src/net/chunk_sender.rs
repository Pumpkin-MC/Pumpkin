use bytes::Bytes;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use std::num::NonZero;
use std::sync::{Arc, Weak};

use crate::net::java::chunk_data::{CChunkData, ChunkLightExt};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{
    CChunkBatchEnd, CChunkBatchStart, CLightUpdate, CUnloadChunk,
};
use pumpkin_protocol::ser::NetworkWriteExt;
use pumpkin_protocol::{ClientPacket, MultiVersionJavaPacket};
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;
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

/// A committed Bedrock chunk plus the token identifying this dispatch of it.
pub struct DispatchedChunk {
    pub position: Vector2<i32>,
    pub chunk: SyncChunk,
    pub delivery_token: u64,
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
    /// Committed Bedrock chunks packets are still being encoded, by dispatch token.
    awaiting_delivery: FxHashMap<Vector2<i32>, u64>,
    /// Monotonic across resets, so a superseded dispatch can never match again.
    next_delivery_token: u64,
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
            awaiting_delivery: FxHashMap::default(),
            next_delivery_token: 0,
            in_flight_batches: 0,
            desired_rate: INITIAL_CHUNKS_PER_TICK,
            send_quota: 0.0,
            max_in_flight: 1,
        }
    }

    pub fn reset(&mut self) {
        self.pending_chunks.clear();
        self.sent_chunks.clear();
        self.awaiting_delivery.clear();
        self.in_flight_batches = 0;
        self.send_quota = 0.0;
    }

    #[must_use]
    pub fn is_chunk_sent(&self, pos: &Vector2<i32>) -> bool {
        self.sent_chunks.contains(pos)
    }

    /// Vanilla `ChunkMap.isChunkTracked` -> the client holds this chunk (view checked by caller).
    #[must_use]
    pub fn is_chunk_ready(&self, pos: &Vector2<i32>) -> bool {
        self.sent_chunks.contains(pos) && !self.awaiting_delivery.contains_key(pos)
    }

    /// Bedrock chunks become ready once `send_chunks` has queued them. Returns the
    /// positions that became ready: a position re-enqueued since its dispatch holds a
    /// newer token and stays not ready until that dispatch completes.
    pub fn mark_delivered(&mut self, deliveries: &[(Vector2<i32>, u64)]) -> Vec<Vector2<i32>> {
        let mut delivered = Vec::with_capacity(deliveries.len());
        for &(pos, token) in deliveries {
            if self.awaiting_delivery.get(&pos) == Some(&token) {
                self.awaiting_delivery.remove(&pos);
                delivered.push(pos);
            }
        }
        delivered
    }

    #[must_use]
    pub fn sent_chunks_count(&self) -> usize {
        self.sent_chunks.len()
    }

    /// Records a chunk sent outside the batch path as held.
    pub fn mark_sent_out_of_band(&mut self, pos: Vector2<i32>) {
        self.pending_chunks.remove(&pos);
        self.awaiting_delivery.remove(&pos);
        self.sent_chunks.insert(pos);
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

    /// Vanilla `ChunkMap.markChunkPendingToSend` -> a held copy stays tracked while re-queued.
    pub fn enqueue_chunk(&mut self, pos: Vector2<i32>) {
        self.awaiting_delivery.remove(&pos);
        self.pending_chunks.insert(pos);
    }

    pub fn unload_chunk(&mut self, client: &ClientPlatform, pos: Vector2<i32>) {
        self.pending_chunks.remove(&pos);
        self.awaiting_delivery.remove(&pos);
        if self.sent_chunks.remove(&pos)
            && let ClientPlatform::Java(java_client) = client
            && !java_client.is_closed()
        {
            java_client.try_send_packet(&CUnloadChunk::new(pos.x, pos.y));
        }
    }

    fn collect_sorted_candidates(
        &self,
        level: &Level,
        center: Vector2<i32>,
        view_distance: NonZero<u8>,
    ) -> Vec<PreparedChunk> {
        let quota_limit = self.send_quota.floor() as usize;
        let mut ready = Vec::with_capacity(quota_limit);

        // If pending_chunks is small, sorting it directly avoids scanning offsets.
        if self.pending_chunks.len() <= 16 {
            let mut sorted: Vec<Vector2<i32>> = self.pending_chunks.iter().copied().collect();
            sorted.sort_unstable_by_key(|pos| {
                let dx = (pos.x - center.x).unsigned_abs() as u64;
                let dz = (pos.y - center.y).unsigned_abs() as u64;
                dx * dx + dz * dz
            });

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
        } else {
            // Re-use precompiled cylindrical chunk view LUT which is already sorted center-outward.
            let offsets = Cylindrical::get_offsets(view_distance.get());
            for &(dx, dy) in offsets {
                if ready.len() >= quota_limit {
                    break;
                }

                let pos = Vector2::new(center.x + i32::from(dx), center.y + i32::from(dy));
                if self.pending_chunks.contains(&pos)
                    && let Some(chunk) = level.loaded_chunks.get(&pos)
                {
                    ready.push(PreparedChunk {
                        position: pos,
                        chunk: chunk.value().clone(),
                    });
                }
            }

            // Fallback for any pending chunks outside the precomputed table
            if ready.is_empty() {
                for &pos in &self.pending_chunks {
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
            }
        }

        ready
    }

    pub fn prepare_batch(
        &mut self,
        level: &Level,
        player_chunk: Vector2<i32>,
        view_distance: NonZero<u8>,
        epoch: u32,
        version: JavaMinecraftVersion,
    ) -> Option<PreparedBatch> {
        if version >= JavaMinecraftVersion::V_1_20_2 && self.in_flight_batches >= self.max_in_flight
        {
            return None;
        }

        let max_batch = self.desired_rate.max(1.0);
        self.send_quota = (self.send_quota + self.desired_rate).min(max_batch);

        if self.send_quota < 1.0 || self.pending_chunks.is_empty() {
            return None;
        }

        let candidates = self.collect_sorted_candidates(level, player_chunk, view_distance);
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
                self.in_flight_batches = self.in_flight_batches.saturating_add(1);
            }

            self.send_quota -= sent_count as f32;
        }

        dispatched_positions
    }

    /// Marks a prepared Bedrock batch as dispatched and returns its chunks for encoding.
    ///
    /// Bedrock chunks use a different encoder from Java chunks, but they must still move from
    /// `pending_chunks` to `sent_chunks`. Otherwise the same batch is selected every tick and the
    /// Bedrock login flow never reaches its minimum-chunk spawn threshold.
    /// They stay not ready until [`Self::mark_delivered`].
    pub fn commit_bedrock_batch(
        &mut self,
        batch: &PreparedBatch,
        current_epoch: u32,
    ) -> Vec<DispatchedChunk> {
        if current_epoch != batch.epoch_snapshot || batch.chunks.is_empty() {
            return Vec::new();
        }

        let mut dispatched_chunks = Vec::with_capacity(batch.chunks.len());
        for candidate in &batch.chunks {
            if !self.pending_chunks.remove(&candidate.position) {
                continue;
            }

            let delivery_token = self.next_delivery_token;
            self.next_delivery_token += 1;
            self.sent_chunks.insert(candidate.position);
            self.awaiting_delivery
                .insert(candidate.position, delivery_token);
            dispatched_chunks.push(DispatchedChunk {
                position: candidate.position,
                chunk: candidate.chunk.clone(),
                delivery_token,
            });
        }

        self.send_quota -= dispatched_chunks.len() as f32;
        dispatched_chunks
    }
}

impl Default for ChunkSender {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bedrock_batch_moves_pending_chunks_to_sent() {
        let position = Vector2::new(3, -2);
        let chunk = ChunkData::empty_sync(position.x, position.y);
        let batch = PreparedBatch {
            chunks: vec![PreparedChunk {
                position,
                chunk: chunk.clone(),
            }],
            epoch_snapshot: 7,
            target_version: JavaMinecraftVersion::V_1_20_2,
        };
        let mut sender = ChunkSender::new();
        sender.enqueue_chunk(position);
        sender.send_quota = 1.0;

        let dispatched = sender.commit_bedrock_batch(&batch, 7);

        assert_eq!(dispatched.len(), 1);
        assert!(Arc::ptr_eq(&dispatched[0].chunk, &chunk));
        assert!(!sender.pending_chunks.contains(&position));
        assert!(sender.is_chunk_sent(&position));
        assert_eq!(sender.sent_chunks_count(), 1);
        assert_eq!(sender.send_quota, 0.0);

        let repeated = sender.commit_bedrock_batch(&batch, 7);

        assert!(repeated.is_empty());
        assert_eq!(sender.sent_chunks_count(), 1);
        assert_eq!(sender.send_quota, 0.0);
    }

    #[test]
    fn bedrock_batch_ignores_a_stale_epoch() {
        let position = Vector2::new(3, -2);
        let batch = PreparedBatch {
            chunks: vec![PreparedChunk {
                position,
                chunk: ChunkData::empty_sync(position.x, position.y),
            }],
            epoch_snapshot: 7,
            target_version: JavaMinecraftVersion::V_1_20_2,
        };
        let mut sender = ChunkSender::new();
        sender.enqueue_chunk(position);

        let dispatched = sender.commit_bedrock_batch(&batch, 8);

        assert!(dispatched.is_empty());
        assert!(sender.pending_chunks.contains(&position));
        assert_eq!(sender.sent_chunks_count(), 0);
    }

    #[test]
    fn watchers_become_ready_on_their_own_delivery() {
        let position = Vector2::new(1, 4);
        let batch = PreparedBatch {
            chunks: vec![PreparedChunk {
                position,
                chunk: ChunkData::empty_sync(position.x, position.y),
            }],
            epoch_snapshot: 0,
            target_version: JavaMinecraftVersion::V_1_20_2,
        };
        let mut first = ChunkSender::new();
        let mut second = ChunkSender::new();
        first.enqueue_chunk(position);
        second.enqueue_chunk(position);
        assert!(!first.is_chunk_ready(&position));
        assert!(!second.is_chunk_ready(&position));

        let first_dispatch = first.commit_bedrock_batch(&batch, 0);
        assert!(!first.is_chunk_ready(&position));
        first.mark_delivered(&[(position, first_dispatch[0].delivery_token)]);
        assert!(first.is_chunk_ready(&position));
        assert!(!second.is_chunk_ready(&position));

        let second_dispatch = second.commit_bedrock_batch(&batch, 0);
        assert!(!second.is_chunk_ready(&position));
        second.mark_delivered(&[(position, second_dispatch[0].delivery_token)]);
        assert!(second.is_chunk_ready(&position));

        // Re-queueing a held chunk keeps it tracked.
        first.enqueue_chunk(position);
        assert!(first.is_chunk_ready(&position));
        assert!(second.is_chunk_ready(&position));
    }

    #[test]
    fn stale_bedrock_delivery_cannot_clear_a_newer_batchs_marker() {
        // A world change resets the sender and re-sends the same relative position
        // under a new epoch while the old dispatch is still awaiting send_chunks.
        let position = Vector2::new(5, -1);
        let mut sender = ChunkSender::new();

        sender.enqueue_chunk(position);
        let stale = sender.commit_bedrock_batch(&batch_for(position, 0), 0);

        sender.reset();
        sender.enqueue_chunk(position);
        sender.commit_bedrock_batch(&batch_for(position, 1), 1);
        assert!(!sender.is_chunk_ready(&position));

        let delivered = sender.mark_delivered(&[(position, stale[0].delivery_token)]);

        assert!(delivered.is_empty());
        assert!(!sender.is_chunk_ready(&position));
    }

    #[test]
    fn same_epoch_re_enqueue_keeps_its_own_delivery_marker() {
        // Re-enqueueing within one epoch dispatches the position again while the
        // first task is still awaiting send_chunks.
        let position = Vector2::new(-3, 8);
        let mut sender = ChunkSender::new();

        sender.enqueue_chunk(position);
        let first = sender.commit_bedrock_batch(&batch_for(position, 0), 0);

        sender.enqueue_chunk(position);
        let second = sender.commit_bedrock_batch(&batch_for(position, 0), 0);
        assert_ne!(first[0].delivery_token, second[0].delivery_token);
        assert!(!sender.is_chunk_ready(&position));

        // First task finishes last: its token is superseded, nothing clears.
        let delivered = sender.mark_delivered(&[(position, first[0].delivery_token)]);
        assert!(delivered.is_empty());
        assert!(!sender.is_chunk_ready(&position));

        let delivered = sender.mark_delivered(&[(position, second[0].delivery_token)]);
        assert_eq!(delivered, vec![position]);
        assert!(sender.is_chunk_ready(&position));
    }

    fn batch_for(position: Vector2<i32>, epoch_snapshot: u32) -> PreparedBatch {
        PreparedBatch {
            chunks: vec![PreparedChunk {
                position,
                chunk: ChunkData::empty_sync(position.x, position.y),
            }],
            epoch_snapshot,
            target_version: JavaMinecraftVersion::V_1_20_2,
        }
    }
}
