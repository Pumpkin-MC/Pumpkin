use std::sync::Arc;

use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use pumpkin_world::{chunk_system::ChunkLoading, level::Level};

use super::AllayEntity;
use crate::world::World;

pub(super) struct NoteBlockChunk {
    level: Arc<Level>,
    position: Vector2<i32>,
}

impl NoteBlockChunk {
    fn new(level: Arc<Level>, position: Vector2<i32>) -> Self {
        {
            let mut loading = level
                .chunk_loading
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            loading.add_ticket(position, ChunkLoading::FULL_CHUNK_LEVEL);
            loading.send_change();
        };
        Self { level, position }
    }
}

impl Drop for NoteBlockChunk {
    fn drop(&mut self) {
        let mut loading = self
            .level
            .chunk_loading
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        loading.remove_ticket(self.position, ChunkLoading::FULL_CHUNK_LEVEL);
        loading.send_change();
    }
}

impl AllayEntity {
    pub(super) fn request_noteblock_chunk(&self, world: &Arc<World>, pos: BlockPos) {
        let position = pos.chunk_and_chunk_relative_position().0;
        let mut chunk = self
            .noteblock_chunk
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if chunk.as_ref().is_some_and(|chunk| {
            chunk.position == position && Arc::ptr_eq(&chunk.level, &world.level)
        }) {
            return;
        }
        *chunk = Some(NoteBlockChunk::new(world.level.clone(), position));
    }

    pub(super) fn finish_noteblock_chunk(&self) {
        let mut chunk = self
            .noteblock_chunk
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if chunk
            .as_ref()
            .is_some_and(|chunk| chunk.level.is_chunk_loaded(&chunk.position))
            || self.mob_entity.is_no_ai()
            || self.mob_entity.living_entity.is_dead_or_dying()
        {
            *chunk = None;
        }
    }
}
