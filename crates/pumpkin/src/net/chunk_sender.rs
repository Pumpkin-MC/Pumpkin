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
/// Bedrock decodes a complete vertical column for every level-chunk packet. Keep initial
/// delivery deliberately smaller than Java's adaptive batch and never overlap these batches.
const BEDROCK_CHUNKS_PER_BATCH: usize = 4;

pub struct PreparedChunk {
    pub position: Vector2<i32>,
    pub chunk: SyncChunk,
}

pub struct PreparedBatch {
    pub chunks: Vec<PreparedChunk>,
    pub epoch_snapshot: u32,
    pub target_version: JavaMinecraftVersion,
}

pub struct BedrockChunkBatch {
    pub chunks: Vec<SyncChunk>,
    pub positions: Vec<Vector2<i32>>,
    pub epoch_snapshot: u32,
    token: u64,
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
    bedrock_ready_chunks: FxHashSet<Vector2<i32>>,
    bedrock_completed_chunks: FxHashSet<Vector2<i32>>,
    bedrock_in_flight_token: Option<u64>,
    next_bedrock_batch_token: u64,
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
            bedrock_ready_chunks: FxHashSet::default(),
            bedrock_completed_chunks: FxHashSet::default(),
            bedrock_in_flight_token: None,
            next_bedrock_batch_token: 0,
            in_flight_batches: 0,
            desired_rate: INITIAL_CHUNKS_PER_TICK,
            send_quota: 0.0,
            max_in_flight: 1,
        }
    }

    pub fn reset(&mut self) {
        self.pending_chunks.clear();
        self.sent_chunks.clear();
        self.bedrock_ready_chunks.clear();
        self.bedrock_completed_chunks.clear();
        self.bedrock_in_flight_token = None;
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

    /// Returns whether the Bedrock level-chunk packet has been queued for this position.
    #[must_use]
    pub fn is_bedrock_chunk_ready(&self, pos: &Vector2<i32>) -> bool {
        self.bedrock_ready_chunks.contains(pos)
    }

    #[must_use]
    pub fn bedrock_ready_chunks_count(&self) -> usize {
        self.bedrock_ready_chunks.len()
    }

    /// Returns chunks whose Bedrock send hook reached a terminal result. This includes
    /// successfully queued chunks and plugin-cancelled chunks, so cancellation cannot
    /// leave login waiting forever. Only `bedrock_ready_chunks` may unlock actors.
    #[must_use]
    pub fn bedrock_completed_chunks_count(&self) -> usize {
        self.bedrock_completed_chunks.len()
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
        self.bedrock_ready_chunks.remove(&pos);
        self.bedrock_completed_chunks.remove(&pos);
        self.pending_chunks.insert(pos);
    }

    pub fn unload_chunk(&mut self, client: &ClientPlatform, pos: Vector2<i32>) {
        self.pending_chunks.remove(&pos);
        self.bedrock_ready_chunks.remove(&pos);
        self.bedrock_completed_chunks.remove(&pos);
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
        if version >= JavaMinecraftVersion::V_1_20_2 && self.in_flight_batches >= self.max_in_flight
        {
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
    pub fn commit_bedrock_batch(
        &mut self,
        batch: &PreparedBatch,
        current_epoch: u32,
    ) -> Option<BedrockChunkBatch> {
        if current_epoch != batch.epoch_snapshot
            || batch.chunks.is_empty()
            || self.bedrock_in_flight_token.is_some()
        {
            return None;
        }

        let capacity = batch.chunks.len().min(BEDROCK_CHUNKS_PER_BATCH);
        let mut dispatched_chunks = Vec::with_capacity(capacity);
        let mut dispatched_positions = Vec::with_capacity(capacity);
        for candidate in &batch.chunks {
            if dispatched_chunks.len() >= BEDROCK_CHUNKS_PER_BATCH {
                break;
            }
            if !self.pending_chunks.remove(&candidate.position) {
                continue;
            }

            self.sent_chunks.insert(candidate.position);
            dispatched_chunks.push(candidate.chunk.clone());
            dispatched_positions.push(candidate.position);
        }

        if dispatched_chunks.is_empty() {
            return None;
        }

        self.send_quota -= dispatched_chunks.len() as f32;
        self.in_flight_batches = self.in_flight_batches.saturating_add(1);
        let token = self.next_bedrock_batch_token;
        self.next_bedrock_batch_token = self.next_bedrock_batch_token.wrapping_add(1);
        self.bedrock_in_flight_token = Some(token);
        Some(BedrockChunkBatch {
            chunks: dispatched_chunks,
            positions: dispatched_positions,
            epoch_snapshot: batch.epoch_snapshot,
            token,
        })
    }

    /// Releases a Bedrock batch and returns the successfully queued positions which became ready
    /// for dependent packets. Failed positions are put back in the pending queue for retry, while
    /// plugin-cancelled positions are treated as terminal for this watch cycle.
    ///
    /// A stale completion must not release a newer batch started after a teleport or world
    /// change. Positions unloaded while encoding are likewise not marked as ready.
    pub fn on_bedrock_batch_completed(
        &mut self,
        batch: &BedrockChunkBatch,
        queued_positions: &[Vector2<i32>],
        cancelled_positions: &[Vector2<i32>],
        current_epoch: u32,
        expected_world_is_current: bool,
    ) -> Vec<Vector2<i32>> {
        if self.bedrock_in_flight_token != Some(batch.token) {
            return Vec::new();
        }

        self.bedrock_in_flight_token = None;
        self.in_flight_batches = self.in_flight_batches.saturating_sub(1);
        if current_epoch != batch.epoch_snapshot || !expected_world_is_current {
            for position in &batch.positions {
                if self.sent_chunks.remove(position) {
                    self.bedrock_ready_chunks.remove(position);
                    self.bedrock_completed_chunks.remove(position);
                    self.pending_chunks.insert(*position);
                }
            }
            return Vec::new();
        }

        let mut ready_positions = Vec::with_capacity(batch.positions.len());
        for position in &batch.positions {
            if !self.sent_chunks.contains(position) {
                continue;
            }

            if queued_positions.contains(position) {
                self.bedrock_completed_chunks.insert(*position);
                if self.bedrock_ready_chunks.insert(*position) {
                    ready_positions.push(*position);
                }
            } else if cancelled_positions.contains(position) {
                // Cancellation means the plugin deliberately suppressed this chunk. Keep it out
                // of the pending queue so the closest cancelled chunk cannot be retried every
                // tick and starve chunks behind it.
                self.bedrock_ready_chunks.remove(position);
                self.bedrock_completed_chunks.insert(*position);
            } else {
                self.sent_chunks.remove(position);
                self.bedrock_ready_chunks.remove(position);
                self.bedrock_completed_chunks.remove(position);
                self.pending_chunks.insert(*position);
            }
        }
        ready_positions
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

        let dispatched = sender
            .commit_bedrock_batch(&batch, 7)
            .expect("current Bedrock batch should be dispatched");

        assert_eq!(dispatched.chunks.len(), 1);
        assert!(Arc::ptr_eq(&dispatched.chunks[0], &chunk));
        assert_eq!(dispatched.positions, vec![position]);
        assert!(!sender.pending_chunks.contains(&position));
        assert!(sender.is_chunk_sent(&position));
        assert!(!sender.is_bedrock_chunk_ready(&position));
        assert_eq!(sender.sent_chunks_count(), 1);
        assert_eq!(sender.send_quota, 0.0);
        assert_eq!(sender.in_flight_batches, 1);

        let repeated = sender.commit_bedrock_batch(&batch, 7);

        assert!(repeated.is_none());
        assert_eq!(sender.sent_chunks_count(), 1);
        assert_eq!(sender.send_quota, 0.0);

        assert_eq!(
            sender.on_bedrock_batch_completed(&dispatched, &dispatched.positions, &[], 7, true,),
            vec![position]
        );
        assert!(sender.is_bedrock_chunk_ready(&position));
        assert_eq!(sender.bedrock_completed_chunks_count(), 1);
        assert!(sender.bedrock_completed_chunks.contains(&position));
        assert_eq!(sender.in_flight_batches, 0);
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

        assert!(dispatched.is_none());
        assert!(sender.pending_chunks.contains(&position));
        assert_eq!(sender.sent_chunks_count(), 0);
        assert_eq!(sender.in_flight_batches, 0);
    }

    #[test]
    fn bedrock_batches_are_bounded_and_do_not_overlap() {
        let positions: Vec<_> = (0..6).map(|x| Vector2::new(x, 0)).collect();
        let batch = PreparedBatch {
            chunks: positions
                .iter()
                .map(|position| PreparedChunk {
                    position: *position,
                    chunk: ChunkData::empty_sync(position.x, position.y),
                })
                .collect(),
            epoch_snapshot: 3,
            target_version: JavaMinecraftVersion::V_1_20_2,
        };
        let mut sender = ChunkSender::new();
        for position in &positions {
            sender.enqueue_chunk(*position);
        }
        sender.send_quota = positions.len() as f32;

        let first = sender
            .commit_bedrock_batch(&batch, 3)
            .expect("first batch should be dispatched");

        assert_eq!(first.chunks.len(), BEDROCK_CHUNKS_PER_BATCH);
        assert_eq!(sender.pending_chunks.len(), 2);
        assert!(sender.commit_bedrock_batch(&batch, 3).is_none());

        assert_eq!(
            sender.on_bedrock_batch_completed(&first, &first.positions, &[], 3, true),
            first.positions
        );
        let second = sender
            .commit_bedrock_batch(&batch, 3)
            .expect("remaining chunks should be dispatched after completion");

        assert_eq!(second.chunks.len(), 2);
        assert!(sender.pending_chunks.is_empty());
    }

    #[test]
    fn bedrock_batch_requeues_chunks_that_were_not_queued() {
        let queued_position = Vector2::new(1, 0);
        let failed_position = Vector2::new(2, 0);
        let batch = PreparedBatch {
            chunks: [queued_position, failed_position]
                .into_iter()
                .map(|position| PreparedChunk {
                    position,
                    chunk: ChunkData::empty_sync(position.x, position.y),
                })
                .collect(),
            epoch_snapshot: 3,
            target_version: JavaMinecraftVersion::V_1_20_2,
        };
        let mut sender = ChunkSender::new();
        sender.enqueue_chunk(queued_position);
        sender.enqueue_chunk(failed_position);
        sender.send_quota = 2.0;
        let committed = sender
            .commit_bedrock_batch(&batch, 3)
            .expect("batch should be dispatched");

        let became_ready =
            sender.on_bedrock_batch_completed(&committed, &[queued_position], &[], 3, true);

        assert_eq!(became_ready, vec![queued_position]);
        assert!(sender.is_bedrock_chunk_ready(&queued_position));
        assert_eq!(sender.bedrock_completed_chunks_count(), 1);
        assert!(sender.bedrock_completed_chunks.contains(&queued_position));
        assert!(sender.is_chunk_sent(&queued_position));
        assert!(sender.pending_chunks.contains(&failed_position));
        assert!(!sender.is_chunk_sent(&failed_position));
        assert!(!sender.is_bedrock_chunk_ready(&failed_position));
        assert!(!sender.bedrock_completed_chunks.contains(&failed_position));
        assert_eq!(sender.in_flight_batches, 0);
    }

    #[test]
    fn bedrock_batch_does_not_retry_plugin_cancelled_chunks() {
        let cancelled_position = Vector2::new(1, 0);
        let failed_position = Vector2::new(2, 0);
        let batch = PreparedBatch {
            chunks: [cancelled_position, failed_position]
                .into_iter()
                .map(|position| PreparedChunk {
                    position,
                    chunk: ChunkData::empty_sync(position.x, position.y),
                })
                .collect(),
            epoch_snapshot: 3,
            target_version: JavaMinecraftVersion::V_1_20_2,
        };
        let mut sender = ChunkSender::new();
        sender.enqueue_chunk(cancelled_position);
        sender.enqueue_chunk(failed_position);
        sender.send_quota = 2.0;
        let committed = sender
            .commit_bedrock_batch(&batch, 3)
            .expect("batch should be dispatched");

        let became_ready =
            sender.on_bedrock_batch_completed(&committed, &[], &[cancelled_position], 3, true);

        assert!(became_ready.is_empty());
        assert!(sender.is_chunk_sent(&cancelled_position));
        assert!(!sender.is_bedrock_chunk_ready(&cancelled_position));
        assert_eq!(sender.bedrock_completed_chunks_count(), 1);
        assert!(
            sender
                .bedrock_completed_chunks
                .contains(&cancelled_position)
        );
        assert!(!sender.bedrock_completed_chunks.contains(&failed_position));
        assert!(!sender.pending_chunks.contains(&cancelled_position));
        assert!(sender.pending_chunks.contains(&failed_position));

        sender.send_quota = 2.0;
        let retried = sender
            .commit_bedrock_batch(&batch, 3)
            .expect("only the failed chunk should be retried");
        assert_eq!(retried.positions, vec![failed_position]);
    }

    #[test]
    fn bedrock_completion_distinguishes_outcomes_and_clears_terminal_state() {
        let queued = Vector2::new(1, 0);
        let cancelled = Vector2::new(2, 0);
        let failed = Vector2::new(3, 0);
        let batch = PreparedBatch {
            chunks: [queued, cancelled, failed]
                .into_iter()
                .map(|position| PreparedChunk {
                    position,
                    chunk: ChunkData::empty_sync(position.x, position.y),
                })
                .collect(),
            epoch_snapshot: 12,
            target_version: JavaMinecraftVersion::V_1_20_2,
        };
        let mut sender = ChunkSender::new();
        for position in [queued, cancelled, failed] {
            sender.enqueue_chunk(position);
        }
        sender.send_quota = 3.0;
        let committed = sender
            .commit_bedrock_batch(&batch, 12)
            .expect("mixed batch should be dispatched");

        let ready =
            sender.on_bedrock_batch_completed(&committed, &[queued], &[cancelled], 12, true);

        assert_eq!(ready, vec![queued]);
        assert_eq!(sender.bedrock_completed_chunks_count(), 2);
        assert!(sender.bedrock_completed_chunks.contains(&queued));
        assert!(sender.bedrock_completed_chunks.contains(&cancelled));
        assert!(!sender.bedrock_completed_chunks.contains(&failed));
        assert!(sender.is_bedrock_chunk_ready(&queued));
        assert!(!sender.is_bedrock_chunk_ready(&cancelled));
        assert!(!sender.is_bedrock_chunk_ready(&failed));
        assert!(!sender.pending_chunks.contains(&queued));
        assert!(!sender.pending_chunks.contains(&cancelled));
        assert!(sender.pending_chunks.contains(&failed));

        sender.enqueue_chunk(cancelled);
        assert_eq!(sender.bedrock_completed_chunks_count(), 1);
        assert!(sender.bedrock_completed_chunks.contains(&queued));
        assert!(!sender.bedrock_completed_chunks.contains(&cancelled));
        assert!(sender.is_bedrock_chunk_ready(&queued));
        assert!(sender.pending_chunks.contains(&cancelled));

        sender.reset();
        assert_eq!(sender.bedrock_completed_chunks_count(), 0);
        assert_eq!(sender.bedrock_ready_chunks_count(), 0);
        assert!(sender.pending_chunks.is_empty());
        assert_eq!(sender.sent_chunks_count(), 0);
    }

    #[test]
    fn bedrock_completion_from_a_different_world_is_requeued() {
        let position = Vector2::new(1, 1);
        let batch = PreparedBatch {
            chunks: vec![PreparedChunk {
                position,
                chunk: ChunkData::empty_sync(position.x, position.y),
            }],
            epoch_snapshot: 4,
            target_version: JavaMinecraftVersion::V_1_20_2,
        };
        let mut sender = ChunkSender::new();
        sender.enqueue_chunk(position);
        sender.send_quota = 1.0;
        let committed = sender
            .commit_bedrock_batch(&batch, 4)
            .expect("batch should be dispatched");

        assert!(
            sender
                .on_bedrock_batch_completed(&committed, &committed.positions, &[], 4, false)
                .is_empty()
        );
        assert!(sender.pending_chunks.contains(&position));
        assert!(!sender.is_chunk_sent(&position));
        assert!(!sender.is_bedrock_chunk_ready(&position));
        assert_eq!(sender.bedrock_completed_chunks_count(), 0);
        assert_eq!(sender.in_flight_batches, 0);
    }

    #[test]
    fn stale_bedrock_completion_does_not_release_a_new_batch() {
        let old_position = Vector2::new(1, 1);
        let old_batch = PreparedBatch {
            chunks: vec![PreparedChunk {
                position: old_position,
                chunk: ChunkData::empty_sync(old_position.x, old_position.y),
            }],
            epoch_snapshot: 4,
            target_version: JavaMinecraftVersion::V_1_20_2,
        };
        let mut sender = ChunkSender::new();
        sender.enqueue_chunk(old_position);
        sender.send_quota = 1.0;
        let old_committed = sender
            .commit_bedrock_batch(&old_batch, 4)
            .expect("old batch should be dispatched");

        sender.reset();
        let new_position = Vector2::new(2, 2);
        let new_batch = PreparedBatch {
            chunks: vec![PreparedChunk {
                position: new_position,
                chunk: ChunkData::empty_sync(new_position.x, new_position.y),
            }],
            epoch_snapshot: 5,
            target_version: JavaMinecraftVersion::V_1_20_2,
        };
        sender.enqueue_chunk(new_position);
        sender.send_quota = 1.0;
        let new_committed = sender
            .commit_bedrock_batch(&new_batch, 5)
            .expect("new batch should be dispatched");

        assert!(
            sender
                .on_bedrock_batch_completed(&old_committed, &old_committed.positions, &[], 5, true,)
                .is_empty()
        );
        assert_eq!(sender.in_flight_batches, 1);
        assert!(!sender.is_bedrock_chunk_ready(&old_position));
        assert!(!sender.is_bedrock_chunk_ready(&new_position));
        assert_eq!(sender.bedrock_completed_chunks_count(), 0);

        assert_eq!(
            sender.on_bedrock_batch_completed(
                &new_committed,
                &new_committed.positions,
                &[],
                5,
                true,
            ),
            vec![new_position]
        );
        assert_eq!(sender.in_flight_batches, 0);
        assert!(sender.is_bedrock_chunk_ready(&new_position));
        assert_eq!(sender.bedrock_completed_chunks_count(), 1);
        assert!(sender.bedrock_completed_chunks.contains(&new_position));
    }

    #[test]
    fn stale_bedrock_completion_releases_its_own_batch_after_epoch_advance() {
        let old_position = Vector2::new(1, 1);
        let old_batch = PreparedBatch {
            chunks: vec![PreparedChunk {
                position: old_position,
                chunk: ChunkData::empty_sync(old_position.x, old_position.y),
            }],
            epoch_snapshot: 10,
            target_version: JavaMinecraftVersion::V_1_20_2,
        };
        let mut sender = ChunkSender::new();
        sender.enqueue_chunk(old_position);
        sender.send_quota = 1.0;
        let old_committed = sender
            .commit_bedrock_batch(&old_batch, 10)
            .expect("old batch should be dispatched");

        // A same-world teleport advances the epoch without resetting the sender. The old send
        // still owns the in-flight slot and must release it, but its chunks must not become ready.
        assert!(
            sender
                .on_bedrock_batch_completed(
                    &old_committed,
                    &old_committed.positions,
                    &[],
                    11,
                    true,
                )
                .is_empty()
        );
        assert_eq!(sender.in_flight_batches, 0);
        assert!(!sender.is_bedrock_chunk_ready(&old_position));
        assert_eq!(sender.bedrock_completed_chunks_count(), 0);
        assert!(sender.pending_chunks.contains(&old_position));
        assert!(!sender.is_chunk_sent(&old_position));

        let new_position = Vector2::new(2, 2);
        let new_batch = PreparedBatch {
            chunks: vec![PreparedChunk {
                position: new_position,
                chunk: ChunkData::empty_sync(new_position.x, new_position.y),
            }],
            epoch_snapshot: 11,
            target_version: JavaMinecraftVersion::V_1_20_2,
        };
        sender.enqueue_chunk(new_position);
        sender.send_quota = 1.0;

        assert!(sender.commit_bedrock_batch(&new_batch, 11).is_some());
        assert_eq!(sender.in_flight_batches, 1);
    }
}
