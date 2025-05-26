use acacia::AcaciaFoliagePlacer;
use blob::BlobFoliagePlacer;
use bush::BushFoliagePlacer;
use fancy::LargeOakFoliagePlacer;
use pumpkin_data::BlockState;
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos, vector3::Vector3},
    random::RandomGenerator,
};
use serde::Deserialize;

use crate::ProtoChunk;

use super::{TreeFeature, TreeNode};

mod acacia;
mod blob;
mod bush;
mod fancy;

#[derive(Deserialize)]
pub struct FoliagePlacer {
    radius: IntProvider,
    offset: IntProvider,
    #[serde(flatten)]
    pub r#type: FoliageType,
}

pub trait LeaveValidator {
    fn is_position_invalid(
        &self,
        random: &mut RandomGenerator,
        dx: i32,
        y: i32,
        dz: i32,
        radius: i32,
        giant_trunk: bool,
    ) -> bool {
        let x = if giant_trunk {
            dx.abs().min((dx - 1).abs())
        } else {
            dx.abs()
        };
        let z = if giant_trunk {
            dz.abs().min((dz - 1).abs())
        } else {
            dz.abs()
        };
        self.is_invalid_for_leaves(random, x, y, z, radius, giant_trunk)
    }

    fn is_invalid_for_leaves(
        &self,
        random: &mut RandomGenerator,
        dx: i32,
        y: i32,
        dz: i32,
        radius: i32,
        giant_trunk: bool,
    ) -> bool;
}

impl FoliagePlacer {
    pub fn generate_square<T: LeaveValidator>(
        validator: &T,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        center_pos: BlockPos,
        radius: i32,
        y: i32,
        giant_trunk: bool,
        foliage_provider: &BlockState,
    ) {
        let i = if giant_trunk { 1 } else { 0 };

        for x in -radius..=(radius + i) {
            for z in -radius..=(radius + i) {
                if validator.is_position_invalid(random, x, y, z, radius, giant_trunk) {
                    continue;
                }
                let pos = center_pos.offset(center_pos.0.add(&Vector3::new(x, y, z)));
                Self::place_foliage_block(chunk, pos, foliage_provider);
            }
        }
    }

    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        node: &TreeNode,
        foliage_height: i32,
        radius: i32,
        foliage_provider: &BlockState,
    ) {
        let offset = self.offset.get(random);
        self.r#type.generate(
            chunk,
            random,
            node,
            foliage_height,
            radius,
            offset,
            foliage_provider,
        );
    }

    pub fn get_random_radius(&self, random: &mut RandomGenerator) -> i32 {
        self.radius.get(random)
    }

    fn place_foliage_block(chunk: &mut ProtoChunk, pos: BlockPos, block_state: &BlockState) {
        if !TreeFeature::can_replace(chunk, &pos) {
            return;
        }
        chunk.set_block_state(&pos.0, block_state);
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum FoliageType {
    #[serde(rename = "minecraft:blob_foliage_placer")]
    Blob(BlobFoliagePlacer),
    #[serde(rename = "minecraft:spruce_foliage_placer")]
    Spruce,
    #[serde(rename = "minecraft:pine_foliage_placer")]
    Pine,
    #[serde(rename = "minecraft:acacia_foliage_placer")]
    Acacia(AcaciaFoliagePlacer),
    #[serde(rename = "minecraft:bush_foliage_placer")]
    Bush(BushFoliagePlacer),
    #[serde(rename = "minecraft:fancy_foliage_placer")]
    Fancy(LargeOakFoliagePlacer),
    #[serde(rename = "minecraft:jungle_foliage_placer")]
    Jungle,
    #[serde(rename = "minecraft:mega_pine_foliage_placer")]
    MegaPine,
    #[serde(rename = "minecraft:dark_oak_foliage_placer")]
    DarkOak,
    #[serde(rename = "minecraft:random_spread_foliage_placer")]
    RandomSpread,
    #[serde(rename = "minecraft:cherry_foliage_placer")]
    Cherry,
}

impl FoliageType {
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        node: &TreeNode,
        foliage_height: i32,
        radius: i32,
        offset: i32,
        foliage_provider: &BlockState,
    ) {
        match self {
            FoliageType::Blob(blob) => blob.generate(
                chunk,
                random,
                node,
                foliage_height,
                radius,
                offset,
                foliage_provider,
            ),
            FoliageType::Spruce => {}
            FoliageType::Pine => {}
            FoliageType::Acacia(acacia) => acacia.generate(
                chunk,
                random,
                node,
                foliage_height,
                radius,
                offset,
                foliage_provider,
            ),
            FoliageType::Bush(bush) => bush.generate(
                chunk,
                random,
                node,
                foliage_height,
                radius,
                offset,
                foliage_provider,
            ),
            FoliageType::Fancy(fancy) => fancy.generate(
                chunk,
                random,
                node,
                foliage_height,
                radius,
                offset,
                foliage_provider,
            ),
            FoliageType::Jungle => {}
            FoliageType::MegaPine => {}
            FoliageType::DarkOak => {}
            FoliageType::RandomSpread => {}
            FoliageType::Cherry => {}
        }
    }

    pub fn get_random_height(&self, random: &mut RandomGenerator) -> i32 {
        match self {
            FoliageType::Blob(blob) => blob.get_random_height(random),
            FoliageType::Spruce => 0,
            FoliageType::Pine => 0,
            FoliageType::Acacia(acacia) => acacia.get_random_height(random),
            FoliageType::Bush(bush) => bush.get_random_height(random),
            FoliageType::Fancy(fancy) => fancy.get_random_height(random),
            FoliageType::Jungle => 0,
            FoliageType::MegaPine => 0,
            FoliageType::DarkOak => 0,
            FoliageType::RandomSpread => 0,
            FoliageType::Cherry => 0,
        }
    }
}
