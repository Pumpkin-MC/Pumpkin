use crate::chunk::format::LightContainer;
use crate::chunk::{ChunkData, ChunkLight};
use crate::chunk_system::Chunk;
use crate::chunk_system::generation_cache::Cache;
use crate::generation::height_limit::HeightLimitView;
use crate::generation::proto_chunk::GenerationCache;
use crate::lighting::section_flags::{self, SectionMask};
use crate::lighting::sky_fill::SkyFill;
use crate::lighting::storage::{get_block_light, get_sky_light, set_block_light, set_sky_light};
use crate::lighting::{decayed, luminance_of, opacity_of, sky_descended};
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::{BlockDirection, BlockStateId};
use pumpkin_util::HeightMap;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::collections::VecDeque;

use crate::ProtoChunk;

pub trait LightProvider {
    fn get_light(cache: &Cache, pos: BlockPos) -> u8;
    fn set_light(cache: &mut Cache, pos: BlockPos, level: u8);
    /// The layer this provider drives, so a caller can take a chunk's light guard once and
    /// then read and write through it instead of locking per nibble.
    ///
    /// [`Self::get_light`] and [`Self::set_light`] derive chunk index, section and local
    /// coordinates from the `BlockPos` all over again. Every caller inside the propagation
    /// loop has them already, so those two exist for callers that do not.
    fn proto_sections(chunk: &ProtoChunk) -> &[LightContainer];
    fn proto_sections_mut(chunk: &mut ProtoChunk) -> &mut [LightContainer];
    fn level_sections_mut(light: &mut ChunkLight) -> &mut [LightContainer];
    /// What a proto chunk reads back where the section is not stored.
    const PROTO_MISSING: u8;
    fn propagate_level(current_level: u8, opacity: u8, dir: BlockDirection) -> u8;
    /// Brightest level a step in `dir` could possibly hand a neighbour, i.e.
    /// [`Self::propagate_level`] at opacity 0. Lets the flood reject a neighbour before
    /// reading its block state. Vanilla: `maxPossibleNewToLevel`.
    fn max_possible(current_level: u8, dir: BlockDirection) -> u8;
}

/// Light of one cell, with the layer's default where the section is absent.
#[inline]
fn light_in(sections: &[LightContainer], idx: usize, lx: usize, ly: usize, lz: usize, missing: u8) -> u8 {
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

pub struct BlockLightProvider;
impl LightProvider for BlockLightProvider {
    #[inline]
    fn get_light(cache: &Cache, pos: BlockPos) -> u8 {
        get_block_light(cache, pos)
    }
    #[inline]
    fn set_light(cache: &mut Cache, pos: BlockPos, level: u8) {
        set_block_light(cache, pos, level);
    }
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
    fn get_light(cache: &Cache, pos: BlockPos) -> u8 {
        get_sky_light(cache, pos)
    }
    #[inline]
    fn set_light(cache: &mut Cache, pos: BlockPos, level: u8) {
        set_sky_light(cache, pos, level);
    }
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
    /// A proto chunk that has not sized its sky storage yet reads as open sky.
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
    /// The level established at `pos` when it was queued. Levels only rise and every rise
    /// queues its own entry, so this is what should spread from here -> reading the cell
    /// back at pop time would only repeat a lookup the pusher already did.
    level: u8,
    skip_direction: Option<BlockDirection>,
}

pub struct VisitedBitSet {
    bits: Vec<u64>,
    min_x: i32,
    min_y: i32,
    min_z: i32,
    size_x: usize,
    size_y: usize,
    size_z: usize,
}

impl Default for VisitedBitSet {
    fn default() -> Self {
        Self::new()
    }
}

impl VisitedBitSet {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bits: Vec::new(),
            min_x: 0,
            min_y: 0,
            min_z: 0,
            size_x: 0,
            size_y: 0,
            size_z: 0,
        }
    }

    pub fn ensure_capacity(
        &mut self,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        size_x: usize,
        size_y: usize,
        size_z: usize,
    ) {
        self.min_x = min_x;
        self.min_y = min_y;
        self.min_z = min_z;
        self.size_x = size_x;
        self.size_y = size_y;
        self.size_z = size_z;
        let total = size_x * size_y * size_z;
        let words = total.div_ceil(64);
        if self.bits.len() == words {
            self.bits.fill(0);
        } else {
            self.bits.resize(words, 0);
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.bits.fill(0);
    }

    #[inline]
    pub fn test_and_set(&mut self, x: i32, y: i32, z: i32) -> bool {
        let lx = (x - self.min_x) as usize;
        let ly = (y - self.min_y) as usize;
        let lz = (z - self.min_z) as usize;
        if lx >= self.size_x || ly >= self.size_y || lz >= self.size_z {
            return false;
        }
        let idx = (ly * self.size_z + lz) * self.size_x + lx;
        let word = idx >> 6;
        let mask = 1u64 << (idx & 63);
        if let Some(w) = self.bits.get_mut(word) {
            let prev = *w;
            if prev & mask != 0 {
                return false;
            }
            *w = prev | mask;
            true
        } else {
            false
        }
    }

    #[inline]
    #[must_use]
    pub fn is_visited(&self, x: i32, y: i32, z: i32) -> bool {
        let lx = (x - self.min_x) as usize;
        let ly = (y - self.min_y) as usize;
        let lz = (z - self.min_z) as usize;
        if lx >= self.size_x || ly >= self.size_y || lz >= self.size_z {
            return true;
        }
        let idx = (ly * self.size_z + lz) * self.size_x + lx;
        let word = idx >> 6;
        let mask = 1u64 << (idx & 63);
        if let Some(&w) = self.bits.get(word) {
            (w & mask) != 0
        } else {
            true
        }
    }
}

