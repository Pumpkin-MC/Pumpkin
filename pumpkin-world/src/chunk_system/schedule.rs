use super::channel::LevelChange;
use super::chunk_holder::ChunkHolder;
use super::chunk_state::{Chunk, StagedChunkEnum};
use super::dag::{DAG, EdgeKey, Node, NodeKey};
use super::generation_cache::Cache;
use super::worker_logic::{RecvChunk, generation_work, io_read_work, io_write_work};
use super::{
    ChunkLevel, ChunkListener, ChunkLoading, ChunkPos, HashMapType, HashSetType, IOLock,
    LevelChannel,
};
use crate::chunk::io::Dirtiable;
use crate::level::{Level, SyncChunk};
use dashmap::DashMap;
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_util::math::vector2::Vector2;
use slotmap::Key;
use std::cmp::{Ordering, max};
use std::collections::{BinaryHeap, HashMap};
use std::mem::swap;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, trace, warn};

pub(crate) struct TaskHeapNode(i8, NodeKey);
impl PartialEq for TaskHeapNode {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl TaskHeapNode {
    #[cfg(test)]
    pub(crate) fn node_key(&self) -> NodeKey {
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
    unload_retry_counts: HashMap<ChunkPos, u32>,

    io_lock: IOLock,
    running_task_count: u16,
    recv_chunk: crossfire::compat::MRx<(ChunkPos, RecvChunk)>,
    io_read: crossfire::compat::MTx<ChunkPos>,
    io_write: crossfire::compat::Tx<Vec<(ChunkPos, Chunk)>>,
    generate: crossfire::compat::MTx<(ChunkPos, Cache, StagedChunkEnum)>,
    listener: Arc<ChunkListener>,
    lighting_config: LightingEngineConfig,
}

impl GenerationSchedule {
    pub fn create(
        io_read_thread_count: usize,
        gen_thread_count: usize,
        level: Arc<Level>,
        level_channel: Arc<LevelChannel>,
        listener: Arc<ChunkListener>,
        thread_tracker: &mut Vec<thread::JoinHandle<()>>,
    ) {
        let (send_chunk, recv_chunk) = crossfire::compat::mpmc::unbounded_blocking();

        let (send_read_io, recv_read_io) =
            crossfire::compat::mpmc::bounded_tx_blocking_rx_async(io_read_thread_count + 5);

        // Use a bounded single-producer/single-consumer channel to prevent unbounded memory growth
        // when IO can't keep up. If the queue reaches capacity, producers will block until
        // the disk catches up.
        let (send_write_io, recv_write_io) =
            crossfire::compat::spsc::bounded_tx_blocking_rx_async(500);

        let (send_gen, recv_gen) = crossfire::compat::mpmc::bounded_blocking(gen_thread_count + 5);

        let io_lock = Arc::new((Mutex::new(HashMapType::default()), Condvar::new()));

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

        for i in 0..gen_thread_count {
            let recv_gen = recv_gen.clone();
            let send_chunk = send_chunk.clone();
            let level_clone = level.clone();

            let handle = thread::Builder::new()
                .name(format!("Gen-{i}")) // Identifying dim helps debugging
                .spawn(move || {
                    generation_work(recv_gen, send_chunk, level_clone);
                })
                .expect("Failed to spawn Generation Thread");

            thread_tracker.push(handle);
        }

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
                    unload_retry_counts: HashMap::new(),
                    io_lock,
                    running_task_count: 0,
                    recv_chunk,
                    io_read: send_read_io,
                    io_write: send_write_io,
                    generate: send_gen,
                    listener,
                    chunk_map: Default::default(),
                    lighting_config,
                };
                scheduler.work(level_sched);
            })
            .expect("Failed to spawn Scheduler Thread");

