use pumpkin_util::{
    biome::TEMPERATURE_NOISE,
    math::{int_provider::IntProvider, position::BlockPos, vector2::Vector2, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;
use std::{collections::HashMap, iter, sync::LazyLock};

use crate::{
    ProtoChunk,
    generation::{block_predicate::BlockPredicate, height_provider::HeightProvider},
};

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
            Self::Named(name) => PLACED_FEATURES.get(name).unwrap(),
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
        min_y: i8,
        height: u16,
        feature_name: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut stream: Vec<BlockPos> = vec![pos];

        for modifier in &self.placement {
            let mut next_stream: Vec<BlockPos> = Vec::new();
            for posx in stream {
                if let Some(positions) = modifier.get_positions(
                    chunk,
                    min_y,
                    height,
                    feature_name,
                    random,
                    BlockPos(Vector3::new(pos.0.x, pos.0.y, pos.0.z)),
                ) {
                    next_stream.extend(positions);
                }
            }
            stream = next_stream;
        }
        let feature = match &self.feature {
            Feature::Named(name) => CONFIGURED_FEATURES
                .get(&name.replace("minecraft:", ""))
                .unwrap(),
            Feature::Inlined(feature) => feature,
        };
        let mut ret = false;
        for pos in stream {
            if feature.generate(chunk, min_y, height, feature_name, random, pos) {
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
    SurfaceRelativeThresholdFilter,
    #[serde(rename = "minecraft:surface_water_depth_filter")]
    SurfaceWaterDepthFilter(SurfaceWaterDepthFilterPlacementModifier),
    #[serde(rename = "minecraft:biome")]
    Biome(BiomePlacementModifier),
    #[serde(rename = "minecraft:count")]
    Count(CountPlacementModifier),
    #[serde(rename = "minecraft:noise_based_count")]
    NoiseBasedCount,
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
        min_y: i8,
        height: u16,
        feature: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> Option<impl Iterator<Item = BlockPos>> {
        match self {
            PlacementModifier::BlockPredicateFilter(modifier) => {
                modifier.get_positions(chunk, feature, random, pos)
            }
            PlacementModifier::RarityFilter(modifier) => {
                modifier.get_positions(chunk, feature, random, pos)
            }
            PlacementModifier::SurfaceRelativeThresholdFilter => None,
            PlacementModifier::SurfaceWaterDepthFilter(modifier) => {
                modifier.get_positions(chunk, feature, random, pos)
            }
            PlacementModifier::Biome(modifier) => {
                modifier.get_positions(chunk, feature, random, pos)
            }
            PlacementModifier::Count(modifier) => Some(modifier.get_positions(random, pos)),
            PlacementModifier::NoiseBasedCount => None,
            PlacementModifier::NoiseThresholdCount(feature) => {
                Some(feature.get_positions(random, pos))
            }
            PlacementModifier::CountOnEveryLayer => None,
            PlacementModifier::EnvironmentScan => None,
            PlacementModifier::Heightmap(modifier) => {
                modifier.get_positions(chunk, min_y, height, random, pos)
            }
            PlacementModifier::HeightRange(modifier) => {
                Some(modifier.get_positions(min_y, height, random, pos))
            }
            PlacementModifier::InSquare(_) => {
                Some(SquarePlacementModifier::get_positions(random, pos))
            }
            PlacementModifier::RandomOffset(modifier) => Some(modifier.get_positions(random, pos)),
            PlacementModifier::FixedPlacement => None,
        }
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
        let noise = TEMPERATURE_NOISE.sample(pos.0.x as f64 / 200.0, pos.0.z as f64, false);
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
        feature: &str,
        chunk: &ProtoChunk,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        // :crying
        self.predicate.test(chunk, &pos)
    }
}

#[derive(Deserialize)]
pub struct RarityFilterPlacementModifier {
    chance: u32,
}

impl ConditionalPlacementModifier for RarityFilterPlacementModifier {
    fn should_place(
        &self,
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
        feature: &str,
        chunk: &ProtoChunk,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let top = chunk.top_block_height_exclusive(&Vector2::new(pos.0.x, pos.0.z));
        // TODO: This is nonsense and broken, add ocean floor
        top <= self.max_water_depth
    }
}

#[derive(Deserialize)]
pub struct BiomePlacementModifier;

impl ConditionalPlacementModifier for BiomePlacementModifier {
    fn should_place(
        &self,
        feature: &str,
        chunk: &ProtoChunk,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        //we check if the current feature can be applied to the biome at the pos
        let features = chunk.get_biome(&pos.0).features.first().unwrap();
        let this_feature = &feature;
        features.contains(this_feature)
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
    heightmap: String,
}

impl HeightmapPlacementModifier {
    pub fn get_positions(
        &self,
        chunk: &ProtoChunk,
        min_y: i8,
        height: u16,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> Option<Box<dyn Iterator<Item = BlockPos>>> {
        let x = pos.0.x;
        let z = pos.0.z;
        let top = chunk.top_block_height_exclusive(&Vector2::new(x, z));
        if top > min_y as i32 {
            return Some(Box::new(iter::once(BlockPos(Vector3::new(x, top, z)))));
        }
        None
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
        chunk: &ProtoChunk,
        feature: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> Option<Box<dyn Iterator<Item = BlockPos>>> {
        if self.should_place(feature, chunk, random, pos) {
            Some(Box::new(iter::once(pos)))
        } else {
            None
        }
    }

    fn should_place(
        &self,
        feature: &str,
        chunk: &ProtoChunk,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool;
}
