use crate::world::World;
use crossbeam::channel::Receiver;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::chunk::ChunkEntityData;
use pumpkin_world::chunk_system::ChunkLoading;
use pumpkin_world::level::Level;
use pumpkin_world::level::SyncChunk;
use pumpkin_world::level::SyncEntityChunk;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Weak;
use std::time::Duration;
use std::time::Instant;

struct HeapNode(i32, Vector2<i32>, Weak<ChunkData>);

impl Eq for HeapNode {}

impl PartialEq<Self> for HeapNode {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd<Self> for HeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

pub struct ChunkManager {
    chunks_per_tick: usize,
    center: Vector2<i32>,
    view_distance: u8,
    chunk_listener: Receiver<(Vector2<i32>, Weak<ChunkData>)>,
    chunk_sent: HashMap<Vector2<i32>, Weak<ChunkData>>,
    chunk_queue: BinaryHeap<HeapNode>,
    entity_chunk_queue: VecDeque<(Vector2<i32>, Weak<ChunkEntityData>)>,
    batches_sent_since_ack: u8,
    last_chunk_batch_sent_at: Instant,
    /// The current world for chunk loading. Updated on dimension change.
    world: Arc<World>,
}

impl ChunkManager {
    pub const NOTCHIAN_BATCHES_WITHOUT_ACK_UNTIL_PAUSE: u8 = 10;
    const ACK_STALL_FALLBACK_DELAY: Duration = Duration::from_millis(250);

    #[must_use]
    pub fn new(
        chunks_per_tick: usize,
        chunk_listener: Receiver<(Vector2<i32>, Weak<ChunkData>)>,
        world: Arc<World>,
    ) -> Self {
        Self {
            chunks_per_tick,
            center: Vector2::<i32>::new(0, 0),
            view_distance: 0,
            chunk_listener,
            chunk_sent: HashMap::new(),
            chunk_queue: BinaryHeap::new(),
            entity_chunk_queue: VecDeque::new(),
            batches_sent_since_ack: 0,
            last_chunk_batch_sent_at: Instant::now(),
            world,
        }
    }

    /// Gets the current world for chunk loading.
    #[must_use]
    pub const fn world(&self) -> &Arc<World> {
        &self.world
    }

    #[must_use]
    pub fn sent_chunks_count(&self) -> usize {
        self.chunk_sent.len()
    }

    fn should_enqueue_chunk(&mut self, position: Vector2<i32>, chunk: &SyncChunk) -> bool {
        self.chunk_sent
            .insert(position, Arc::downgrade(chunk))
            .and_then(|old_chunk| old_chunk.upgrade())
            .is_none_or(|old_chunk| !Arc::ptr_eq(&old_chunk, chunk))
    }

    #[must_use]
    const fn ack_window_open(&self) -> bool {
        self.batches_sent_since_ack < Self::NOTCHIAN_BATCHES_WITHOUT_ACK_UNTIL_PAUSE
    }

    #[must_use]
    fn ack_fallback_ready(&self) -> bool {
        !self.ack_window_open()
            && self.last_chunk_batch_sent_at.elapsed() >= Self::ACK_STALL_FALLBACK_DELAY
    }

    pub fn pull_new_chunks(&mut self) {
        while let Ok((pos, chunk_weak)) = self.chunk_listener.try_recv() {
            let dst = Self::chebyshev(pos, self.center);
            if dst > i32::from(self.view_distance) {
                continue;
            }
            if let Some(chunk) = chunk_weak.upgrade()
                && self.should_enqueue_chunk(pos, &chunk)
            {
                self.chunk_queue.push(HeapNode(dst, pos, chunk_weak));
            }
        }
    }

    fn chebyshev(a: Vector2<i32>, b: Vector2<i32>) -> i32 {
        (a.x - b.x).abs().max((a.y - b.y).abs())
    }

