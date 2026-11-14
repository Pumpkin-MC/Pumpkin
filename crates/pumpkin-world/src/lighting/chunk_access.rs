//! Runtime chunk cursor. Memoizes the last `DashMap` lookup.

use super::stats::{Counter, LocalCounters};
use crate::chunk::ChunkData;
use crate::chunk::io::Dirtiable;
use crate::chunk::palette::BlockPalette;
use crate::level::Level;
use pumpkin_data::BlockStateId;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub(super) enum VerticalInChunk {
    Below,
    Inside {
        section_index: usize,
        y_in_section: usize,
        local_x: usize,
        local_z: usize,
    },
    Above,
}

const fn vertical_in(chunk: &ChunkData, relative: &Vector3<i32>) -> VerticalInChunk {
    let rel_y = relative.y - chunk.section.min_y;
    if rel_y < 0 {
        return VerticalInChunk::Below;
    }
    let section_index = (rel_y as usize) / BlockPalette::SIZE;
    if section_index >= chunk.section.count {
        return VerticalInChunk::Above;
    }
    VerticalInChunk::Inside {
        section_index,
        y_in_section: (rel_y as usize) % BlockPalette::SIZE,
        local_x: relative.x as usize,
        local_z: relative.z as usize,
    }
}

/// Holds `Arc<ChunkData>`, not a `DashMap` guard (deadlock vs waiting writers).
pub(super) struct ChunkCursor<'a> {
    pub(super) level: &'a Level,
    pub(super) counters: &'a LocalCounters<'a>,
    memo: Option<(Vector2<i32>, Option<Arc<ChunkData>>)>,
}

impl<'a> ChunkCursor<'a> {
    pub(super) const fn new(level: &'a Level, counters: &'a LocalCounters<'a>) -> Self {
        Self {
            level,
            counters,
            memo: None,
        }
    }

    pub(super) fn chunk_at(&mut self, chunk_pos: Vector2<i32>) -> Option<&Arc<ChunkData>> {
        if !matches!(&self.memo, Some((cached, _)) if *cached == chunk_pos) {
            let chunk = self
                .level
                .loaded_chunks
                .get(&chunk_pos)
                .map(|entry| entry.value().clone());
            self.memo = Some((chunk_pos, chunk));
        }
        self.memo.as_ref().and_then(|(_, chunk)| chunk.as_ref())
    }

    pub(super) fn chunk_for(&mut self, pos: &BlockPos) -> Option<&Arc<ChunkData>> {
        let (chunk_pos, _) = pos.chunk_and_chunk_relative_position();
        self.chunk_at(chunk_pos)
    }

    pub(super) fn resolve(&mut self, pos: &BlockPos) -> Option<(&Arc<ChunkData>, VerticalInChunk)> {
        let (chunk_pos, relative) = pos.chunk_and_chunk_relative_position();
        let chunk = self.chunk_at(chunk_pos)?;
        let cell = vertical_in(chunk, &relative);
        Some((chunk, cell))
    }

    pub(super) fn sky_light(&mut self, pos: &BlockPos) -> u8 {
        self.counters.bump(Counter::GetSky);
        self.resolve(pos)
            .map_or(0, |(chunk, cell)| Self::sky_light_at(chunk, cell))
    }

    pub(super) fn block_light(&mut self, pos: &BlockPos) -> Option<u8> {
        self.counters.bump(Counter::GetBlockLight);
        self.resolve(pos)
            .and_then(|(chunk, cell)| Self::block_light_at(chunk, cell))
    }

