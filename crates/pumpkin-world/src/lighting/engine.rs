use crate::chunk::format::LightContainer;
use crate::chunk::{ChunkData, ChunkHeightmapType, ChunkLight};
use crate::chunk_system::Chunk;
use crate::chunk_system::generation_cache::Cache;
use crate::generation::height_limit::HeightLimitView;
use crate::generation::proto_chunk::GenerationCache;
use crate::lighting::occlusion;
use crate::lighting::section_flags::{self, SectionMask};
use crate::lighting::sky_fill::SkyFill;
use crate::lighting::{decayed, luminance_of, opacity_of, sky_descended};
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::{BlockDirection, BlockState, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::collections::VecDeque;

use crate::ProtoChunk;

pub trait LightProvider {
    fn proto_sections(chunk: &ProtoChunk) -> &[LightContainer];
    fn proto_sections_mut(chunk: &mut ProtoChunk) -> &mut [LightContainer];
    fn level_sections_mut(light: &mut ChunkLight) -> &mut [LightContainer];
    /// Missing proto section.
    const PROTO_MISSING: u8;
    fn propagate_level(current_level: u8, opacity: u8, dir: BlockDirection) -> u8;
    /// Vanilla `maxPossibleNewToLevel`: opacity 0, skip the block read.
    fn max_possible(current_level: u8, dir: BlockDirection) -> u8;
}

/// Light of one cell, with the layer's default where the section is absent.
#[inline]
fn light_in(
    sections: &[LightContainer],
    idx: usize,
    lx: usize,
    ly: usize,
    lz: usize,
    missing: u8,
) -> u8 {
    sections.get(idx).map_or(missing, |s| s.get(lx, ly, lz))
}

/// `false` when the section is absent and the write cannot land.
#[inline]
fn set_light_in(
    sections: &mut [LightContainer],
    idx: usize,
    lx: usize,
    ly: usize,
    lz: usize,
    level: u8,
) -> bool {
    sections.get_mut(idx).is_some_and(|s| {
        s.set(lx, ly, lz, level);
        true
    })
}

/// One block of the flood: where it sits in the world and in its chunk's sections.
#[derive(Clone, Copy)]
struct LightCell {
    y: i32,
    min_y: i32,
    section_idx: usize,
    local_x: usize,
    local_y: usize,
    local_z: usize,
}

impl LightCell {
    /// Y relative to the chunk bottom, as `ProtoChunk` indexes it.
    #[inline]
    const fn rel_y(self) -> i32 {
        self.y - self.min_y
    }
}

/// The popped cell, positioned in the world and in the cache so a step can be taken from it.
#[derive(Clone, Copy)]
struct Source {
    x: i32,
    y: i32,
    z: i32,
    rel_x: i32,
    rel_z: i32,
    local_x: usize,
    local_z: usize,
    chunk_idx: usize,
}

/// One neighbour of a [`Source`], already resolved to its chunk in the cache.
#[derive(Clone, Copy)]
struct Neighbor {
    x: i32,
    y: i32,
    z: i32,
    chunk_idx: usize,
}

impl Source {
    /// One step, or `None` when it leaves the cache. Only the stepped axis is tested.
    #[inline]
    const fn step(self, dir: BlockDirection, bounds: CacheBounds) -> Option<Neighbor> {
        match dir {
            BlockDirection::Down => {
                if self.y == bounds.min_y {
                    return None;
                }
                Some(Neighbor {
                    x: self.x,
                    y: self.y - 1,
                    z: self.z,
                    chunk_idx: self.chunk_idx,
                })
            }
            BlockDirection::Up => {
                if self.y + 1 >= bounds.max_y {
                    return None;
                }
                Some(Neighbor {
                    x: self.x,
                    y: self.y + 1,
                    z: self.z,
                    chunk_idx: self.chunk_idx,
                })
            }
            BlockDirection::North if self.local_z == 0 => {
                if self.rel_z == 0 {
                    return None;
                }
                Some(Neighbor {
                    x: self.x,
                    y: self.y,
                    z: self.z - 1,
                    chunk_idx: self.chunk_idx - 1,
                })
            }
            BlockDirection::North => Some(Neighbor {
                x: self.x,
                y: self.y,
                z: self.z - 1,
                chunk_idx: self.chunk_idx,
            }),
            BlockDirection::South if self.local_z == 15 => {
                if self.rel_z + 1 >= bounds.size {
                    return None;
                }
                Some(Neighbor {
                    x: self.x,
                    y: self.y,
                    z: self.z + 1,
                    chunk_idx: self.chunk_idx + 1,
                })
            }
            BlockDirection::South => Some(Neighbor {
                x: self.x,
                y: self.y,
                z: self.z + 1,
                chunk_idx: self.chunk_idx,
            }),
            BlockDirection::West if self.local_x == 0 => {
                if self.rel_x == 0 {
                    return None;
                }
                Some(Neighbor {
                    x: self.x - 1,
                    y: self.y,
                    z: self.z,
                    chunk_idx: self.chunk_idx - bounds.size as usize,
                })
            }
            BlockDirection::West => Some(Neighbor {
                x: self.x - 1,
                y: self.y,
                z: self.z,
                chunk_idx: self.chunk_idx,
            }),
            BlockDirection::East if self.local_x == 15 => {
                if self.rel_x + 1 >= bounds.size {
                    return None;
                }
                Some(Neighbor {
                    x: self.x + 1,
                    y: self.y,
                    z: self.z,
                    chunk_idx: self.chunk_idx + bounds.size as usize,
                })
            }
            BlockDirection::East => Some(Neighbor {
                x: self.x + 1,
                y: self.y,
                z: self.z,
                chunk_idx: self.chunk_idx,
            }),
        }
    }
}

/// The 3x3 cache's extent, constant for one [`LightPropagator::propagate`] drain.
#[derive(Clone, Copy)]
struct CacheBounds {
    min_y: i32,
    max_y: i32,
    size: i32,
}

pub struct BlockLightProvider;
impl LightProvider for BlockLightProvider {
    #[inline]
    fn proto_sections(chunk: &ProtoChunk) -> &[LightContainer] {
        &chunk.light.block_light
    }
    #[inline]
    fn proto_sections_mut(chunk: &mut ProtoChunk) -> &mut [LightContainer] {
        &mut chunk.light.block_light
    }
    #[inline]
    fn level_sections_mut(light: &mut ChunkLight) -> &mut [LightContainer] {
        &mut light.block_light
    }
    const PROTO_MISSING: u8 = 0;
    #[inline]
    fn propagate_level(current_level: u8, opacity: u8, _dir: BlockDirection) -> u8 {
        decayed(current_level, opacity)
    }
    #[inline]
    fn max_possible(current_level: u8, _dir: BlockDirection) -> u8 {
        current_level.saturating_sub(1)
    }
}

pub struct SkyLightProvider;
impl LightProvider for SkyLightProvider {
    #[inline]
    fn proto_sections(chunk: &ProtoChunk) -> &[LightContainer] {
        &chunk.light.sky_light
    }
    #[inline]
    fn proto_sections_mut(chunk: &mut ProtoChunk) -> &mut [LightContainer] {
        &mut chunk.light.sky_light
    }
    #[inline]
    fn level_sections_mut(light: &mut ChunkLight) -> &mut [LightContainer] {
        &mut light.sky_light
    }
    /// Unsized proto sky = open sky.
    const PROTO_MISSING: u8 = 15;
    #[inline]
    fn propagate_level(current_level: u8, opacity: u8, dir: BlockDirection) -> u8 {
        if dir == BlockDirection::Down {
            sky_descended(current_level, opacity)
        } else {
            decayed(current_level, opacity)
        }
    }
    #[inline]
    fn max_possible(current_level: u8, dir: BlockDirection) -> u8 {
        // Straight down, a full 15 passes through transparent blocks undimmed.
        if dir == BlockDirection::Down {
            current_level
        } else {
            current_level.saturating_sub(1)
        }
    }
}

#[derive(Clone, Copy)]
pub struct PropagationEntry {
    pos: BlockPos,
    /// Queued level. Do not re-read at pop: pusher already stored it.
    level: u8,
    skip_direction: Option<BlockDirection>,
}

pub struct LightPropagator<P: LightProvider> {
    pub(crate) queue: VecDeque<PropagationEntry>,
    _marker: std::marker::PhantomData<P>,
}

impl<P: LightProvider> LightPropagator<P> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: VecDeque::with_capacity(8192),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    fn neighbor_state(chunk: &Chunk, cell: LightCell) -> BlockStateId {
        match chunk {
            Chunk::Proto(c) => {
                c.get_block_state_raw(cell.local_x as i32, cell.rel_y(), cell.local_z as i32)
            }
            Chunk::Level(lvl) => lvl
                .section
                .get_block_absolute_y(cell.local_x, cell.y, cell.local_z)
                .unwrap_or(BlockStateId::AIR),
        }
    }

    /// One light lock: `max_possible`, `shapeOccludes`, raise. Lock order: light then section.
    fn try_raise(
        chunk: &mut Chunk,
        from_id: BlockStateId,
        dir: BlockDirection,
        current_light: u8,
        cell: LightCell,
    ) -> Option<u8> {
        let max_possible = P::max_possible(current_light, dir);
        match chunk {
            Chunk::Proto(c) => {
                let stored = light_in(
                    P::proto_sections(c),
                    cell.section_idx,
                    cell.local_x,
                    cell.local_y,
                    cell.local_z,
                    P::PROTO_MISSING,
                );
                if stored >= max_possible {
                    return None;
                }
                let state_id =
                    c.get_block_state_raw(cell.local_x as i32, cell.rel_y(), cell.local_z as i32);
                if occlusion::shape_occludes(from_id, state_id, dir) {
                    return None;
                }
                let new_level = P::propagate_level(current_light, opacity_of(state_id), dir);
                if new_level <= stored {
                    return None;
                }
                if !set_light_in(
                    P::proto_sections_mut(c),
                    cell.section_idx,
                    cell.local_x,
                    cell.local_y,
                    cell.local_z,
                    new_level,
                ) {
                    return None;
                }
                Some(new_level)
            }
            Chunk::Level(lvl) => {
                let mut light = lvl
                    .light_engine
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let stored = light_in(
                    P::level_sections_mut(&mut light),
                    cell.section_idx,
                    cell.local_x,
                    cell.local_y,
                    cell.local_z,
                    0,
                );
                if stored >= max_possible {
                    return None;
                }
                let state_id = lvl
                    .section
                    .get_block_absolute_y(cell.local_x, cell.y, cell.local_z)
                    .unwrap_or(BlockStateId::AIR);
                if occlusion::shape_occludes(from_id, state_id, dir) {
                    return None;
                }
                let new_level = P::propagate_level(current_light, opacity_of(state_id), dir);
                if new_level <= stored {
                    return None;
                }
                if !set_light_in(
                    P::level_sections_mut(&mut light),
                    cell.section_idx,
                    cell.local_x,
                    cell.local_y,
                    cell.local_z,
                    new_level,
                ) {
                    return None;
                }
                drop(light);
                lvl.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
                Some(new_level)
            }
        }
    }

    pub fn propagate(&mut self, cache: &mut Cache) {
        let cache_x = cache.x;
        let cache_z = cache.z;
        let cache_size = cache.size;
        let min_y = cache.bottom_y() as i32;
        let max_y = min_y + cache.height() as i32;
        let bounds = CacheBounds {
            min_y,
            max_y,
            size: cache_size,
        };

        while let Some(entry) = self.queue.pop_front() {
            let pos = entry.pos;
            let current_light = entry.level;
            if current_light <= 1 {
                continue;
            }

            let px = pos.0.x;
            let py = pos.0.y;
            let pz = pos.0.z;
            let rel_x = (px >> 4) - cache_x;
            let rel_z = (pz >> 4) - cache_z;
            if py < min_y
                || py >= max_y
                || rel_x < 0
                || rel_x >= cache_size
                || rel_z < 0
                || rel_z >= cache_size
            {
                continue;
            }
            let local_x = (px & 15) as usize;
            let local_y = (py & 15) as usize;
            let local_z = (pz & 15) as usize;
            let chunk_idx = (rel_x * cache_size + rel_z) as usize;
            let from_id = Self::neighbor_state(
                &cache.chunks[chunk_idx],
                LightCell {
                    y: py,
                    min_y,
                    section_idx: ((py - min_y) >> 4) as usize,
                    local_x,
                    local_y,
                    local_z,
                },
            );
            let source = Source {
                x: px,
                y: py,
                z: pz,
                rel_x,
                rel_z,
                local_x,
                local_z,
                chunk_idx,
            };

            for dir in BlockDirection::all() {
                if let Some(skip_dir) = entry.skip_direction
                    && dir == skip_dir
                {
                    continue;
                }

                let Some(neighbor) = source.step(dir, bounds) else {
                    continue;
                };

                let cell = LightCell {
                    y: neighbor.y,
                    min_y,
                    section_idx: ((neighbor.y - min_y) >> 4) as usize,
                    local_x: (neighbor.x & 15) as usize,
                    local_y: (neighbor.y & 15) as usize,
                    local_z: (neighbor.z & 15) as usize,
                };

                // Relaxation: levels only rise, no visited set.
                if let Some(new_level) = Self::try_raise(
                    &mut cache.chunks[neighbor.chunk_idx],
                    from_id,
                    dir,
                    current_light,
                    cell,
                ) && new_level > 1
                {
                    self.queue.push_back(PropagationEntry {
                        pos: BlockPos(Vector3::new(neighbor.x, neighbor.y, neighbor.z)),
                        level: new_level,
                        skip_direction: Some(dir.opposite()),
                    });
                }
            }
        }
    }
}

