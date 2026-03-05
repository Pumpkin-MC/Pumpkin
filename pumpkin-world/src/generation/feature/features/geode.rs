use std::collections::{HashMap, HashSet};

use crate::block::RawBlockState;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::RandomGenerator,
};

use crate::generation::feature::features::spring_feature::BlockWrapper;
use crate::{
    block::BlockStateCodec, generation::proto_chunk::GenerationCache, world::BlockRegistryExt,
};
use pumpkin_util::random::RandomImpl;
use std::cmp::Ordering;

/// Minimal wrapper around the perlin sampler used for geode generation, to avoid exposing the entire noise module to this file.
struct NormalNoise(crate::generation::noise::perlin::DoublePerlinNoiseSampler);

impl NormalNoise {
    fn create(rand: &mut RandomGenerator, first_octave: i32, amplitudes: &[f64]) -> Self {
        NormalNoise(
            crate::generation::noise::perlin::DoublePerlinNoiseSampler::new(
                rand,
                first_octave,
                amplitudes,
                true,
            ),
        )
    }

    #[inline]
    fn get_value(&self, x: f64, y: f64, z: f64) -> f64 {
        self.0.sample(x, y, z)
    }
}

pub struct GeodeFeature {
    /* Block settings */
    pub filling_provider: crate::generation::block_state_provider::BlockStateProvider,
    pub inner_layer_provider: crate::generation::block_state_provider::BlockStateProvider,
    pub alternate_inner_layer_provider: crate::generation::block_state_provider::BlockStateProvider,
    pub middle_layer_provider: crate::generation::block_state_provider::BlockStateProvider,
    pub outer_layer_provider: crate::generation::block_state_provider::BlockStateProvider,
    pub inner_placements: Vec<BlockStateCodec>,
    pub cannot_replace: BlockWrapper,
    pub invalid_blocks: BlockWrapper,
    /* Layer settings */
    pub filling: f64,
    pub inner_layer: f64,
    pub middle_layer: f64,
    pub outer_layer: f64,
    /* Crack settings */
    pub generate_crack_chance: f64,
    pub base_crack_size: f64,
    pub crack_point_offset: i32,
    /* Other */
    pub use_potential_placements_chance: f64,
    pub use_alternate_layer0_chance: f64,
    pub placements_require_layer0_alternate: bool,
    pub outer_wall_distance: IntProvider,
    pub distribution_points: IntProvider,
    pub point_offset: IntProvider,
    pub min_gen_offset: i32,
    pub max_gen_offset: i32,
    pub noise_multiplier: f64,
    pub invalid_blocks_threshold: i32,
}

impl GeodeFeature {
    fn safe_set_block<T: GenerationCache>(
        &self,
        chunk: &mut T,
        pos: BlockPos,
        state: &'static BlockState,
        can_replace: &dyn Fn(&BlockState) -> bool,
    ) {
        let existing = GenerationCache::get_block_state(chunk, &pos.0).to_state();
        if can_replace(existing) {
            chunk.set_block_state(&pos.0, state);
        }
    }

