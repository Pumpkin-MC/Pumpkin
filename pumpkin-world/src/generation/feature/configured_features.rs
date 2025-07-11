use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};
use serde::Deserialize;

use crate::{ProtoChunk, level::Level, world::BlockRegistryExt};

use super::features::{
    bamboo::BambooFeature,
    basalt_columns::BasaltColumnsFeature,
    basalt_pillar::BasaltPillarFeature,
    block_column::BlockColumnFeature,
    block_pile::BlockPileFeature,
    blue_ice::BlueIceFeature,
    bonus_chest::BonusChestFeature,
    chorus_plant::ChorusPlantFeature,
    coral::{
        coral_claw::CoralClawFeature, coral_mushroom::CoralMushroomFeature,
        coral_tree::CoralTreeFeature,
    },
    delta_feature::DeltaFeatureFeature,
    desert_well::DesertWellFeature,
    disk::DiskFeature,
    drip_stone::{
        cluster::DripstoneClusterFeature, large::LargeDripstoneFeature,
        small::SmallDripstoneFeature,
    },
    end_gateway::EndGatewayFeature,
    end_island::EndIslandFeature,
    end_platform::EndPlatformFeature,
    end_spike::EndSpikeFeature,
    fallen_tree::FallenTreeFeature,
    fill_layer::FillLayerFeature,
    forest_rock::ForestRockFeature,
    fossil::FossilFeature,
    freeze_top_layer::FreezeTopLayerFeature,
    geode::GeodeFeature,
    glowstone_blob::GlowstoneBlobFeature,
    huge_brown_mushroom::HugeBrownMushroomFeature,
    huge_fungus::HugeFungusFeature,
    huge_red_mushroom::HugeRedMushroomFeature,
    ice_spike::IceSpikeFeature,
    iceberg::IcebergFeature,
    kelp::KelpFeature,
    lake::LakeFeature,
    monster_room::DungeonFeature,
    multiface_growth::MultifaceGrowthFeature,
    nether_forest_vegetation::NetherForestVegetationFeature,
    netherrack_replace_blobs::ReplaceBlobsFeature,
    ore::OreFeature,
    random_boolean_selector::RandomBooleanFeature,
    random_patch::RandomPatchFeature,
    random_selector::RandomFeature,
    replace_single_block::ReplaceSingleBlockFeature,
    root_system::RootSystemFeature,
    scattered_ore::ScatteredOreFeature,
    sculk_patch::SculkPatchFeature,
    sea_pickle::SeaPickleFeature,
    seagrass::SeagrassFeature,
    simple_block::SimpleBlockFeature,
    simple_random_selector::SimpleRandomFeature,
    spring_feature::SpringFeatureFeature,
    tree::TreeFeature,
    twisting_vines::TwistingVinesFeature,
    underwater_magma::UnderwaterMagmaFeature,
    vegetation_patch::VegetationPatchFeature,
    vines::VinesFeature,
    void_start_platform::VoidStartPlatformFeature,
    waterlogged_vegetation_patch::WaterloggedVegetationPatchFeature,
    weeping_vines::WeepingVinesFeature,
};

pub static CONFIGURED_FEATURES: LazyLock<HashMap<String, ConfiguredFeature>> =
    LazyLock::new(|| {
        serde_json::from_str(include_str!("../../../../assets/configured_features.json"))
            .expect("Could not parse configured_features.json registry.")
    });