    pub fn update_center_and_view_distance(
        &mut self,
        center: Vector2<i32>,
        mut view_distance: u8,
        level: &Arc<Level>,
        loading_chunks: &[Vector2<i32>],
        unloading_chunks: &[Vector2<i32>],
    ) {
        view_distance += 1; // Margin for loading
        let old_center = self.center;
        let old_view_distance = self.view_distance;

        {
            let mut lock = level.chunk_loading.lock().unwrap();
            let new_level = ChunkLoading::get_level_from_view_distance(view_distance);
            lock.add_ticket(center, new_level);

            if old_center != center || old_view_distance != view_distance {
                let old_level = ChunkLoading::get_level_from_view_distance(old_view_distance);
                // Don't remove if it would be the same ticket
                if old_center != center || old_level != new_level {
                    lock.remove_ticket(old_center, old_level);
                }
            }
            lock.send_change();
        };

        self.center = center;
        self.view_distance = view_distance;
        let view_distance_i32 = i32::from(view_distance);
        let unloading_chunks: HashSet<Vector2<i32>> = unloading_chunks.iter().copied().collect();

        self.chunk_sent.retain(|pos, _| {
            (pos.x - center.x).abs().max((pos.y - center.y).abs()) <= view_distance_i32
                && !unloading_chunks.contains(pos)
        });

        self.entity_chunk_queue.retain(|(pos, _)| {
            (pos.x - center.x).abs().max((pos.y - center.y).abs()) <= view_distance_i32
                && !unloading_chunks.contains(pos)
        });

        let mut tasks: Vec<_> = self
            .chunk_queue
            .drain()
            .filter_map(|node| {
                let dst = Self::chebyshev(node.1, center);
                (dst <= view_distance_i32 && !unloading_chunks.contains(&node.1))
                    .then(|| HeapNode(dst, node.1, node.2))
            })
            .collect();

        for pos in loading_chunks {
            if !self.chunk_sent.contains_key(pos)
                && let Some(chunk) = level.loaded_chunks.get(pos)
            {
                let chunk = chunk.value().clone();
                if self.should_enqueue_chunk(*pos, &chunk) {
                    let dst = (pos.x - center.x).abs().max((pos.y - center.y).abs());
                    tasks.push(HeapNode(dst, *pos, Arc::downgrade(&chunk)));
                }
            }
        }
        self.chunk_queue = BinaryHeap::from(tasks);
    }

    pub fn clean_up(&mut self, level: &Arc<Level>) {
        let mut lock = level.chunk_loading.lock().unwrap();
        lock.remove_ticket(
            self.center,
            ChunkLoading::get_level_from_view_distance(self.view_distance),
        );
        lock.send_change();
        let (_rx, tx) = crossbeam::channel::unbounded();
        // drop old channel
        self.chunk_listener = tx;

        // Drop any held chunk references to allow chunks to be unloaded.
        self.chunk_sent.clear();
        self.chunk_queue.clear();
        self.entity_chunk_queue.clear();
        self.batches_sent_since_ack = 0;
        self.last_chunk_batch_sent_at = Instant::now();
    }

    pub fn change_world(&mut self, old_level: &Arc<Level>, new_world: Arc<World>) {
        let mut lock = old_level.chunk_loading.lock().unwrap();
        lock.remove_ticket(
            self.center,
            ChunkLoading::get_level_from_view_distance(self.view_distance),
        );
        lock.send_change();
        drop(lock);
        self.chunk_listener = new_world.level.chunk_listener.add_global_chunk_listener();
        self.chunk_sent.clear();
        self.chunk_queue.clear();
        self.world = new_world;
        // Reset batch state so chunks can be sent immediately in the new dimension
        self.batches_sent_since_ack = 0;
        self.last_chunk_batch_sent_at = Instant::now();
    }

    pub const fn handle_acknowledge(&mut self, chunks_per_tick: f32) {
        self.batches_sent_since_ack = 0;
        self.chunks_per_tick = chunks_per_tick.ceil() as usize;
    }

