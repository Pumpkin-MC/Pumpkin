use crate::chunk_system::Chunk;
use crate::chunk_system::generation_cache::Cache;
use crate::generation::height_limit::HeightLimitView;
use crate::generation::proto_chunk::GenerationCache;
use crate::lighting::storage::{get_block_light, get_sky_light, set_block_light, set_sky_light};
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
    fn get_light_proto(
        chunk: &ProtoChunk,
        section_idx: usize,
        lx: usize,
        ly: usize,
        lz: usize,
    ) -> u8;
    fn set_light_proto(
        chunk: &mut ProtoChunk,
        section_idx: usize,
        lx: usize,
        ly: usize,
        lz: usize,
        level: u8,
    );
    fn propagate_level(current_level: u8, opacity: u8, dir: BlockDirection) -> u8;
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
    fn get_light_proto(
        chunk: &ProtoChunk,
        section_idx: usize,
        lx: usize,
        ly: usize,
        lz: usize,
    ) -> u8 {
        chunk
            .light
            .block_light
            .get(section_idx)
            .map_or(0, |c| c.get(lx, ly, lz))
    }
    #[inline]
    fn set_light_proto(
        chunk: &mut ProtoChunk,
        section_idx: usize,
        lx: usize,
        ly: usize,
        lz: usize,
        level: u8,
    ) {
        if let Some(c) = chunk.light.block_light.get_mut(section_idx) {
            c.set(lx, ly, lz, level);
        }
    }
    #[inline]
    fn propagate_level(current_level: u8, opacity: u8, _dir: BlockDirection) -> u8 {
        current_level.saturating_sub(opacity.max(1))
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
    fn get_light_proto(
        chunk: &ProtoChunk,
        section_idx: usize,
        lx: usize,
        ly: usize,
        lz: usize,
    ) -> u8 {
        chunk
            .light
            .sky_light
            .get(section_idx)
            .map_or(15, |c| c.get(lx, ly, lz))
    }
    #[inline]
    fn set_light_proto(
        chunk: &mut ProtoChunk,
        section_idx: usize,
        lx: usize,
        ly: usize,
        lz: usize,
        level: u8,
    ) {
        if let Some(c) = chunk.light.sky_light.get_mut(section_idx) {
            c.set(lx, ly, lz, level);
        }
    }
    #[inline]
    fn propagate_level(current_level: u8, opacity: u8, dir: BlockDirection) -> u8 {
        if current_level == 15 && dir == BlockDirection::Down && opacity == 0 {
            return 15;
        }

        current_level.saturating_sub(opacity.max(1))
    }
}

#[derive(Clone, Copy)]
pub struct PropagationEntry {
    pos: BlockPos,
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

            let current_light = P::get_light(cache, pos);
            if current_light <= 1 {
                continue;
            }

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

                if self.visited.is_visited(nx, ny, nz) {
                    continue;
                }

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

                let (opacity, neighbor_light) = match &cache.chunks[chunk_idx] {
                    Chunk::Proto(c) => {
                        let local_y_proto = ny - min_y;
                        let state_id =
                            c.get_block_state_raw(local_x as i32, local_y_proto, local_z as i32);
                        let op = if state_id == BlockStateId::AIR {
                            0
                        } else {
                            state_id.to_state().opacity
                        };
                        let light = P::get_light_proto(c, section_idx, local_x, local_y, local_z);
                        (op, light)
                    }
                    Chunk::Level(lvl) => {
                        let state_id = lvl
                            .section
                            .get_block_absolute_y(local_x, ny, local_z)
                            .unwrap_or(BlockStateId::AIR);
                        let op = if state_id == BlockStateId::AIR {
                            0
                        } else {
                            state_id.to_state().opacity
                        };
                        (op, P::get_light(cache, neighbor_pos))
                    }
                };

                let new_level = P::propagate_level(current_light, opacity, dir);

