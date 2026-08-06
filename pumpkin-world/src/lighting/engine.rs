use crate::chunk_system::Chunk;
use crate::chunk_system::generation_cache::Cache;
use crate::generation::height_limit::HeightLimitView;
use crate::generation::proto_chunk::GenerationCache;
use crate::lighting::storage::{get_block_light, get_sky_light, set_block_light, set_sky_light};
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::BlockDirection;
use pumpkin_util::HeightMap;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::collections::VecDeque;
//use std::time::Instant;

const LIGHTING_REGION_WIDTH: usize = 18;

/// Dense cache-relative visitation map used by light propagation.
///
/// Propagation never leaves the chunk cache, so hashing absolute block positions
/// wastes work and memory. A bit per cache block also lets the hot path combine
/// the cache-boundary and visited checks.
struct DenseVisited {
    bits: Vec<u64>,
    origin_x: i32,
    origin_z: i32,
    min_y: i32,
    width: usize,
    height: usize,
}

impl DenseVisited {
    const fn new() -> Self {
        Self {
            bits: Vec::new(),
            origin_x: 0,
            origin_z: 0,
            min_y: 0,
            width: 0,
            height: 0,
        }
    }

    fn ensure_layout(&mut self, cache: &Cache) {
        let width = cache.size as usize * 16;
        let height = cache.height() as usize;
        let origin_x = cache.x * 16;
        let origin_z = cache.z * 16;
        let min_y = cache.bottom_y() as i32;

        if self.width == width
            && self.height == height
            && self.origin_x == origin_x
            && self.origin_z == origin_z
            && self.min_y == min_y
        {
            return;
        }

        self.origin_x = origin_x;
        self.origin_z = origin_z;
        self.min_y = min_y;
        self.width = width;
        self.height = height;

        let bit_count = width
            .checked_mul(width)
            .and_then(|area| area.checked_mul(height))
            .expect("light cache dimensions overflow");
        self.bits.resize(bit_count.div_ceil(64), 0);
        self.bits.fill(0);
    }

    fn clear(&mut self) {
        self.bits.fill(0);
    }

    fn index(&self, pos: BlockPos) -> Option<usize> {
        let x = usize::try_from(pos.0.x - self.origin_x).ok()?;
        let y = usize::try_from(pos.0.y - self.min_y).ok()?;
        let z = usize::try_from(pos.0.z - self.origin_z).ok()?;
        if x >= self.width || y >= self.height || z >= self.width {
            return None;
        }

        Some((y * self.width + z) * self.width + x)
    }

    /// Returns true for positions already visited or outside the cache.
    fn contains_or_out_of_bounds(&self, pos: BlockPos) -> bool {
        let Some(index) = self.index(pos) else {
            return true;
        };
        self.bits[index >> 6] & (1u64 << (index & 63)) != 0
    }

    /// Marks an in-bounds position and reports whether it was newly visited.
    fn insert(&mut self, pos: BlockPos) -> bool {
        let Some(index) = self.index(pos) else {
            return false;
        };
        let word = &mut self.bits[index >> 6];
        let mask = 1u64 << (index & 63);
        let newly_inserted = *word & mask == 0;
        *word |= mask;
        newly_inserted
    }
}

/// Trait to unify Block and Sky light logic
pub trait LightProvider {
    fn get_light(cache: &Cache, pos: BlockPos) -> u8;
    fn set_light(cache: &mut Cache, pos: BlockPos, level: u8);
    fn propagate_level(current_level: u8, opacity: u8, dir: BlockDirection) -> u8;
}

pub struct BlockLightProvider;
impl LightProvider for BlockLightProvider {
    fn get_light(cache: &Cache, pos: BlockPos) -> u8 {
        get_block_light(cache, pos)
    }
    fn set_light(cache: &mut Cache, pos: BlockPos, level: u8) {
        set_block_light(cache, pos, level);
    }
    fn propagate_level(current_level: u8, opacity: u8, _dir: BlockDirection) -> u8 {
        current_level.saturating_sub(opacity.max(1))
    }
}

pub struct SkyLightProvider;
impl LightProvider for SkyLightProvider {
    fn get_light(cache: &Cache, pos: BlockPos) -> u8 {
        get_sky_light(cache, pos)
    }
    fn set_light(cache: &mut Cache, pos: BlockPos, level: u8) {
        set_sky_light(cache, pos, level);
    }
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
    skip_direction: Option<BlockDirection>, // direction from which the light came, used to prevent back-propagation
}

