use super::chunk_holder::ChunkHolder;
use super::chunk_state::{Chunk, StagedChunkEnum};
use super::dag::{DAG, Node, NodeKey};
use super::generation_cache::Cache;
use super::worker_logic::{RecvChunk, generation_work, io_read_work, io_write_work};
use super::{
    ChunkLevel, ChunkListener, ChunkLoading, ChunkPos, HashMapType, HashSetType, IOLock,
    LevelChannel,
};
use crate::level::{Level, SyncChunk};
use dashmap::DashMap;
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_util::math::vector2::Vector2;
use slotmap::Key;
use std::cmp::{Ordering, max};
use std::collections::{BinaryHeap, HashMap};
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Mutex};
use std::thread;

mod resort;
mod run;

static CHUNKS_LOADED_DISK: AtomicU64 = AtomicU64::new(0);
static CHUNKS_GEN_FULL: AtomicU64 = AtomicU64::new(0);

pub(crate) struct TaskHeapNode(i8, NodeKey);
impl PartialEq for TaskHeapNode {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl TaskHeapNode {
    #[cfg(test)]
    pub(crate) const fn node_key(&self) -> NodeKey {
        self.1
    }
}
impl Eq for TaskHeapNode {}
impl PartialOrd for TaskHeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for TaskHeapNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

pub struct GenerationSchedule {
    queue: BinaryHeap<TaskHeapNode>,
    graph: DAG,

    last_level: ChunkLevel,
    last_high_priority: Vec<ChunkPos>,
    send_level: Arc<LevelChannel>,

    public_chunk_map: Arc<DashMap<Vector2<i32>, SyncChunk>>,
    chunk_map: HashMap<ChunkPos, ChunkHolder>,
    unload_chunks: HashSetType<ChunkPos>,

    /// Tasks that are graph-ready (`in_degree` == 0) but cannot yet run because
    /// one or more of their required neighbor chunks haven't been delivered yet.
    /// Parked here and re-queued by `check_waiting_tasks()` as chunk data arrives.
    waiting_for_chunks: HashSetType<NodeKey>,