pub type BlockLightPropagator = LightPropagator<BlockLightProvider>;
pub type SkyLightPropagator = LightPropagator<SkyLightProvider>;

impl<P: LightProvider> Default for LightPropagator<P> {
    fn default() -> Self {
        Self::new()
    }
}

/// One column of the block light seeding scan: where it sits in the world and in its chunk.
#[derive(Clone, Copy)]
struct SeedColumn {
    x: i32,
    z: i32,
    local_x: usize,
    local_z: usize,
    min_y: i32,
    max_y: i32,
    on_rim: bool,
}

impl BlockLightPropagator {
    /// Writes a cell's own emission and queues it when something can still spread from it.
    fn seed_cell(
        &mut self,
        container: &mut LightContainer,
        col: SeedColumn,
        y: i32,
        local_y: usize,
        emission: u8,
    ) {
        let stored = if col.on_rim {
            container.get(col.local_x, local_y, col.local_z)
        } else {
            0
        };
        if emission > stored {
            container.set(col.local_x, local_y, col.local_z, emission);
        }
        let level = emission.max(stored);
        if level > 1 {
            self.queue.push_back(PropagationEntry {
                pos: BlockPos(Vector3::new(col.x, y, col.z)),
                level,
                skip_direction: None,
            });
        }
    }

