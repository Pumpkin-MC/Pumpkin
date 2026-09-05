use std::collections::{HashMap, HashSet};

use pumpkin_data::{Block, BlockDirection, BlockId, BlockState, BlockStateId, tag::Taggable};
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::{RandomGenerator, legacy_rand::LegacyRand},
};

use crate::generation::feature::features::spring_feature::BlockWrapper;
use crate::{
    block::BlockStateCodec, generation::proto_chunk::GenerationCache, world::WorldPortalExt,
};
use pumpkin_util::random::RandomImpl;
use std::cmp::Ordering;

/// Minimal wrapper around the perlin sampler used for geode generation, to avoid exposing the entire noise module to this file.
struct NormalNoise(crate::generation::noise::perlin::DoublePerlinNoiseSampler);

impl NormalNoise {
    fn create(rand: &mut RandomGenerator, first_octave: i32, amplitudes: &[f64]) -> Self {
        Self(
            crate::generation::noise::perlin::DoublePerlinNoiseSampler::new(
                rand,
                first_octave,
                amplitudes,
                false,
            ),
        )
    }

    #[inline]
    fn get_value(&self, x: f64, y: f64, z: f64) -> f64 {
        f64::from(self.0.sample(x, y, z))
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
    /// The geode's shape noise: vanilla creates it from a `LegacyRandomSource(level.getSeed())`,
    /// i.e. it is a function of the world seed only and consumes nothing from the feature random
    /// (`NormalNoise.create(new WorldgenRandom(new LegacyRandomSource(level.getSeed())), -4, 1.0)`).
    fn shape_noise(world_seed: u64) -> NormalNoise {
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(world_seed));
        NormalNoise::create(&mut random, -4, &[1.0])
    }

    /// Vanilla geode noise value for testing: `GeodeFeature.place`'s `normalNoise.getValue(x, y, z)`.
    #[cfg(test)]
    pub(crate) fn shape_noise_value(world_seed: u64, x: f64, y: f64, z: f64) -> f64 {
        Self::shape_noise(world_seed).get_value(x, y, z)
    }

    /// Resolves a `BlockWrapper` (block names and/or `#tags`, as vanilla `HolderSet`s) to the
    /// set of block ids it matches.
    fn resolve_block_set(wrapper: &BlockWrapper) -> HashSet<BlockId> {
        let mut ids = HashSet::new();
        let entries: Vec<&str> = match wrapper {
            BlockWrapper::Single(s) => vec![s.as_str()],
            BlockWrapper::Multi(v) => v.iter().map(String::as_str).collect(),
        };
        for entry in entries {
            if let Some(tag) = entry.strip_prefix('#') {
                for name in Block::get_tag_values(tag).into_iter().flatten() {
                    if let Some(block) = Block::from_name(name) {
                        ids.insert(block.id);
                    }
                }
            } else if let Some(block) = Block::from_name(entry) {
                ids.insert(block.id);
            }
        }
        ids
    }

    fn safe_set_block<T: GenerationCache>(
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
        block
            .properties(state.id)
            .is_some_and(|props| props.to_props().iter().any(|(k, _)| *k == property))
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_lines)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        _min_y: i8,
        _height: u16,
        _feature_name: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let origin = pos;
        let num_points = self.distribution_points.get(random);
        let noise = Self::shape_noise(chunk.get_world_seed());