    io_lock: IOLock,
    running_task_count: u16,
    max_in_flight: u16,
    queue_dirty: bool,
    recv_chunk: crossfire::compat::MRx<(ChunkPos, RecvChunk)>,
    io_read: crossfire::compat::MTx<Vec<ChunkPos>>,
    io_write: crossfire::compat::Tx<Vec<(ChunkPos, Chunk)>>,
    generate: crossfire::compat::MTx<(ChunkPos, Cache, StagedChunkEnum)>,
    send_chunk: crossfire::compat::MTx<(ChunkPos, RecvChunk)>,
    gen_pool: Option<Arc<rayon::ThreadPool>>,
    listener: Arc<ChunkListener>,
    lighting_config: LightingEngineConfig,
    last_unload: std::time::Instant,
}

impl GenerationSchedule {
    pub fn create(
        io_read_thread_count: usize,
        gen_thread_count: usize,
        level: Arc<Level>,
        level_channel: Arc<LevelChannel>,
        listener: Arc<ChunkListener>,
        thread_tracker: &mut Vec<thread::JoinHandle<()>>,
        gen_pool: Option<Arc<rayon::ThreadPool>>,
    ) {
        let (send_chunk, recv_chunk) = crossfire::compat::mpmc::unbounded_blocking();

        let (send_read_io, recv_read_io) =
            crossfire::compat::mpmc::bounded_tx_blocking_rx_async(io_read_thread_count + 5);

        let (send_write_io, recv_write_io) =
            crossfire::compat::spsc::bounded_tx_blocking_rx_async(500);

        let (send_gen, recv_gen) = crossfire::compat::mpmc::bounded_blocking(gen_thread_count + 5);

        let io_lock = Arc::new((
            Mutex::new(HashMapType::default()),
            tokio::sync::Notify::new(),
        ));

        for _ in 0..io_read_thread_count {
            level.chunk_system_tasks.spawn(io_read_work(
                recv_read_io.clone(),
                send_chunk.clone(),
                level.clone(),
                io_lock.clone(),
            ));
        }

        level.chunk_system_tasks.spawn(io_write_work(
            recv_write_io,
            level.clone(),
            io_lock.clone(),
        ));

        if gen_pool.is_none() {
            for i in 0..gen_thread_count {
                let recv_gen = recv_gen.clone();
                let send_chunk = send_chunk.clone();
                let level_clone = level.clone();

                let handle = thread::Builder::new()
                    .name(format!("Gen-{i}"))
                    .spawn(move || {
                        generation_work(&recv_gen, &send_chunk, &level_clone);
                    })
                    .expect("Failed to spawn Generation Thread");

                thread_tracker.push(handle);
            }
        }

        // Backpressure follows the actual generation pool size, not the core
        // count: the pool may be sized smaller than the machine (see
        // `world.chunk_generation_threads`) to leave CPU headroom for the
        // tokio runtime and networking. 4x the pool keeps every generation
        // thread fed without flooding the queue with swapped-out chunk data.
        let max_in_flight = if let Some(pool) = &gen_pool {
            (pool.current_num_threads().max(1) * 4).min(usize::from(u16::MAX)) as u16
        } else {
            gen_thread_count as u16
        };

        let level_sched = level;
        let lighting_config = level_sched.lighting_config;
        let handle = thread::Builder::new()
            .name("Schedule".to_string())
            .spawn(move || {
                let scheduler = Self {
                    queue: BinaryHeap::new(),
                    graph: DAG::default(),
                    last_level: ChunkLevel::default(),
                    last_high_priority: Vec::new(),
                    send_level: level_channel,
                    public_chunk_map: level_sched.loaded_chunks.clone(),
                    unload_chunks: HashSetType::default(),
                    waiting_for_chunks: HashSetType::default(),
                    io_lock,
                    running_task_count: 0,
                    max_in_flight,
                    queue_dirty: false,
                    recv_chunk,
                    io_read: send_read_io,
                    io_write: send_write_io,
                    generate: send_gen,
                    send_chunk,
                    gen_pool,
                    listener,
                    chunk_map: HashMap::default(),
                    lighting_config,
                    last_unload: std::time::Instant::now(),
                };
                scheduler.work(&level_sched);
            })
            .expect("Failed to spawn Scheduler Thread");

        thread_tracker.push(handle);
    }

    fn apply_lighting_override(&self, chunk: &SyncChunk) {
        match self.lighting_config {
            LightingEngineConfig::Full => {
                let mut engine = chunk.light_engine.lock().unwrap();
                for section in &mut engine.block_light {
                    section.fill(15);
                }
                for section in &mut engine.sky_light {
                    section.fill(15);
                }
                chunk.dirty.store(true, Relaxed);
            }
            LightingEngineConfig::Dark => {
                let mut engine = chunk.light_engine.lock().unwrap();
                for section in &mut engine.block_light {
                    section.fill(0);
                }
                for section in &mut engine.sky_light {
                    section.fill(0);
                }
                chunk.dirty.store(true, Relaxed);
            }
            LightingEngineConfig::Default => {}
        }
    }

    fn calc_priority(
        last_level: &ChunkLevel,
        last_high_priority: &[ChunkPos],
        pos: ChunkPos,
        stage: StagedChunkEnum,
    ) -> i8 {
        if last_high_priority.is_empty() {
            return *last_level.get(&pos).unwrap_or(&ChunkLoading::MAX_LEVEL) + (stage as i8);
        }
        for i in last_high_priority {
            let dst = max((i.x - pos.x).abs(), (i.y - pos.y).abs());
            if dst <= StagedChunkEnum::FULL_RADIUS
                && stage <= StagedChunkEnum::FULL_DEPENDENCIES[dst as usize]
            {
                return *last_level.get(&pos).unwrap_or(&ChunkLoading::MAX_LEVEL) + (stage as i8)
                    - 100;
            }
        }
        *last_level.get(&pos).unwrap_or(&ChunkLoading::MAX_LEVEL) + (stage as i8)
    }