pub struct LightPropagator<P: LightProvider> {
    pub(crate) queue: VecDeque<PropagationEntry>,
    visited: DenseVisited,
    pub(crate) decrease_queue: VecDeque<(BlockPos, u8)>,
    _marker: std::marker::PhantomData<P>,
}

impl<P: LightProvider> LightPropagator<P> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: VecDeque::with_capacity(4096),
            visited: DenseVisited::new(),
            decrease_queue: VecDeque::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.visited.clear();
        self.decrease_queue.clear();
    }

    /// Core Propagation Logic (BFS).
    ///
    /// Reads and writes light directly through the light storage (a fast array
    /// lookup) instead of maintaining a separate hashed shadow cache and batched
    /// write buffer; the storage is the single source of truth.
    pub fn propagate(&mut self, cache: &mut Cache) {
        self.visited.ensure_layout(cache);

        while let Some(entry) = self.queue.pop_front() {
            let pos = entry.pos;

            let current_light = P::get_light(cache, pos);
            if current_light <= 1 {
                continue;
            }

            for dir in BlockDirection::all() {
                // Skip the direction we came from (if specified)
                if let Some(skip_dir) = entry.skip_direction
                    && dir == skip_dir
                {
                    continue;
                }

                let neighbor_pos = pos.offset(dir.to_offset());

                // One dense lookup replaces both hashing and cache bounds checks.
                if self.visited.contains_or_out_of_bounds(neighbor_pos) {
                    continue;
                }

                // Get block opacity
                let state = cache.get_block_state(&neighbor_pos.0);
                let opacity = state.to_state().opacity;

                let new_level = P::propagate_level(current_light, opacity, dir);
                let neighbor_light = P::get_light(cache, neighbor_pos);

                if new_level > neighbor_light {
                    P::set_light(cache, neighbor_pos, new_level);

                    // Add to propagation queue if bright enough
                    if new_level > 1 && self.visited.insert(neighbor_pos) {
                        self.queue.push_back(PropagationEntry {
                            pos: neighbor_pos,
                            skip_direction: Some(dir.opposite()),
                        });
                    }
                }
            }
        }
    }

    /// Handle light removal
    pub fn process_decrease_queue(&mut self, cache: &mut Cache) {
        {
            // Cache metadata for bounds checking
            let cache_x = cache.x;
            let cache_z = cache.z;
            let cache_size = cache.size;

            while let Some((pos, old_val)) = self.decrease_queue.pop_front() {
                for dir in BlockDirection::all() {
                    let neighbor_pos = pos.offset(dir.to_offset());

                    // Bounds check
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
                        // Darken
                        P::set_light(cache, neighbor_pos, 0);
                        self.decrease_queue
                            .push_back((neighbor_pos, neighbor_light));
                    } else if neighbor_light >= old_val {
                        // Re-illuminate from this bright neighbor
                        self.queue.push_back(PropagationEntry {
                            pos: neighbor_pos,
                            skip_direction: None,
                        });
                        self.visited.insert(neighbor_pos);
                    }
                }
            }
        }

        self.propagate(cache); // Re-propagate from survivors
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
        self.visited.ensure_layout(cache);

        //let scan_start = Instant::now();

        let min_y = cache.bottom_y() as i32;
        let max_y = min_y + cache.height() as i32;
        let center_x = cache.x + (cache.size / 2);
        let center_z = cache.z + (cache.size / 2);

        let start_x = center_x * 16 - 1;
        let start_z = center_z * 16 - 1;
        let end_x = start_x + 18;
        let end_z = start_z + 18;

        for y in min_y..max_y {
            for z in start_z..end_z {
                for x in start_x..end_x {
                    let pos_vec = Vector3::new(x, y, z);
                    let state = cache.get_block_state(&pos_vec);
                    let emission = state.to_state().luminance;
                    if emission > 0 {
                        let pos = BlockPos(pos_vec);
                        set_block_light(cache, pos, emission);
                        if self.visited.insert(pos) {
                            // Block light propagates in all directions
                            self.queue.push_back(PropagationEntry {
                                pos,
                                skip_direction: None,
                            });
                        }
                    }
                }
            }
        }
        //let scan_elapsed = scan_start.elapsed();
        //let propagate_start = Instant::now();

        self.propagate(cache);

        //let propagate_elapsed = propagate_start.elapsed();
        //log::info!("Block light timing - Scan: {:?}, Propagate: {:?}", scan_elapsed, propagate_elapsed);
    }
}