    fn seed_proto_column(&mut self, chunk: &mut ProtoChunk, seeds: SectionMask, col: SeedColumn) {
        for section_idx in 0..chunk.light.block_light.len() {
            if !seeds.contains(section_idx) {
                continue;
            }
            for local_y in 0..16usize {
                let relative_y = (section_idx * 16 + local_y) as i32;
                let y = col.min_y + relative_y;
                if y >= col.max_y {
                    break;
                }
                let emission = luminance_of(chunk.get_block_state_raw(
                    col.local_x as i32,
                    relative_y,
                    col.local_z as i32,
                ));
                let container = &mut chunk.light.block_light[section_idx];
                self.seed_cell(container, col, y, local_y, emission);
            }
        }
    }

    fn seed_level_column(&mut self, chunk: &ChunkData, seeds: SectionMask, col: SeedColumn) {
        let mut light = chunk
            .light_engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // One sections guard for the whole column instead of one per block.
        chunk.section.with_blocks(|sections| {
            for (section_idx, section) in sections.iter().enumerate() {
                if !seeds.contains(section_idx) {
                    continue;
                }
                let Some(container) = light.block_light.get_mut(section_idx) else {
                    continue;
                };
                for local_y in 0..16usize {
                    let y = col.min_y + (section_idx * 16 + local_y) as i32;
                    if y >= col.max_y {
                        break;
                    }
                    let emission = luminance_of(section.get(col.local_x, local_y, col.local_z));
                    self.seed_cell(container, col, y, local_y, emission);
                }
            }
        });
    }