    fn sort_queue(&mut self) {
        if self.queue.is_empty() {
            return;
        }
        let mut tasks: Vec<_> = self.queue.drain().collect();
        for i in &mut tasks {
            if let Some(node) = self.graph.nodes.get(i.1) {
                i.0 = Self::calc_priority(
                    &self.last_level,
                    &self.last_high_priority,
                    node.pos,
                    node.stage,
                );
            }
        }
        self.queue = BinaryHeap::from(tasks);
    }

    /// TODO: will remove at some point
    pub(crate) fn restore_ready_tasks(
        graph: &mut DAG,
        queue: &mut BinaryHeap<TaskHeapNode>,
        chunk_map: &HashMap<ChunkPos, ChunkHolder>,
        last_level: &ChunkLevel,
        last_high_priority: &[ChunkPos],
        waiting_for_chunks: &HashSetType<NodeKey>,
    ) -> usize {
        debug_assert!(queue.is_empty());

        let mut ready = Vec::new();
        for (key, node) in &mut graph.nodes {
            node.in_queue = false;
            if node.stage == StagedChunkEnum::None
                || node.in_degree != 0
                || waiting_for_chunks.contains(&key)
            {
                continue;
            }
            let Some(holder) = chunk_map.get(&node.pos) else {
                continue;
            };
            if holder.current_stage >= node.stage || holder.tasks[node.stage as usize] != key {
                continue;
            }
            ready.push((key, node.pos, node.stage));
        }

        for (key, pos, stage) in &ready {
            let Some(node) = graph.nodes.get_mut(*key) else {
                continue;
            };
            node.in_queue = true;
            queue.push(TaskHeapNode(
                Self::calc_priority(last_level, last_high_priority, *pos, *stage),
                *key,
            ));
        }

        ready.len()
    }