pub struct LightPropagator<P: LightProvider> {
    pub(crate) queue: VecDeque<PropagationEntry>,
    pub(crate) visited: VisitedBitSet,
    pub(crate) decrease_queue: VecDeque<(BlockPos, u8)>,
    _marker: std::marker::PhantomData<P>,
}

impl<P: LightProvider> LightPropagator<P> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: VecDeque::with_capacity(8192),
            visited: VisitedBitSet::new(),
            decrease_queue: VecDeque::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.visited.clear();
        self.decrease_queue.clear();
    }

    /// Proto and level chunks differ only in where their light lives. Split from
    /// [`Self::neighbor_opacity`] so the block read can be skipped when the level alone
    /// already rules the neighbour out.
    fn neighbor_light(
        chunk: &Chunk,
        section_idx: usize,
        local_x: usize,
        local_y: usize,
        local_z: usize,
    ) -> u8 {
        match chunk {
            Chunk::Proto(c) => light_in(
                P::proto_sections(c),
                section_idx,
                local_x,
                local_y,
                local_z,
                P::PROTO_MISSING,
            ),
            Chunk::Level(lvl) => {
                let mut light = lvl
                    .light_engine
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                light_in(
                    P::level_sections_mut(&mut light),
                    section_idx,
                    local_x,
                    local_y,
                    local_z,
                    0,
                )
            }
        }
    }

    fn neighbor_opacity(chunk: &Chunk, ny: i32, min_y: i32, local_x: usize, local_z: usize) -> u8 {
        match chunk {
            Chunk::Proto(c) => {
                opacity_of(c.get_block_state_raw(local_x as i32, ny - min_y, local_z as i32))
            }
            Chunk::Level(lvl) => opacity_of(
                lvl.section
                    .get_block_absolute_y(local_x, ny, local_z)
                    .unwrap_or(BlockStateId::AIR),
            ),
        }
    }

    fn set_neighbor_light(
        chunk: &mut Chunk,
        section_idx: usize,
        local_x: usize,
        local_y: usize,
        local_z: usize,
        level: u8,
    ) {
        match chunk {
            Chunk::Proto(c) => {
                set_light_in(
                    P::proto_sections_mut(c),
                    section_idx,
                    local_x,
                    local_y,
                    local_z,
                    level,
                );
            }
            Chunk::Level(lvl) => {
                let wrote = {
                    let mut light = lvl
                        .light_engine
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    set_light_in(
                        P::level_sections_mut(&mut light),
                        section_idx,
                        local_x,
                        local_y,
                        local_z,
                        level,
                    )
                };
                if wrote {
                    lvl.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
    }

    pub fn propagate(&mut self, cache: &mut Cache) {
        let cache_x = cache.x;
        let cache_z = cache.z;
        let cache_size = cache.size;
        let min_y = cache.bottom_y() as i32;
        let max_y = min_y + cache.height() as i32;

        let min_x = cache_x * 16;
        let min_z = cache_z * 16;
        let size_x = (cache_size * 16) as usize;
        let size_z = (cache_size * 16) as usize;
        let size_y = (max_y - min_y) as usize;
        self.visited
            .ensure_capacity(min_x, min_y, min_z, size_x, size_y, size_z);

        while let Some(entry) = self.queue.pop_front() {
            let pos = entry.pos;

            let current_light = entry.level;
            if current_light <= 1 {
                continue;
            }

            // TODO: Once level reads are performant enough, skip entries whose cell is already
            // brighter than the queued level.

            for dir in BlockDirection::all() {
                if let Some(skip_dir) = entry.skip_direction
                    && dir == skip_dir
                {
                    continue;
                }

                let neighbor_pos = pos.offset(dir.to_offset());
                let nx = neighbor_pos.0.x;
                let ny = neighbor_pos.0.y;
                let nz = neighbor_pos.0.z;

                if ny < min_y || ny >= max_y {
                    continue;
                }

                let rel_x = (nx >> 4) - cache_x;
                let rel_z = (nz >> 4) - cache_z;
                if rel_x < 0 || rel_x >= cache_size || rel_z < 0 || rel_z >= cache_size {
                    continue;
                }

                let chunk_idx = (rel_x * cache_size + rel_z) as usize;
                let local_x = (nx & 15) as usize;
                let local_z = (nz & 15) as usize;
                let section_idx = ((ny - min_y) >> 4) as usize;
                let local_y = (ny & 15) as usize;

                // Deliberately not `P::get_light`/`P::set_light`: those take a `BlockPos`
                // and re-derive the chunk index, the section and the local coordinates
                // that are all sitting right above.
                let neighbor_light = Self::neighbor_light(
                    &cache.chunks[chunk_idx],
                    section_idx,
                    local_x,
                    local_y,
                    local_z,
                );

                // Nothing this step could hand over beats what the neighbour already has.
                // Checked before the block read, which is the expensive half.
                if neighbor_light >= P::max_possible(current_light, dir) {
                    continue;
                }

                let opacity =
                    Self::neighbor_opacity(&cache.chunks[chunk_idx], ny, min_y, local_x, local_z);
                let new_level = P::propagate_level(current_light, opacity, dir);

                if new_level > neighbor_light {
                    Self::set_neighbor_light(
                        &mut cache.chunks[chunk_idx],
                        section_idx,
                        local_x,
                        local_y,
                        local_z,
                        new_level,
                    );

                    // `new_level > neighbor_light` is the relaxation guard: levels only ever
                    // rise, and are bounded by 15, so the flood terminates without a visited
                    // set. A visited set here would freeze whichever seed reached a cell
                    // first, whether or not it was the brightest.
                    if new_level > 1 {
                        self.queue.push_back(PropagationEntry {
                            pos: neighbor_pos,
                            level: new_level,
                            skip_direction: Some(dir.opposite()),
                        });
                    }
                }
            }
        }
    }

    pub fn process_decrease_queue(&mut self, cache: &mut Cache) {
        let cache_x = cache.x;
        let cache_z = cache.z;
        let cache_size = cache.size;

        while let Some((pos, old_val)) = self.decrease_queue.pop_front() {
            for dir in BlockDirection::all() {
                let neighbor_pos = pos.offset(dir.to_offset());

                let (cx, _rel) = neighbor_pos.chunk_and_chunk_relative_position();
                let rel_x = cx.x - cache_x;
                let rel_z = cx.y - cache_z;

                if rel_x < 0 || rel_x >= cache_size || rel_z < 0 || rel_z >= cache_size {
                    continue;
                }

                let neighbor_light = P::get_light(cache, neighbor_pos);
                if neighbor_light == 0 {
                    continue;
                }

                let state = cache.get_block_state(&neighbor_pos.0);
                let opacity = state.to_state().opacity;

                let predicted = P::propagate_level(old_val, opacity, dir);

                if neighbor_light == predicted || neighbor_light < old_val {
                    P::set_light(cache, neighbor_pos, 0);
                    self.decrease_queue
                        .push_back((neighbor_pos, neighbor_light));
                } else if neighbor_light >= old_val {
                    let nx = neighbor_pos.0.x;
                    let ny = neighbor_pos.0.y;
                    let nz = neighbor_pos.0.z;
                    self.queue.push_back(PropagationEntry {
                        pos: neighbor_pos,
                        level: neighbor_light,
                        skip_direction: None,
                    });
                    self.visited.test_and_set(nx, ny, nz);
                }
            }
        }

        self.propagate(cache);
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

        let min_x = cache.x * 16;
        let min_z = cache.z * 16;
        let size_x = (cache.size * 16) as usize;
        let size_z = (cache.size * 16) as usize;
        let size_y = (max_y - min_y) as usize;
        self.visited
            .ensure_capacity(min_x, min_y, min_z, size_x, size_y, size_z);

        // One mask per chunk, not per column: the seeds of a section are a property of the
        // chunk, and all 256 of its columns ask the same question.
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
                    // The rim columns sit in the neighbours, which may already be lit by a
                    // source too deep inside them to be seen from here. Their stored light
                    // is seeded alongside the emitters so it can flow into the chunk.
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

impl SkyLightPropagator {
    #[expect(clippy::too_many_lines)]
    pub fn convert_light(&mut self, cache: &mut Cache) {
        self.clear();

        let center_x = cache.x + (cache.size / 2);
        let center_z = cache.z + (cache.size / 2);
        let start_x = center_x * 16 - 1;
        let start_z = center_z * 16 - 1;
        let end_x = start_x + 18;
        let end_z = start_z + 18;

        let bottom_y = cache.bottom_y() as i32;
        let max_y = bottom_y + cache.height() as i32;

        let min_x = cache.x * 16;
        let min_z = cache.z * 16;
        let size_x = (cache.size * 16) as usize;
        let size_z = (cache.size * 16) as usize;
        let size_y = (max_y - bottom_y) as usize;
        self.visited
            .ensure_capacity(min_x, bottom_y, min_z, size_x, size_y, size_z);

        let mut surface_heights = [0i32; 18 * 18];

        for z in start_z..end_z {
            let lz = (z - start_z) as usize;
            for x in start_x..end_x {
                let lx = (x - start_x) as usize;
                surface_heights[lx * 18 + lz] = cache.get_top_y(&HeightMap::WorldSurface, x, z);
            }
        }

        // The centre chunk sits at rim offsets 1..17, so its own columns are already in the table.
        let center_idx = ((cache.size / 2) * cache.size + (cache.size / 2)) as usize;
        let center_tops = || {
            (1..17).flat_map(|lx: usize| (1..17).map(move |lz: usize| surface_heights[lx * 18 + lz]))
        };
        let sky_fill = match &mut cache.chunks[center_idx] {
            Chunk::Proto(c) => {
                let fill =
                    SkyFill::from_surface(center_tops(), bottom_y, c.light.sky_light.len());
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

        for z in start_z..end_z {
            let chunk_z = z >> 4;
            let local_z = (z & 15) as usize;
            let lz = (z - start_z) as usize;

            for x in start_x..end_x {
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

                match &mut cache.chunks[chunk_idx] {
                    Chunk::Proto(c) => {
                        let sections = c.light.sky_light.len();
                        let fill_end = if is_center {
                            sky_fill.fill_end()
                        } else {
                            sections
                        };
                        let top_local_y = (top_y + 1 - bottom_y).max(0) as usize;
                        let top_sec = top_local_y >> 4;
                        let top_rem = top_local_y & 15;
                        if top_sec < sections {
                            c.light.sky_light[top_sec]
                                .set_column_y_range(local_x, local_z, top_rem, 16, 15);
                            for sec in (top_sec + 1)..fill_end {
                                c.light.sky_light[sec]
                                    .set_column_y_range(local_x, local_z, 0, 16, 15);
                            }
                        }

                        let mut light: u8 = 15;
                        for y in (bottom_y..=top_y).rev() {
                            let local_y_proto = y - bottom_y;
                            let state_id = c.get_block_state_raw(
                                local_x as i32,
                                local_y_proto,
                                local_z as i32,
                            );

                            light = sky_descended(light, opacity_of(state_id));
                            let section_idx = (local_y_proto >> 4) as usize;
                            let local_y = (y & 15) as usize;

                            if section_idx < c.light.sky_light.len() {
                                c.light.sky_light[section_idx]
                                    .set(local_x, local_y, local_z, light);
                            }

                            if light == 0 {
                                break;
                            }
                        }
                    }
                    Chunk::Level(c) => {
                        let mut light_engine = c
                            .light_engine
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);

                        let fill_top = if is_center {
                            sky_fill.open_sky_y(bottom_y).min(max_y)
                        } else {
                            max_y
                        };
                        for y in (top_y + 1)..fill_top {
                            let section_idx = ((y - bottom_y) >> 4) as usize;
                            let local_y = (y & 15) as usize;
                            if section_idx < light_engine.sky_light.len() {
                                light_engine.sky_light[section_idx]
                                    .set(local_x, local_y, local_z, 15);
                            }
                        }

                        // One sections guard for the walk down, for the same reason as in
                        // `propagate_light` -> shorter here, because it stops at the first
                        // block that swallows the last light level.
                        c.section.with_blocks(|sections| {
                            let mut light: u8 = 15;
                            for y in (bottom_y..=top_y).rev() {
                                let section_idx = ((y - bottom_y) >> 4) as usize;
                                let local_y = (y & 15) as usize;

                                let state_id = sections
                                    .get(section_idx)
                                    .map_or(BlockStateId::AIR, |section| {
                                        section.get(local_x, local_y, local_z)
                                    });

                                light = sky_descended(light, opacity_of(state_id));

                                if section_idx < light_engine.sky_light.len() {
                                    light_engine.sky_light[section_idx]
                                        .set(local_x, local_y, local_z, light);
                                }

                                if light == 0 {
                                    break;
                                }
                            }
                        });
                    }
                }
            }
        }

        for z in start_z..end_z {
            let lz = (z - start_z) as usize;
            for x in start_x..end_x {
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

                for y in (bottom_y..=max_check_y).rev() {
                    let pos = BlockPos(Vector3::new(x, y, z));
                    let light = get_sky_light(cache, pos);

                    if light == 0 {
                        if y <= top_y {
                            break;
                        }
                        continue;
                    }

                    let is_at_surface = y == top_y;
                    let below_neighbor =
                        y < north_top || y < south_top || y < west_top || y < east_top;

                    if is_at_surface || below_neighbor {
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

        self.propagate(cache);
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

    /// Worldgen pass (`ProtoChunk`), not `ThreadedLevelLightEngine`. Runtime
    /// [`crate::lighting::DynamicLightEngine`] can restitch after the chunk goes live.
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

        // Cut height + quadrant flags now here, after carvers and features
        //
        // A blanket 0x00 fill below the cut is not needed -> untouched sections stay
        // `LightContainer::Empty(0)`, and `set(.., 0)` on `Empty(0)` is a no-op
        let center = cache.get_center_chunk_mut();
        center.sky_light_height = crate::lighting::SkyLightHeight::compute_from_proto(center).raw();
    }

    pub fn update_block_light(
        &mut self,
        cache: &mut Cache,
        pos: BlockPos,
        old_luminance: u8,
        new_luminance: u8,
    ) {
        if old_luminance > new_luminance {
            let current_light = get_block_light(cache, pos);
            if current_light > 0 {
                self.block_light
                    .decrease_queue
                    .push_back((pos, current_light));
                set_block_light(cache, pos, 0);
            }
        }

        if new_luminance > 0 {
            set_block_light(cache, pos, new_luminance);
            if self
                .block_light
                .visited
                .test_and_set(pos.0.x, pos.0.y, pos.0.z)
            {
                self.block_light.queue.push_back(PropagationEntry {
                    pos,
                    level: new_luminance,
                    skip_direction: None,
                });
            }
        }
    }

    pub fn run_light_updates(&mut self, cache: &mut Cache) {
        if !self.block_light.decrease_queue.is_empty() {
            self.block_light.process_decrease_queue(cache);
        }
        if !self.block_light.queue.is_empty() {
            self.block_light.propagate(cache);
            self.block_light.visited.clear();
        }
        if !self.sky_light.decrease_queue.is_empty() {
            self.sky_light.process_decrease_queue(cache);
        }
        if !self.sky_light.queue.is_empty() {
            self.sky_light.propagate(cache);
            self.sky_light.visited.clear();
        }
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
}
