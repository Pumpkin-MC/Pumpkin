use pumpkin_data::block_properties::is_air;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::HeightMap;
use std::collections::HashMap;
use std::iter;
use std::sync::LazyLock;

use pumpkin_util::biome::FOLIAGE_NOISE;
use pumpkin_util::math::int_provider::IntProvider;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{RandomGenerator, RandomImpl};

use crate::block::RawBlockState;
use crate::generation::block_predicate::BlockPredicate;
use crate::generation::height_provider::HeightProvider;
use crate::generation::proto_chunk::GenerationCache;
use crate::world::BlockRegistryExt;

use super::configured_features::{CONFIGURED_FEATURES, ConfiguredFeature};

pub static PLACED_FEATURES: LazyLock<HashMap<String, PlacedFeature>> =
    LazyLock::new(build_placed_features);

pub enum PlacedFeatureWrapper {
    Direct(PlacedFeature),
    Named(String),
}

impl PlacedFeatureWrapper {
    pub fn get(&self) -> &PlacedFeature {
        match self {
            Self::Named(name) => PLACED_FEATURES
                .get(name.strip_prefix("minecraft:").unwrap_or(name))
                .unwrap(),
            Self::Direct(feature) => feature,
        }
    }
}

pub struct PlacedFeature {
    /// The name of the configuired feature
    pub feature: Feature,
    pub placement: Vec<PlacementModifier>,
}

pub enum Feature {
    Named(String),
    Inlined(Box<ConfiguredFeature>),
}

impl PlacedFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn BlockRegistryExt,
        min_y: i8,
        height: u16,
        feature_name: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut stream: Vec<BlockPos> = vec![pos];
        for modifier in &self.placement {
            let mut new_stream = Vec::with_capacity(stream.len());

            for block_pos in stream {
                let positions = modifier.get_positions(
                    chunk,
                    block_registry,
                    min_y,
                    height,
                    feature_name,
                    random,
                    block_pos,
                );
                new_stream.extend(positions);
            }

            stream = new_stream;
        }

        let feature = match &self.feature {
            Feature::Named(name) => CONFIGURED_FEATURES
                .get(name.strip_prefix("minecraft:").unwrap_or(name))
                .unwrap(),
            Feature::Inlined(feature) => feature,
        };

        let mut ret = false;
        for pos in stream {
            if feature.generate(
                chunk,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ) {
                ret = true;
            }
        }
        ret
    }
}

pub enum PlacementModifier {
    BlockPredicateFilter(BlockFilterPlacementModifier),
    RarityFilter(RarityFilterPlacementModifier),
    SurfaceRelativeThresholdFilter(SurfaceThresholdFilterPlacementModifier),
    SurfaceWaterDepthFilter(SurfaceWaterDepthFilterPlacementModifier),
    Biome(BiomePlacementModifier),
    Count(CountPlacementModifier),
    NoiseBasedCount(NoiseBasedCountPlacementModifier),
    NoiseThresholdCount(NoiseThresholdCountPlacementModifier),
    CountOnEveryLayer(CountOnEveryLayerPlacementModifier),
    EnvironmentScan(EnvironmentScanPlacementModifier),
    Heightmap(HeightmapPlacementModifier),
    HeightRange(HeightRangePlacementModifier),
    InSquare(SquarePlacementModifier),
    RandomOffset(RandomOffsetPlacementModifier),
    FixedPlacement,
}