    /// Ensure that the dependency chain for `req_stage` exists on `holder` (for chunk at
    /// `chunk_pos`) and wire it to depend on `dependency_task`.
    ///
    /// Bumps `holder.dependency_stage` (NOT `target_stage`) to at least `req_stage` so
    /// that neighbor chunks pulled in as generation dependencies are not discarded before
    /// their dependency is satisfied. `target_stage` is left alone so the level-change
    /// bookkeeping invariant (`old_stage == holder.target_stage`) is never violated.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn ensure_dependency_chain(
        graph: &mut DAG,
        queue: &mut BinaryHeap<TaskHeapNode>,
        last_level: &ChunkLevel,
        last_high_priority: &[ChunkPos],
        dependency_task: NodeKey,
        chunk_pos: ChunkPos,
        holder: &mut ChunkHolder,
        req_stage: StagedChunkEnum,
    ) {
        // Insert occupied_by edge head
        holder.occupied_by = graph.edges.insert(crate::chunk_system::dag::Edge::new(
            dependency_task,
            holder.occupied_by,
        ));

        if !holder.occupied.is_null() {
            graph.add_edge(holder.occupied, dependency_task);
        }

        // Bump dependency_stage so this chunk's IO/generation tasks are scheduled and
        // kept alive even if target_stage is None (outside player view radius).
        // We deliberately do NOT touch target_stage — that field is owned by resort_work
        // and must match the level-change bookkeeping or the debug_assert will fire.
        if holder.dependency_stage < req_stage {
            holder.dependency_stage = req_stage;
        }

        // Effective target is the max of what the player wants and what dependencies need.
        let effective_target = holder.target_stage.max(holder.dependency_stage);

        // Create any missing tasks from current_stage+1 up to effective_target.
        // We do this even when current_stage >= req_stage, because dependency_stage may
        // require tasks beyond req_stage that haven't been created yet.
        if holder.current_stage < effective_target {
            let empty = StagedChunkEnum::Empty as usize;
            let start = (holder.current_stage as usize + 1).max(empty);
            let end = effective_target as u8 as usize;
            let mut newly_created = [false; StagedChunkEnum::COUNT];

            for (i, flag) in newly_created[start..=end].iter_mut().enumerate() {
                let stage_i = start + i;
                if holder.tasks[stage_i].is_null() {
                    let new_node = graph
                        .nodes
                        .insert(Node::new(chunk_pos, StagedChunkEnum::from(stage_i as u8)));
                    holder.tasks[stage_i] = new_node;
                    *flag = true;
                    if !holder.occupied.is_null() {
                        graph.add_edge(holder.occupied, new_node);
                    }
                }
            }

            for stage_i in start..=end {
                if !newly_created[stage_i] {
                    continue;
                }
                let cur = holder.tasks[stage_i];

                if stage_i > empty {
                    let prev = holder.tasks[stage_i - 1];
                    if !prev.is_null() {
                        graph.add_edge(prev, cur);
                    }
                }
                if stage_i < end {
                    let next = holder.tasks[stage_i + 1];
                    if !next.is_null() && !newly_created[stage_i + 1] {
                        graph.add_edge(cur, next);
                    }
                }
            }

            // Queue the entry task (lowest unblocked stage)
            let entry_task = holder.tasks[start];
            if !entry_task.is_null()
                && let Some(n) = graph.nodes.get_mut(entry_task)
                && n.in_degree == 0
                && !n.in_queue
            {
                n.in_queue = true;
                queue.push(TaskHeapNode(
                    Self::calc_priority(
                        last_level,
                        last_high_priority,
                        chunk_pos,
                        StagedChunkEnum::from(start as u8),
                    ),
                    entry_task,
                ));
            }
        }

        // If req_stage is already satisfied, dependency_task doesn't need to wait —
        // it was only blocked on `occupied` (handled above) and the stage itself is done.
        // Do NOT add an edge here: tasks[req_stage] is null (completed and dropped).
        if holder.current_stage >= req_stage {
            return;
        }

        // Wire req_stage task → dependency_task so dependency_task can't run until
        // this chunk reaches req_stage. tasks[req_stage] is guaranteed non-null here:
        // effective_target >= req_stage (we just set dependency_stage = req_stage) and
        // current_stage < req_stage, so the task was created in the loop above (or
        // already existed).
        let req_end = req_stage as u8 as usize;
        let ano_task = holder.tasks[req_end];
        debug_assert!(
            !ano_task.is_null(),
            "holder.tasks[req_stage] must not be null before adding edge"
        );
        graph.add_edge(ano_task, dependency_task);
    }

    /// Check if any tasks parked in `waiting_for_chunks` now have all their neighbor
    /// chunk data available, and re-queue them if so.
    /// Must be called after every `receive_chunk` call.
    fn check_waiting_tasks(&mut self) {
        if self.waiting_for_chunks.is_empty() {
            return;
        }

        let mut now_ready: Vec<NodeKey> = Vec::new();

        self.waiting_for_chunks.retain(|&node_key| {
            let Some(node) = self.graph.nodes.get(node_key) else {
                return false; // node was dropped, discard silently
            };
            let write_radius = node.stage.get_write_radius();
            let pos = node.pos;
            let all_ready = (-write_radius..=write_radius).all(|dx| {
                (-write_radius..=write_radius).all(|dy| {
                    self.chunk_map
                        .get(&pos.add_raw(dx, dy))
                        .is_some_and(|h| h.chunk.is_some())
                })
            });
            if all_ready {
                now_ready.push(node_key);
                false
            } else {
                true
            }
        });

        for node_key in now_ready {
            if let Some(n) = self.graph.nodes.get_mut(node_key)
                && n.in_degree == 0
                && !n.in_queue
            {
                n.in_queue = true;
                let priority =
                    Self::calc_priority(&self.last_level, &self.last_high_priority, n.pos, n.stage);
                self.queue.push(TaskHeapNode(priority, node_key));
            }
            // If in_degree > 0, drop_node will re-queue when unblocked
        }
    }