    pub fn propagate_light(&mut self, cache: &mut Cache) {
        self.clear();

        let min_y = cache.bottom_y() as i32;
        let max_y = min_y + cache.height() as i32;
        let center_x = cache.x + (cache.size / 2);
        let center_z = cache.z + (cache.size / 2);

        let start_x = center_x * 16 - 1;
        let start_z = center_z * 16 - 1;
        let end_x = start_x + 18;
        let end_z = start_z + 18;

        // One mask per chunk, not per column.
        let seeds: Vec<SectionMask> = cache
            .chunks
            .iter()
            .enumerate()
            .map(|(idx, chunk)| {
                let rim = (idx / cache.size as usize) as i32 + cache.x != center_x
                    || (idx % cache.size as usize) as i32 + cache.z != center_z;
                section_flags::block_light_seeds(chunk, rim)
            })
            .collect();

        for z in start_z..end_z {
            let rel_z = (z >> 4) - cache.z;
            let local_z = (z & 15) as usize;

            for x in start_x..end_x {
                let rel_x = (x >> 4) - cache.x;
                if rel_x < 0 || rel_x >= cache.size || rel_z < 0 || rel_z >= cache.size {
                    continue;
                }
                let chunk_idx = (rel_x * cache.size + rel_z) as usize;
                let local_x = (x & 15) as usize;
                let seeds = seeds[chunk_idx];

                let column = SeedColumn {
                    x,
                    z,
                    local_x,
                    local_z,
                    min_y,
                    max_y,
                    // Rim: seed stored light so it can flow in.
                    on_rim: (x >> 4) != center_x || (z >> 4) != center_z,
                };

                match &mut cache.chunks[chunk_idx] {
                    Chunk::Proto(c) => self.seed_proto_column(c, seeds, column),
                    Chunk::Level(lvl) => self.seed_level_column(lvl, seeds, column),
                }
            }
        }

        self.propagate(cache);
    }
}

fn fill_sky_above(
    sky: &mut [LightContainer],
    local_x: usize,
    local_z: usize,
    from_rel_y: usize,
    until_section: usize,
) {
    let top_sec = from_rel_y >> 4;
    if top_sec >= sky.len() {
        return;
    }
    let fill_end = until_section.min(sky.len());
    sky[top_sec].set_column_y_range(local_x, local_z, from_rel_y & 15, 16, 15);
    for section in &mut sky[top_sec + 1..fill_end] {
        section.set_column_y_range(local_x, local_z, 0, 16, 15);
    }
}

/// First air above the top non-air. Proto already stores that; level `WorldSurface` is inclusive.
fn exclusive_column_top(cache: &Cache, x: i32, z: i32) -> i32 {
    let rel_x = (x >> 4) - cache.x;
    let rel_z = (z >> 4) - cache.z;
    if rel_x < 0 || rel_z < 0 || rel_x >= cache.size || rel_z >= cache.size {
        return cache.bottom_y() as i32;
    }
    let idx = (rel_x * cache.size + rel_z) as usize;
    match &cache.chunks[idx] {
        Chunk::Proto(c) => c.top_block_height_exclusive(x, z),
        Chunk::Level(c) => {
            let min_y = c.section.min_y;
            c.heightmap
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(ChunkHeightmapType::WorldSurface, x, z, min_y)
                + 1
        }
    }
}

fn descend_sky_column(
    sky: &mut [LightContainer],
    local_x: usize,
    local_z: usize,
    bottom_y: i32,
    top_y: i32,
    mut state_at: impl FnMut(i32) -> BlockStateId,
) {
    let mut light = 15u8;
    let mut above = BlockStateId::AIR;
    for y in (bottom_y..=top_y).rev() {
        let state_id = state_at(y);
        light = if occlusion::shape_occludes(above, state_id, BlockDirection::Down) {
            0
        } else {
            sky_descended(light, opacity_of(state_id))
        };
        above = state_id;
        let rel = (y - bottom_y) as usize;
        if let Some(section) = sky.get_mut(rel >> 4) {
            section.set(local_x, (y & 15) as usize, local_z, light);
        }
        if light == 0 {
            break;
        }
    }
}

