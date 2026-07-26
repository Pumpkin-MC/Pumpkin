use super::{GenerationSchedule, TaskHeapNode};
use crate::chunk::io::Dirtiable;
use crate::chunk_system::channel::LevelChange;
use crate::chunk_system::chunk_state::{Chunk, StagedChunkEnum};
use crate::chunk_system::dag::{EdgeKey, Node, NodeKey};
use crate::chunk_system::{ChunkLoading, ChunkPos, HashSetType};
use slotmap::Key;
use std::mem::swap;
use tracing::{error, info};

impl GenerationSchedule {
    #[expect(clippy::too_many_lines)]
    pub(super) fn resort_work(
        &mut self,
        new_data: (Option<LevelChange>, Option<Vec<ChunkPos>>),
    ) -> bool {
        if new_data.0.is_none() && new_data.1.is_none() {
            return false;
        }
        if let Some(high_priority) = new_data.1 {
            self.last_high_priority = high_priority;
            self.queue_dirty = true;
        }
        let Some(new_level) = new_data.0 else {
            return true;
        };
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

            // Effective target is what we actually need to schedule tasks up to.
            let effective_old = old_stage.max(holder.dependency_stage);
            let effective_new = new_stage.max(holder.dependency_stage);

            if effective_old > effective_new {
                for i in (effective_new.max(holder.current_stage) as usize + 1)
                    ..=(effective_old as usize)
                {
                    let task = &mut holder.tasks[i];
                    if !task.is_null() {
                        let is_in_flight = self.graph.nodes.get(*task).is_some_and(|n| n.in_flight);
                        if !is_in_flight {
                            self.waiting_for_chunks.remove(task);
                            self.drop_node(*task);
                            *task = NodeKey::null();
                        }
                    }
                }
                if new_stage == StagedChunkEnum::None
                    && holder.dependency_stage == StagedChunkEnum::None
                {
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
                for i in (effective_old.max(holder.current_stage) as u8 + 1)..=(effective_new as u8)
                {
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
                    if node.in_degree == 0 && !node.in_queue {
                        node.in_queue = true;
                        self.queue.push(TaskHeapNode(0, task));
                    }
                }
            }
            self.chunk_map.insert(pos, holder);
        }
        self.last_level = new_level.1;
        self.queue_dirty = true;
        true
    }

    pub(super) fn garbage_collect_dependencies(&mut self) {
        // Garbage collect stranded dependencies
        let mut stranded = Vec::new();
        for (pos, holder) in &self.chunk_map {
            if holder.target_stage == StagedChunkEnum::None
                && holder.dependency_stage != StagedChunkEnum::None
            {
                stranded.push(*pos);
            }
        }

        for pos in stranded {
            let holder = self.chunk_map.get_mut(&pos).unwrap();
            if !holder.occupied.is_null() && self.graph.nodes.contains_key(holder.occupied) {
                continue;
            }

            let mut cur_edge = holder.occupied_by;
            let mut prev_edge = EdgeKey::null();
            let mut change_head = None;
            let mut has_valid_task = false;

            while !cur_edge.is_null() {
                let edge = self.graph.edges.get(cur_edge).unwrap();
                if self.graph.nodes.contains_key(edge.to) {
                    prev_edge = cur_edge;
                    cur_edge = edge.next;
                    has_valid_task = true;
                } else {
                    let next = edge.next;
                    self.graph.edges.remove(cur_edge);
                    cur_edge = next;
                    if prev_edge.is_null() {
                        change_head = Some(next);
                    } else {
                        self.graph.edges.get_mut(prev_edge).unwrap().next = next;
                    }
                }
            }
            if let Some(next) = change_head {
                holder.occupied_by = next;
            }

            if !has_valid_task {
                holder.dependency_stage = StagedChunkEnum::None;
                self.unload_chunks.insert(pos);
            }
        }
    }

    pub(super) fn process_unload_queue(&mut self) {
        if self.unload_chunks.is_empty() {
            return;
        }

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

                        // Forcefully drop chunks when unloaded to prevent memory leaks
                        // from dangling strong references
                        if chunk.is_dirty() {
                            chunks.push((pos, Chunk::Level(chunk)));
                        }
                        self.chunk_map.remove(&pos);
                    }
                    Chunk::Proto(chunk) => {
                        debug_assert!(!holder.public);
                        chunks.push((pos, Chunk::Proto(chunk)));
                        self.chunk_map.remove(&pos);
                    }
                }
            } else {
                self.unload_chunks.insert(pos);
            }
        }
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

    pub(super) fn save_all_chunk(&mut self, save_proto_chunk: bool) {
        let mut chunks = Vec::with_capacity(self.chunk_map.len());

        for (pos, holder) in &mut self.chunk_map {
            if let Some(chunk) = &holder.chunk {
                let should_save = match chunk {
                    Chunk::Level(sync_chunk) => sync_chunk.is_dirty(),
                    Chunk::Proto(proto) => {
                        save_proto_chunk
                            && !matches!(
                                proto.stage,
                                crate::chunk_system::chunk_state::StagedChunkEnum::Empty
                                    | crate::chunk_system::chunk_state::StagedChunkEnum::None
                            )
                    }
                };

                if should_save {
                    let chunk_to_save = match chunk {
                        Chunk::Level(sync_chunk) => Chunk::Level(sync_chunk.clone()),
                        Chunk::Proto(_) => holder.chunk.take().unwrap(),
                    };
                    chunks.push((*pos, chunk_to_save));
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
        for (pos, _) in &chunks {
            *data.entry(*pos).or_insert(0) += 1;
        }
        drop(data);

        if let Err(e) = self.io_write.send(chunks) {
            error!("Failed to send chunks to io write thread: {:?}", e);
        }
    }
}