    fn has_property(codec: &BlockStateCodec, property: &str) -> bool {
        let state = codec.get_state();
        // Obtain the block corresponding to this state id
        let block = Block::from_state_id(state.id);
        if let Some(props) = block.properties(state.id) {
            props.to_props().iter().any(|(k, _)| *k == property)
        } else {
            false
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _block_registry: &dyn BlockRegistryExt,
        _min_y: i8,
        _height: u16,
        _feature_name: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let origin = pos;
        let num_points = self.distribution_points.get(random);
        let noise = NormalNoise::create(random, -4, &[1.0]);

        // Precompute sets of raw block ids for fast lookups
        let mut invalid_raw_ids: HashSet<u16> = HashSet::new();
        match &self.invalid_blocks {
            BlockWrapper::Single(s) => {
                if let Some(b) = pumpkin_data::Block::from_name(s.as_str()) {
                    invalid_raw_ids.insert(pumpkin_data::Block::get_raw_id_from_state_id(
                        b.default_state.id,
                    ));
                }
            }
            BlockWrapper::Multi(v) => {
                for s in v {
                    if let Some(b) = pumpkin_data::Block::from_name(s.as_str()) {
                        invalid_raw_ids.insert(pumpkin_data::Block::get_raw_id_from_state_id(
                            b.default_state.id,
                        ));
                    }
                }
            }
        }

        let mut cannot_replace_raw_ids: HashSet<u16> = HashSet::new();
        match &self.cannot_replace {
            BlockWrapper::Single(s) => {
                if let Some(b) = pumpkin_data::Block::from_name(s.as_str()) {
                    cannot_replace_raw_ids.insert(pumpkin_data::Block::get_raw_id_from_state_id(
                        b.default_state.id,
                    ));
                }
            }
            BlockWrapper::Multi(v) => {
                for s in v {
                    if let Some(b) = pumpkin_data::Block::from_name(s.as_str()) {
                        cannot_replace_raw_ids.insert(
                            pumpkin_data::Block::get_raw_id_from_state_id(b.default_state.id),
                        );
                    }
                }
            }
        }

        let mut points: Vec<(BlockPos, i32)> = Vec::with_capacity(num_points as usize);
        let mut crack_points: Vec<BlockPos> = Vec::new();
        let crack_size_adjustment = num_points as f64 / (self.outer_wall_distance.get_max() as f64);
        let inner_air = 1.0 / self.filling.sqrt();
        let innermost_block_layer = 1.0 / (self.inner_layer + crack_size_adjustment).sqrt();
        let inner_crust = 1.0 / (self.middle_layer + crack_size_adjustment).sqrt();
        let outer_crust = 1.0 / (self.outer_layer + crack_size_adjustment).sqrt();
        let crack_size = 1.0
            / (self.base_crack_size
                + random.next_f64() / 2.0
                + if num_points > 3 {
                    crack_size_adjustment
                } else {
                    0.0
                })
            .sqrt();
        let should_generate_crack = random.next_f32() < self.generate_crack_chance as f32;
        let mut num_invalid_points = 0;

        // Sample distribution points; bail out early if too many fall into invalid blocks
        for _ in 0..num_points {
            let x = self.outer_wall_distance.get(random);
            let y = self.outer_wall_distance.get(random);
            let z = self.outer_wall_distance.get(random);
            let p = origin.add(x, y, z);
            let raw = GenerationCache::get_block_state(chunk, &p.0);
            let state = raw.to_state();

            // Check against precomputed invalid raw ids
            let raw_id = pumpkin_data::Block::get_raw_id_from_state_id(state.id);
            if state.is_air() || invalid_raw_ids.contains(&raw_id) {
                num_invalid_points += 1;
                if num_invalid_points > self.invalid_blocks_threshold {
                    return false;
                }
            }
            let offset = self.point_offset.get(random);
            points.push((p, offset));
        }

        if should_generate_crack {
            let offset_index = random.next_bounded_i32(4);
            let crack_offset = num_points * 2 + 1;
            match offset_index {
                0 => {
                    crack_points.push(origin.add(crack_offset, 7, 0));
                    crack_points.push(origin.add(crack_offset, 5, 0));
                    crack_points.push(origin.add(crack_offset, 1, 0));
                }
                1 => {
                    crack_points.push(origin.add(0, 7, crack_offset));
                    crack_points.push(origin.add(0, 5, crack_offset));
                    crack_points.push(origin.add(0, 1, crack_offset));
                }
                2 => {
                    crack_points.push(origin.add(crack_offset, 7, crack_offset));
                    crack_points.push(origin.add(crack_offset, 5, crack_offset));
                    crack_points.push(origin.add(crack_offset, 1, crack_offset));
                }
                _ => {
                    crack_points.push(origin.add(0, 7, 0));
                    crack_points.push(origin.add(0, 5, 0));
                    crack_points.push(origin.add(0, 1, 0));
                }
            }
        }

        let mut potential_crystal_placements: Vec<BlockPos> = Vec::new();

        // Determine which blocks can be replaced
        let can_replace_pred = |state: &BlockState| {
            let raw_id = pumpkin_data::Block::get_raw_id_from_state_id(state.id);
            state.is_air() || !cannot_replace_raw_ids.contains(&raw_id)
        };

        let min = origin.add(
            self.min_gen_offset,
            self.min_gen_offset,
            self.min_gen_offset,
        );
        let max = origin.add(
            self.max_gen_offset,
            self.max_gen_offset,
            self.max_gen_offset,
        );

        for point_inside in BlockPos::iterate(min, max) {
            let noise_offset = noise.get_value(
                point_inside.0.x as f64,
                point_inside.0.y as f64,
                point_inside.0.z as f64,
            ) * self.noise_multiplier;
            let mut dist_sum_shell = 0.0;
            let mut dist_sum_crack = 0.0;
            for (pt, off) in &points {
                let dx = (point_inside.0.x - pt.0.x) as f64;
                let dy = (point_inside.0.y - pt.0.y) as f64;
                let dz = (point_inside.0.z - pt.0.z) as f64;
                let dist_sq = dx * dx + dy * dy + dz * dz;
                dist_sum_shell += 1.0 / (dist_sq + (*off as f64)).sqrt() + noise_offset;
            }
            for pt in &crack_points {
                let dx = (point_inside.0.x - pt.0.x) as f64;
                let dy = (point_inside.0.y - pt.0.y) as f64;
                let dz = (point_inside.0.z - pt.0.z) as f64;
                let dist_sq = dx * dx + dy * dy + dz * dz;
                dist_sum_crack +=
                    1.0 / (dist_sq + self.crack_point_offset as f64).sqrt() + noise_offset;
            }
            if matches!(
                dist_sum_shell.partial_cmp(&outer_crust),
                Some(Ordering::Greater) | Some(Ordering::Equal)
            ) {
                if should_generate_crack
                    && dist_sum_crack >= crack_size
                    && dist_sum_shell < inner_air
                {
                    // Carve crack
                    self.safe_set_block(
                        chunk,
                        point_inside,
                        RawBlockState::AIR.to_state(),
                        &can_replace_pred,
                    );
                } else if dist_sum_shell >= inner_air {
                    let state = self.filling_provider.get(random, point_inside);
                    self.safe_set_block(chunk, point_inside, state, &can_replace_pred);
                } else if dist_sum_shell >= innermost_block_layer {
                    let use_alternate = random.next_f32() < self.use_alternate_layer0_chance as f32;
                    if use_alternate {
                        let state = self
                            .alternate_inner_layer_provider
                            .get(random, point_inside);
                        self.safe_set_block(chunk, point_inside, state, &can_replace_pred);
                    } else {
                        let state = self.inner_layer_provider.get(random, point_inside);
                        self.safe_set_block(chunk, point_inside, state, &can_replace_pred);
                    }
                    if (!self.placements_require_layer0_alternate || use_alternate)
                        && random.next_f32() < self.use_potential_placements_chance as f32
                    {
                        potential_crystal_placements.push(point_inside);
                    }
                } else if dist_sum_shell >= inner_crust {
                    let state = self.middle_layer_provider.get(random, point_inside);
                    self.safe_set_block(chunk, point_inside, state, &can_replace_pred);
                } else if dist_sum_shell >= outer_crust {
                    let state = self.outer_layer_provider.get(random, point_inside);
                    self.safe_set_block(chunk, point_inside, state, &can_replace_pred);
                }
            }
        }

        // Iterate through potential placements (these are the budding blocks)
        for crystal_pos in potential_crystal_placements {
            // Pick a random base placement codec for this budding block
            if let Some(base_codec) = self
                .inner_placements
                .get(random.next_bounded_i32(self.inner_placements.len() as i32) as usize)
            {
                for dir in &BlockDirection::all() {
                    let offset = dir.to_offset();
                    let place_pos = crystal_pos.add(offset.x, offset.y, offset.z);
                    let place_raw = GenerationCache::get_block_state(chunk, &place_pos.0);
                    let place_state = place_raw.to_state();

                    // Only place if the target block is replaceable (air/water)
                    let is_air = place_state.is_air();
                    let is_water = place_raw.to_block().name == "water";

                    if is_air || is_water {
                        let mut final_codec = base_codec.clone();

                        // Set facing based on direction
                        let dir_name = match dir {
                            pumpkin_data::BlockDirection::Up => "up",
                            pumpkin_data::BlockDirection::Down => "down",
                            pumpkin_data::BlockDirection::North => "north",
                            pumpkin_data::BlockDirection::South => "south",
                            pumpkin_data::BlockDirection::East => "east",
                            pumpkin_data::BlockDirection::West => "west",
                        };

                        if Self::has_property(&final_codec, "facing") {
                            final_codec
                                .properties
                                .get_or_insert_with(HashMap::new)
                                .insert("facing".to_string(), dir_name.to_string());
                        }

                        // Handle waterlogging dynamically based on the block we are replacing
                        if Self::has_property(&final_codec, "waterlogged") {
                            final_codec
                                .properties
                                .get_or_insert_with(HashMap::new)
                                .insert("waterlogged".to_string(), is_water.to_string());
                        }

                        let final_state = final_codec.get_state();
                        self.safe_set_block(chunk, place_pos, final_state, &can_replace_pred);

                        // Only place one crystal per budding block per geode gen
                        break;
                    }
                }
            }
        }

        true
    }
}
