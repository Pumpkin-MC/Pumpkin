//! Chunk resolution and per-block light access for the runtime light engine.
//!
//! Every `level.read_chunk_sync` is a `DashMap` lookup, and the propagation loops hit the
//! same chunk many times in a row. [`ChunkCursor`] memoizes the last one; the `*_in`
//! functions then operate on an already-resolved chunk so a caller can resolve once and
//! reuse it.

use super::stats::{Counter, LightCounters};
use crate::chunk::ChunkData;
use crate::chunk::io::Dirtiable;
use crate::chunk::palette::BlockPalette;
use crate::level::Level;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use std::sync::Arc;

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

pub(super) const fn vertical_in_chunk(chunk: &ChunkData, pos: &BlockPos) -> VerticalInChunk {
    let (_, relative) = pos.chunk_and_chunk_relative_position();
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

/// Memoized chunk handle for one lighting operation.
///
/// Every `level.read_chunk_sync` is a `DashMap` lookup: hash, shard `RwLock`, table probe,
/// `Arc` deref -> several potential cache misses and an atomic. A single
/// sky propagation step touches 6 neighbours, each with "loaded?", read, opacity and
/// write, so up to 24 such lookups -> and at least two thirds of them land in the
/// same chunk as the origin position. The cursor turns those into a compare plus
/// a pointer deref.
///
/// Deliberately holds the `Arc<ChunkData>` and not the `DashMap` guard: keeping a shard read
/// guard alive across further lookups can deadlock against a waiting writer on the same
/// shard (`parking_lot` lets `read()` block as soon as a writer is queued).
pub(super) struct ChunkCursor<'a> {
    pub(super) level: &'a Level,
    pub(super) counters: &'a LightCounters,
    memo: Option<(Vector2<i32>, Option<Arc<ChunkData>>)>,
}

impl<'a> ChunkCursor<'a> {
    pub(super) const fn new(level: &'a Level, counters: &'a LightCounters) -> Self {
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

    pub(super) fn sky_light(&mut self, pos: &BlockPos) -> u8 {
        self.counters.bump(Counter::GetSky);
        let Some(chunk) = self.chunk_for(pos) else {
            return 0;
        };
        Self::sky_light_in(chunk, pos)
    }

    pub(super) fn block_light(&mut self, pos: &BlockPos) -> Option<u8> {
        self.counters.bump(Counter::GetBlockLight);
        let chunk = self.chunk_for(pos)?;
        Self::block_light_in(chunk, pos)
    }

    /// `false` if the write cannot land (chunk not loaded, Y outside the
    /// chunk height). Callers must not re-queue such positions.
    pub(super) fn set_sky_light(&mut self, pos: &BlockPos, light_level: u8) -> bool {
        self.counters.bump(Counter::SetSky);
        self.chunk_for(pos)
            .is_some_and(|chunk| Self::write_light(chunk, pos, light_level, false))
    }

    pub(super) fn set_block_light(&mut self, pos: &BlockPos, light_level: u8) -> bool {
        self.counters.bump(Counter::SetBlockLight);
        self.chunk_for(pos)
            .is_some_and(|chunk| Self::write_light(chunk, pos, light_level, true))
    }

    // Resolving a chunk costs a `chunk_and_chunk_relative_position` (shifts and masks) plus
    // the memo compare, even on a hit. A caller that touches the same position several
    // times in a row; every propagation step reads "loaded?", the level, the opacity and
    // then writes, this would pay that four times over for one and the same chunk, whose
    // address obviously never changes in between.
    //
    // These take the resolved `&ChunkData` instead, so the caller resolves once, keeps the
    // pointer in a register across all four, and the repeat work disappears. It matters
    // most in `has_open_sky_above`, where the whole column is one chunk by construction and
    // the loop runs up to the world height.
    //
    // They deliberately do not bump counters: the caller already did that when it resolved.

    pub(super) fn block_state_in(chunk: &ChunkData, pos: &BlockPos) -> &'static pumpkin_data::BlockState {
        let (_, relative) = pos.chunk_and_chunk_relative_position();
        chunk
            .section
            .get_block_absolute_y(relative.x as usize, relative.y, relative.z as usize)
            .unwrap_or(pumpkin_data::Block::VOID_AIR.default_state.id)
            .to_state()
    }

    pub(super) fn opacity_in(chunk: &ChunkData, pos: &BlockPos) -> u8 {
        Self::block_state_in(chunk, pos).opacity
    }

    pub(super) fn sky_light_in(chunk: &ChunkData, pos: &BlockPos) -> u8 {
        match vertical_in_chunk(chunk, pos) {
            // Vanilla: sky below the world is 0, above the world is 15.
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

    pub(super) fn block_light_in(chunk: &ChunkData, pos: &BlockPos) -> Option<u8> {
        let VerticalInChunk::Inside {
            section_index,
            y_in_section,
            local_x,
            local_z,
        } = vertical_in_chunk(chunk, pos)
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

    pub(super) fn write_light(
        chunk: &ChunkData,
        pos: &BlockPos,
        light_level: u8,
        block_light: bool,
    ) -> bool {
        let VerticalInChunk::Inside {
            section_index,
            y_in_section,
            local_x,
            local_z,
        } = vertical_in_chunk(chunk, pos)
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
}
