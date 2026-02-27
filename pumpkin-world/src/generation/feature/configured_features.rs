use std::{collections::HashMap, sync::LazyLock};

use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

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
use crate::generation::proto_chunk::GenerationCache;
use crate::world::BlockRegistryExt;

pub static CONFIGURED_FEATURES: LazyLock<HashMap<String, ConfiguredFeature>> =
    LazyLock::new(build_configured_features);

pub enum ConfiguredFeature {
    NoOp,
    Tree(Box<TreeFeature>),
    FallenTree(FallenTreeFeature),
    Flower(RandomPatchFeature),
    NoBonemealFlower(RandomPatchFeature),
    RandomPatch(RandomPatchFeature),
    BlockPile(BlockPileFeature),
    SpringFeature(SpringFeatureFeature),
    ChorusPlant(ChorusPlantFeature),
    ReplaceSingleBlock(ReplaceSingleBlockFeature),
    VoidStartPlatform(VoidStartPlatformFeature),
    DesertWell(DesertWellFeature),
    Fossil(FossilFeature),
    HugeRedMushroom(HugeRedMushroomFeature),
    HugeBrownMushroom(HugeBrownMushroomFeature),
    IceSpike(IceSpikeFeature),
    GlowstoneBlob(GlowstoneBlobFeature),
    FreezeTopLayer(FreezeTopLayerFeature),
    Vines(VinesFeature),
    BlockColumn(BlockColumnFeature),
    VegetationPatch(VegetationPatchFeature),
    WaterloggedVegetationPatch(WaterloggedVegetationPatchFeature),
    RootSystem(RootSystemFeature),
    MultifaceGrowth(MultifaceGrowthFeature),
    UnderwaterMagma(UnderwaterMagmaFeature),
    MonsterRoom(DungeonFeature),
    BlueIce(BlueIceFeature),
    Iceberg(IcebergFeature),
    ForestRock(ForestRockFeature),
    Disk(DiskFeature),
    Lake(LakeFeature),
    Ore(OreFeature),
    EndPlatform(EndPlatformFeature),
    EndSpike(EndSpikeFeature),
    EndIsland(EndIslandFeature),
    EndGateway(EndGatewayFeature),
    Seagrass(SeagrassFeature),
    Kelp(KelpFeature),
    CoralTree(CoralTreeFeature),
    CoralMushroom(CoralMushroomFeature),
    CoralClaw(CoralClawFeature),
    SeaPickle(SeaPickleFeature),
    SimpleBlock(SimpleBlockFeature),
    Bamboo(BambooFeature),
    HugeFungus(HugeFungusFeature),
    NetherForestVegetation(NetherForestVegetationFeature),
    WeepingVines(WeepingVinesFeature),
    TwistingVines(TwistingVinesFeature),
    BasaltColumns(BasaltColumnsFeature),
    DeltaFeature(DeltaFeatureFeature),
    NetherrackReplaceBlobs(ReplaceBlobsFeature),
    FillLayer(FillLayerFeature),
    BonusChest(BonusChestFeature),
    BasaltPillar(BasaltPillarFeature),
    ScatteredOre(ScatteredOreFeature),
    RandomSelector(RandomFeature),
    SimpleRandomSelector(SimpleRandomFeature),
    RandomBooleanSelector(RandomBooleanFeature),
    Geode(GeodeFeature),
    DripstoneCluster(DripstoneClusterFeature),
    LargeDripstone(LargeDripstoneFeature),
    PointedDripstone(SmallDripstoneFeature),
    SculkPatch(SculkPatchFeature),
}

// Yes this may look ugly and you wonder why this is hard coded, but it makes sense to hardcode since we have to add logic for these in code

impl ConfiguredFeature {
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
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            Self::NoBonemealFlower(feature) => feature.generate(
                chunk,
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
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            Self::RandomBooleanSelector(feature) => feature.generate(
                chunk,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            Self::Tree(feature) => {
                feature.generate(chunk, min_y, height, feature_name, random, pos)
            }
            Self::RandomSelector(feature) => feature.generate(
                chunk,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            ),
            Self::SimpleRandomSelector(feature) => feature.generate(
                chunk,
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
            Self::UnderwaterMagma(feature) => {
                feature.generate(chunk, min_y, height, feature_name, random, pos)
            }
            Self::SeaPickle(feature) => {
                feature.generate(chunk, min_y, height, feature_name, random, pos)
            }
            Self::Kelp(feature) => {
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

// generated code is now placed alongside other codegen outputs
// in `src/generated` so we don’t hide it deep under `generation/feature`.
include!("../../../../pumpkin-data/src/generated/configured_features_generated.rs");