impl PlacementModifier {
    #[expect(clippy::too_many_arguments)]
    pub fn get_positions<T: GenerationCache>(
        &self,
        chunk: &T,
        block_registry: &dyn BlockRegistryExt,
        min_y: i8,
        height: u16,
        feature: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> Box<dyn Iterator<Item = BlockPos>> {
        match self {
            Self::BlockPredicateFilter(modifier) => {
                modifier.get_positions(block_registry, chunk, feature, random, pos)
            }
            Self::RarityFilter(modifier) => {
                modifier.get_positions(block_registry, chunk, feature, random, pos)
            }
            Self::SurfaceRelativeThresholdFilter(modifier) => {
                modifier.get_positions(block_registry, chunk, feature, random, pos)
            }
            Self::SurfaceWaterDepthFilter(modifier) => {
                modifier.get_positions(block_registry, chunk, feature, random, pos)
            }
            Self::Biome(modifier) => {
                modifier.get_positions(block_registry, chunk, feature, random, pos)
            }
            Self::Count(modifier) => modifier.get_positions(random, pos),
            Self::NoiseBasedCount(modifier) => Box::new(modifier.get_positions(random, pos)),
            Self::NoiseThresholdCount(modifier) => modifier.get_positions(random, pos),
            Self::CountOnEveryLayer(modifier) => modifier.get_positions(random, chunk, pos),
            Self::EnvironmentScan(modifier) => modifier.get_positions(chunk, block_registry, pos),
            Self::Heightmap(modifier) => modifier.get_positions(chunk, min_y, height, random, pos),
            Self::HeightRange(modifier) => modifier.get_positions(min_y, height, random, pos),
            Self::InSquare(_) => SquarePlacementModifier::get_positions(random, pos),
            Self::RandomOffset(modifier) => modifier.get_positions(random, pos),
            Self::FixedPlacement => Box::new(iter::empty()),
        }
    }
}

pub struct NoiseBasedCountPlacementModifier {
    pub noise_to_count_ratio: i32,
    pub noise_factor: f64,
    pub noise_offset: f64,
}

impl CountPlacementModifierBase for NoiseBasedCountPlacementModifier {
    fn get_count(&self, _random: &mut RandomGenerator, pos: BlockPos) -> i32 {
        let noise = FOLIAGE_NOISE
            .sample(
                pos.0.x as f64 / self.noise_factor,
                pos.0.z as f64 / self.noise_factor,
                false,
            )
            .max(0.0); // TODO: max is wrong
        ((noise + self.noise_offset) * self.noise_to_count_ratio as f64).ceil() as i32
    }
}

pub struct NoiseThresholdCountPlacementModifier {
    pub noise_level: f64,
    pub below_noise: i32,
    pub above_noise: i32,
}

impl CountPlacementModifierBase for NoiseThresholdCountPlacementModifier {
    fn get_count(&self, _random: &mut RandomGenerator, pos: BlockPos) -> i32 {
        let noise = FOLIAGE_NOISE.sample(pos.0.x as f64 / 200.0, pos.0.z as f64 / 200.0, false);
        if noise < self.noise_level {
            self.below_noise
        } else {
            self.above_noise
        }
    }
}

pub struct EnvironmentScanPlacementModifier {
    pub direction_of_search: BlockDirection,
    pub target_condition: BlockPredicate,
    pub allowed_search_condition: Option<BlockPredicate>,
    pub max_steps: i32,
}

impl EnvironmentScanPlacementModifier {
    pub fn get_positions<T: GenerationCache>(
        &self,
        chunk: &T,
        block_registry: &dyn BlockRegistryExt,
        pos: BlockPos,
    ) -> Box<dyn Iterator<Item = BlockPos>> {
        let allowed_search_condition = self
            .allowed_search_condition
            .as_ref()
            .unwrap_or(&BlockPredicate::AlwaysTrue);

        if !allowed_search_condition.test(block_registry, chunk, &pos) {
            return Box::new(iter::empty());
        }
        let mut pos = pos;
        for _ in 0..self.max_steps {
            if self.target_condition.test(block_registry, chunk, &pos) {
                return Box::new(iter::once(pos));
            }
            pos = pos.offset(self.direction_of_search.to_offset());

            if chunk.out_of_height(pos.0.y as i16) {
                return Box::new(iter::empty());
            }

            if !allowed_search_condition.test(block_registry, chunk, &pos) {
                break;
            }
        }
        if self.target_condition.test(block_registry, chunk, &pos) {
            return Box::new(iter::once(pos));
        }

        Box::new(iter::empty())
    }
}

pub struct RandomOffsetPlacementModifier {
    pub xz_spread: IntProvider,
    pub y_spread: IntProvider,
}

impl RandomOffsetPlacementModifier {
    pub fn get_positions(
        &self,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> Box<dyn Iterator<Item = BlockPos>> {
        let x = pos.0.x + self.xz_spread.get(random);
        let y = pos.0.y + self.y_spread.get(random);
        let z = pos.0.z + self.xz_spread.get(random);
        Box::new(iter::once(BlockPos(Vector3::new(x, y, z))))
    }
}

pub struct CountOnEveryLayerPlacementModifier {
    pub count: IntProvider,
}

impl CountOnEveryLayerPlacementModifier {
    pub fn get_positions<T: GenerationCache>(
        &self,
        random: &mut RandomGenerator,
        chunk: &T,
        pos: BlockPos,
    ) -> Box<dyn Iterator<Item = BlockPos>> {
        let mut positions = Vec::new(); // Using a Vec to collect results, analogous to Stream.builder()
        let mut i = 0; // Represents the 'targetY' in findPos
        let mut bl;

        loop {
            bl = false;
            for _j in 0..self.count.get(random) {
                let x = random.next_bounded_i32(16) + pos.0.x;
                let z = random.next_bounded_i32(16) + pos.0.z;
                let y = chunk.top_motion_blocking_block_height_exclusive(x, z);

                let n = Self::find_pos(chunk, x, y, z, i);

                if n == i32::MAX {
                    continue;
                }
                positions.push(BlockPos::new(x, n, z));
                bl = true;
            }
            i += 1;
            if !bl {
                break;
            }
        }
        Box::new(positions.into_iter())
    }