    fn drop_node(&mut self, node: NodeKey) {
        let Some(old) = self.graph.nodes.remove(node) else {
            return;
        };
        let mut edge = old.edge;
        while !edge.is_null() {
            let cur = self.graph.edges.remove(edge).unwrap();
            if let Some(node) = self.graph.nodes.get_mut(cur.to) {
                debug_assert!(node.in_degree >= 1);
                node.in_degree -= 1;
                if node.in_degree == 0 && !node.in_queue {
                    // Don't queue if parked in waiting_for_chunks — check_waiting_tasks()
                    // will re-queue it once chunk data arrives.
                    if !self.waiting_for_chunks.contains(&cur.to) {
                        self.queue.push(TaskHeapNode(
                            Self::calc_priority(
                                &self.last_level,
                                &self.last_high_priority,
                                node.pos,
                                node.stage,
                            ),
                            cur.to,
                        ));
                        node.in_queue = true;
                    }
                }
            }
            edge = cur.next;
        }
    }

    fn drop_satisfied_tasks(&mut self, holder: &mut ChunkHolder, stage: StagedChunkEnum) {
        for task_idx in (holder.current_stage as usize + 1)..=(stage as usize) {
            if !holder.tasks[task_idx].is_null() {
                self.waiting_for_chunks.remove(&holder.tasks[task_idx]);
                self.drop_node(holder.tasks[task_idx]);
                holder.tasks[task_idx] = NodeKey::null();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BinaryHeap, ChunkLevel, ChunkLoading, ChunkPos, GenerationSchedule, HashMap, HashSetType,
        StagedChunkEnum, TaskHeapNode,
    };
    use crate::chunk_system::dag::{DAG, NodeKey};
    use slotmap::Key;

    #[test]
    fn task_heap_pops_lowest_priority_first() {
        let mut queue = BinaryHeap::new();
        queue.push(TaskHeapNode(5, NodeKey::null()));
        queue.push(TaskHeapNode(-3, NodeKey::null()));
        queue.push(TaskHeapNode(1, NodeKey::null()));

        let order: Vec<i8> = std::iter::from_fn(|| queue.pop().map(|node| node.0)).collect();
        assert_eq!(order, [-3, 1, 5]);
    }

    #[test]
    fn calc_priority_defaults_to_max_level() {
        let level = ChunkLevel::default();
        let pos = ChunkPos::new(3, -2);
        let priority = GenerationSchedule::calc_priority(&level, &[], pos, StagedChunkEnum::Full);
        assert_eq!(
            priority,
            ChunkLoading::MAX_LEVEL + StagedChunkEnum::Full as i8
        );
    }

    #[test]
    fn calc_priority_boosts_chunks_near_high_priority_positions() {
        let level = ChunkLevel::default();
        let pos = ChunkPos::new(0, 0);
        let base = GenerationSchedule::calc_priority(&level, &[], pos, StagedChunkEnum::Empty);
        let boosted = GenerationSchedule::calc_priority(
            &level,
            &[ChunkPos::new(1, 1)],
            pos,
            StagedChunkEnum::Empty,
        );
        assert_eq!(boosted, base - 100);
    }

    #[test]
    fn restore_ready_tasks_handles_empty_graph() {
        let mut graph = DAG::default();
        let mut queue = BinaryHeap::new();
        let chunk_map = HashMap::default();
        let level = ChunkLevel::default();
        let waiting = HashSetType::default();

        let restored = GenerationSchedule::restore_ready_tasks(
            &mut graph,
            &mut queue,
            &chunk_map,
            &level,
            &[],
            &waiting,
        );
        assert_eq!(restored, 0);
        assert!(queue.is_empty());
    }
}
