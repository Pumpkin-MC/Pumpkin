use super::ChunkPos;
use crate::level::SyncChunk;
use crossbeam::channel::{Receiver, Sender};
use dashmap::DashSet;
use std::sync::Arc;
use std::sync::{Mutex, Weak};
use tokio::sync::oneshot;

#[expect(clippy::type_complexity)]
pub struct ChunkListener {
    single: Mutex<Vec<(ChunkPos, oneshot::Sender<SyncChunk>)>>,
    global: Mutex<Vec<Sender<(ChunkPos, Weak<crate::chunk::ChunkData>)>>>,
    chunks_with_scheduled_ticks: Arc<DashSet<ChunkPos>>,
}

impl ChunkListener {
    #[must_use]
    pub const fn new(chunks_with_scheduled_ticks: Arc<DashSet<ChunkPos>>) -> Self {
        Self {
            single: Mutex::new(Vec::new()),
            global: Mutex::new(Vec::new()),
            chunks_with_scheduled_ticks,
        }
    }

    pub fn add_single_chunk_listener(&self, pos: ChunkPos) -> oneshot::Receiver<SyncChunk> {
        let (tx, rx) = oneshot::channel();
        self.single.lock().unwrap().push((pos, tx));
        rx
    }

    pub fn add_global_chunk_listener(&self) -> Receiver<(ChunkPos, Weak<crate::chunk::ChunkData>)> {
        let (tx, rx) = crossbeam::channel::unbounded();
        self.global.lock().unwrap().push(tx);
        rx
    }

    pub fn process_new_chunk(&self, pos: ChunkPos, chunk: &SyncChunk) {
        // Vanilla `LevelChunk.registerTickContainerInLevel` registers a chunk's block and fluid
        // tick containers with the level every time the chunk reaches FULL. Without this, ticks
        // that arrive with the chunk (loaded from region NBT, or scheduled during worldgen) are
        // never stepped, because `Level::tick_scheduled` only visits chunks that some later
        // `schedule_*_tick` call happened to register.
        if chunk.block_ticks.has_ticks() || chunk.fluid_ticks.has_ticks() {
            self.chunks_with_scheduled_ticks.insert(pos);
        }
        {
            let mut single = self.single.lock().unwrap();
            let mut i = 0;
            let mut len = single.len();
            while i < len {
                if single[i].0 == pos {
                    let (_, send) = single.remove(i);
                    let _ = send.send(chunk.clone());
                    // log::debug!("single listener {i} send {pos:?}");
                    len -= 1;
                    continue;
                }
                i += 1;
            }
        }
        {
            let weak = Arc::downgrade(chunk);
            let mut global = self.global.lock().unwrap();
            let mut i = 0;
            let mut len = global.len();
            while i < len {
                if matches!(global[i].send((pos, weak.clone())), Ok(())) {
                    // log::debug!("global listener {i} send {pos:?}");
                } else {
                    // log::debug!("one global listener dropped");
                    global.remove(i);
                    len -= 1;
                    continue;
                }
                i += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicU64};

    use dashmap::DashSet;
    use pumpkin_data::fluid::Fluid;
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector2::Vector2;

    use super::ChunkListener;
    use pumpkin_data::chunk::ChunkStatus;

    use crate::chunk::format::LightContainer;
    use crate::chunk::{ChunkData, ChunkLight, ChunkSections};
    use crate::tick::scheduler::ChunkTickScheduler;
    use crate::tick::{ScheduledTick, TickPriority};

    fn chunk_with_pending_fluid_tick(x: i32, z: i32) -> Arc<ChunkData> {
        let fluid_ticks = ChunkTickScheduler::from_iter([ScheduledTick {
            delay: 5,
            priority: TickPriority::Normal,
            position: BlockPos::new(x * 16, 64, z * 16),
            value: &Fluid::FLOWING_WATER,
        }]);

        Arc::new(ChunkData {
            section: ChunkSections::new(1, 0),
            heightmap: std::sync::Mutex::default(),
            x,
            z,
            block_ticks: ChunkTickScheduler::default(),
            fluid_ticks,
            pending_block_entities: std::sync::Mutex::default(),
            light_engine: std::sync::Mutex::new(ChunkLight {
                sky_light: vec![LightContainer::new_empty(0); 1].into_boxed_slice(),
                block_light: vec![LightContainer::new_empty(0); 1].into_boxed_slice(),
            }),
            light_populated: AtomicBool::new(false),
            status: ChunkStatus::Full,
            blending_data: None,
            unknown_nbt: NbtCompound::new(),
            dirty: AtomicBool::new(false),
            inhabited_time: AtomicU64::new(0),
        })
    }

    /// Regression test for water frozen mid-flow after a chunk reload.
    ///
    /// `Level::tick_scheduled` only steps chunks listed in `chunks_with_scheduled_ticks`, and the
    /// only writers of that set are `schedule_block_tick`/`schedule_fluid_tick`. A chunk that
    /// arrives already carrying ticks - deserialized from the region file's `fluid_ticks` list, or
    /// carried over from `ProtoChunk` at the worldgen handoff - was therefore never stepped, so
    /// in-flight flowing water stayed at whatever level it had when the chunk was saved.
    ///
    /// Vanilla registers both tick containers whenever a chunk reaches FULL
    /// (`LevelChunk.registerTickContainerInLevel`, 1.21.4 Mojang-named source).
    #[test]
    fn publishing_a_chunk_registers_its_pending_ticks() {
        let set: Arc<DashSet<Vector2<i32>>> = Arc::new(DashSet::new());
        let listener = ChunkListener::new(set.clone());

        let pos = Vector2::new(-32, 307);
        let chunk = chunk_with_pending_fluid_tick(pos.x, pos.y);
        assert!(chunk.fluid_ticks.has_ticks());

        listener.process_new_chunk(pos, &chunk);

        assert!(
            set.contains(&pos),
            "a chunk published with pending fluid ticks must be registered for stepping"
        );
    }

    /// The set is shared by both tick kinds, so a chunk with no pending ticks must not be added.
    #[test]
    fn publishing_a_tickless_chunk_registers_nothing() {
        let set: Arc<DashSet<Vector2<i32>>> = Arc::new(DashSet::new());
        let listener = ChunkListener::new(set.clone());

        let pos = Vector2::new(1, 2);
        let chunk = chunk_with_pending_fluid_tick(pos.x, pos.y);
        // Drain the queued tick so the chunk is genuinely idle.
        let _ = chunk.fluid_ticks.step_tick();
        let _ = chunk.fluid_ticks.step_tick();
        let _ = chunk.fluid_ticks.step_tick();
        let _ = chunk.fluid_ticks.step_tick();
        let _ = chunk.fluid_ticks.step_tick();
        assert!(!chunk.fluid_ticks.has_ticks());

        listener.process_new_chunk(pos, &chunk);

        assert!(set.is_empty());
    }
}
