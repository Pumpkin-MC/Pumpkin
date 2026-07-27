use super::{CHUNKS_GEN_FULL, CHUNKS_LOADED_DISK, GenerationSchedule, TaskHeapNode};
use crate::chunk_system::chunk_state::{Chunk, StagedChunkEnum};
use crate::chunk_system::dag::{EdgeKey, Node, NodeKey};
use crate::chunk_system::generation_cache::Cache;
use crate::chunk_system::worker_logic::RecvChunk;
use crate::chunk_system::{ChunkLoading, ChunkPos};
use crate::level::Level;
use pumpkin_data::chunk_gen_settings::GenerationSettings;
use slotmap::Key;
use std::mem::swap;
use std::sync::Arc;
use std::sync::atomic::Ordering as AtomicOrdering;
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, trace, warn};

impl GenerationSchedule {
    #[expect(clippy::too_many_lines)]
    fn receive_chunk(&mut self, pos: ChunkPos, data: RecvChunk) {
        match data {
            RecvChunk::IO(chunk) => {
                let mut holder = self.chunk_map.remove(&pos).unwrap();
                if holder.chunk.is_some() {
                    warn!(
                        "receive_chunk(IO): holder already has chunk at {:?}; replacing",
                        pos
                    );
                }
                debug_assert_eq!(holder.current_stage, StagedChunkEnum::None);

                let stage = StagedChunkEnum::from(chunk.get_stage_id());
                self.drop_satisfied_tasks(&mut holder, stage);
                holder.current_stage = stage;
                debug_assert!(self.graph.nodes.contains_key(holder.occupied));
                self.drop_node(holder.occupied);
                holder.occupied = NodeKey::null();

                match &chunk {
                    Chunk::Level(data) => {
                        self.apply_lighting_override(data);
                        let result = self.public_chunk_map.insert(pos, data.clone());
                        if result.is_some() {
                            warn!(
                                "receive_chunk(IO): replacing existing public chunk at {:?}",
                                pos
                            );
                        }
                        holder.public = true;
                        if pumpkin_config::development_mode() {
                            let n = CHUNKS_LOADED_DISK.fetch_add(1, AtomicOrdering::Relaxed) + 1;
                            if n == 1 || n.is_multiple_of(32) {
                                info!(
                                    "Chunk load (disk): {:?} ({} full chunks loaded this run)",
                                    pos, n
                                );
                            } else {
                                debug!(
                                    "Notifying players: chunk {:?} loaded from disk (Full status)",
                                    pos
                                );
                            }
                        } else {
                            trace!(
                                "Notifying players: chunk {:?} loaded from disk (Full status)",
                                pos
                            );
                        }
                        self.listener.process_new_chunk(pos, data);
                    }
                    Chunk::Proto(_) => {
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

                // A new chunk arrived — unblock any waiting generation tasks
                self.check_waiting_tasks();
            }
            RecvChunk::Generation(data) => {
                let mut dx = 0;
                let mut dy = 0;
                for chunk in data.chunks {
                    let new_pos = ChunkPos::new(data.x + dx, data.z + dy);
                    match chunk {
                        Chunk::Level(chunk) => {
                            let mut holder = self.chunk_map.remove(&new_pos).unwrap();
                            let stage = StagedChunkEnum::Full;
                            if new_pos == pos {
                                if holder.current_stage != StagedChunkEnum::Spawn {
                                    warn!(
                                        "receive_chunk(Level): holder at {:?} for pos {:?} expected {:?}; aligning",
                                        holder.current_stage,
                                        new_pos,
                                        StagedChunkEnum::Spawn
                                    );
                                    holder.current_stage = StagedChunkEnum::Spawn;
                                }
                                self.drop_satisfied_tasks(&mut holder, stage);
                                if self.graph.nodes.contains_key(holder.occupied) {
                                    self.drop_node(holder.occupied);
                                }
                                holder.current_stage = stage;

                                let was_public = holder.public;
                                self.apply_lighting_override(&chunk);
                                let public_chunk = chunk.clone();
                                if was_public {
                                    self.public_chunk_map.insert(new_pos, public_chunk);
                                    info!(
                                        "Notifying players: regenerated chunk at {:?} (was already public)",
                                        new_pos
                                    );
                                    self.listener.process_new_chunk(new_pos, &chunk);
                                    holder.chunk = Some(Chunk::Level(chunk));
                                } else {
                                    holder.chunk = Some(Chunk::Level(chunk));
                                    let result =
                                        self.public_chunk_map.insert(new_pos, public_chunk);
                                    holder.public = true;
                                    if result.is_some() {
                                        warn!(
                                            "public_chunk_map.insert returned existing chunk for {new_pos:?}"
                                        );
                                    }
                                    if let Some(pc) = self.public_chunk_map.get(&new_pos) {
                                        if pumpkin_config::development_mode() {
                                            let n = CHUNKS_GEN_FULL
                                                .fetch_add(1, AtomicOrdering::Relaxed)
                                                + 1;
                                            if n == 1 || n.is_multiple_of(16) {
                                                info!(
                                                    "Terrain ready: new full chunk {:?} ({} generated this run)",
                                                    new_pos, n
                                                );
                                            } else {
                                                debug!(
                                                    "Notifying players: new chunk at {:?} (generation complete)",
                                                    new_pos
                                                );
                                            }
                                        } else {
                                            trace!(
                                                "Notifying players: new chunk at {:?} (generation complete)",
                                                new_pos
                                            );
                                        }
                                        self.listener.process_new_chunk(new_pos, &pc);
                                    } else {
                                        error!(
                                            "CRITICAL: Failed to retrieve chunk {:?} from public_chunk_map immediately after insert!",
                                            new_pos
                                        );
                                    }
                                }
                            } else {
                                self.drop_satisfied_tasks(&mut holder, stage);
                                holder.current_stage = stage;
                                holder.chunk = Some(Chunk::Level(chunk));
                            }

                            if !holder.occupied.is_null()
                                && self.graph.nodes.contains_key(holder.occupied)
                            {
                                self.drop_node(holder.occupied);
                            }
                            holder.occupied = NodeKey::null();

                            // If this chunk was only loaded for a dependency or cancelled
                            // and is no longer needed, clear dependency_stage and queue unload.
                            if holder.target_stage == StagedChunkEnum::None
                                && holder.current_stage >= holder.dependency_stage
                            {
                                holder.dependency_stage = StagedChunkEnum::None;
                                self.unload_chunks.insert(new_pos);
                            }

                            self.chunk_map.insert(new_pos, holder);
                        }
                        Chunk::Proto(chunk) => {
                            let mut holder = self.chunk_map.remove(&new_pos).unwrap();

                            let stage = StagedChunkEnum::from(chunk.stage_id());
                            self.drop_satisfied_tasks(&mut holder, stage);

                            if new_pos == pos {
                                debug_assert_ne!(holder.current_stage, StagedChunkEnum::None);
                                if self.graph.nodes.contains_key(holder.occupied) {
                                    self.drop_node(holder.occupied);
                                }
                                holder.current_stage = stage;
                            } else {
                                if holder.current_stage < stage {
                                    holder.current_stage = stage;
                                }
                                if !holder.occupied.is_null()
                                    && self.graph.nodes.contains_key(holder.occupied)
                                {
                                    self.drop_node(holder.occupied);
                                }
                            }

                            // Clear dependency_stage and queue unload if no longer needed
                            if holder.target_stage == StagedChunkEnum::None
                                && holder.current_stage >= holder.dependency_stage
                            {
                                holder.dependency_stage = StagedChunkEnum::None;
                                self.unload_chunks.insert(new_pos);
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

                // Neighbor chunks returned to holders — unblock waiting tasks
                self.check_waiting_tasks();
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

                if let Some(mut holder) = self.chunk_map.remove(&pos) {
                    let target_stage = holder.target_stage;

                    if !holder.occupied.is_null() {
                        if self.graph.nodes.contains_key(holder.occupied) {
                            self.drop_node(holder.occupied);
                        }
                        holder.occupied = NodeKey::null();
                    }

                    for i in 0..holder.tasks.len() {
                        if !holder.tasks[i].is_null() {
                            self.waiting_for_chunks.remove(&holder.tasks[i]);
                            self.drop_node(holder.tasks[i]);
                            holder.tasks[i] = NodeKey::null();
                        }
                    }

                    holder.current_stage = StagedChunkEnum::None;
                    holder.dependency_stage = StagedChunkEnum::None;
                    holder.chunk = None;

                    for i in (StagedChunkEnum::None as usize + 1)..=(target_stage as usize) {
                        let stage_enum = StagedChunkEnum::from(i as u8);
                        let task_node = Node::new(pos, stage_enum);
                        holder.tasks[i] = self.graph.nodes.insert(task_node);

                        if i > (StagedChunkEnum::None as usize + 1) {
                            self.graph.add_edge(holder.tasks[i - 1], holder.tasks[i]);
                        }
                    }

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
                            ) - 50,
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

    #[expect(clippy::too_many_lines)]
    pub(super) fn work(mut self, level: &Arc<Level>) {
        debug!(
            "schedule thread start id: {:?} name: {}",
            thread::current().id(),
            thread::current().name().unwrap_or("unknown")
        );
        loop {
            if level.should_unload.swap(false, Relaxed) {
                self.garbage_collect_dependencies();
                self.process_unload_queue();
            }
            if level.should_save.swap(false, Relaxed) {
                self.save_all_chunk(false);
            }
            if level.shut_down_chunk_system.load(Relaxed) {
                info!("Saving chunks before shutdown...");
                self.garbage_collect_dependencies();
                self.process_unload_queue();
                self.save_all_chunk(true);
                break;
            }

            // 1. Get latest world state (player moves, etc)
            self.resort_work(self.send_level.get());

            // Process unload queue periodically (every 1 second) to batch writes together
            // and act as a brief memory cache if a player walks back into the chunk.
            if !self.unload_chunks.is_empty()
                && self.last_unload.elapsed() >= std::time::Duration::from_secs(1)
            {
                self.process_unload_queue();
                self.last_unload = std::time::Instant::now();
            }

            // 2. Process all pending chunk results from workers
            while let Ok((pos, data)) = self.recv_chunk.try_recv() {
                self.receive_chunk(pos, data);
            }

            // 3. Re-sort if world state changed or new tasks added
            if self.queue_dirty {
                self.sort_queue();
                self.queue_dirty = false;
            }

            // 4. Process ready tasks in the queue (up to max_in_flight).
            //
            // This is Pumpkin's analog of vanilla's ThrottlingChunkTaskDispatcher:
            // stop admitting work while the in-execution count is at the bound
            // (/root/Vanilla/src/net/minecraft/server/level/ThrottlingChunkTaskDispatcher.java:42),
            // increment on dispatch (:47), decrement on completion (:37).
            // `running_task_count` is a faithful stand-in for vanilla's
            // `chunkPositionsInExecution` LongSet without needing a separate set:
            // dispatching a task installs `holder.occupied` as a graph predecessor
            // of every remaining task of that chunk, so a chunk can never have two
            // tasks counted at once and the count equals the number of distinct
            // chunk positions in execution.
            let mut io_batch = Vec::with_capacity(16);
            'out2: while let Some(task) = self.queue.pop() {
                if level.shut_down_chunk_system.load(Relaxed) {
                    self.queue.push(task);
                    info!("Shutdown detected during task processing, saving chunks...");
                    self.save_all_chunk(true);
                    break 'out2;
                }

                if self.running_task_count >= self.max_in_flight {
                    self.queue.push(task);
                    break 'out2;
                }

                // Briefly check for high-priority results or world changes to avoid stalling
                while let Ok((pos, data)) = self.recv_chunk.try_recv() {
                    self.receive_chunk(pos, data);
                    if self.resort_work(self.send_level.get()) {
                        // If world state changed, we MUST re-sort before continuing
                        self.queue.push(task);
                        self.queue_dirty = true;
                        break 'out2;
                    }
                }

                if let Some(node) = self.graph.nodes.get_mut(task.1) {
                    if node.in_degree != 0 {
                        node.in_queue = false;
                        continue;
                    }
                    node.in_flight = true;
                    let node = node.clone();

                    // A chunk can be advanced as part of a neighboring task's write cache.
                    // In that case its queued node may survive even though the returned
                    // ProtoChunk has already reached this stage. Dispatching the stale node
                    // would run the same stage twice and trip ProtoChunk's stage invariant.
                    let actual_stage = self
                        .chunk_map
                        .get(&node.pos)
                        .and_then(|holder| holder.chunk.as_ref())
                        .map(Chunk::get_stage_id);
                    if actual_stage.is_some_and(|stage| stage >= node.stage as u8) {
                        if let Some(holder) = self.chunk_map.get_mut(&node.pos) {
                            holder.current_stage = holder
                                .current_stage
                                .max(StagedChunkEnum::from(actual_stage.expect("checked above")));
                            let task_slot = &mut holder.tasks[node.stage as usize];
                            if *task_slot == task.1 {
                                *task_slot = NodeKey::null();
                            }
                        }
                        self.waiting_for_chunks.remove(&task.1);
                        self.drop_node(task.1);
                        continue;
                    }

                    if node.stage == StagedChunkEnum::Empty {
                        self.running_task_count += 1;
                        let holder = self.chunk_map.get_mut(&node.pos).unwrap();
                        debug_assert!(holder.occupied.is_null());
                        debug_assert_eq!(holder.current_stage, StagedChunkEnum::None);
                        let occupy = self.graph.nodes.insert(Node::new(
                            ChunkPos::new(i32::MAX, i32::MAX),
                            StagedChunkEnum::None,
                        ));
                        let effective_target = holder.target_stage.max(holder.dependency_stage);
                        for i in (holder.current_stage as usize + 1)..=(effective_target as usize) {
                            self.graph.add_edge(occupy, holder.tasks[i]);
                        }
                        holder.occupied = occupy;

                        io_batch.push(node.pos);
                        if io_batch.len() >= 16
                            && self.io_read.send(std::mem::take(&mut io_batch)).is_err()
                        {
                            info!("IO read thread closed, saving remaining chunks...");
                            self.save_all_chunk(true);
                            break 'out2;
                        }
                    } else {
                        // Send any pending IO batch before starting generation
                        if !io_batch.is_empty()
                            && self.io_read.send(std::mem::take(&mut io_batch)).is_err()
                        {
                            info!("IO read thread closed, saving remaining chunks...");
                            self.save_all_chunk(true);
                            break 'out2;
                        }

                        let write_radius = node.stage.get_write_radius();

                        // Pre-validate that every chunk in the write area (including the
                        // center for write_radius==0 stages like Biomes, StructureStart,
                        // Noise, Surface) has its data present before we swap anything out.
                        //
                        // The dependency graph ensures predecessor *tasks* are complete, but
                        // there is a brief window between a task completing on a generation
                        // thread and its chunk data being placed back into the holder. Any
                        // stage whose write area overlaps with a currently-running task will
                        // see chunk==None in that window. We park here and let
                        // check_waiting_tasks() re-queue once all data has arrived.
                        {
                            let all_ready = (-write_radius..=write_radius).all(|dx| {
                                (-write_radius..=write_radius).all(|dy| {
                                    self.chunk_map
                                        .get(&node.pos.add_raw(dx, dy))
                                        .is_some_and(|h| h.chunk.is_some())
                                })
                            });

                            if !all_ready {
                                if let Some(n) = self.graph.nodes.get_mut(task.1) {
                                    n.in_queue = false;
                                    n.in_flight = false;
                                }
                                self.waiting_for_chunks.insert(task.1);
                                // Close the TOCTOU window: the chunk we're waiting for may
                                // have arrived in the recv_chunk drain that happened earlier
                                // in this same loop iteration, before this task was parked.
                                // If so, check_waiting_tasks() will immediately re-queue it
                                // so it isn't stranded with running_task_count==0.
                                self.check_waiting_tasks();
                                continue;
                            }
                        }

                        let mut cache = Cache::new(
                            node.pos.x - write_radius,
                            node.pos.y - write_radius,
                            write_radius << 1 | 1,
                        );

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
                                let Some(tmp) = tmp else {
                                    panic!(
                                        "Missing chunk for position {:?} while processing generation task for {:?} stage {:?}",
                                        new_pos, node.pos, node.stage
                                    )
                                };
                                match tmp {
                                    Chunk::Level(chunk) => {
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

                        self.running_task_count += 1;
                        if let Some(pool) = &self.gen_pool {
                            let pos = node.pos;
                            let stage = node.stage;
                            let send_chunk = self.send_chunk.clone();
                            let level = level.clone();
                            let settings =
                                GenerationSettings::from_dimension(level.world_gen.dimension());

                            pool.spawn(move || {
                                let result = crate::chunk_system::worker_logic::run_generation(
                                    pos, cache, stage, &level, settings,
                                );
                                let _ = send_chunk.send((pos, result));
                            });
                        } else if self.generate.send((node.pos, cache, node.stage)).is_err() {
                            self.running_task_count = self.running_task_count.saturating_sub(1);
                            info!("Generation thread closed, saving remaining chunks...");
                            self.save_all_chunk(true);
                            break 'out2;
                        }
                    }
                }
            }

            // Flush any remaining IO batch
            if !io_batch.is_empty() && self.io_read.send(std::mem::take(&mut io_batch)).is_err() {
                info!("IO read thread closed, saving remaining chunks...");
                self.save_all_chunk(true);
            }

            // 3. If queue is empty, wait for work or results
            if self.queue.is_empty() {
                // If we have tasks in flight, wait for them with timeout
                if self.running_task_count > 0 || !self.waiting_for_chunks.is_empty() {
                    match self.recv_chunk.recv_timeout(Duration::from_millis(5)) {
                        Ok((pos, data)) => {
                            self.receive_chunk(pos, data);
                            self.resort_work(self.send_level.get());
                        }
                        Err(crossfire::compat::RecvTimeoutError::Timeout) => {
                            // Periodically check LevelChannel for new requests
                            self.resort_work(self.send_level.get());
                        }
                        Err(crossfire::compat::RecvTimeoutError::Disconnected) => break,
                    }
                } else {
                    // No tasks in flight, wait indefinitely for LevelChannel changes
                    let restored = Self::restore_ready_tasks(
                        &mut self.graph,
                        &mut self.queue,
                        &self.chunk_map,
                        &self.last_level,
                        &self.last_high_priority,
                        &self.waiting_for_chunks,
                    );
                    if restored > 0 {
                        warn!("Restored {restored} stranded ready chunk tasks to generation queue");
                        continue;
                    }
                    debug_assert!(self.debug_check());
                    debug_assert_eq!(self.running_task_count, 0);
                    self.resort_work(self.send_level.wait_and_get(level));
                }
                if self.queue_dirty {
                    self.sort_queue();
                    self.queue_dirty = false;
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
            if let Ok((pos, data)) = self.recv_chunk.try_recv() {
                self.receive_chunk(pos, data);
                wait_iterations = 0;
            } else {
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

        if self.running_task_count > 0 {
            warn!(
                "Cancelling {} in-flight generation tasks",
                self.running_task_count
            );
            let mut nodes_to_drop = Vec::new();

            for holder in self.chunk_map.values_mut() {
                for task in &mut holder.tasks {
                    if !task.is_null() {
                        self.waiting_for_chunks.remove(task);
                        nodes_to_drop.push(*task);
                        *task = NodeKey::null();
                    }
                }

                if !holder.occupied.is_null()
                    && let Some(node) = self.graph.nodes.get(holder.occupied)
                    && node.pos.x == i32::MAX
                    && node.pos.y == i32::MAX
                {
                    nodes_to_drop.push(holder.occupied);
                    holder.occupied = NodeKey::null();
                }
            }

            for node_key in nodes_to_drop {
                self.drop_node(node_key);
            }

            self.running_task_count = 0;
        }

        drop(self.io_write);

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
            let effective = holder.target_stage.max(holder.dependency_stage);
            debug_assert!(holder.current_stage >= effective);
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