impl SkyLightPropagator {
    pub fn convert_light(&mut self, cache: &mut Cache) {
        self.clear();

        let center_x = cache.x + (cache.size / 2);
        let center_z = cache.z + (cache.size / 2);
        let start_x = center_x * 16 - 1;
        let start_z = center_z * 16 - 1;
        let end_x = start_x + 18;
        let end_z = start_z + 18;

        let bottom_y = cache.bottom_y() as i32;

        let mut surface_heights = [0i32; 18 * 18];
        for z in start_z..end_z {
            let lz = (z - start_z) as usize;
            for x in start_x..end_x {
                let lx = (x - start_x) as usize;
                surface_heights[lx * 18 + lz] = exclusive_column_top(cache, x, z);
            }
        }

        // The centre chunk sits at rim offsets 1..17, so its own columns are already in the table.
        let center_idx = ((cache.size / 2) * cache.size + (cache.size / 2)) as usize;
        let center_tops = || {
            (1..17)
                .flat_map(|lx: usize| (1..17).map(move |lz: usize| surface_heights[lx * 18 + lz]))
        };
        let sky_fill = match &mut cache.chunks[center_idx] {
            Chunk::Proto(c) => {
                let fill = SkyFill::from_surface(center_tops(), bottom_y, c.light.sky_light.len());
                fill.mark(&mut c.light.sky_light);
                fill
            }
            Chunk::Level(c) => {
                let mut light = c
                    .light_engine
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let fill = SkyFill::from_surface(center_tops(), bottom_y, light.sky_light.len());
                fill.mark(&mut light.sky_light);
                fill
            }
        };

        Self::fill_and_descend(
            cache,
            (start_x, start_z),
            bottom_y,
            &surface_heights,
            center_idx,
            sky_fill,
        );

        self.seed_shadowed_columns(
            cache,
            (start_x, start_z),
            bottom_y,
            &surface_heights,
            center_idx,
        );

        self.propagate(cache);
    }