    fn find_pos<T: GenerationCache>(chunk: &T, x: i32, y: i32, z: i32, target_y: i32) -> i32 {
        let mut mutable_pos = BlockPos::new(x, y, z);
        let mut found_count = 0;
        let mut current_block_state = GenerationCache::get_block_state(chunk, &mutable_pos.0);

        for j in (chunk.bottom_y() as i32 + 1..=y).rev() {
            mutable_pos.0.y = j - 1;
            let next_block_state = GenerationCache::get_block_state(chunk, &mutable_pos.0);

            if !Self::blocks_spawn(&next_block_state)
                && Self::blocks_spawn(&current_block_state)
                && next_block_state.to_block_id() != Block::BEDROCK
            {
                if found_count == target_y {
                    return mutable_pos.0.y + 1;
                }
                found_count += 1;
            }
            current_block_state = next_block_state;
        }
        i32::MAX
    }

    fn blocks_spawn(state: &RawBlockState) -> bool {
        let block = state.to_block_id();
        is_air(state.0) || block == Block::WATER || block == Block::LAVA
    }
}

pub struct BlockFilterPlacementModifier {
    pub predicate: BlockPredicate,
}

impl ConditionalPlacementModifier for BlockFilterPlacementModifier {
    fn should_place<T: GenerationCache>(
        &self,
        block_registry: &dyn BlockRegistryExt,
        _feature: &str,
        chunk: &T,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        self.predicate.test(block_registry, chunk, &pos)
    }
}

pub struct SurfaceThresholdFilterPlacementModifier {
    pub heightmap: HeightMap,
    pub min_inclusive: Option<i32>,
    pub max_inclusive: Option<i32>,
}

impl ConditionalPlacementModifier for SurfaceThresholdFilterPlacementModifier {
    fn should_place<T: GenerationCache>(
        &self,
        _block_registry: &dyn BlockRegistryExt,
        _feature: &str,
        chunk: &T,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let y = chunk.get_top_y(&self.heightmap, pos.0.x, pos.0.z);
        let min = y.saturating_add(self.min_inclusive.unwrap_or(i32::MIN));
        let max = y.saturating_add(self.max_inclusive.unwrap_or(i32::MAX));
        min <= pos.0.y && pos.0.y <= max
    }
}

pub struct RarityFilterPlacementModifier {
    pub chance: u32,
}

impl ConditionalPlacementModifier for RarityFilterPlacementModifier {
    fn should_place<T: GenerationCache>(
        &self,
        _block_registry: &dyn BlockRegistryExt,
        _feature: &str,
        _chunk: &T,
        random: &mut RandomGenerator,
        _pos: BlockPos,
    ) -> bool {
        random.next_f32() < 1.0 / self.chance as f32
    }
}

pub struct SquarePlacementModifier;

impl SquarePlacementModifier {
    pub fn get_positions(
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> Box<dyn Iterator<Item = BlockPos>> {
        let x = random.next_bounded_i32(16) + pos.0.x;
        let z = random.next_bounded_i32(16) + pos.0.z;
        Box::new(iter::once(BlockPos(Vector3::new(x, pos.0.y, z))))
    }
}

pub struct CountPlacementModifier {
    pub count: IntProvider,
}

impl CountPlacementModifierBase for CountPlacementModifier {
    fn get_count(&self, random: &mut RandomGenerator, _pos: BlockPos) -> i32 {
        self.count.get(random)
    }
}

pub struct SurfaceWaterDepthFilterPlacementModifier {
    pub max_water_depth: i32,
}

impl ConditionalPlacementModifier for SurfaceWaterDepthFilterPlacementModifier {
    fn should_place<T: GenerationCache>(
        &self,
        _block_registry: &dyn BlockRegistryExt,
        _feature: &str,
        chunk: &T,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let world_top = chunk.top_block_height_exclusive(pos.0.x, pos.0.z);
        let ocean = chunk.ocean_floor_height_exclusive(pos.0.x, pos.0.z);
        world_top - ocean <= self.max_water_depth
    }
}

pub struct BiomePlacementModifier;

impl ConditionalPlacementModifier for BiomePlacementModifier {
    fn should_place<T: GenerationCache>(
        &self,
        _block_registry: &dyn BlockRegistryExt,
        this_feature: &str,
        chunk: &T,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let name = format!("minecraft:{this_feature}");
        let biome = chunk.get_biome_for_terrain_gen(pos.0.x, pos.0.y, pos.0.z);

        for feature in biome.features {
            if feature.contains(&&*name) {
                return true;
            }
        }
        false
    }
}

pub struct HeightRangePlacementModifier {
    pub height: HeightProvider,
}

impl HeightRangePlacementModifier {
    pub fn get_positions(
        &self,
        min_y: i8,
        height: u16,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> Box<dyn Iterator<Item = BlockPos>> {
        let mut pos = pos;
        pos.0.y = self.height.get(random, min_y, height);
        Box::new(iter::once(pos))
    }
}

pub struct HeightmapPlacementModifier {
    pub heightmap: HeightMap,
}

impl HeightmapPlacementModifier {
    pub fn get_positions<T: GenerationCache>(
        &self,
        chunk: &T,
        min_y: i8,
        _height: u16,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> Box<dyn Iterator<Item = BlockPos>> {
        let x = pos.0.x;
        let z = pos.0.z;
        let top = chunk.get_top_y(&self.heightmap, x, z);
        if top > min_y as i32 {
            return Box::new(iter::once(BlockPos(Vector3::new(x, top, z))));
        }
        Box::new(iter::empty())
    }
}

pub trait CountPlacementModifierBase {
    fn get_positions(
        &self,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> Box<dyn Iterator<Item = BlockPos>> {
        let count = self.get_count(random, pos);
        Box::new(std::iter::repeat_n(pos, count as usize))
    }

    fn get_count(&self, random: &mut RandomGenerator, pos: BlockPos) -> i32;
}

pub trait ConditionalPlacementModifier {
    fn get_positions<T: GenerationCache>(
        &self,
        block_registry: &dyn BlockRegistryExt,
        chunk: &T,
        feature: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> Box<dyn Iterator<Item = BlockPos>> {
        if self.should_place(block_registry, feature, chunk, random, pos) {
            Box::new(iter::once(pos))
        } else {
            Box::new(iter::empty())
        }
    }

    fn should_place<T: GenerationCache>(
        &self,
        block_registry: &dyn BlockRegistryExt,
        feature: &str,
        chunk: &T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool;
}

// generated code is now placed alongside other codegen outputs
// in `src/generated` so it’s easier to find when upgrading MC versions.
// the path is relative to this file (up two levels to reach `src`).
include!("../../../../pumpkin-data/src/generated/placed_features_generated.rs");
