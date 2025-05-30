use serde::Deserialize;
use std::collections::HashMap;
use std::iter;
use std::ops::Deref;
use std::sync::LazyLock;

use pumpkin_util::biome::{FOLIAGE_NOISE, TEMPERATURE_NOISE};
use pumpkin_util::math::int_provider::IntProvider;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{RandomGenerator, RandomImpl};

use crate::generation::block_predicate::BlockPredicate;
use crate::generation::height_provider::HeightProvider;
use crate::world::BlockRegistryExt;
use crate::{HeightMap, ProtoChunk};

use super::configured_features::{CONFIGURED_FEATURES, ConfiguredFeature};

pub static PLACED_FEATURES: LazyLock<HashMap<String, PlacedFeature>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../../assets/placed_feature.json"))
        .expect("Could not parse placed_feature.json registry.")
});

#[derive(Deserialize)]
#[serde(untagged)]
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

#[derive(Deserialize)]
pub struct PlacedFeature {
    /// The name of the configuired feature
    feature: Feature,
    placement: Vec<PlacementModifier>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Feature {
    Named(String),
    Inlined(ConfiguredFeature),
}

impl PlacedFeature {
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
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

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum PlacementModifier {
    #[serde(rename = "minecraft:block_predicate_filter")]
    BlockPredicateFilter(BlockFilterPlacementModifier),
    #[serde(rename = "minecraft:rarity_filter")]
    RarityFilter(RarityFilterPlacementModifier),
    #[serde(rename = "minecraft:surface_relative_threshold_filter")]
    SurfaceRelativeThresholdFilter(SurfaceThresholdFilterPlacementModifier),
    #[serde(rename = "minecraft:surface_water_depth_filter")]
    SurfaceWaterDepthFilter(SurfaceWaterDepthFilterPlacementModifier),
    #[serde(rename = "minecraft:biome")]
    Biome(BiomePlacementModifier),
    #[serde(rename = "minecraft:count")]
    Count(CountPlacementModifier),
    #[serde(rename = "minecraft:noise_based_count")]
    NoiseBasedCount(NoiseBasedCountPlacementModifier),
    #[serde(rename = "minecraft:noise_threshold_count")]
    NoiseThresholdCount(NoiseThresholdCountPlacementModifier),
    #[serde(rename = "minecraft:count_on_every_layer")]
    CountOnEveryLayer,
    #[serde(rename = "minecraft:environment_scan")]
    EnvironmentScan,
    #[serde(rename = "minecraft:heightmap")]
    Heightmap(HeightmapPlacementModifier),
    #[serde(rename = "minecraft:height_range")]
    HeightRange(HeightRangePlacementModifier),
    #[serde(rename = "minecraft:in_square")]
    InSquare(SquarePlacementModifier),
    #[serde(rename = "minecraft:random_offset")]
    RandomOffset(RandomOffsetPlacementModifier),
    #[serde(rename = "minecraft:fixed_placement")]
    FixedPlacement,
}

impl PlacementModifier {
    pub fn get_positions(
        &self,
        chunk: &ProtoChunk,
        block_registry: &dyn BlockRegistryExt,
        min_y: i8,
        height: u16,
        feature: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> Box<dyn Iterator<Item = BlockPos>> {
        match self {
            PlacementModifier::BlockPredicateFilter(modifier) => {
                modifier.get_positions(block_registry, chunk, feature, random, pos)
            }
            PlacementModifier::RarityFilter(modifier) => {
                modifier.get_positions(block_registry, chunk, feature, random, pos)
            }
            PlacementModifier::SurfaceRelativeThresholdFilter(modifier) => {
                modifier.get_positions(block_registry, chunk, feature, random, pos)
            }
            PlacementModifier::SurfaceWaterDepthFilter(modifier) => {
                modifier.get_positions(block_registry, chunk, feature, random, pos)
            }
            PlacementModifier::Biome(modifier) => {
                modifier.get_positions(block_registry, chunk, feature, random, pos)
            }
            PlacementModifier::Count(modifier) => modifier.get_positions(random, pos),
            PlacementModifier::NoiseBasedCount(modifier) => {
                Box::new(modifier.get_positions(random, pos))
            }
            PlacementModifier::NoiseThresholdCount(feature) => feature.get_positions(random, pos),
            PlacementModifier::CountOnEveryLayer => Box::new(iter::empty()),
            PlacementModifier::EnvironmentScan => Box::new(iter::empty()),
            PlacementModifier::Heightmap(modifier) => {
                modifier.get_positions(chunk, min_y, height, random, pos)
            }
            PlacementModifier::HeightRange(modifier) => {
                modifier.get_positions(min_y, height, random, pos)
            }
            PlacementModifier::InSquare(_) => SquarePlacementModifier::get_positions(random, pos),
            PlacementModifier::RandomOffset(modifier) => modifier.get_positions(random, pos),
            PlacementModifier::FixedPlacement => Box::new(iter::empty()),
        }
    }
}

#[derive(Deserialize)]
pub struct NoiseBasedCountPlacementModifier {
    noise_to_count_ratio: i32,
    noise_factor: f64,
    noise_offset: f64,
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

#[derive(Deserialize)]
pub struct NoiseThresholdCountPlacementModifier {
    noise_level: f64,
    below_noise: i32,
    above_noise: i32,
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

#[derive(Deserialize)]
pub struct RandomOffsetPlacementModifier {
    xz_spread: IntProvider,
    y_spread: IntProvider,
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

#[derive(Deserialize)]
pub struct BlockFilterPlacementModifier {
    predicate: BlockPredicate,
}

impl ConditionalPlacementModifier for BlockFilterPlacementModifier {
    fn should_place(
        &self,
        block_registry: &dyn BlockRegistryExt,
        _feature: &str,
        chunk: &ProtoChunk,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        self.predicate.test(block_registry, chunk, &pos)
    }
}

#[derive(Deserialize)]
pub struct SurfaceThresholdFilterPlacementModifier {
    heightmap: HeightMap,
    min_inclusive: Option<i32>,
    max_inclusive: Option<i32>,
}

impl ConditionalPlacementModifier for SurfaceThresholdFilterPlacementModifier {
    fn should_place(
        &self,
        _block_registry: &dyn BlockRegistryExt,
        _feature: &str,
        chunk: &ProtoChunk,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let y = chunk.get_top_y(&self.heightmap, &pos.0.to_vec2_i32());
        let min = y.saturating_add(self.min_inclusive.unwrap_or(i32::MIN) as i64);
        let max = y.saturating_add(self.max_inclusive.unwrap_or(i32::MAX) as i64);
        min <= pos.0.y as i64 && pos.0.y as i64 <= max
    }
}

#[derive(Deserialize)]
pub struct RarityFilterPlacementModifier {
    chance: u32,
}

impl ConditionalPlacementModifier for RarityFilterPlacementModifier {
    fn should_place(
        &self,
        block_registry: &dyn BlockRegistryExt,
        feature: &str,
        chunk: &ProtoChunk,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        random.next_f32() < 1.0 / self.chance as f32
    }
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub struct CountPlacementModifier {
    count: IntProvider,
}

impl CountPlacementModifierBase for CountPlacementModifier {
    fn get_count(&self, random: &mut RandomGenerator, _pos: BlockPos) -> i32 {
        self.count.get(random)
    }
}

#[derive(Deserialize)]
pub struct SurfaceWaterDepthFilterPlacementModifier {
    max_water_depth: i32,
}

impl ConditionalPlacementModifier for SurfaceWaterDepthFilterPlacementModifier {
    fn should_place(
        &self,
        block_registry: &dyn BlockRegistryExt,

        _feature: &str,
        chunk: &ProtoChunk,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let world_top = chunk.top_block_height_exclusive(&Vector2::new(pos.0.x, pos.0.z)) as i32;
        let ocean = chunk.ocean_floor_height_exclusive(&Vector2::new(pos.0.x, pos.0.z)) as i32;
        world_top - ocean <= self.max_water_depth
    }
}

#[derive(Deserialize)]
pub struct BiomePlacementModifier;

impl ConditionalPlacementModifier for BiomePlacementModifier {
    fn should_place(
        &self,
        _block_registry: &dyn BlockRegistryExt,

        this_feature: &str,
        chunk: &ProtoChunk,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        // we check if the current feature can be applied to the biome at the pos
        let name = format!("minecraft:{this_feature}");
        let biome = chunk.get_biome_for_terrain_gen(&pos.0);

        for feature in biome.features {
            if feature.contains(&name.deref()) {
                return true;
            }
        }
        false
    }
}

#[derive(Deserialize)]
pub struct HeightRangePlacementModifier {
    height: HeightProvider,
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

#[derive(Deserialize)]
pub struct HeightmapPlacementModifier {
    heightmap: HeightMap,
}

impl HeightmapPlacementModifier {
    pub fn get_positions(
        &self,
        chunk: &ProtoChunk,
        min_y: i8,
        height: u16,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> Box<dyn Iterator<Item = BlockPos>> {
        let x = pos.0.x;
        let z = pos.0.z;
        let top = chunk.get_top_y(&self.heightmap, &Vector2::new(x, z)) as i32;
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
    fn get_positions(
        &self,
        block_registry: &dyn BlockRegistryExt,
        chunk: &ProtoChunk,
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

    fn should_place(
        &self,
        block_registry: &dyn BlockRegistryExt,

        feature: &str,
        chunk: &ProtoChunk,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool;
}
