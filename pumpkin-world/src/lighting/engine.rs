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
use std::collections::{HashMap, HashSet, VecDeque};
//use std::time::Instant;

type FastHashSet<K> = HashSet<K>;
type FastHashMap<K, V> = HashMap<K, V>;

/// Trait to unify Block and Sky light logic
pub trait LightProvider {
    fn get_light(cache: &Cache, pos: BlockPos) -> u8;
    fn set_light(cache: &mut Cache, pos: BlockPos, level: u8);
    fn propagate_level(current_level: u8, opacity: u8, dir: BlockDirection) -> u8;
}

pub struct BlockLightProvider;
impl LightProvider for BlockLightProvider {
    #[inline(always)]
    fn get_light(cache: &Cache, pos: BlockPos) -> u8 {
        get_block_light(cache, pos)
    }
    #[inline(always)]
    fn set_light(cache: &mut Cache, pos: BlockPos, level: u8) {
        set_block_light(cache, pos, level)
    }
    #[inline(always)]
    fn propagate_level(current_level: u8, opacity: u8, _dir: BlockDirection) -> u8 {
        current_level.saturating_sub(opacity.max(1))
    }
}

pub struct SkyLightProvider;
impl LightProvider for SkyLightProvider {
    #[inline(always)]
    fn get_light(cache: &Cache, pos: BlockPos) -> u8 {
        get_sky_light(cache, pos)
    }
    #[inline(always)]
    fn set_light(cache: &mut Cache, pos: BlockPos, level: u8) {
        set_sky_light(cache, pos, level)
    }
    #[inline(always)]
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
    pub(crate) visited: FastHashSet<BlockPos>,
    pub(crate) decrease_queue: VecDeque<(BlockPos, u8)>,

    // Batched updates
    pending_updates: FastHashMap<(i32, i32), Vec<(BlockPos, u8)>>,
    shadow_cache: FastHashMap<BlockPos, u8>,
    _marker: std::marker::PhantomData<P>,
}

impl<P: LightProvider> LightPropagator<P> {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::with_capacity(4096),
            visited: FastHashSet::default(),
            decrease_queue: VecDeque::new(),
            pending_updates: FastHashMap::default(),
            shadow_cache: FastHashMap::default(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.visited.clear();
        self.decrease_queue.clear();
        self.pending_updates.clear();
        self.shadow_cache.clear();
    }

    /// Flushes pending updates to chunk storage
    fn apply_updates(&mut self, cache: &mut Cache) {
        if self.pending_updates.is_empty() {
            return;
        }

        for (_, updates) in self.pending_updates.drain() {
            for (pos, val) in updates {
                P::set_light(cache, pos, val);
            }
        }
    }

    /// Core Propagation Logic (BFS)
    pub fn propagate(&mut self, cache: &mut Cache) {
        self.shadow_cache.clear();

        // Cache metadata for bounds checking
        let cache_x = cache.x;
        let cache_z = cache.z;
        let cache_size = cache.size;

        while let Some(entry) = self.queue.pop_front() {
            let pos = entry.pos;

            // Check shadow cache first, fall back to storage
            let current_light = self
                .shadow_cache
                .get(&pos)
                .copied()
                .unwrap_or_else(|| P::get_light(cache, pos));

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

                // Skip if already visited (critical early-exit optimization)
                if self.visited.contains(&neighbor_pos) {
                    continue;
                }

                let (cx, _rel) = neighbor_pos.chunk_and_chunk_relative_position();
                let rel_x = cx.x - cache_x;
                let rel_z = cx.y - cache_z;

                if rel_x < 0 || rel_x >= cache_size || rel_z < 0 || rel_z >= cache_size {
                    continue;
                }

                // Get block opacity
                let state = cache.get_block_state(&neighbor_pos.0);
                let opacity = state.to_state().opacity;

                let new_level = P::propagate_level(current_light, opacity, dir);

                // Check shadow cache first, fall back to storage
                let neighbor_light = self
                    .shadow_cache
                    .get(&neighbor_pos)
                    .copied()
                    .unwrap_or_else(|| P::get_light(cache, neighbor_pos));

                if new_level > neighbor_light {
                    // Update shadow cache for this propagation cycle
                    self.shadow_cache.insert(neighbor_pos, new_level);

                    // Queue for batch write
                    let chunk_x = neighbor_pos.0.x >> 4;
                    let chunk_z = neighbor_pos.0.z >> 4;
                    self.pending_updates
                        .entry((chunk_x, chunk_z))
                        .or_default()
                        .push((neighbor_pos, new_level));

                    // Add to propagation queue if bright enough
                    if new_level > 1 && self.visited.insert(neighbor_pos) {
                        self.queue.push_back(PropagationEntry {
                            pos: neighbor_pos,
                            skip_direction: Some(dir.opposite()),
                        });
                    }
                }
            }

            // Batch write every 64 chunks worth of updates
            if self.pending_updates.len() > 64 {
                self.apply_updates(cache);
            }
        }

        // Final flush of any remaining updates
        self.apply_updates(cache);
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
    pub fn convert_light(&mut self, cache: &mut Cache) {
        self.clear();

        //let scan_start = Instant::now();

        let center_x = cache.x + (cache.size / 2);
        let center_z = cache.z + (cache.size / 2);
        let start_x = center_x * 16 - 1;
        let start_z = center_z * 16 - 1;
        let end_x = start_x + 18;
        let end_z = start_z + 18;

        let bottom_y = cache.bottom_y() as i32;
        let max_y = bottom_y + cache.height() as i32;

        // Pre-allocate with exact size needed
        let capacity = ((end_x - start_x) * (end_z - start_z)) as usize;
        let mut surface_heights =
            FastHashMap::with_capacity_and_hasher(capacity, Default::default());

        // Process in Z-outer, X-inner order for better cache locality
        for z in start_z..end_z {
            let chunk_z = z >> 4;
            let local_z = (z & 15) as usize;

            for x in start_x..end_x {
                let chunk_x = x >> 4;
                let local_x = (x & 15) as usize;

                // Get heightmap (top solid blocks)
                let top_y = cache.get_top_y(&HeightMap::WorldSurface, x, z);
                surface_heights.insert((x, z), top_y);

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
                            let mut light_engine = c.light_engine.lock().unwrap();
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
                            let mut light_engine = c.light_engine.lock().unwrap();
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
                let top_y = surface_heights[&(x, z)];

                let north_top = surface_heights.get(&(x, z - 1)).copied().unwrap_or(top_y);
                let south_top = surface_heights.get(&(x, z + 1)).copied().unwrap_or(top_y);
                let west_top = surface_heights.get(&(x - 1, z)).copied().unwrap_or(top_y);
                let east_top = surface_heights.get(&(x + 1, z)).copied().unwrap_or(top_y);

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
                        // Safe to break if we're under our own roof
                        else {
                            continue;
                        }
                    }

                    let is_at_surface = y == top_y;
                    let below_neighbor =
                        y < north_top || y < south_top || y < west_top || y < east_top;

                    if (is_at_surface || below_neighbor) && self.visited.insert(pos) {
                        let skip_dir = if y >= top_y {
                            Some(BlockDirection::Up)
                        } else {
                            None
                        };

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