// Yes this may look ugly and you wonder why this is hard coded, but its makes sense to hardcode since we have to add logic for these in code
#[derive(Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum ConfiguredFeature {
    #[serde(rename = "minecraft:no_op")]
    NoOp,
    #[serde(rename = "minecraft:tree")]
    Tree(Box<TreeFeature>),
    #[serde(rename = "minecraft:fallen_tree")]
    FallenTree(FallenTreeFeature),
    #[serde(rename = "minecraft:flower")]
    Flower(RandomPatchFeature),
    #[serde(rename = "minecraft:no_bonemeal_flower")]
    NoBonemealFlower(RandomPatchFeature),
    #[serde(rename = "minecraft:random_patch")]
    RandomPatch(RandomPatchFeature),
    #[serde(rename = "minecraft:block_pile")]
    BlockPile(BlockPileFeature),
    #[serde(rename = "minecraft:spring_feature")]
    SpringFeature(SpringFeatureFeature),
    #[serde(rename = "minecraft:chorus_plant")]
    ChorusPlant(ChorusPlantFeature),
    #[serde(rename = "minecraft:replace_single_block")]
    ReplaceSingleBlock(ReplaceSingleBlockFeature),
    #[serde(rename = "minecraft:void_start_platform")]
    VoidStartPlatform(VoidStartPlatformFeature),
    #[serde(rename = "minecraft:desert_well")]
    DesertWell(DesertWellFeature),
    #[serde(rename = "minecraft:fossil")]
    Fossil(FossilFeature),
    #[serde(rename = "minecraft:huge_red_mushroom")]
    HugeRedMushroom(HugeRedMushroomFeature),
    #[serde(rename = "minecraft:huge_brown_mushroom")]
    HugeBrownMushroom(HugeBrownMushroomFeature),
    #[serde(rename = "minecraft:ice_spike")]
    IceSpike(IceSpikeFeature),
    #[serde(rename = "minecraft:glowstone_blob")]
    GlowstoneBlob(GlowstoneBlobFeature),
    #[serde(rename = "minecraft:freeze_top_layer")]
    FreezeTopLayer(FreezeTopLayerFeature),
    #[serde(rename = "minecraft:vines")]
    Vines(VinesFeature),
    #[serde(rename = "minecraft:block_column")]
    BlockColumn(BlockColumnFeature),
    #[serde(rename = "minecraft:vegetation_patch")]
    VegetationPatch(VegetationPatchFeature),
    #[serde(rename = "minecraft:waterlogged_vegetation_patch")]
    WaterloggedVegetationPatch(WaterloggedVegetationPatchFeature),
    #[serde(rename = "minecraft:root_system")]
    RootSystem(RootSystemFeature),
    #[serde(rename = "minecraft:multiface_growth")]
    MultifaceGrowth(MultifaceGrowthFeature),
    #[serde(rename = "minecraft:underwater_magma")]
    UnderwaterMagma(UnderwaterMagmaFeature),
    #[serde(rename = "minecraft:monster_room")]
    MonsterRoom(DungeonFeature),
    #[serde(rename = "minecraft:blue_ice")]
    BlueIce(BlueIceFeature),
    #[serde(rename = "minecraft:iceberg")]
    Iceberg(IcebergFeature),
    #[serde(rename = "minecraft:forest_rock")]
    ForestRock(ForestRockFeature),
    #[serde(rename = "minecraft:disk")]
    Disk(DiskFeature),
    #[serde(rename = "minecraft:lake")]
    Lake(LakeFeature),
    #[serde(rename = "minecraft:ore")]
    Ore(OreFeature),
    #[serde(rename = "minecraft:end_platform")]
    EndPlatform(EndPlatformFeature),
    #[serde(rename = "minecraft:end_spike")]
    EndSpike(EndSpikeFeature),
    #[serde(rename = "minecraft:end_island")]
    EndIsland(EndIslandFeature),
    #[serde(rename = "minecraft:end_gateway")]
    EndGateway(EndGatewayFeature),
    #[serde(rename = "minecraft:seagrass")]
    Seagrass(SeagrassFeature),
    #[serde(rename = "minecraft:kelp")]
    Kelp(KelpFeature),
    #[serde(rename = "minecraft:coral_tree")]
    CoralTree(CoralTreeFeature),
    #[serde(rename = "minecraft:coral_mushroom")]
    CoralMushroom(CoralMushroomFeature),
    #[serde(rename = "minecraft:coral_claw")]
    CoralClaw(CoralClawFeature),
    #[serde(rename = "minecraft:sea_pickle")]
    SeaPickle(SeaPickleFeature),
    #[serde(rename = "minecraft:simple_block")]
    SimpleBlock(SimpleBlockFeature),
    #[serde(rename = "minecraft:bamboo")]
    Bamboo(BambooFeature),
    #[serde(rename = "minecraft:huge_fungus")]
    HugeFungus(HugeFungusFeature),
    #[serde(rename = "minecraft:nether_forest_vegetation")]
    NetherForestVegetation(NetherForestVegetationFeature),
    #[serde(rename = "minecraft:weeping_vines")]
    WeepingVines(WeepingVinesFeature),
    #[serde(rename = "minecraft:twisting_vines")]
    TwistingVines(TwistingVinesFeature),
    #[serde(rename = "minecraft:basalt_columns")]
    BasaltColumns(BasaltColumnsFeature),
    #[serde(rename = "minecraft:delta_feature")]
    DeltaFeature(DeltaFeatureFeature),
    #[serde(rename = "minecraft:netherrack_replace_blobs")]
    NetherrackReplaceBlobs(ReplaceBlobsFeature),
    #[serde(rename = "minecraft:fill_layer")]
    FillLayer(FillLayerFeature),
    #[serde(rename = "minecraft:bonus_chest")]
    BonusChest(BonusChestFeature),
    #[serde(rename = "minecraft:basalt_pillar")]
    BasaltPillar(BasaltPillarFeature),
    #[serde(rename = "minecraft:scattered_ore")]
    ScatteredOre(ScatteredOreFeature),
    #[serde(rename = "minecraft:random_selector")]
    RandomSelector(RandomFeature),
    #[serde(rename = "minecraft:simple_random_selector")]
    SimpleRandomSelector(SimpleRandomFeature),
    #[serde(rename = "minecraft:random_boolean_selector")]
    RandomBooleanSelector(RandomBooleanFeature),
    #[serde(rename = "minecraft:geode")]
    Geode(GeodeFeature),
    #[serde(rename = "minecraft:dripstone_cluster")]
    DripstoneCluster(DripstoneClusterFeature),
    #[serde(rename = "minecraft:large_dripstone")]
    LargeDripstone(LargeDripstoneFeature),
    #[serde(rename = "minecraft:pointed_dripstone")]
    PointedDripstone(SmallDripstoneFeature),
    #[serde(rename = "minecraft:sculk_patch")]
    SculkPatch(SculkPatchFeature),
}

