//! Chunk resolution and per-block light access for the runtime light engine.
//!
//! Every `level.read_chunk_sync` is a `DashMap` lookup, and the propagation loops hit the
//! same chunk many times in a row. [`ChunkCursor`] memoizes the last one, and
//! [`ChunkCursor::resolve`] hands out that chunk together with the in-chunk indices: one
//! position is decoded once, and the light read, the opacity and the write of that step
//! all reuse it.

use super::stats::{Counter, LocalCounters};
use crate::chunk::ChunkData;
use crate::chunk::io::Dirtiable;
use crate::chunk::palette::BlockPalette;
use crate::level::Level;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

/// A position resolved against its chunk: where its Y sits, and for `Inside` the section
/// and in-section indices every accessor below needs.
///
/// Small and `Copy` on purpose -> it is handed to three or four accessors per propagation
/// step, and each of them used to derive it again from the raw [`BlockPos`].
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

/// Derives the cell from coordinates that are already chunk-relative.
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
    /// A shared reference for cursor, so it can be
    /// copied out and handed to a closure while the cursor itself is borrowed mutably.
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

    /// Resolves a position once: the chunk it lives in, plus its cell inside that chunk.
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

    /// The block state at `pos`, or void air if the chunk is not loaded.
    pub(super) fn block_state(&mut self, pos: &BlockPos) -> &'static pumpkin_data::BlockState {
        self.counters.bump(Counter::BlockState);
        self.resolve(pos).map_or_else(
            || pumpkin_data::Block::VOID_AIR.default_state,
            |(chunk, cell)| Self::block_state_at(chunk, cell),
        )
    }

    /// `false` if the write cannot land (chunk not loaded, Y outside the
    /// chunk height). Callers must not re-queue such positions.
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

    // The accessors below take an already resolved chunk and cell. A propagation step asks
    // "loaded?", reads the light level, reads the opacity and then writes.
    //
    // They deliberately do not bump counters: the caller already did that when it resolved.

    pub(super) fn block_state_at(
        chunk: &ChunkData,
        cell: VerticalInChunk,
    ) -> &'static pumpkin_data::BlockState {
        let VerticalInChunk::Inside {
            section_index,
            y_in_section,
            local_x,
            local_z,
        } = cell
        else {
            return pumpkin_data::Block::VOID_AIR.default_state;
        };
        chunk.section.with_blocks(|sections| {
            sections.get(section_index).map_or_else(
                || pumpkin_data::Block::VOID_AIR.default_state,
                |section| section.get(local_x, y_in_section, local_z).to_state(),
            )
        })
    }

    /// Opacity without materialising the state: `opacity_of` answers air from the id alone,
    /// where `block_state_at` always pays the state table lookup and its dereference.
    pub(super) fn opacity_at(chunk: &ChunkData, cell: VerticalInChunk) -> u8 {
        let VerticalInChunk::Inside {
            section_index,
            y_in_section,
            local_x,
            local_z,
        } = cell
        else {
            return pumpkin_data::Block::VOID_AIR.default_state.opacity;
        };
        chunk.section.with_blocks(|sections| {
            sections.get(section_index).map_or(
                pumpkin_data::Block::VOID_AIR.default_state.opacity,
                |section| crate::lighting::opacity_of(section.get(local_x, y_in_section, local_z)),
            )
        })
    }

    pub(super) fn sky_light_at(chunk: &ChunkData, cell: VerticalInChunk) -> u8 {
        match cell {
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
}
