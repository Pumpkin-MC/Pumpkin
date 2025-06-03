use std::sync::Arc;

use acacia::AcaciaFoliagePlacer;
use blob::BlobFoliagePlacer;
use bush::BushFoliagePlacer;
use cherry::CherryFoliagePlacer;
use dark_oak::DarkOakFoliagePlacer;
use fancy::LargeOakFoliagePlacer;
use jungle::JungleFoliagePlacer;
use mega_pine::MegaPineFoliagePlacer;
use pine::PineFoliagePlacer;
use pumpkin_data::BlockState;
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos, vector3::Vector3},
    random::RandomGenerator,
};
use random_spread::RandomSpreadFoliagePlacer;
use serde::Deserialize;
use spruce::SpruceFoliagePlacer;

use crate::{ProtoChunk, level::Level};

use super::{TreeFeature, TreeNode};

mod acacia;
mod blob;
mod bush;
mod cherry;
mod dark_oak;
mod fancy;
mod jungle;
mod mega_pine;
mod pine;
mod random_spread;
mod spruce;

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
    #[expect(clippy::too_many_arguments)]
    pub async fn generate_square<T: LeaveValidator>(
        validator: &T,
        chunk: &mut ProtoChunk<'_>,
        level: &Arc<Level>,
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
                let pos = BlockPos(center_pos.0.add(&Vector3::new(x, y, z)));
                Self::place_foliage_block(chunk, level, pos, foliage_provider).await;
            }
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub async fn generate(
        &self,
        chunk: &mut ProtoChunk<'_>,
        level: &Arc<Level>,
        random: &mut RandomGenerator,
        node: &TreeNode,
        foliage_height: i32,
        radius: i32,
        foliage_provider: &BlockState,
    ) {
        let offset = self.offset.get(random);
        self.r#type
            .generate(
                chunk,
                level,
                random,
                node,
                foliage_height,
                radius,
                offset,
                foliage_provider,
            )
            .await;
    }

    pub fn get_random_radius(&self, random: &mut RandomGenerator) -> i32 {
        self.radius.get(random)
    }

    pub async fn place_foliage_block(
        chunk: &mut ProtoChunk<'_>,
        _level: &Arc<Level>,
        pos: BlockPos,
        block_state: &BlockState,
    ) {
        let block = chunk.get_block_state(&pos.0);
        if !TreeFeature::can_replace(&block.to_state(), &block.to_block()) {
            return;
        }
        if chunk.chunk_pos == pos.chunk_and_chunk_relative_position().0 {
            chunk.set_block_state(&pos.0, block_state);
        } else {
            // level.set_block_state(&pos, block_state.id).await;
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum FoliageType {
    #[serde(rename = "minecraft:blob_foliage_placer")]
    Blob(BlobFoliagePlacer),
    #[serde(rename = "minecraft:spruce_foliage_placer")]
    Spruce(SpruceFoliagePlacer),
    #[serde(rename = "minecraft:pine_foliage_placer")]
    Pine(PineFoliagePlacer),
    #[serde(rename = "minecraft:acacia_foliage_placer")]
    Acacia(AcaciaFoliagePlacer),
    #[serde(rename = "minecraft:bush_foliage_placer")]
    Bush(BushFoliagePlacer),
    #[serde(rename = "minecraft:fancy_foliage_placer")]
    Fancy(LargeOakFoliagePlacer),
    #[serde(rename = "minecraft:jungle_foliage_placer")]
    Jungle(JungleFoliagePlacer),
    #[serde(rename = "minecraft:mega_pine_foliage_placer")]
    MegaPine(MegaPineFoliagePlacer),
    #[serde(rename = "minecraft:dark_oak_foliage_placer")]
    DarkOak(DarkOakFoliagePlacer),
    #[serde(rename = "minecraft:random_spread_foliage_placer")]
    RandomSpread(RandomSpreadFoliagePlacer),
    #[serde(rename = "minecraft:cherry_foliage_placer")]
    Cherry(CherryFoliagePlacer),
}

impl FoliageType {
    #[expect(clippy::too_many_arguments)]
    pub async fn generate(
        &self,
        chunk: &mut ProtoChunk<'_>,
        level: &Arc<Level>,
        random: &mut RandomGenerator,
        node: &TreeNode,
        foliage_height: i32,
        radius: i32,
        offset: i32,
        foliage_provider: &BlockState,
    ) {
        match self {
            FoliageType::Blob(blob) => {
                blob.generate(
                    chunk,
                    level,
                    random,
                    node,
                    foliage_height,
                    radius,
                    offset,
                    foliage_provider,
                )
                .await
            }
            FoliageType::Spruce(spruce) => {
                spruce
                    .generate(
                        chunk,
                        level,
                        random,
                        node,
                        foliage_height,
                        radius,
                        offset,
                        foliage_provider,
                    )
                    .await
            }
            FoliageType::Pine(pine) => {
                pine.generate(
                    chunk,
                    level,
                    random,
                    node,
                    foliage_height,
                    radius,
                    offset,
                    foliage_provider,
                )
                .await
            }
            FoliageType::Acacia(acacia) => {
                acacia
                    .generate(
                        chunk,
                        level,
                        random,
                        node,
                        foliage_height,
                        radius,
                        offset,
                        foliage_provider,
                    )
                    .await
            }
            FoliageType::Bush(bush) => {
                bush.generate(
                    chunk,
                    level,
                    random,
                    node,
                    foliage_height,
                    radius,
                    offset,
                    foliage_provider,
                )
                .await
            }
            FoliageType::Fancy(fancy) => {
                fancy
                    .generate(
                        chunk,
                        level,
                        random,
                        node,
                        foliage_height,
                        radius,
                        offset,
                        foliage_provider,
                    )
                    .await
            }
            FoliageType::Jungle(jungle) => {
                jungle
                    .generate(
                        chunk,
                        level,
                        random,
                        node,
                        foliage_height,
                        radius,
                        offset,
                        foliage_provider,
                    )
                    .await
            }
            FoliageType::MegaPine(mega_pine) => {
                mega_pine
                    .generate(
                        chunk,
                        level,
                        random,
                        node,
                        foliage_height,
                        radius,
                        offset,
                        foliage_provider,
                    )
                    .await
            }
            FoliageType::DarkOak(dark_oak) => {
                dark_oak
                    .generate(
                        chunk,
                        level,
                        random,
                        node,
                        foliage_height,
                        radius,
                        offset,
                        foliage_provider,
                    )
                    .await
            }
            FoliageType::RandomSpread(random_spread) => {
                random_spread
                    .generate(
                        chunk,
                        level,
                        random,
                        node,
                        foliage_height,
                        radius,
                        offset,
                        foliage_provider,
                    )
                    .await
            }
            FoliageType::Cherry(cherry) => {
                cherry
                    .generate(
                        chunk,
                        level,
                        random,
                        node,
                        foliage_height,
                        radius,
                        offset,
                        foliage_provider,
                    )
                    .await
            }
        }
    }

    pub fn get_random_height(&self, random: &mut RandomGenerator, trunk_height: i32) -> i32 {
        match self {
            FoliageType::Blob(blob) => blob.get_random_height(random),
            FoliageType::Spruce(spruce) => spruce.get_random_height(random, trunk_height),
            FoliageType::Pine(pine) => pine.get_random_height(random, trunk_height),
            FoliageType::Acacia(acacia) => acacia.get_random_height(random),
            FoliageType::Bush(bush) => bush.get_random_height(random),
            FoliageType::Fancy(fancy) => fancy.get_random_height(random),
            FoliageType::Jungle(jungle) => jungle.get_random_height(random, trunk_height),
            FoliageType::MegaPine(mega_pine) => mega_pine.get_random_height(random, trunk_height),
            FoliageType::DarkOak(dark_oak) => dark_oak.get_random_height(random),
            FoliageType::RandomSpread(random_spread) => {
                random_spread.get_random_height(random, trunk_height)
            }
            FoliageType::Cherry(cherry) => cherry.get_random_height(random),
        }
    }
}