impl ConfiguredFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk<'_>,
        level: &Arc<Level>,
        block_registry: &dyn BlockRegistryExt,
        min_y: i8,
        height: u16,
        feature_name: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        match self {
            Self::NetherrackReplaceBlobs(feature) => feature.generate(
                chunk,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            Self::NetherForestVegetation(feature) => feature.generate(
                chunk,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            Self::PointedDripstone(feature) => feature.generate(chunk, random, pos),
            Self::CoralMushroom(feature) => {
                feature.generate(chunk, min_y, height, feature_name, random, pos)
            }
            Self::CoralTree(feature) => {
                feature.generate(chunk, min_y, height, feature_name, random, pos)
            }
            Self::CoralClaw(feature) => {
                feature.generate(chunk, min_y, height, feature_name, random, pos)
            }
            Self::EndPlatform(feature) => feature.generate(
                chunk,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            Self::EndSpike(feature) => feature.generate(
                chunk,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            Self::SpringFeature(feature) => feature.generate(block_registry, chunk, random, pos),
            Self::SimpleBlock(feature) => feature.generate(block_registry, chunk, random, pos),
            Self::Flower(feature) => feature.generate(
                chunk,
                level,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            Self::NoBonemealFlower(feature) => feature.generate(
                chunk,
                level,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            Self::DesertWell(feature) => {
                feature.generate(chunk, min_y, height, feature_name, random, pos)
            }
            Self::Bamboo(feature) => feature.generate(
                chunk,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            Self::BlockColumn(feature) => feature.generate(
                chunk,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            Self::RandomPatch(feature) => feature.generate(
                chunk,
                level,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            Self::RandomBooleanSelector(feature) => feature.generate(
                chunk,
                level,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            Self::Tree(feature) => {
                feature.generate(chunk, level, min_y, height, feature_name, random, pos)
            }
            Self::RandomSelector(feature) => feature.generate(
                chunk,
                level,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            Self::SimpleRandomSelector(feature) => feature.generate(
                chunk,
                level,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            Self::Vines(feature) => feature.generate(
                chunk,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            Self::Seagrass(feature) => {
                feature.generate(chunk, min_y, height, feature_name, random, pos)
            }
            Self::SeaPickle(feature) => {
                feature.generate(chunk, min_y, height, feature_name, random, pos)
            }
            Self::Ore(feature) => feature.generate(
                chunk,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            _ => false, // TODO
        }
    }
}