        thread_tracker.push(handle);
    }

    fn apply_lighting_override(&self, chunk: &SyncChunk) {
        match self.lighting_config {
            LightingEngineConfig::Full => {
                let mut engine = chunk.light_engine.lock().unwrap();

                for section in engine.block_light.iter_mut() {
                    section.fill(15);
                }
                for section in engine.sky_light.iter_mut() {
                    section.fill(15);
                }

                chunk.dirty.store(true, Relaxed);
            }
            LightingEngineConfig::Dark => {
                let mut engine = chunk.light_engine.lock().unwrap();

                for section in engine.block_light.iter_mut() {
                    section.fill(0);
                }
                for section in engine.sky_light.iter_mut() {
                    section.fill(0);
                }

                chunk.dirty.store(true, Relaxed);
            }
            _ => {}
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
        let mut new_queue = BinaryHeap::with_capacity(self.queue.len());
        for i in &self.queue {
            if let Some(node) = self.graph.nodes.get(i.1) {
                new_queue.push(TaskHeapNode(
                    Self::calc_priority(
                        &self.last_level,
                        &self.last_high_priority,
                        node.pos,
                        node.stage,
                    ),
                    i.1,
                ));
            }
        }
        self.queue = new_queue;
    }

    // Ensure that the dependency chain for `req_stage` exists on `holder` (for chunk at
    // `chunk_pos`) and wire it to depend on `dependency_task`.
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

        if holder.current_stage >= req_stage {
            return;
        }

        let empty = StagedChunkEnum::Empty as usize;
        let start = (holder.current_stage as usize + 1).max(empty);
        let end = req_stage as u8 as usize;
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

        let ano_task = holder.tasks[end];
        debug_assert!(
            !ano_task.is_null(),
            "holder.tasks[end] must not be null before adding edge"
        );
        graph.add_edge(ano_task, dependency_task);
    }

    fn resort_work(&mut self, new_data: (Option<LevelChange>, Option<Vec<ChunkPos>>)) -> bool {
        // true -> updated | false -> not update
        if new_data.0.is_none() && new_data.1.is_none() {
            return false;
        }
        if let Some(high_priority) = new_data.1 {
            // debug!("receive new priority");
            self.last_high_priority = high_priority;
        }
        let Some(new_level) = new_data.0 else {
            self.sort_queue();
            return true;
        };
        // debug!("receive new level");
        for (pos, (old_stage, new_stage)) in new_level.0 {
            debug_assert_ne!(old_stage, new_stage);
            debug_assert_eq!(
                new_stage,
                StagedChunkEnum::level_to_stage(
                    *new_level.1.get(&pos).unwrap_or(&ChunkLoading::MAX_LEVEL)
                )
            );
            let mut holder = self.chunk_map.remove(&pos).unwrap_or_default();
            debug_assert_eq!(holder.target_stage, old_stage);
            holder.target_stage = new_stage;
            if old_stage > new_stage {
                for i in (new_stage.max(holder.current_stage) as usize + 1)..=(old_stage as usize) {
                    let task = &mut holder.tasks[i];
                    if !task.is_null() {
                        self.drop_node(*task); // Properly handles incoming/outgoing edges and in_degree
                        *task = NodeKey::null();
                    }
                }
                if new_stage == StagedChunkEnum::None {
                    self.unload_chunks.insert(pos);
                }
            } else {
                if old_stage == StagedChunkEnum::None {
                    self.unload_chunks.remove(&pos);
                    if holder.current_stage == StagedChunkEnum::Full && !holder.public {
                        holder.public = true;
                        match holder.chunk.as_ref().unwrap() {
                            Chunk::Level(chunk) => {
                                self.apply_lighting_override(chunk);
                                self.public_chunk_map.insert(pos, chunk.clone());
                                self.listener.process_new_chunk(pos, chunk);
                            }
                            Chunk::Proto(_) => panic!(),
                        }
                    }
                }
                for i in (old_stage.max(holder.current_stage) as u8 + 1)..=(new_stage as u8) {
                    let task = &mut holder.tasks[i as usize];
                    if task.is_null() {
                        *task = self.graph.nodes.insert(Node::new(pos, i.into()));
                        if !holder.occupied.is_null() {
                            self.graph.add_edge(holder.occupied, *task);
                        }
                    }
                    let task = *task;
                    if i > 1 {
                        let stage = StagedChunkEnum::from(i);
                        let dependency = stage.get_direct_dependencies();
                        let radius = stage.get_direct_radius();
                        for dx in -radius..=radius {
                            for dz in -radius..=radius {
                                let new_pos = pos.add_raw(dx, dz);
                                let req_stage = dependency[dx.abs().max(dz.abs()) as usize];
                                if new_pos == pos {
                                    Self::ensure_dependency_chain(
                                        &mut self.graph,
                                        &mut self.queue,
                                        &self.last_level,
                                        &self.last_high_priority,
                                        task,
                                        new_pos,
                                        &mut holder,
                                        req_stage,
                                    );
                                    continue;
                                }

                                let ano_chunk = self.chunk_map.entry(new_pos).or_default();
                                Self::ensure_dependency_chain(
                                    &mut self.graph,
                                    &mut self.queue,
                                    &self.last_level,
                                    &self.last_high_priority,
                                    task,
                                    new_pos,
                                    ano_chunk,
                                    req_stage,
                                );
                            }
                        }
                    }
                    let node = self.graph.nodes.get_mut(task).unwrap();
                    if node.in_degree == 0 {
                        node.in_queue = true;
                        self.queue.push(TaskHeapNode(0, task));
                    }
                }
            }
            self.chunk_map.insert(pos, holder);
        }
        self.last_level = new_level.1;
        self.sort_queue();
        true
    }

    fn unload_chunk(&mut self) {
        let mut unload_chunks = HashSetType::default();
        swap(&mut unload_chunks, &mut self.unload_chunks);
        let mut chunks = Vec::with_capacity(unload_chunks.len());
        for pos in unload_chunks {
            let holder = self.chunk_map.get_mut(&pos).unwrap();
            debug_assert_eq!(holder.target_stage, StagedChunkEnum::None);
            if holder.occupied.is_null() {
                let mut tmp = None;
                swap(&mut holder.chunk, &mut tmp);
                let Some(tmp) = tmp else {
                    continue;
                };
                match tmp {
                    Chunk::Level(chunk) => {
                        if holder.public {
                            self.public_chunk_map.remove(&pos);
                            holder.public = false;
                        }
                        let sc = Arc::strong_count(&chunk);
                        let force_unload = if sc > 1 {
                            let retry_count = self.unload_retry_counts.entry(pos).or_insert(0);
                            *retry_count += 1;
                            let count = *retry_count;

                            if count < 120 {
                                if count == 1 || count.is_multiple_of(20) {
                                    warn!(
                                        "unload_chunk: chunk {pos:?} still has {sc} strong refs; \
                                         retry {count}/120. holder.public={}",
                                        holder.public
                                    );
                                }
                                self.unload_chunks.insert(pos);
                                holder.chunk = Some(Chunk::Level(chunk));
                                continue;
                            }
                            warn!(
                                "unload_chunk: chunk {pos:?} force-unloading after {count} retries \
                                 (still has {sc} strong refs, holder.public={})",
                                holder.public
                            );
                            true
                        } else {
                            false
                        };
                        if force_unload {
                            self.unload_retry_counts.remove(&pos);
                        }
                        chunks.push((pos, Chunk::Level(chunk)));
                        self.chunk_map.remove(&pos);
                    }
                    Chunk::Proto(chunk) => {
                        debug_assert!(!holder.public);
                        // debug!("unload proto chunk {pos:?} to file");
                        chunks.push((pos, Chunk::Proto(chunk)));
                        self.chunk_map.remove(&pos);
                    }
                }
            }
        }
        // debug!("send {} unloaded chunks to io write", chunks.len());
        if chunks.is_empty() {
            return;
        }
        let mut data = self.io_lock.0.lock().unwrap();
        for (pos, _chunk) in &chunks {
            *data.entry(*pos).or_insert(0) += 1;
        }
        drop(data);
        if let Err(e) = self.io_write.send(chunks) {
            error!(
                "Failed to send chunks to io write thread during save (may have shut down): {:?}",
                e
            );
        }
    }

    fn save_all_chunk(&self, save_proto_chunk: bool) {
        let mut chunks = Vec::with_capacity(self.chunk_map.len());
        for (pos, holder) in &self.chunk_map {
            if let Some(chunk) = &holder.chunk {
                match chunk {
                    Chunk::Level(sync_chunk) => {
                        // Only save level chunks that are marked dirty
                        if sync_chunk.is_dirty() {
                            chunks.push((*pos, Chunk::Level(sync_chunk.clone())));
                        }
                    }
                    Chunk::Proto(proto) => {
                        if save_proto_chunk {
                            chunks.push((*pos, Chunk::Proto(proto.clone())));
                        }
                    }
                }
            }
        }
        if chunks.is_empty() {
            return;
        }
        info!(
            "Saving {} chunks (collected from {} holders)...",
            chunks.len(),
            self.chunk_map.len()
        );
        let mut data = self.io_lock.0.lock().unwrap();
        for (pos, _chunk) in &chunks {
            *data.entry(*pos).or_insert(0) += 1;
        }
        drop(data);
        if let Err(e) = self.io_write.send(chunks) {
            error!(
                "Failed to send chunks to io write thread during unload (may have shut down): {:?}",
                e
            );
        }
    }

    fn drop_node(&mut self, node: NodeKey) {
        let Some(old) = self.graph.nodes.remove(node) else {
            return;
        };
        // debug!("drop node {node:?}");
        let mut edge = old.edge;
        while !edge.is_null() {
            let cur = self.graph.edges.remove(edge).unwrap();
            if let Some(node) = self.graph.nodes.get_mut(cur.to) {
                debug_assert!(node.in_degree >= 1);
                node.in_degree -= 1;
                if node.in_degree == 0 && !node.in_queue {
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
            edge = cur.next;
        }
    }

    fn receive_chunk(&mut self, pos: ChunkPos, data: RecvChunk) {
        match data {
            RecvChunk::IO(chunk) => {
                // debug!("receive io chunk pos {pos:?}");
                let mut holder = self.chunk_map.remove(&pos).unwrap();
                if holder.chunk.is_some() {
                    warn!(
                        "receive_chunk(IO): holder already has chunk at {:?}; replacing",
                        pos
                    );
                }
                debug_assert_eq!(holder.current_stage, StagedChunkEnum::None);

                for i in (holder.current_stage as usize + 1)..=(chunk.get_stage_id() as usize) {
                    self.drop_node(holder.tasks[i]);
                    holder.tasks[i] = NodeKey::null();
                }
                holder.current_stage = StagedChunkEnum::from(chunk.get_stage_id());
                debug_assert!(self.graph.nodes.contains_key(holder.occupied));
                self.drop_node(holder.occupied);
                holder.occupied = NodeKey::null();

                match &chunk {
                    Chunk::Level(data) => {
                        // Full chunk from IO - mark as public and notify
                        self.apply_lighting_override(data);
                        let result = self.public_chunk_map.insert(pos, data.clone());
                        if result.is_some() {
                            warn!(
                                "receive_chunk(IO): replacing existing public chunk at {:?}",
                                pos
                            );
                        }
                        holder.public = true;
                        trace!(
                            "Notifying players: chunk {:?} loaded from disk (Full status)",
                            pos
                        );
                        self.listener.process_new_chunk(pos, data);
                    }
                    Chunk::Proto(_) => {
                        // Proto chunk from IO (downgraded for relighting) - mark as non-public
                        // so players get notified when it finishes generation
                        if holder.public {
                            debug!(
                                "Chunk {:?} downgraded to Proto for relighting, marking as non-public",
                                pos
                            );
                            self.public_chunk_map.remove(&pos);
                            holder.public = false;
                        }
                    }
                }
                holder.chunk = Some(chunk);
                self.chunk_map.insert(pos, holder);
            }
            RecvChunk::Generation(data) => {
                // debug!("receive gen chunk pos {pos:?}");
                let mut dx = 0;
                let mut dy = 0;
                for chunk in data.chunks {
                    let new_pos = ChunkPos::new(data.x + dx, data.z + dy);
                    match chunk {
                        Chunk::Level(chunk) => {
                            let mut holder = self.chunk_map.remove(&new_pos).unwrap();
                            if new_pos == pos {
                                // Expect the holder to be one stage before Full (Lighting). If not,
                                // log and align so we don't panic in production/debug runs.
                                if holder.current_stage != StagedChunkEnum::Lighting {
                                    warn!(
                                        "receive_chunk(Level): holder at {:?} for pos {:?} expected {:?}; aligning",
                                        holder.current_stage,
                                        new_pos,
                                        StagedChunkEnum::Lighting
                                    );
                                    holder.current_stage = StagedChunkEnum::Lighting;
                                }
                                self.drop_node(holder.tasks[StagedChunkEnum::Full as usize]);
                                holder.tasks[StagedChunkEnum::Full as usize] = NodeKey::null();
                                if self.graph.nodes.contains_key(holder.occupied) {
                                    self.drop_node(holder.occupied);
                                }
                                holder.current_stage = StagedChunkEnum::Full;

                                // Check if this is the first time becoming public
                                let was_public = holder.public;

                                if !was_public {
                                    self.apply_lighting_override(&chunk);
                                    // Clone once for public_chunk_map (needed for player access)
                                    let public_chunk = chunk.clone();
                                    holder.chunk = Some(Chunk::Level(chunk));
                                    let result =
                                        self.public_chunk_map.insert(new_pos, public_chunk);
                                    holder.public = true;
                                    if result.is_some() {
                                        warn!(
                                            "public_chunk_map.insert returned existing chunk for {new_pos:?}"
                                        );
                                    }
                                    // Notify players about the new chunk
                                    if let Some(pc) = self.public_chunk_map.get(&new_pos) {
                                        trace!(
                                            "Notifying players: new chunk at {:?} (generation complete)",
                                            new_pos
                                        );
                                        self.listener.process_new_chunk(new_pos, &pc);
                                    } else {
                                        error!(
                                            "CRITICAL: Failed to retrieve chunk {:?} from public_chunk_map immediately after insert!",
                                            new_pos
                                        );
                                    }
                                } else {
                                    // Was already public but chunk has been regenerated (e.g., after relighting)
                                    // Update the chunk data and re-notify players
                                    self.apply_lighting_override(&chunk);
                                    holder.chunk = Some(Chunk::Level(chunk.clone()));
                                    self.public_chunk_map.insert(new_pos, chunk.clone());
                                    info!(
                                        "Notifying players: regenerated chunk at {:?} (was already public)",
                                        new_pos
                                    );
                                    self.listener.process_new_chunk(new_pos, &chunk);
                                }
                            } else {
                                // Non-center chunk... just restore it to holder without cloning
                                holder.chunk = Some(Chunk::Level(chunk));
                            }

                            // Always drop the occupied node before nulling it
                            if !holder.occupied.is_null()
                                && self.graph.nodes.contains_key(holder.occupied)
                            {
                                self.drop_node(holder.occupied);
                            }
                            holder.occupied = NodeKey::null();
                            self.chunk_map.insert(new_pos, holder);
                        }
                        Chunk::Proto(chunk) => {
                            let mut holder = self.chunk_map.remove(&new_pos).unwrap();

                            // Clean up the task node for the stage that just finished.
                            let stage = chunk.stage_id();
                            if stage < holder.tasks.len() as u8 {
                                let task_idx = stage as usize;
                                if !holder.tasks[task_idx].is_null() {
                                    self.drop_node(holder.tasks[task_idx]);
                                    holder.tasks[task_idx] = NodeKey::null();
                                }
                            }

                            if new_pos == pos {
                                // Center Chunk Logic
                                debug_assert_ne!(holder.current_stage, StagedChunkEnum::None);
                                if self.graph.nodes.contains_key(holder.occupied) {
                                    self.drop_node(holder.occupied);
                                }
                                holder.current_stage = StagedChunkEnum::from(stage);
                            } else {
                                // Neighbor Logic
                                if holder.current_stage < StagedChunkEnum::from(stage) {
                                    holder.current_stage = StagedChunkEnum::from(stage);
                                }

                                if !holder.occupied.is_null()
                                    && self.graph.nodes.contains_key(holder.occupied)
                                {
                                    self.drop_node(holder.occupied);
                                }
                            }

                            holder.occupied = NodeKey::null();
                            holder.chunk = Some(Chunk::Proto(chunk));
                            self.chunk_map.insert(new_pos, holder);
                        }
                    }
                    dy += 1;
                    if dy == data.size {
                        dy = 0;
                        dx += 1;
                    }
                }
            }
            RecvChunk::GenerationFailure {
                pos: fail_pos,
                stage,
                error,
            } => {
                error!(
                    "Received generation failure notification for chunk {:?} at stage {:?}: {}",
                    fail_pos, stage, error
                );

                // Clean up the holder
                if let Some(mut holder) = self.chunk_map.remove(&pos) {
                    let target_stage = holder.target_stage;

                    // Clean up occupied node
                    if !holder.occupied.is_null() {
                        if self.graph.nodes.contains_key(holder.occupied) {
                            self.drop_node(holder.occupied);
                        }
                        holder.occupied = NodeKey::null();
                    }

                    // Clean up all task nodes
                    for i in 0..holder.tasks.len() {
                        if !holder.tasks[i].is_null() {
                            self.drop_node(holder.tasks[i]);
                            holder.tasks[i] = NodeKey::null();
                        }
                    }

                    // Reset chunk to None so it will be regenerated
                    holder.current_stage = StagedChunkEnum::None;
                    holder.chunk = None;

                    // Recreate tasks from None to target_stage
                    for i in (StagedChunkEnum::None as usize + 1)..=(target_stage as usize) {
                        let stage_enum = StagedChunkEnum::from(i as u8);
                        let task_node = Node::new(pos, stage_enum);
                        holder.tasks[i] = self.graph.nodes.insert(task_node);

                        // Add dependencies
                        if i > (StagedChunkEnum::None as usize + 1) {
                            self.graph.add_edge(holder.tasks[i - 1], holder.tasks[i]);
                        }
                    }

                    // Add first task to queue with high priority (use a boost to retry faster)
                    if target_stage > StagedChunkEnum::None {
                        let first_task = holder.tasks[StagedChunkEnum::None as usize + 1];
                        if let Some(node) = self.graph.nodes.get_mut(first_task) {
                            node.in_queue = true;
                        }
                        self.queue.push(TaskHeapNode(
                            Self::calc_priority(
                                &self.last_level,
                                &self.last_high_priority,
                                pos,
                                StagedChunkEnum::from(1),
                            ) - 50, // Priority boost for retry
                            first_task,
                        ));
                    }

                    self.chunk_map.insert(pos, holder);

                    warn!(
                        "Chunk {:?} reset to None and re-queued for regeneration (target: {:?})",
                        pos, target_stage
                    );
                } else {
                    error!("Failed to find holder for failed chunk {:?}", pos);
                }
            }
        }
        self.running_task_count -= 1;
    }

    fn work(mut self, level: Arc<Level>) {
        debug!(
            "schedule thread start id: {:?} name: {}",
            thread::current().id(),
            thread::current().name().unwrap_or("unknown")
        );
        // let mut clock = Instant::now();
        loop {
            if level.should_unload.swap(false, Relaxed) {
                self.unload_chunk();
            }
            if level.should_save.swap(false, Relaxed) {
                self.save_all_chunk(false);
            }
            if level.shut_down_chunk_system.load(Relaxed) {
                // Save all chunks BEFORE breaking the loop to ensure IO write thread processes them
                info!("Saving chunks before shutdown...");
                self.save_all_chunk(true);
                break;
            }

            'out2: while let Some(task) = self.queue.pop() {
                // Check shutdown flag again before processing tasks to avoid IO errors
                if level.shut_down_chunk_system.load(Relaxed) {
                    // Don't process any more tasks, just break to save chunks
                    self.queue.push(task);
                    info!("Shutdown detected during task processing, saving chunks...");
                    self.save_all_chunk(true);
                    break 'out2;
                }

                if self.resort_work(self.send_level.get()) {
                    self.queue.push(task);
                    break 'out2;
                }
                while let Ok((pos, data)) = self.recv_chunk.try_recv() {
                    self.receive_chunk(pos, data);
                }
                if let Some(node) = self.graph.nodes.get_mut(task.1) {
                    if node.in_degree != 0 {
                        node.in_queue = false;
                        continue;
                    }
                    let node = node.clone();
                    if node.stage == StagedChunkEnum::Empty {
                        self.running_task_count += 1;
                        let holder = self.chunk_map.get_mut(&node.pos).unwrap();
                        debug_assert!(holder.occupied.is_null());
                        debug_assert_eq!(holder.current_stage, StagedChunkEnum::None);
                        let occupy = self.graph.nodes.insert(Node::new(
                            ChunkPos::new(i32::MAX, i32::MAX),
                            StagedChunkEnum::None,
                        ));
                        for i in
                            (holder.current_stage as usize + 1)..=(holder.target_stage as usize)
                        {
                            self.graph.add_edge(occupy, holder.tasks[i]);
                        }
                        holder.occupied = occupy;

                        // debug!("send task {:?} {node:?}", task.1);

                        if self.io_read.send(node.pos).is_err() {
                            // IO thread closed (likely due to shutdown), save and exit cleanly
                            info!("IO read thread closed, saving remaining chunks...");
                            self.save_all_chunk(true);
                            break 'out2;
                        }
                    } else {
                        let write_radius = node.stage.get_write_radius();
                        let mut cache = Cache::new(
                            node.pos.x - write_radius,
                            node.pos.y - write_radius,
                            write_radius << 1 | 1,
                        );

                        // Only create the occupy node after we know all chunks are available
                        // and we're actually proceeding with the task
                        let occupy = self.graph.nodes.insert(Node::new(
                            ChunkPos::new(i32::MAX, i32::MAX),
                            StagedChunkEnum::None,
                        ));

                        for dx in -write_radius..=write_radius {
                            for dy in -write_radius..=write_radius {
                                let new_pos = node.pos.add_raw(dx, dy);
                                let holder = self.chunk_map.get_mut(&new_pos).unwrap();
                                let mut tmp = None;
                                swap(&mut tmp, &mut holder.chunk);
                                let tmp = match tmp {
                                    Some(v) => v,
                                    None => panic!(
                                        "Missing chunk for position {:?} while processing generation task for {:?} stage {:?}",
                                        new_pos, node.pos, node.stage
                                    ),
                                };
                                match tmp {
                                    Chunk::Level(chunk) => {
                                        // Don't clone Level chunks. Move them into cache to avoid
                                        // extra Arc references that prevent unloading.
                                        cache.chunks.push(Chunk::Level(chunk));
                                    }
                                    Chunk::Proto(chunk) => {
                                        cache.chunks.push(Chunk::Proto(chunk));
                                    }
                                }

                                debug_assert!(holder.occupied.is_null());

                                let mut cur_edge = holder.occupied_by;
                                let mut prev_edge = EdgeKey::null();
                                let mut change_head = None;
                                while !cur_edge.is_null() {
                                    let edge = self.graph.edges.get(cur_edge).unwrap();
                                    if self.graph.nodes.contains_key(edge.to) {
                                        prev_edge = cur_edge;
                                        cur_edge = edge.next;
                                        self.graph.add_edge(occupy, edge.to);
                                    } else {
                                        let next = edge.next;
                                        self.graph.edges.remove(cur_edge);
                                        cur_edge = next;
                                        if prev_edge.is_null() {
                                            change_head = Some(next);
                                        } else {
                                            self.graph.edges.get_mut(prev_edge).unwrap().next =
                                                next;
                                        }
                                    }
                                }
                                if let Some(next) = change_head {
                                    holder.occupied_by = next;
                                }

                                holder.occupied = occupy;
                            }
                        }

                        // debug!("send task {:?} {node:?}", task.1);

                        self.running_task_count += 1;
                        if self.generate.send((node.pos, cache, node.stage)).is_err() {
                            // revert running task count increment and exit cleanly with save
                            self.running_task_count = self.running_task_count.saturating_sub(1);
                            info!("Generation thread closed, saving remaining chunks...");
                            self.save_all_chunk(true);
                            break 'out2;
                        }
                    }
                }
            }

            if self.queue.is_empty() {
                // debug!("the queue is empty. thread sleep");
                while self.running_task_count > 0 && self.queue.is_empty() {
                    match self.recv_chunk.try_recv() {
                        Ok((pos, data)) => {
                            self.receive_chunk(pos, data);
                            self.resort_work(self.send_level.get());
                        }
                        Err(_) => {
                            if level.shut_down_chunk_system.load(Relaxed) {
                                break;
                            }
                            thread::sleep(Duration::from_millis(50));
                        }
                    }
                }
                if self.queue.is_empty() {
                    // debug!("no work to do. thread sleep");
                    debug_assert!(self.debug_check());
                    debug_assert_eq!(self.running_task_count, 0);
                    self.resort_work(self.send_level.wait_and_get(&level));
                }
            }
        }
        info!(
            "schedule: waiting for {} generation tasks to finish",
            self.running_task_count
        );
        let mut wait_iterations = 0;
        let max_wait_iterations = 100; // 5 seconds max wait
        while self.running_task_count > 0 && wait_iterations < max_wait_iterations {
            match self.recv_chunk.try_recv() {
                Ok((pos, data)) => {
                    self.receive_chunk(pos, data);
                    wait_iterations = 0; // Reset counter when we receive data
                }
                Err(_) => {
                    wait_iterations += 1;
                    if wait_iterations % 20 == 0 {
                        warn!(
                            "Still waiting for {} tasks to complete (waited {}ms)",
                            self.running_task_count,
                            wait_iterations * 50
                        );
                    }
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }

        if self.running_task_count > 0 {
            warn!(
                "Cancelling {} in-flight generation tasks",
                self.running_task_count
            );
            // Collect nodes to drop
            let mut nodes_to_drop = Vec::new();

            // Clear occupy nodes for cancelled tasks
            for holder in self.chunk_map.values_mut() {
                // Collect any task nodes associated with this holder to ensure full cleanup
                for task in &mut holder.tasks {
                    if !task.is_null() {
                        nodes_to_drop.push(*task);
                        *task = NodeKey::null();
                    }
                }

                if !holder.occupied.is_null()
                    && let Some(node) = self.graph.nodes.get(holder.occupied)
                    && node.pos.x == i32::MAX
                    && node.pos.y == i32::MAX
                {
                    // This is an occupy node for an in-flight task
                    nodes_to_drop.push(holder.occupied);
                    holder.occupied = NodeKey::null();
                }
            }

            // Safely drop them (using `drop_node`, which decrements in_degree on targets)
            for node_key in nodes_to_drop {
                self.drop_node(node_key);
            }

            self.running_task_count = 0;
        }

        // Drop the io_write sender to signal the write thread to exit
        // Chunks were already saved during loop exit above
        drop(self.io_write);

        // Clean up any remaining graph structures
        let unreleased_count = self.graph.nodes.len();
        if unreleased_count > 0 {
            warn!(
                "Cleaning up {} unreleased nodes from incomplete tasks",
                unreleased_count
            );
        }
        self.graph.edges.clear();
    }

    fn debug_check(&self) -> bool {
        if !self.graph.nodes.is_empty() {
            // error!("nodes: {:?}", self.graph.nodes);
            for (key, value) in &self.graph.nodes {
                error!("unrelease node {key:?}: {value:?}");
            }
            panic!("nodes count error");
        }
        for (pos, holder) in &self.chunk_map {
            for i in &holder.tasks {
                debug_assert!(i.is_null());
            }
            debug_assert_eq!(
                holder.target_stage,
                StagedChunkEnum::level_to_stage(
                    *self.last_level.get(pos).unwrap_or(&ChunkLoading::MAX_LEVEL)
                )
            );
            debug_assert!(holder.current_stage >= holder.target_stage);
            debug_assert!(holder.occupied.is_null());
            if holder.current_stage != StagedChunkEnum::None {
                debug_assert_eq!(
                    holder.chunk.as_ref().unwrap().get_stage_id(),
                    holder.current_stage as u8
                );
            }
        }
        true
    }
}