                if new_level > neighbor_light {
                    match &mut cache.chunks[chunk_idx] {
                        Chunk::Proto(c) => {
                            P::set_light_proto(
                                c,
                                section_idx,
                                local_x,
                                local_y,
                                local_z,
                                new_level,
                            );
                        }
                        Chunk::Level(_) => {
                            P::set_light(cache, neighbor_pos, new_level);
                        }
                    }

                    if new_level > 1 && self.visited.test_and_set(nx, ny, nz) {
                        self.queue.push_back(PropagationEntry {
                            pos: neighbor_pos,
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

impl BlockLightPropagator {
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

                match &mut cache.chunks[chunk_idx] {
                    Chunk::Proto(c) => {
                        for y in min_y..max_y {
                            let local_y_proto = y - min_y;
                            let state_id = c.get_block_state_raw(
                                local_x as i32,
                                local_y_proto,
                                local_z as i32,
                            );
                            if state_id == BlockStateId::AIR {
                                continue;
                            }
                            let emission = state_id.to_state().luminance;
                            if emission > 0 {
                                let section_idx = (local_y_proto >> 4) as usize;
                                let local_y = (y & 15) as usize;
                                if section_idx < c.light.block_light.len() {
                                    c.light.block_light[section_idx]
                                        .set(local_x, local_y, local_z, emission);
                                }
                                if self.visited.test_and_set(x, y, z) {
                                    let pos = BlockPos(Vector3::new(x, y, z));
                                    self.queue.push_back(PropagationEntry {
                                        pos,
                                        skip_direction: None,
                                    });
                                }
                            }
                        }
                    }
                    Chunk::Level(lvl) => {
                        let mut light_engine = lvl
                            .light_engine
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        for y in min_y..max_y {
                            let state_id = lvl
                                .section
                                .get_block_absolute_y(local_x, y, local_z)
                                .unwrap_or(BlockStateId::AIR);
                            if state_id == BlockStateId::AIR {
                                continue;
                            }
                            let emission = state_id.to_state().luminance;
                            if emission > 0 {
                                let section_idx = ((y - min_y) >> 4) as usize;
                                let local_y = (y & 15) as usize;
                                if section_idx < light_engine.block_light.len() {
                                    light_engine.block_light[section_idx]
                                        .set(local_x, local_y, local_z, emission);
                                }
                                if self.visited.test_and_set(x, y, z) {
                                    let pos = BlockPos(Vector3::new(x, y, z));
                                    self.queue.push_back(PropagationEntry {
                                        pos,
                                        skip_direction: None,
                                    });
                                }
                            }
                        }
                    }
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
            let chunk_z = z >> 4;
            let local_z = (z & 15) as usize;
            let lz = (z - start_z) as usize;

            for x in start_x..end_x {
                let chunk_x = x >> 4;
                let local_x = (x & 15) as usize;
                let lx = (x - start_x) as usize;

                let top_y = cache.get_top_y(&HeightMap::WorldSurface, x, z);
                surface_heights[lx * 18 + lz] = top_y;

                let rel_x = chunk_x - cache.x;
                let rel_z = chunk_z - cache.z;

                if rel_x < 0 || rel_x >= cache.size || rel_z < 0 || rel_z >= cache.size {
                    continue;
                }

                let chunk_idx = (rel_x * cache.size + rel_z) as usize;

                match &mut cache.chunks[chunk_idx] {
                    Chunk::Proto(c) => {
                        let top_local_y = (top_y + 1 - bottom_y).max(0) as usize;
                        let top_sec = top_local_y >> 4;
                        let top_rem = top_local_y & 15;
                        if top_sec < c.light.sky_light.len() {
                            c.light.sky_light[top_sec]
                                .set_column_y_range(local_x, local_z, top_rem, 16, 15);
                            for sec in (top_sec + 1)..c.light.sky_light.len() {
                                c.light.sky_light[sec]
                                    .set_column_y_range(local_x, local_z, 0, 16, 15);
                            }
                        }

                        let mut light: i32 = 15;
                        for y in (bottom_y..=top_y).rev() {
                            let local_y_proto = y - bottom_y;
                            let state_id = c.get_block_state_raw(
                                local_x as i32,
                                local_y_proto,
                                local_z as i32,
                            );
                            let opacity = if state_id == BlockStateId::AIR {
                                0
                            } else {
                                state_id.to_state().opacity as i32
                            };

                            light = light.saturating_sub(opacity);
                            let light_val = if light <= 0 { 0 } else { light as u8 };
                            let section_idx = (local_y_proto >> 4) as usize;
                            let local_y = (y & 15) as usize;

                            if section_idx < c.light.sky_light.len() {
                                c.light.sky_light[section_idx]
                                    .set(local_x, local_y, local_z, light_val);
                            }

                            if light <= 0 {
                                break;
                            }
                        }
                    }
                    Chunk::Level(c) => {
                        let mut light_engine = c
                            .light_engine
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);

                        for y in (top_y + 1)..max_y {
                            let section_idx = ((y - bottom_y) >> 4) as usize;
                            let local_y = (y & 15) as usize;
                            if section_idx < light_engine.sky_light.len() {
                                light_engine.sky_light[section_idx]
                                    .set(local_x, local_y, local_z, 15);
                            }
                        }

                        let mut light: i32 = 15;
                        for y in (bottom_y..=top_y).rev() {
                            let section_idx = ((y - bottom_y) >> 4) as usize;
                            let local_y = (y & 15) as usize;

                            let state_id = c
                                .section
                                .get_block_absolute_y(local_x, y, local_z)
                                .unwrap_or(BlockStateId::AIR);
                            let opacity = if state_id == BlockStateId::AIR {
                                0
                            } else {
                                state_id.to_state().opacity as i32
                            };

                            light = light.saturating_sub(opacity);
                            let light_val = if light <= 0 { 0 } else { light as u8 };

                            if section_idx < light_engine.sky_light.len() {
                                light_engine.sky_light[section_idx]
                                    .set(local_x, local_y, local_z, light_val);
                            }

                            if light <= 0 {
                                break;
                            }
                        }
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

                    if (is_at_surface || below_neighbor) && self.visited.test_and_set(x, y, z) {
                        let skip_dir = (y >= top_y).then_some(BlockDirection::Up);

                        self.queue.push_back(PropagationEntry {
                            pos,
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