        // Precompute sets of raw block ids for fast lookups (tags included).
        let invalid_raw_ids = Self::resolve_block_set(&self.invalid_blocks);
        let cannot_replace_raw_ids = Self::resolve_block_set(&self.cannot_replace);

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
            let raw_id = raw.to_block_id();
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
            let block_id = state.id.to_block_id();
            state.is_air() || !cannot_replace_raw_ids.contains(&block_id)
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
                Some(Ordering::Greater | Ordering::Equal)
            ) {
                if should_generate_crack
                    && dist_sum_crack >= crack_size
                    && dist_sum_shell < inner_air
                {
                    // Carve crack
                    Self::safe_set_block(
                        chunk,
                        point_inside,
                        BlockStateId::AIR.to_state(),
                        &can_replace_pred,
                    );
                } else if dist_sum_shell >= inner_air {
                    let state =
                        self.filling_provider
                            .get(random, point_inside, chunk, block_registry);
                    Self::safe_set_block(chunk, point_inside, state, &can_replace_pred);
                } else if dist_sum_shell >= innermost_block_layer {
                    let use_alternate = random.next_f32() < self.use_alternate_layer0_chance as f32;
                    if use_alternate {
                        let state = self.alternate_inner_layer_provider.get(
                            random,
                            point_inside,
                            chunk,
                            block_registry,
                        );
                        Self::safe_set_block(chunk, point_inside, state, &can_replace_pred);
                    } else {
                        let state = self.inner_layer_provider.get(
                            random,
                            point_inside,
                            chunk,
                            block_registry,
                        );
                        Self::safe_set_block(chunk, point_inside, state, &can_replace_pred);
                    }
                    if (!self.placements_require_layer0_alternate || use_alternate)
                        && random.next_f32() < self.use_potential_placements_chance as f32
                    {
                        potential_crystal_placements.push(point_inside);
                    }
                } else if dist_sum_shell >= inner_crust {
                    let state =
                        self.middle_layer_provider
                            .get(random, point_inside, chunk, block_registry);
                    Self::safe_set_block(chunk, point_inside, state, &can_replace_pred);
                } else if dist_sum_shell >= outer_crust {
                    let state =
                        self.outer_layer_provider
                            .get(random, point_inside, chunk, block_registry);
                    Self::safe_set_block(chunk, point_inside, state, &can_replace_pred);
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
                    let is_water = place_raw.to_block_id() == BlockId::WATER;

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
                        Self::safe_set_block(chunk, place_pos, final_state, &can_replace_pred);

                        // Only place one crystal per budding block per geode gen
                        break;
                    }
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::feature::configured_features::CONFIGURED_FEATURES;
    use crate::generation::feature::configured_features::ConfiguredFeature;
    use pumpkin_util::math::int_provider::NormalIntProvider;

    fn amethyst_geode() -> &'static GeodeFeature {
        match CONFIGURED_FEATURES
            .get(&pumpkin_data::configured_feature::ConfiguredFeature::AmethystGeode)
        {
            Some(ConfiguredFeature::Geode(feature)) => feature,
            _ => panic!("amethyst_geode is not a geode feature"),
        }
    }

    fn uniform_bounds(provider: &IntProvider) -> (i32, i32) {
        match provider {
            IntProvider::Object(NormalIntProvider::Uniform(uniform)) => {
                (uniform.min_inclusive, uniform.max_inclusive)
            }
            _ => panic!("expected a uniform int provider"),
        }
    }

    /// Vanilla `GeodeConfiguration.CODEC` fills the fields the `amethyst_geode` datapack entry
    /// omits with `UniformInt.of(3, 4)` (`distribution_points`) and `UniformInt.of(1, 2)`
    /// (`point_offset`); `outer_wall_distance` is given explicitly as uniform 4..6.
    #[test]
    fn amethyst_geode_uses_vanilla_codec_defaults() {
        let geode = amethyst_geode();
        assert_eq!(uniform_bounds(&geode.distribution_points), (3, 4));
        assert_eq!(uniform_bounds(&geode.point_offset), (1, 2));
        assert_eq!(uniform_bounds(&geode.outer_wall_distance), (4, 6));
        assert_eq!(geode.invalid_blocks_threshold, 1);
    }

    /// Values printed by the real 26.2 server classes for seed 13579:
    /// `NormalNoise.create(new WorldgenRandom(new LegacyRandomSource(13579L)), -4, 1.0)
    ///     .getValue(x, y, z)` (see `GeodeFeature.place`).
    #[test]
    fn shape_noise_is_seeded_from_the_world_seed() {
        let cases = [
            ((0.0, 0.0, 0.0), 0.293_497_274_324_212_6f64),
            ((-24.0, -17.0, 135.0), -0.141_043_541_033_730_46f64),
            ((-121.0, -28.0, 36.0), -0.041_936_309_071_201_51f64),
            ((100.5, -40.0, -7.0), 0.092_317_015_824_239_66f64),
        ];
        for ((x, y, z), expected) in cases {
            let actual = GeodeFeature::shape_noise_value(13579, x, y, z);
            assert!(
                (actual - expected).abs() < 1e-6,
                "noise at ({x}, {y}, {z}) = {actual}, vanilla {expected}"
            );
        }
    }

    /// Vanilla resolves `#minecraft:geode_invalid_blocks` / `#minecraft:features_cannot_replace`
    /// as `HolderSet`s (`BlockState.is(HolderSet)`); a tag string must not silently resolve to
    /// an empty set.
    #[test]
    fn amethyst_geode_resolves_block_tags() {
        let geode = amethyst_geode();
        let invalid = GeodeFeature::resolve_block_set(&geode.invalid_blocks);
        assert!(invalid.contains(&Block::WATER.id));
        assert!(invalid.contains(&Block::LAVA.id));
        assert!(invalid.contains(&Block::BEDROCK.id));
        assert!(!invalid.contains(&Block::STONE.id));
        let cannot_replace = GeodeFeature::resolve_block_set(&geode.cannot_replace);
        assert!(cannot_replace.contains(&Block::BEDROCK.id));
        assert!(!cannot_replace.contains(&Block::DEEPSLATE.id));
    }
}