    pub fn push_chunk(&mut self, position: Vector2<i32>, chunk: &SyncChunk) {
        if self.should_enqueue_chunk(position, chunk) {
            let dst = (position.x - self.center.x)
                .abs()
                .max((position.y - self.center.y).abs());
            self.chunk_queue
                .push(HeapNode(dst, position, Arc::downgrade(chunk)));
        }
    }

    pub fn push_entity(&mut self, position: Vector2<i32>, chunk: &SyncEntityChunk) {
        self.entity_chunk_queue
            .push_back((position, Arc::downgrade(chunk)));
    }

    #[must_use]
    pub fn can_send_chunk(&self) -> bool {
        let state_available = self.ack_window_open() || self.ack_fallback_ready();

        state_available && !self.chunk_queue.is_empty()
    }

    pub fn next_chunk(&mut self) -> Box<[SyncChunk]> {
        let take = self.chunk_queue.len().min(self.chunks_per_tick.max(1));
        let mut chunks = Vec::with_capacity(take);
        while chunks.len() < take
            && let Some(node) = self.chunk_queue.pop()
        {
            if let Some(chunk) = node.2.upgrade() {
                chunks.push(chunk);
            }
        }
        self.batches_sent_since_ack = self.batches_sent_since_ack.saturating_add(1);
        self.last_chunk_batch_sent_at = Instant::now();

        chunks.into_boxed_slice()
    }

    pub fn next_entity(&mut self) -> Box<[SyncEntityChunk]> {
        let chunk_size = self
            .entity_chunk_queue
            .len()
            .min(self.chunks_per_tick.max(1));

        let mut chunks = Vec::with_capacity(chunk_size);
        while chunks.len() < chunk_size
            && let Some((_, weak_chunk)) = self.entity_chunk_queue.pop_front()
        {
            if let Some(chunk) = weak_chunk.upgrade() {
                chunks.push(chunk);
            }
        }

        self.batches_sent_since_ack = self.batches_sent_since_ack.saturating_add(1);
        self.last_chunk_batch_sent_at = Instant::now();

        chunks.into_boxed_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    fn node(distance: i32, pos: Vector2<i32>) -> HeapNode {
        HeapNode(distance, pos, Weak::new())
    }

    #[test]
    fn chunk_queue_pops_nearest_chunk_first() {
        let mut queue = BinaryHeap::new();
        queue.push(node(5, Vector2::new(5, 0)));
        queue.push(node(1, Vector2::new(0, 1)));
        queue.push(node(3, Vector2::new(3, 3)));

        assert_eq!(queue.pop().unwrap().0, 1);
        assert_eq!(queue.pop().unwrap().0, 3);
        assert_eq!(queue.pop().unwrap().0, 5);
        assert!(queue.pop().is_none());
    }

    #[test]
    fn heap_node_ordering_is_reversed_and_ignores_position() {
        // Reversed ordering: the smaller distance is the "greater" node so
        // `BinaryHeap` (a max-heap) yields it first.
        assert_eq!(
            node(1, Vector2::new(9, 9)).cmp(&node(2, Vector2::new(0, 0))),
            Ordering::Greater
        );
        assert_eq!(
            node(2, Vector2::new(0, 0)).cmp(&node(1, Vector2::new(0, 0))),
            Ordering::Less
        );
        assert!(node(4, Vector2::new(1, 2)) == node(4, Vector2::new(-7, 5)));
    }

    #[test]
    fn chebyshev_distance_is_the_maximum_axis_delta() {
        assert_eq!(
            ChunkManager::chebyshev(Vector2::new(3, -4), Vector2::new(0, 0)),
            4
        );
        assert_eq!(
            ChunkManager::chebyshev(Vector2::new(5, 2), Vector2::new(1, 3)),
            4
        );
        assert_eq!(
            ChunkManager::chebyshev(Vector2::new(-2, 7), Vector2::new(-2, 7)),
            0
        );
    }
}