    pub(super) fn block_state(&mut self, pos: &BlockPos) -> &'static pumpkin_data::BlockState {
        self.counters.bump(Counter::BlockState);
        self.resolve(pos).map_or_else(
            || Block::VOID_AIR.default_state,
            |(chunk, cell)| Self::block_state_at(chunk, cell),
        )
    }

    pub(super) fn set_sky_light(&mut self, pos: &BlockPos, light_level: u8) -> bool {
        self.counters.bump(Counter::SetSky);
        self.resolve(pos)
            .is_some_and(|(chunk, cell)| Self::write_light_at(chunk, cell, light_level, false))
    }

    pub(super) fn set_block_light(&mut self, pos: &BlockPos, light_level: u8) -> bool {
        self.counters.bump(Counter::SetBlockLight);
        self.resolve(pos)
            .is_some_and(|(chunk, cell)| Self::write_light_at(chunk, cell, light_level, true))
    }

    pub(super) fn block_state_at(
        chunk: &ChunkData,
        cell: VerticalInChunk,
    ) -> &'static pumpkin_data::BlockState {
        Self::state_id_at(chunk, cell).to_state()
    }

    pub(super) fn state_id_at(chunk: &ChunkData, cell: VerticalInChunk) -> BlockStateId {
        let VerticalInChunk::Inside {
            section_index,
            y_in_section,
            local_x,
            local_z,
        } = cell
        else {
            return Block::VOID_AIR.default_state.id;
        };
        chunk.section.with_blocks(|sections| {
            sections
                .get(section_index)
                .map_or(Block::VOID_AIR.default_state.id, |section| {
                    section.get(local_x, y_in_section, local_z)
                })
        })
    }

    pub(super) fn sky_light_at(chunk: &ChunkData, cell: VerticalInChunk) -> u8 {
        match cell {
            // Vanilla: below world = 0, above = 15.
            VerticalInChunk::Below => 0,
            VerticalInChunk::Above => 15,
            VerticalInChunk::Inside {
                section_index,
                y_in_section,
                local_x,
                local_z,
            } => chunk
                .light_engine
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .sky_light
                .get(section_index)
                .map_or(15, |s| s.get(local_x, y_in_section, local_z)),
        }
    }

    pub(super) fn block_light_at(chunk: &ChunkData, cell: VerticalInChunk) -> Option<u8> {
        let VerticalInChunk::Inside {
            section_index,
            y_in_section,
            local_x,
            local_z,
        } = cell
        else {
            return None;
        };
        chunk
            .light_engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .block_light
            .get(section_index)
            .map(|section| section.get(local_x, y_in_section, local_z))
    }

    pub(super) fn write_light_at(
        chunk: &ChunkData,
        cell: VerticalInChunk,
        light_level: u8,
        block_light: bool,
    ) -> bool {
        let VerticalInChunk::Inside {
            section_index,
            y_in_section,
            local_x,
            local_z,
        } = cell
        else {
            return false;
        };
        let mut light_engine = chunk
            .light_engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let sections = if block_light {
            &mut light_engine.block_light
        } else {
            &mut light_engine.sky_light
        };
        let Some(section) = sections.get_mut(section_index) else {
            return false;
        };
        section.set(local_x, y_in_section, local_z, light_level);
        drop(light_engine);

        if !chunk.is_dirty() {
            chunk.mark_dirty(true);
        }
        true
    }

    /// One light lock: `max_possible` skip, shape occlusion, raise. Lock order: light then section.
    pub(super) fn raise_light(
        chunk: &ChunkData,
        cell: VerticalInChunk,
        from: BlockStateId,
        dir: BlockDirection,
        incoming: u8,
        block_light: bool,
    ) -> Option<u8> {
        let VerticalInChunk::Inside {
            section_index,
            y_in_section,
            local_x,
            local_z,
        } = cell
        else {
            return None;
        };

        let mut engine = chunk
            .light_engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let sections = if block_light {
            &mut engine.block_light
        } else {
            &mut engine.sky_light
        };
        let section = sections.get_mut(section_index)?;
        let stored = section.get(local_x, y_in_section, local_z);
        let max_possible = if !block_light && dir == BlockDirection::Down {
            incoming
        } else {
            incoming.saturating_sub(1)
        };
        if stored >= max_possible {
            return None;
        }

        let state_id = chunk.section.with_blocks(|sections| {
            sections
                .get(section_index)
                .map_or(BlockStateId::AIR, |s| s.get(local_x, y_in_section, local_z))
        });
        if crate::lighting::occlusion::shape_occludes(from, state_id, dir) {
            return None;
        }

        let opacity = crate::lighting::opacity_of(state_id);
        let new_level = if !block_light && dir == BlockDirection::Down {
            crate::lighting::sky_descended(incoming, opacity)
        } else {
            crate::lighting::decayed(incoming, opacity)
        };
        if new_level <= stored {
            return None;
        }

        section.set(local_x, y_in_section, local_z, new_level);
        drop(engine);
        if !chunk.is_dirty() {
            chunk.mark_dirty(true);
        }
        Some(new_level)
    }
}