impl SkyLightPropagator {
    #[expect(clippy::too_many_lines)]
    pub fn convert_light(&mut self, cache: &mut Cache) {
        self.clear();
        self.visited.ensure_layout(cache);

        //let scan_start = Instant::now();

        let center_x = cache.x + (cache.size / 2);
        let center_z = cache.z + (cache.size / 2);
        let start_x = center_x * 16 - 1;
        let start_z = center_z * 16 - 1;
        let end_x = start_x + 18;
        let end_z = start_z + 18;

        let bottom_y = cache.bottom_y() as i32;
        let max_y = bottom_y + cache.height() as i32;

        let mut surface_heights = [0i32; LIGHTING_REGION_WIDTH * LIGHTING_REGION_WIDTH];

        // Process in Z-outer, X-inner order for better cache locality
        for z in start_z..end_z {
            let chunk_z = z >> 4;
            let local_z = (z & 15) as usize;

            for x in start_x..end_x {
                let chunk_x = x >> 4;
                let local_x = (x & 15) as usize;

                // Get heightmap (top solid blocks)
                let top_y = cache.get_top_y(&HeightMap::WorldSurface, x, z);
                let height_index =
                    (z - start_z) as usize * LIGHTING_REGION_WIDTH + (x - start_x) as usize;
                surface_heights[height_index] = top_y;

                // Get chunk index once per column
                let rel_x = chunk_x - cache.x;
                let rel_z = chunk_z - cache.z;

                if rel_x < 0 || rel_x >= cache.size || rel_z < 0 || rel_z >= cache.size {
                    continue;
                }

                let chunk_idx = (rel_x * cache.size + rel_z) as usize;

                // Fill everything above heightmap with 15 immediately
                for y in (top_y + 1)..max_y {
                    let section_idx = ((y - bottom_y) >> 4) as usize;
                    let local_y = (y & 15) as usize;

                    // Direct array access - skip all function call overhead
                    match &mut cache.chunks[chunk_idx] {
                        Chunk::Proto(c) => {
                            if section_idx < c.light.sky_light.len() {
                                c.light.sky_light[section_idx].set(local_x, local_y, local_z, 15);
                            }
                        }
                        Chunk::Level(c) => {
                            let mut light_engine = c
                                .light_engine
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner);
                            if section_idx < light_engine.sky_light.len() {
                                light_engine.sky_light[section_idx]
                                    .set(local_x, local_y, local_z, 15);
                            }
                        }
                    }
                }

                // Only iterate from top_y DOWN - not from max_y
                let mut light: i32 = 15;

                for y in (bottom_y..=top_y).rev() {
                    let section_idx = ((y - bottom_y) >> 4) as usize;
                    let local_y = (y & 15) as usize;

                    // Get block opacity
                    let opacity = {
                        let pos_vec = Vector3::new(x, y, z);
                        let state = cache.get_block_state(&pos_vec);
                        state.to_state().opacity
                    } as i32;

                    // Reduce light by opacity
                    light = light.saturating_sub(opacity);

                    // Set the light value directly
                    let light_val = if light <= 0 { 0 } else { light as u8 };

                    match &mut cache.chunks[chunk_idx] {
                        Chunk::Proto(c) => {
                            if section_idx < c.light.sky_light.len() {
                                c.light.sky_light[section_idx]
                                    .set(local_x, local_y, local_z, light_val);
                            }
                        }
                        Chunk::Level(c) => {
                            let mut light_engine = c
                                .light_engine
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner);
                            if section_idx < light_engine.sky_light.len() {
                                light_engine.sky_light[section_idx]
                                    .set(local_x, local_y, local_z, light_val);
                            }
                        }
                    }

                    // Early exit when light hits 0
                    if light <= 0 {
                        break;
                    }
                }
            }
        }

        // Enqueue horizontal propagation
        for z in start_z..end_z {
            for x in start_x..end_x {
                let local_x = (x - start_x) as usize;
                let local_z = (z - start_z) as usize;
                let height_index = local_z * LIGHTING_REGION_WIDTH + local_x;
                let top_y = surface_heights[height_index];

                let north_top = local_z.checked_sub(1).map_or(top_y, |z| {
                    surface_heights[z * LIGHTING_REGION_WIDTH + local_x]
                });
                let south_top = if local_z + 1 < LIGHTING_REGION_WIDTH {
                    surface_heights[(local_z + 1) * LIGHTING_REGION_WIDTH + local_x]
                } else {
                    top_y
                };
                let west_top = local_x.checked_sub(1).map_or(top_y, |x| {
                    surface_heights[local_z * LIGHTING_REGION_WIDTH + x]
                });
                let east_top = if local_x + 1 < LIGHTING_REGION_WIDTH {
                    surface_heights[local_z * LIGHTING_REGION_WIDTH + local_x + 1]
                } else {
                    top_y
                };

                // We must check up to the highest neighbor to catch the "air sources"
                let max_check_y = top_y
                    .max(north_top)
                    .max(south_top)
                    .max(west_top)
                    .max(east_top);

                for y in (bottom_y..=max_check_y).rev() {
                    let pos = BlockPos(Vector3::new(x, y, z));
                    let light = get_sky_light(cache, pos);

                    // Use continue, or only break if we are safely below all possible side-light
                    if light == 0 {
                        if y <= top_y {
                            break;
                        }
                        continue;
                    }

                    let is_at_surface = y == top_y;
                    let below_neighbor =
                        y < north_top || y < south_top || y < west_top || y < east_top;

                    if (is_at_surface || below_neighbor) && self.visited.insert(pos) {
                        let skip_dir = (y >= top_y).then_some(BlockDirection::Up);

                        self.queue.push_back(PropagationEntry {
                            pos,
                            skip_direction: skip_dir,
                        });
                    }
                }
            }
        }

        //let propagate_start = Instant::now();

        self.propagate(cache);

        //let propagate_elapsed = propagate_start.elapsed();
        //let scan_elapsed = scan_start.elapsed();
        //log::info!("Sky light timing - Scan: {:?}, Propagate: {:?}", scan_elapsed, propagate_elapsed);
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
        self.block_light.visited.ensure_layout(cache);

        // Decrease Logic
        if old_luminance > new_luminance {
            let current_light = get_block_light(cache, pos);
            if current_light > 0 {
                self.block_light
                    .decrease_queue
                    .push_back((pos, current_light));
                set_block_light(cache, pos, 0);
            }
        }

        // Increase Logic
        if new_luminance > 0 {
            set_block_light(cache, pos, new_luminance);
            if self.block_light.visited.insert(pos) {
                self.block_light.queue.push_back(PropagationEntry {
                    pos,
                    skip_direction: None,
                });
            }
        }
    }

    pub fn run_light_updates(&mut self, cache: &mut Cache) {
        self.block_light.visited.ensure_layout(cache);
        self.sky_light.visited.ensure_layout(cache);

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
    use super::DenseVisited;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector3::Vector3;

    fn visited_map() -> DenseVisited {
        let width = 48usize;
        let height = 384usize;
        DenseVisited {
            bits: vec![0; (width * width * height).div_ceil(64)],
            origin_x: -32,
            origin_z: 48,
            min_y: -64,
            width,
            height,
        }
    }

    fn pos(x: i32, y: i32, z: i32) -> BlockPos {
        BlockPos(Vector3::new(x, y, z))
    }

    #[test]
    fn dense_visited_indexes_cache_bounds_and_negative_coordinates() {
        let mut visited = visited_map();

        for position in [
            pos(-32, -64, 48),
            pos(15, 319, 95),
            pos(-7, 80, 73),
            pos(0, 0, 64),
        ] {
            assert!(!visited.contains_or_out_of_bounds(position));
            assert!(visited.insert(position));
            assert!(visited.contains_or_out_of_bounds(position));
            assert!(!visited.insert(position));
        }

        for position in [
            pos(-33, 0, 48),
            pos(16, 0, 48),
            pos(-32, -65, 48),
            pos(-32, 320, 48),
            pos(-32, 0, 47),
            pos(-32, 0, 96),
        ] {
            assert!(visited.contains_or_out_of_bounds(position));
            assert!(!visited.insert(position));
        }
    }

    #[test]
    fn dense_visited_positions_do_not_alias_and_can_be_reused() {
        let mut visited = visited_map();
        let positions = [
            pos(-32, -64, 48),
            pos(-31, -64, 48),
            pos(-32, -63, 48),
            pos(-32, -64, 49),
            pos(15, 319, 95),
        ];

        for &position in &positions {
            assert!(visited.insert(position));
        }
        for &position in &positions {
            assert!(visited.contains_or_out_of_bounds(position));
        }

        visited.clear();
        for position in positions {
            assert!(!visited.contains_or_out_of_bounds(position));
            assert!(visited.insert(position));
        }
    }
}