    /// Queues every lit cell a taller neighbouring column shadows.
    fn seed_shadowed_columns(
        &mut self,
        cache: &Cache,
        (start_x, start_z): (i32, i32),
        bottom_y: i32,
        surface_heights: &[i32; 18 * 18],
        center_idx: usize,
    ) {
        for z in start_z..start_z + 18 {
            let lz = (z - start_z) as usize;
            for x in start_x..start_x + 18 {
                let lx = (x - start_x) as usize;
                let top_y = surface_heights[lx * 18 + lz];

                let north_top = if lz > 0 {
                    surface_heights[lx * 18 + (lz - 1)]
                } else {
                    top_y
                };
                let south_top = if lz + 1 < 18 {
                    surface_heights[lx * 18 + (lz + 1)]
                } else {
                    top_y
                };
                let west_top = if lx > 0 {
                    surface_heights[(lx - 1) * 18 + lz]
                } else {
                    top_y
                };
                let east_top = if lx + 1 < 18 {
                    surface_heights[(lx + 1) * 18 + lz]
                } else {
                    top_y
                };

                let max_check_y = top_y
                    .max(north_top)
                    .max(south_top)
                    .max(west_top)
                    .max(east_top);

                // One chunk + one light guard for the column.
                let rel_x = (x >> 4) - cache.x;
                let rel_z = (z >> 4) - cache.z;
                if rel_x < 0 || rel_x >= cache.size || rel_z < 0 || rel_z >= cache.size {
                    continue;
                }
                let chunk_idx = (rel_x * cache.size + rel_z) as usize;
                let local_x = (x & 15) as usize;
                let local_z = (z & 15) as usize;

                let mut level_guard = match &cache.chunks[chunk_idx] {
                    Chunk::Proto(_) => None,
                    Chunk::Level(c) => Some(
                        c.light_engine
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner),
                    ),
                };
                let column: &[LightContainer] = match (&cache.chunks[chunk_idx], &mut level_guard) {
                    (Chunk::Proto(c), _) => &c.light.sky_light,
                    (Chunk::Level(_), Some(guard)) => &guard.sky_light,
                    (Chunk::Level(_), None) => unreachable!("guard taken for every level chunk"),
                };

                for y in (bottom_y..=max_check_y).rev() {
                    // Missing section = 0, not PROTO_MISSING.
                    let light = light_in(
                        column,
                        ((y - bottom_y) >> 4) as usize,
                        local_x,
                        (y & 15) as usize,
                        local_z,
                        0,
                    );

                    if light == 0 {
                        // Centre descent is monotonic; rim light from disk is not.
                        if y <= top_y && chunk_idx == center_idx {
                            break;
                        }
                        continue;
                    }
                    let pos = BlockPos(Vector3::new(x, y, z));

                    // Seed only if a taller neighbour shadows this Y.
                    let below_neighbor =
                        y < north_top || y < south_top || y < west_top || y < east_top;

                    if below_neighbor {
                        let skip_dir = (y >= top_y).then_some(BlockDirection::Up);

                        self.queue.push_back(PropagationEntry {
                            pos,
                            level: light,
                            skip_direction: skip_dir,
                        });
                    }
                }
            }
        }
    }

    /// Open sky above each of the 18x18 columns, then the descent through it.
    fn fill_and_descend(
        cache: &mut Cache,
        (start_x, start_z): (i32, i32),
        bottom_y: i32,
        surface_heights: &[i32; 18 * 18],
        center_idx: usize,
        sky_fill: SkyFill,
    ) {
        for z in start_z..start_z + 18 {
            let chunk_z = z >> 4;
            let local_z = (z & 15) as usize;
            let lz = (z - start_z) as usize;

            for x in start_x..start_x + 18 {
                let chunk_x = x >> 4;
                let local_x = (x & 15) as usize;
                let lx = (x - start_x) as usize;

                let top_y = surface_heights[lx * 18 + lz];

                let rel_x = chunk_x - cache.x;
                let rel_z = chunk_z - cache.z;

                if rel_x < 0 || rel_x >= cache.size || rel_z < 0 || rel_z >= cache.size {
                    continue;
                }

                let chunk_idx = (rel_x * cache.size + rel_z) as usize;
                // Sections the centre already holds as one uniform 15 need no column fill.
                let is_center = chunk_idx == center_idx;
                // `top_y` is exclusive: fill 15 from that air cell up, descend the solid below.
                let from_rel_y = (top_y - bottom_y).max(0) as usize;
                let descend_top = top_y - 1;

                // A lit rim chunk already holds what reached it horizontally; the descent
                // only knows this column and would overwrite that with a darker value.
                if !is_center && cache.chunks[chunk_idx].is_lit() {
                    continue;
                }

                match &mut cache.chunks[chunk_idx] {
                    Chunk::Proto(c) => {
                        let fill_end = if is_center {
                            sky_fill.fill_end()
                        } else {
                            c.light.sky_light.len()
                        };
                        fill_sky_above(
                            &mut c.light.sky_light,
                            local_x,
                            local_z,
                            from_rel_y,
                            fill_end,
                        );
                        let col_height = c.height() as usize;
                        let map = c.flat_block_map.as_ref();
                        descend_sky_column(
                            &mut c.light.sky_light,
                            local_x,
                            local_z,
                            bottom_y,
                            descend_top,
                            |y| {
                                let rel = (y - bottom_y) as usize;
                                map[col_height * 16 * local_x + 16 * rel + local_z]
                            },
                        );
                    }
                    Chunk::Level(c) => {
                        let mut light_engine = c
                            .light_engine
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        let fill_end = if is_center {
                            sky_fill.fill_end()
                        } else {
                            light_engine.sky_light.len()
                        };
                        fill_sky_above(
                            &mut light_engine.sky_light,
                            local_x,
                            local_z,
                            from_rel_y,
                            fill_end,
                        );
                        c.section.with_blocks(|sections| {
                            descend_sky_column(
                                &mut light_engine.sky_light,
                                local_x,
                                local_z,
                                bottom_y,
                                descend_top,
                                |y| {
                                    let rel = (y - bottom_y) as usize;
                                    sections.get(rel >> 4).map_or(BlockStateId::AIR, |section| {
                                        section.get(local_x, (y & 15) as usize, local_z)
                                    })
                                },
                            );
                        });
                    }
                }
            }
        }
    }
}

pub struct LightEngine {
    block_light: BlockLightPropagator,
    sky_light: SkyLightPropagator,
}

