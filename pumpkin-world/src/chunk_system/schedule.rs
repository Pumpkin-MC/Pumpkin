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
use crate::level::{Level, SyncChunk};
use dashmap::DashMap;
use log::error;
use num_traits::abs;
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

struct TaskHeapNode(i8, NodeKey);
impl PartialEq for TaskHeapNode {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
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

        let (send_write_io, recv_write_io) = crossfire::compat::spsc::unbounded_async();

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
                .name(format!("Gen-{}", i)) // Identifying dim helps debugging
                .spawn(move || {
                    generation_work(recv_gen, send_chunk, level_clone);
                })
                .expect("Failed to spawn Generation Thread");

            thread_tracker.push(handle);
        }

        let level_sched = level.clone();
        let lighting_config = level.lighting_config;
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
                let mut chunk = chunk.blocking_write();
                for section in chunk.light_engine.block_light.iter_mut() {
                    section.fill(15);
                }
                for section in chunk.light_engine.sky_light.iter_mut() {
                    section.fill(15);
                }
            }
            LightingEngineConfig::Dark => {
                let mut chunk = chunk.blocking_write();
                for section in chunk.light_engine.block_light.iter_mut() {
                    section.fill(0);
                }
                for section in chunk.light_engine.sky_light.iter_mut() {
                    section.fill(0);
                }
            }
            _ => {}
        }
    }

    fn calc_priority(
        last_level: &ChunkLevel,
        last_high_priority: &Vec<ChunkPos>,
        pos: ChunkPos,
        stage: StagedChunkEnum,
    ) -> i8 {
        if last_high_priority.is_empty() {
            return *last_level.get(&pos).unwrap_or(&ChunkLoading::MAX_LEVEL) + (stage as i8);
        }
        for i in last_high_priority {
            let dst = max(abs(i.x - pos.x), abs(i.y - pos.y));
            if dst <= StagedChunkEnum::FULL_RADIUS
                && stage <= StagedChunkEnum::FULL_DEPENDENCIES[dst as usize]
            {
                return *last_level.get(&pos).unwrap_or(&ChunkLoading::MAX_LEVEL) + (stage as i8) - 100;
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

    fn resort_work(&mut self, new_data: (Option<LevelChange>, Option<Vec<ChunkPos>>)) -> bool {
        // true -> updated | false -> not update
        if new_data.0.is_none() && new_data.1.is_none() {
            return false;
        }
        if let Some(high_priority) = new_data.1 {
            // log::debug!("receive new priority");
            self.last_high_priority = high_priority;
        }
        let Some(new_level) = new_data.0 else {
            self.sort_queue();
            return true;
        };
        // log::debug!("receive new level");
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
                    self.graph.fast_drop_node(*task);
                    *task = NodeKey::null();
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
                                    // TODO
                                    holder.occupied_by = self.graph.edges.insert(
                                        crate::chunk_system::dag::Edge::new(
                                            task,
                                            holder.occupied_by,
                                        ),
                                    );
                                    if holder.current_stage >= req_stage {
                                        continue;
                                    }
                                    let ano_task = &mut holder.tasks[req_stage as usize];
                                    if ano_task.is_null() {
                                        *ano_task =
                                            self.graph.nodes.insert(Node::new(new_pos, req_stage));
                                        
                                        // Ensure the implicitly created task is queued
                                        let node = self.graph.nodes.get_mut(*ano_task).unwrap();
                                        node.in_queue = true;
                                        self.queue.push(TaskHeapNode(
                                            Self::calc_priority(
                                                &self.last_level,
                                                &self.last_high_priority,
                                                new_pos,
                                                req_stage,
                                            ),
                                            *ano_task,
                                        ));
                                    }
                                    self.graph.add_edge(*ano_task, task); // task depend on ano_task
                                    continue;
                                }
                                let ano_chunk = self.chunk_map.entry(new_pos).or_default();
                                ano_chunk.occupied_by =
                                    self.graph.edges.insert(crate::chunk_system::dag::Edge::new(
                                        task,
                                        ano_chunk.occupied_by,
                                    ));

                                if !ano_chunk.occupied.is_null() {
                                    self.graph.add_edge(ano_chunk.occupied, task);
                                }

                                if ano_chunk.current_stage >= req_stage {
                                    continue;
                                }
                                let ano_task = &mut ano_chunk.tasks[req_stage as usize];
                                if ano_task.is_null() {
                                    *ano_task =
                                        self.graph.nodes.insert(Node::new(new_pos, req_stage));

                                    // Ensure the implicitly created task is queued
                                    let node = self.graph.nodes.get_mut(*ano_task).unwrap();
                                    node.in_queue = true;
                                    self.queue.push(TaskHeapNode(
                                        Self::calc_priority(
                                            &self.last_level,
                                            &self.last_high_priority,
                                            new_pos,
                                            req_stage,
                                        ),
                                        *ano_task,
                                    ));
                                }
                                self.graph.add_edge(*ano_task, task); // task depend on ano_task
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
                        if Arc::strong_count(&chunk) == 1 {
                            // log::debug!("unload chunk {pos:?} to file");
                            chunks.push((pos, Chunk::Level(chunk)));
                            self.chunk_map.remove(&pos);
                        } else {
                            // log::debug!(
                            //     "chunk {pos:?} is still used somewhere. it can't be unloaded"
                            // );
                            self.unload_chunks.insert(pos);
                            holder.chunk = Some(Chunk::Level(chunk));
                        }
                    }
                    Chunk::Proto(chunk) => {
                        debug_assert!(!holder.public);
                        // log::debug!("unload proto chunk {pos:?} to file");
                        chunks.push((pos, Chunk::Proto(chunk)));
                        self.chunk_map.remove(&pos);
                    }
                }
            }
        }
        // log::debug!("send {} unloaded chunks to io write", chunks.len());
        if chunks.is_empty() {
            return;
        }
        let mut data = self.io_lock.0.lock().unwrap();
        for (pos, _chunk) in &chunks {
            *data.entry(*pos).or_insert(0) += 1;
        }
        drop(data);
        if let Err(e) = self.io_write.send(chunks) {
            log::error!("Failed to send chunks to io write thread during save (may have shut down): {:?}", e);
        }
    }

    fn save_all_chunk(&self, save_proto_chunk: bool) {
        let mut chunks = Vec::with_capacity(self.chunk_map.len());
        for (pos, holder) in &self.chunk_map {
            if let Some(chunk) = &holder.chunk {
                match chunk {
                    Chunk::Level(sync_chunk) => {
                        // Only save level chunks that are marked dirty
                        let dirty = sync_chunk.blocking_read().dirty;
                        if dirty {
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
        log::info!("Saving {} chunks...", chunks.len());
        let mut data = self.io_lock.0.lock().unwrap();
        for (pos, _chunk) in &chunks {
            *data.entry(*pos).or_insert(0) += 1;
        }
        drop(data);
        if let Err(e) = self.io_write.send(chunks) {
            log::error!("Failed to send chunks to io write thread during unload (may have shut down): {:?}", e);
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
                debug_assert!(holder.chunk.is_none());
                debug_assert_eq!(holder.current_stage, StagedChunkEnum::None);

                for i in (holder.current_stage as usize + 1)..=(chunk.get_stage_id() as usize) {
                    self.drop_node(holder.tasks[i]);
                    holder.tasks[i] = NodeKey::null();
                }
                holder.current_stage = StagedChunkEnum::from(chunk.get_stage_id());
                debug_assert!(self.graph.nodes.contains_key(holder.occupied));
                self.drop_node(holder.occupied);
                holder.occupied = NodeKey::null();

                debug_assert!(!holder.public);
                match &chunk {
                    Chunk::Level(data) => {
                        self.apply_lighting_override(data);
                        let result = self.public_chunk_map.insert(pos, data.clone());
                        debug_assert!(result.is_none());
                        holder.public = true;
                        self.listener.process_new_chunk(pos, data);
                    }
                    Chunk::Proto(_) => {}
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
                                    log::warn!(
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

                                holder.chunk = Some(Chunk::Level(chunk.clone()));
                                if !holder.public {
                                    self.apply_lighting_override(&chunk);
                                    let result = self.public_chunk_map.insert(new_pos, chunk.clone());
                                    holder.public = true;
                                    if result.is_some() {
                                        log::warn!("public_chunk_map.insert returned existing chunk for {new_pos:?}");
                                    }
                                    self.listener.process_new_chunk(new_pos, &chunk);
                                }
                            }

                            holder.occupied = NodeKey::null();
                            self.chunk_map.insert(new_pos, holder);
                        }
                        Chunk::Proto(chunk) => {
                            let mut holder = self.chunk_map.remove(&new_pos).unwrap();
                            debug_assert!(holder.chunk.is_none());
                            let expected_stage = if new_pos == pos {
                                chunk.stage_id().saturating_sub(1)
                            } else {
                                chunk.stage_id()
                            };
                            if holder.current_stage as u8 != expected_stage {
                                log::warn!(
                                    "receive_chunk: stage mismatch at {:?}: holder {} expected {}",
                                    new_pos,
                                    holder.current_stage as u8,
                                    expected_stage
                                );
                                // Align holder to the expected stage to continue safely.
                                holder.current_stage = StagedChunkEnum::from(expected_stage);
                            }

                            if new_pos == pos {
                                debug_assert_ne!(holder.current_stage, StagedChunkEnum::None);
                                let stage = chunk.stage_id();
                                self.drop_node(holder.tasks[stage as usize]);
                                holder.tasks[stage as usize] = NodeKey::null();
                                debug_assert!(self.graph.nodes.contains_key(holder.occupied));
                                self.drop_node(holder.occupied);
                                holder.current_stage = StagedChunkEnum::from(stage);
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
        }
        self.running_task_count -= 1;
    }

    fn work(mut self, level: Arc<Level>) {
        log::debug!(
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
                log::info!("Saving chunks before shutdown...");
                self.save_all_chunk(true);
                break;
            }

            'out2: while let Some(task) = self.queue.pop() {
                // Check shutdown flag again before processing tasks to avoid IO errors
                if level.shut_down_chunk_system.load(Relaxed) {
                    // Don't process any more tasks, just break to save chunks
                    self.queue.push(task);
                    log::info!("Shutdown detected during task processing, saving chunks...");
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
                            log::info!("IO read thread closed, saving remaining chunks...");
                            self.save_all_chunk(true);
                            break 'out2;
                        }
                    } else {
                        let occupy = self.graph.nodes.insert(Node::new(
                            ChunkPos::new(i32::MAX, i32::MAX),
                            StagedChunkEnum::None,
                        ));
                        let write_radius = node.stage.get_write_radius();
                        let mut cache = Cache::new(
                            node.pos.x - write_radius,
                            node.pos.y - write_radius,
                            write_radius << 1 | 1,
                        );
                        // Ensure all direct dependencies are satisfied before executing.
                        // If not ready, requeue the task so dependencies can finish first.
                        {
                            let dp = node.stage.get_direct_dependencies();
                            let r = node.stage.get_direct_radius();
                            let mut ready = true;
                            // Ensure entries exist first to avoid mutable/immutable borrow conflicts
                            for dx in -r..=r {
                                for dy in -r..=r {
                                    let new_pos = node.pos.add_raw(dx, dy);
                                    if !self.chunk_map.contains_key(&new_pos) {
                                        self.chunk_map.insert(new_pos, ChunkHolder::default());
                                    }
                                }
                            }

                            for dx in -r..=r {
                                for dy in -r..=r {
                                    let new_pos = node.pos.add_raw(dx, dy);
                                    let holder = self.chunk_map.get(&new_pos).unwrap();
                                    let dst = dy.abs().max(dx.abs());
                                    if holder.current_stage < dp[dst as usize] {
                                        ready = false;
                                        break;
                                    }
                                }
                                if !ready {
                                    break;
                                }
                            }
                            if !ready {
                                // requeue this task for later
                                if let Some(n) = self.graph.nodes.get_mut(task.1) {
                                    n.in_queue = true;
                                }
                                self.queue.push(TaskHeapNode(
                                    Self::calc_priority(
                                        &self.last_level,
                                        &self.last_high_priority,
                                        node.pos,
                                        node.stage,
                                    ),
                                    task.1,
                                ));
                                continue;
                            }
                        }

                        for dx in -write_radius..=write_radius {
                            for dy in -write_radius..=write_radius {
                                let new_pos = node.pos.add_raw(dx, dy);
                                let holder = self.chunk_map.get_mut(&new_pos).unwrap();
                                let mut tmp = None;
                                swap(&mut tmp, &mut holder.chunk);
                                match tmp.unwrap() {
                                    Chunk::Level(chunk) => {
                                        cache.chunks.push(Chunk::Level(chunk.clone()));
                                        holder.chunk = Some(Chunk::Level(chunk));
                                    }
                                    Chunk::Proto(chunk) => cache.chunks.push(Chunk::Proto(chunk)),
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
                            log::info!("Generation thread closed, saving remaining chunks...");
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
        log::info!("schedule: waiting for {} generation tasks to finish", self.running_task_count);
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
                        log::warn!("Still waiting for {} tasks to complete (waited {}ms)", self.running_task_count, wait_iterations * 50);
                    }
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }
        
        if self.running_task_count > 0 {
            log::warn!("Cancelling {} in-flight generation tasks", self.running_task_count);
            // Clear occupy nodes for cancelled tasks
            for (_pos, holder) in &mut self.chunk_map {
                if !holder.occupied.is_null() {
                    if let Some(node) = self.graph.nodes.get(holder.occupied) {
                        if node.pos.x == i32::MAX && node.pos.y == i32::MAX {
                            // This is an occupy node for an in-flight task
                            self.graph.nodes.remove(holder.occupied);
                            holder.occupied = NodeKey::null();
                        }
                    }
                }
            }
            self.running_task_count = 0;
        }
        
        // Drop the io_write sender to signal the write thread to exit
        // Chunks were already saved during loop exit above
        drop(self.io_write);
        
        // Clean up any remaining graph structures
        if !self.graph.nodes.is_empty() {
            log::warn!("Cleaning up {} unreleased nodes from incomplete tasks", self.graph.nodes.len());
            self.graph.nodes.clear();
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