impl LightEngine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            block_light: BlockLightPropagator::new(),
            sky_light: SkyLightPropagator::new(),
        }
    }

    /// Worldgen pass. Runtime restitch is `DynamicLightEngine`.
    pub fn initialize_light(&mut self, cache: &mut Cache, config: &LightingEngineConfig) {
        if *config != LightingEngineConfig::Default {
            return;
        }

        let should_skip = {
            let center_chunk = cache.get_center_chunk();
            center_chunk.stage >= crate::chunk_system::chunk_state::StagedChunkEnum::Lighting
        };
        if should_skip {
            return;
        }

        self.sky_light.convert_light(cache);
        self.block_light.propagate_light(cache);

        self.block_light.clear();
        self.sky_light.clear();

        // After carvers/features. Empty(0) already covers below-cut.
        let center = cache.get_center_chunk_mut();
        center.sky_light_height = crate::lighting::SkyLightHeight::compute_from_proto(center).raw();
    }

    /// Whether this state's occlusion shape is empty. Vanilla `LightEngine.isEmptyShape`.
    #[inline]
    #[must_use]
    pub const fn is_empty_shape(state: &BlockState) -> bool {
        !state.can_occlude()
    }

    /// Whether a block change can move any light. Vanilla
    /// `LightEngine.hasDifferentLightProperties`.
    #[inline]
    #[must_use]
    pub fn has_different_light_properties(old_state: &BlockState, new_state: &BlockState) -> bool {
        if std::ptr::eq(old_state, new_state) || old_state.id == new_state.id {
            return false;
        }

        // Vanilla: opacity, emission, or either uses shape occlusion (stair rotate, etc.).
        old_state.opacity != new_state.opacity
            || old_state.luminance != new_state.luminance
            || occlusion::uses_shape_for_light_occlusion(old_state)
            || occlusion::uses_shape_for_light_occlusion(new_state)
    }

    /// Light lost crossing one block. Vanilla `LightEngine.getOpacity`.
    #[inline]
    #[must_use]
    pub const fn get_opacity(state: &BlockState) -> u8 {
        if state.opacity > 1 { state.opacity } else { 1 }
    }

    /// Light lost moving from one block into another. Vanilla
    /// `LightEngine.getLightDampeningInto`, which returns 16 -- fully blocking -- when the
    /// merged face shapes occlude.
    #[inline]
    #[must_use]
    pub const fn get_light_dampening_into(
        from_state: &BlockState,
        to_state: &BlockState,
        _direction: BlockDirection,
        simple_opacity: u8,
    ) -> u8 {
        let from_empty = Self::is_empty_shape(from_state);
        let to_empty = Self::is_empty_shape(to_state);
        if from_empty && to_empty {
            return simple_opacity;
        }
        if to_state.can_occlude() && to_state.is_solid_render() {
            return 16;
        }
        simple_opacity
    }
}
impl Default for LightEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::LightEngine;
    use crate::ProtoChunk;
    use crate::chunk::ChunkData;
    use crate::chunk::format::LightContainer;
    use crate::chunk_system::Chunk;
    use crate::chunk_system::generation_cache::Cache;
    use pumpkin_config::lighting::LightingEngineConfig;
    use pumpkin_data::Block;
    use pumpkin_data::dimension::Dimension;
    use std::sync::Arc;

    const SECTIONS: usize = 24;
    const MIN_Y: i32 = -64;
    const SURFACE: i32 = 60;

    fn sky_light(chunk: &ChunkData, local_x: usize, y: i32, local_z: usize) -> u8 {
        let relative = (y - MIN_Y) as usize;
        chunk
            .light_engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .sky_light[relative / 16]
            .get(local_x, relative % 16, local_z)
    }

    fn block_light(chunk: &ChunkData, local_x: usize, y: i32, local_z: usize) -> u8 {
        let relative = (y - MIN_Y) as usize;
        chunk
            .light_engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .block_light[relative / 16]
            .get(local_x, relative % 16, local_z)
    }

    /// A loaded level chunk with solid ground, sized light storage, and whatever `carve`
    /// puts into it.
    fn level_chunk(
        x: i32,
        z: i32,
        carve: impl Fn(&mut Vec<(usize, i32, usize, pumpkin_data::BlockStateId)>),
    ) -> Arc<ChunkData> {
        let chunk = ChunkData::empty(x, z);
        let mut updates = Vec::new();
        for local_x in 0..16usize {
            for local_z in 0..16usize {
                for y in MIN_Y..=SURFACE {
                    updates.push((local_x, y, local_z, Block::STONE.default_state.id));
                }
            }
        }
        carve(&mut updates);
        chunk.set_blocks_batch(updates);
        *chunk
            .heightmap
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = chunk.calculate_heightmap();

        // `ChunkData::empty` starts with zero-length light storage; a loaded chunk has one
        // container per section.
        let mut light = chunk
            .light_engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        light.sky_light = (0..SECTIONS)
            .map(|_| LightContainer::new_empty(0))
            .collect();
        light.block_light = (0..SECTIONS)
            .map(|_| LightContainer::new_empty(0))
            .collect();
        drop(light);

        Arc::new(chunk)
    }

    fn proto_chunk(x: i32, z: i32) -> ProtoChunk {
        use crate::generation::generator::{GeneratorInit, VanillaGenerator, WorldGenerator};
        use pumpkin_util::world_seed::Seed;

        let world_gen = WorldGenerator::Noise(Box::new(VanillaGenerator::new(
            Seed(42),
            Dimension::OVERWORLD,
        )));
        ProtoChunk::new(x, z, &world_gen)
    }

    /// The worldgen pass has to light the already loaded chunks around the proto chunk it
    /// is generating, not only the proto chunk itself.
    ///
    /// That level branch is the one that reads blocks through the chunk's section lock and
    /// writes light through its light mutex, and it is not reachable from any other test.
    #[test]
    fn the_worldgen_pass_lights_a_loaded_neighbour_chunk() {
        let mut cache = Cache::new(-1, -1, 3);
        for dx in 0..3 {
            for dz in 0..3 {
                let (x, z) = (-1 + dx, -1 + dz);
                cache.chunks.push(if (x, z) == (0, 0) {
                    Chunk::Proto(Box::new(proto_chunk(x, z)))
                } else if (x, z) == (1, 0) {
                    // The rim column x=16 sits in this chunk: a light source buried in
                    // rock, with one air pocket beside it for the flood to reach.
                    Chunk::Level(level_chunk(x, z, |updates| {
                        updates.push((0, 30, 5, Block::GLOWSTONE.default_state.id));
                        updates.push((0, 30, 6, Block::AIR.default_state.id));
                    }))
                } else {
                    Chunk::Level(level_chunk(x, z, |_| {}))
                });
            }
        }

        LightEngine::new().initialize_light(&mut cache, &LightingEngineConfig::Default);

        let Chunk::Level(lit) = &cache.chunks[(2 * 3 + 1) as usize] else {
            panic!("the neighbour at (1, 0) is not a level chunk");
        };

        assert_eq!(
            sky_light(lit, 0, SURFACE + 5, 5),
            15,
            "the open sky above the neighbour's surface stayed dark"
        );
        assert_eq!(
            sky_light(lit, 0, SURFACE, 5),
            0,
            "sky light reached into solid stone"
        );
        assert_eq!(
            block_light(lit, 0, 30, 5),
            Block::GLOWSTONE.default_state.luminance,
            "the buried light source in the neighbour was never seeded"
        );
        assert_eq!(
            block_light(lit, 0, 30, 6),
            Block::GLOWSTONE.default_state.luminance - 1,
            "the light source did not propagate into the pocket beside it"
        );
    }

    /// A lit neighbour's sky light already accounts for what reached it horizontally, which a
    /// descent down one column cannot see. Re-descending it overwrites that with a darker
    /// value, and the centre's edge then receives light from the darkened cell.
    #[test]
    fn a_lit_neighbour_keeps_the_light_the_descent_cannot_reproduce() {
        // Leaves let the descent through while dimming it, so it reaches the cell and lands
        // below the stored value. Under solid stone it would stop and overwrite nothing.
        const DEPTH: i32 = 4;
        const BURIED: i32 = SURFACE - DEPTH;
        const DESCENT_REACHES: u8 = 15 - (DEPTH as u8 + 1);
        const STORED: u8 = DESCENT_REACHES + 2;

        let mut cache = Cache::new(-1, -1, 3);
        for dx in 0..3 {
            for dz in 0..3 {
                let (x, z) = (-1 + dx, -1 + dz);
                cache.chunks.push(if (x, z) == (0, 0) {
                    let mut proto = proto_chunk(x, z);
                    // Seal the face the shaft shares with the centre, so only the descent
                    // can reach the buried cell.
                    for y in BURIED..=SURFACE {
                        proto.set_block_state(15, y, 5, Block::STONE.default_state);
                    }
                    Chunk::Proto(Box::new(proto))
                } else if (x, z) == (1, 0) {
                    let chunk = level_chunk(x, z, |updates| {
                        for y in BURIED..=SURFACE {
                            updates.push((0, y, 5, Block::OAK_LEAVES.default_state.id));
                        }
                    });
                    chunk
                        .light_populated
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                    let mut light = chunk
                        .light_engine
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    let relative = (BURIED - MIN_Y) as usize;
                    light.sky_light[relative / 16].set(0, relative % 16, 5, STORED);
                    drop(light);
                    Chunk::Level(chunk)
                } else {
                    Chunk::Level(level_chunk(x, z, |_| {}))
                });
            }
        }

        LightEngine::new().initialize_light(&mut cache, &LightingEngineConfig::Default);

        let Chunk::Level(neighbour) = &cache.chunks[(2 * 3 + 1) as usize] else {
            panic!("the neighbour at (1, 0) is not a level chunk");
        };
        assert_eq!(
            sky_light(neighbour, 0, BURIED, 5),
            STORED,
            "the pass re-descended a lit neighbour and overwrote its stored light"
        );
    }

    /// The emitter mask replaces a sweep over the block map, so it may name a section that no
    /// longer emits, but never miss one that does -> a missed section is light that never
    /// gets seeded.
    #[test]
    fn the_emitter_mask_names_every_section_that_emits() {
        let mut proto = proto_chunk(0, 0);
        let bottom_y = proto.bottom_y() as i32;

        for (index, y) in [bottom_y + 5, bottom_y + 100, bottom_y + 200]
            .into_iter()
            .enumerate()
        {
            proto.set_block_state(index as i32, y, 0, Block::GLOWSTONE.default_state);
        }

        for local_y in 0..proto.height() as i32 {
            for local_x in 0..16 {
                for local_z in 0..16 {
                    let state = proto.get_block_state_raw(local_x, local_y, local_z);
                    assert!(
                        super::luminance_of(state) == 0
                            || proto.emitter_sections.contains((local_y >> 4) as usize),
                        "section {} emits but is not in the mask",
                        local_y >> 4
                    );
                }
            }
        }
    }

    /// Replacing the only emitter leaves the bit set; the pass then scans a section that turns
    /// out to be dark, which costs a scan and never light.
    #[test]
    fn the_emitter_mask_is_allowed_to_go_stale_upwards() {
        let mut proto = proto_chunk(0, 0);
        let y = proto.bottom_y() as i32 + 5;
        let section = ((y - proto.bottom_y() as i32) >> 4) as usize;

        proto.set_block_state(0, y, 0, Block::GLOWSTONE.default_state);
        assert!(proto.emitter_sections.contains(section));

        proto.set_block_state(0, y, 0, Block::AIR.default_state);
        assert!(
            proto.emitter_sections.contains(section),
            "clearing the bit would need a count, and over-reporting is the safe direction"
        );
    }
}
