/* This file is generated. Do not edit manually. */
#[allow(
    clippy::all,
    unused_imports,
    dead_code,
    clippy::large_stack_frames,
    clippy::too_many_lines
)]
fn build_configured_features()
-> std::collections::HashMap<pumpkin_data::configured_feature::ConfiguredFeature, ConfiguredFeature>
{
    use crate::block::BlockStateCodec;
    use crate::generation::block_predicate::{
        AllOfBlockPredicate, AnyOfBlockPredicate, BlockPredicate, HasSturdyFacePredicate,
        InsideWorldBoundsBlockPredicate, MatchingBlockTagPredicate, MatchingBlocksBlockPredicate,
        MatchingBlocksWrapper, MatchingFluidsBlockPredicate, NotBlockPredicate,
        OffsetBlocksBlockPredicate, ReplaceableBlockPredicate, SolidBlockPredicate,
        WouldSurviveBlockPredicate,
    };
    use crate::generation::block_state_provider::{
        BlockStateProvider, BlockStateRule, DualNoiseBlockStateProvider, NoiseBlockStateProvider,
        NoiseBlockStateProviderBase, NoiseThresholdBlockStateProvider, PillarBlockStateProvider,
        RandomizedIntBlockStateProvider, RuleBasedBlockStateProvider, SimpleStateProvider,
        WeightedBlockStateProvider,
    };
    use crate::generation::feature::features::drip_stone::small::SmallDripstoneFeature;
    use crate::generation::feature::features::{
        bamboo::BambooFeature,
        block_column::{BlockColumnFeature, Layer},
        end_spike::{EndSpikeFeature, Spike},
        fallen_tree::FallenTreeFeature,
        geode::GeodeFeature,
        nether_forest_vegetation::NetherForestVegetationFeature,
        netherrack_replace_blobs::ReplaceBlobsFeature,
        ore::{OreFeature, OreTarget},
        random_boolean_selector::RandomBooleanFeature,
        random_patch::RandomPatchFeature,
        random_selector::{RandomFeature, RandomFeatureEntry},
        sea_pickle::SeaPickleFeature,
        seagrass::SeagrassFeature,
        simple_block::SimpleBlockFeature,
        simple_random_selector::SimpleRandomFeature,
        spring_feature::{BlockWrapper, SpringFeatureFeature},
        tree::TreeFeature,
        tree::decorator::{
            TreeDecorator, alter_ground::AlterGroundTreeDecorator,
            attached_to_leaves::AttachedToLeavesTreeDecorator,
            attached_to_logs::AttachedToLogsTreeDecorator, beehive::BeehiveTreeDecorator,
            cocoa::CocoaTreeDecorator, creaking_heart::CreakingHeartTreeDecorator,
            leave_vine::LeavesVineTreeDecorator, pale_moss::PaleMossTreeDecorator,
            place_on_ground::PlaceOnGroundTreeDecorator, trunk_vine::TrunkVineTreeDecorator,
        },
        tree::foliage::{
            FoliagePlacer, FoliageType, acacia::AcaciaFoliagePlacer, blob::BlobFoliagePlacer,
            bush::BushFoliagePlacer, cherry::CherryFoliagePlacer, dark_oak::DarkOakFoliagePlacer,
            fancy::LargeOakFoliagePlacer, jungle::JungleFoliagePlacer,
            mega_pine::MegaPineFoliagePlacer, pine::PineFoliagePlacer,
            random_spread::RandomSpreadFoliagePlacer, spruce::SpruceFoliagePlacer,
        },
        tree::root::{
            RootPlacer,
            mangrove::{AboveRootPlacement, MangroveRootPlacement, MangroveRootPlacer},
        },
        tree::trunk::{
            TrunkPlacer, TrunkType, bending::BendingTrunkPlacer, cherry::CherryTrunkPlacer,
            dark_oak::DarkOakTrunkPlacer, fancy::FancyTrunkPlacer, forking::ForkingTrunkPlacer,
            giant::GiantTrunkPlacer, mega_jungle::MegaJungleTrunkPlacer,
            straight::StraightTrunkPlacer, upwards_branching::UpwardsBranchingTrunkPlacer,
        },
        vegetation_patch::VegetationPatchFeature,
        waterlogged_vegetation_patch::WaterloggedVegetationPatchFeature,
    };
    use crate::generation::feature::placed_features::{
        BiomePlacementModifier, BlockFilterPlacementModifier, CountOnEveryLayerPlacementModifier,
        CountPlacementModifier, EnvironmentScanPlacementModifier, Feature,
        HeightRangePlacementModifier, HeightmapPlacementModifier, NoiseBasedCountPlacementModifier,
        NoiseThresholdCountPlacementModifier, PlacedFeature, PlacedFeatureWrapper,
        PlacementModifier, RandomOffsetPlacementModifier, RarityFilterPlacementModifier,
        SquarePlacementModifier, SurfaceThresholdFilterPlacementModifier,
        SurfaceWaterDepthFilterPlacementModifier,
    };
    use crate::generation::feature::size::{
        FeatureSize, FeatureSizeType, ThreeLayersFeatureSize, TwoLayersFeatureSize,
    };
    use crate::generation::height_provider::{
        HeightProvider, TrapezoidHeightProvider, UniformHeightProvider,
        VeryBiasedToBottomHeightProvider,
    };
    use crate::generation::rule::{
        RuleTest, block_match::BlockMatchRuleTest, block_state_match::BlockStateMatchRuleTest,
        random_block_match::RandomBlockMatchRuleTest,
        random_block_state_match::RandomBlockStateMatchRuleTest, tag_match::TagMatchRuleTest,
    };
    use pumpkin_data::{Block, BlockDirection};
    use pumpkin_util::DoublePerlinNoiseParametersCodec;
    use pumpkin_util::HeightMap;
    use pumpkin_util::math::int_provider::{
        BiasedToBottomIntProvider, ClampedIntProvider, ClampedNormalIntProvider,
        ConstantIntProvider, IntProvider, NormalIntProvider, TrapezoidIntProvider,
        UniformIntProvider, WeightedEntry, WeightedListIntProvider,
    };
    use pumpkin_util::math::pool::Weighted;
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_util::y_offset::{AboveBottom, Absolute, BelowTop, YOffset};
    let mut map = std::collections::HashMap::new();
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: Acacia , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: ACACIA_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 5u8 , height_rand_a : 2u8 , height_rand_b : 2u8 , r#type : TrunkType :: Forking (ForkingTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: ACACIA_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Acacia (AcaciaFoliagePlacer) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 2u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [] , root_placer : None , }))) ;
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::AmethystGeode,
        ConfiguredFeature::Geode(Box::new(GeodeFeature {
            filling_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::AIR.default_state,
            }),
            inner_layer_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::AMETHYST_BLOCK.default_state,
            }),
            alternate_inner_layer_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::BUDDING_AMETHYST.default_state,
            }),
            middle_layer_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::CALCITE.default_state,
            }),
            outer_layer_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::SMOOTH_BASALT.default_state,
            }),
            inner_placements: vec![
                {
                    let mut props = std::collections::HashMap::new();
                    props.insert("waterlogged".to_string(), "false".to_string());
                    props.insert("facing".to_string(), "up".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::SMALL_AMETHYST_BUD,
                        properties: Some(props),
                    }
                },
                {
                    let mut props = std::collections::HashMap::new();
                    props.insert("waterlogged".to_string(), "false".to_string());
                    props.insert("facing".to_string(), "up".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::MEDIUM_AMETHYST_BUD,
                        properties: Some(props),
                    }
                },
                {
                    let mut props = std::collections::HashMap::new();
                    props.insert("waterlogged".to_string(), "false".to_string());
                    props.insert("facing".to_string(), "up".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::LARGE_AMETHYST_BUD,
                        properties: Some(props),
                    }
                },
                {
                    let mut props = std::collections::HashMap::new();
                    props.insert("waterlogged".to_string(), "false".to_string());
                    props.insert("facing".to_string(), "up".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::AMETHYST_CLUSTER,
                        properties: Some(props),
                    }
                },
            ],
            cannot_replace: BlockWrapper::Single("#minecraft:features_cannot_replace".to_string()),
            invalid_blocks: BlockWrapper::Single("#minecraft:geode_invalid_blocks".to_string()),
            filling: 1.7f64,
            inner_layer: 2.2f64,
            middle_layer: 3.2f64,
            outer_layer: 4.2f64,
            generate_crack_chance: 0.95f64,
            base_crack_size: 2f64,
            crack_point_offset: 2i32,
            use_potential_placements_chance: 0.35f64,
            use_alternate_layer0_chance: 0.083f64,
            placements_require_layer0_alternate: true,
            outer_wall_distance: IntProvider::Object(NormalIntProvider::Uniform(
                UniformIntProvider {
                    min_inclusive: 4i32,
                    max_inclusive: 6i32,
                },
            )),
            distribution_points: IntProvider::Constant(0),
            point_offset: IntProvider::Constant(0),
            min_gen_offset: -16i32,
            max_gen_offset: 16i32,
            noise_multiplier: 0.05f64,
            invalid_blocks_threshold: 1i32,
        })),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::AzaleaTree,
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LOG,
                        properties: Some(props),
                    }
                    .get_state()
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 4u8,
                height_rand_a: 2u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Bending(BendingTrunkPlacer {
                    min_height_for_leaves: 3u32,
                    bend_length: IntProvider::Object(NormalIntProvider::Uniform(
                        UniformIntProvider {
                            min_inclusive: 1i32,
                            max_inclusive: 2i32,
                        },
                    )),
                }),
            },
            foliage_provider: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("waterlogged".to_string(), "false".to_string());
                            props.insert("persistent".to_string(), "false".to_string());
                            props.insert("distance".to_string(), "7".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::AZALEA_LEAVES,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 3i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("waterlogged".to_string(), "false".to_string());
                            props.insert("persistent".to_string(), "false".to_string());
                            props.insert("distance".to_string(), "7".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::FLOWERING_AZALEA_LEAVES,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                ],
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(3i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::RandomSpread(RandomSpreadFoliagePlacer {
                    foliage_height: IntProvider::Constant(2i32),
                    leaf_placement_attempts: 50i32,
                }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 1u8,
                }),
            },
            ignore_vines: false,
            below_trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::ROOTED_DIRT.default_state,
            }),
            decorators: vec![],
            root_placer: None,
        })),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::BambooNoPodzol,
        ConfiguredFeature::Bamboo(BambooFeature { probability: 0f32 }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::BambooSomePodzol,
        ConfiguredFeature::Bamboo(BambooFeature {
            probability: 0.2f32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::BambooVegetation,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FancyOakChecked,
                    ),
                    chance: 0.05f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::JungleBush,
                    ),
                    chance: 0.15f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::MegaJungleTreeChecked,
                    ),
                    chance: 0.7f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Direct(PlacedFeature {
                feature: Feature::Named(
                    pumpkin_data::configured_feature::ConfiguredFeature::GrassJungle,
                ),
                placement: vec![
                    PlacementModifier::Count(CountPlacementModifier {
                        count: IntProvider::Constant(32i32),
                    }),
                    PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                        xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                            TrapezoidIntProvider {
                                min_inclusive: -7i32,
                                max_inclusive: 7i32,
                                plateau: 0i32,
                            },
                        )),
                        y_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                            TrapezoidIntProvider {
                                min_inclusive: -3i32,
                                max_inclusive: 3i32,
                                plateau: 0i32,
                            },
                        )),
                    }),
                    PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                        predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                            predicates: vec![
                                BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                                }),
                                BlockPredicate::Not(NotBlockPredicate {
                                    predicate: Box::new(BlockPredicate::MatchingBlocks(
                                        MatchingBlocksBlockPredicate {
                                            offset: OffsetBlocksBlockPredicate {
                                                offset: Some(Vector3::new(0i32, -1i32, 0i32)),
                                            },
                                            blocks: MatchingBlocksWrapper::Single(
                                                "minecraft:podzol".to_string(),
                                            ),
                                        },
                                    )),
                                }),
                            ],
                        }),
                    }),
                ],
            })),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::BasaltBlobs,
        ConfiguredFeature::NetherrackReplaceBlobs(ReplaceBlobsFeature {
            target: pumpkin_data::Block::NETHERRACK.default_state,
            state: {
                let mut props = std::collections::HashMap::new();
                props.insert("axis".to_string(), "y".to_string());
                BlockStateCodec {
                    name: &pumpkin_data::Block::BASALT,
                    properties: Some(props),
                }
                .get_state()
            },
            radius: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                min_inclusive: 3i32,
                max_inclusive: 7i32,
            })),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::BasaltPillar,
        ConfiguredFeature::BasaltPillar(
            crate::generation::feature::features::basalt_pillar::BasaltPillarFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::BerryBush,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("age".to_string(), "3".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::SWEET_BERRY_BUSH,
                        properties: Some(props),
                    }
                    .get_state()
                },
            }),
            schedule_tick: None,
        }),
    );
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: Birch , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: BIRCH_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 5u8 , height_rand_a : 2u8 , height_rand_b : 0u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: BIRCH_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Blob (BlobFoliagePlacer { height : 3i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 1u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: BirchBees0002 , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: BIRCH_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 5u8 , height_rand_a : 2u8 , height_rand_b : 0u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: BIRCH_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Blob (BlobFoliagePlacer { height : 3i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 1u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: Beehive (BeehiveTreeDecorator { probability : 0.002f32 })] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: BirchBees0002LeafLitter , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: BIRCH_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 5u8 , height_rand_a : 2u8 , height_rand_b : 0u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: BIRCH_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Blob (BlobFoliagePlacer { height : 3i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 1u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: Beehive (BeehiveTreeDecorator { probability : 0.002f32 }) , TreeDecorator :: PlaceOnGround (PlaceOnGroundTreeDecorator { tries : 96i32 , radius : 4i32 , height : 2i32 , block_state_provider : BlockStateProvider :: Weighted (WeightedBlockStateProvider { entries : vec ! [Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 }] , }) , }) , TreeDecorator :: PlaceOnGround (PlaceOnGroundTreeDecorator { tries : 150i32 , radius : 1i32 , height : 2i32 , block_state_provider : BlockStateProvider :: Weighted (WeightedBlockStateProvider { entries : vec ! [Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 }] , }) , })] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: BirchBees002 , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: BIRCH_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 5u8 , height_rand_a : 2u8 , height_rand_b : 0u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: BIRCH_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Blob (BlobFoliagePlacer { height : 3i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 1u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: Beehive (BeehiveTreeDecorator { probability : 0.02f32 })] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: BirchBees005 , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: BIRCH_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 5u8 , height_rand_a : 2u8 , height_rand_b : 0u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: BIRCH_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Blob (BlobFoliagePlacer { height : 3i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 1u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: Beehive (BeehiveTreeDecorator { probability : 0.05f32 })] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: BirchLeafLitter , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: BIRCH_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 5u8 , height_rand_a : 2u8 , height_rand_b : 0u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: BIRCH_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Blob (BlobFoliagePlacer { height : 3i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 1u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: PlaceOnGround (PlaceOnGroundTreeDecorator { tries : 96i32 , radius : 4i32 , height : 2i32 , block_state_provider : BlockStateProvider :: Weighted (WeightedBlockStateProvider { entries : vec ! [Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 }] , }) , }) , TreeDecorator :: PlaceOnGround (PlaceOnGroundTreeDecorator { tries : 150i32 , radius : 1i32 , height : 2i32 , block_state_provider : BlockStateProvider :: Weighted (WeightedBlockStateProvider { entries : vec ! [Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 }] , }) , })] , root_placer : None , }))) ;
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::BirchTall,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FallenSuperBirchTree,
                    ),
                    chance: 0.00625f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::SuperBirchBees0002,
                    ),
                    chance: 0.5f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FallenBirchTree,
                    ),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::BirchBees0002,
            )),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::BlackstoneBlobs,
        ConfiguredFeature::NetherrackReplaceBlobs(ReplaceBlobsFeature {
            target: pumpkin_data::Block::NETHERRACK.default_state,
            state: pumpkin_data::Block::BLACKSTONE.default_state,
            radius: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                min_inclusive: 3i32,
                max_inclusive: 7i32,
            })),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::BlueIce,
        ConfiguredFeature::BlueIce(
            crate::generation::feature::features::blue_ice::BlueIceFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::BonusChest,
        ConfiguredFeature::BonusChest(
            crate::generation::feature::features::bonus_chest::BonusChestFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::BrownMushroom,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::BROWN_MUSHROOM.default_state,
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::Bush,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::BUSH.default_state,
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::Cactus,
        ConfiguredFeature::BlockColumn(BlockColumnFeature {
            layers: vec![
                Layer {
                    height: IntProvider::Object(NormalIntProvider::BiasedToBottom(
                        BiasedToBottomIntProvider {
                            min_inclusive: 1i32,
                            max_inclusive: 3i32,
                        },
                    )),
                    provider: BlockStateProvider::Simple(SimpleStateProvider {
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("age".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::CACTUS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                    }),
                },
                Layer {
                    height: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(0i32),
                                    weight: 3i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(1i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                    provider: BlockStateProvider::Simple(SimpleStateProvider {
                        state: pumpkin_data::Block::CACTUS_FLOWER.default_state,
                    }),
                },
            ],
            direction: BlockDirection::Up,
            allowed_placement: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                offset: OffsetBlocksBlockPredicate { offset: None },
                tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
            }),
            prioritize_tip: false,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::CaveVine,
        ConfiguredFeature::BlockColumn(BlockColumnFeature {
            layers: vec![
                Layer {
                    height: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Object(NormalIntProvider::Uniform(
                                        UniformIntProvider {
                                            min_inclusive: 0i32,
                                            max_inclusive: 19i32,
                                        },
                                    )),
                                    weight: 2i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Object(NormalIntProvider::Uniform(
                                        UniformIntProvider {
                                            min_inclusive: 0i32,
                                            max_inclusive: 2i32,
                                        },
                                    )),
                                    weight: 3i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Object(NormalIntProvider::Uniform(
                                        UniformIntProvider {
                                            min_inclusive: 0i32,
                                            max_inclusive: 6i32,
                                        },
                                    )),
                                    weight: 10i32,
                                },
                            ],
                        },
                    )),
                    provider: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                        entries: vec![
                            Weighted {
                                data: {
                                    let mut props = std::collections::HashMap::new();
                                    props.insert("berries".to_string(), "false".to_string());
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::CAVE_VINES_PLANT,
                                        properties: Some(props),
                                    }
                                    .get_state()
                                },
                                weight: 4i32,
                            },
                            Weighted {
                                data: {
                                    let mut props = std::collections::HashMap::new();
                                    props.insert("berries".to_string(), "true".to_string());
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::CAVE_VINES_PLANT,
                                        properties: Some(props),
                                    }
                                    .get_state()
                                },
                                weight: 1i32,
                            },
                        ],
                    }),
                },
                Layer {
                    height: IntProvider::Constant(1i32),
                    provider: BlockStateProvider::RandomizedInt(RandomizedIntBlockStateProvider {
                        source: Box::new(BlockStateProvider::Weighted(
                            WeightedBlockStateProvider {
                                entries: vec![
                                    Weighted {
                                        data: {
                                            let mut props = std::collections::HashMap::new();
                                            props
                                                .insert("berries".to_string(), "false".to_string());
                                            props.insert("age".to_string(), "0".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::CAVE_VINES,
                                                properties: Some(props),
                                            }
                                            .get_state()
                                        },
                                        weight: 4i32,
                                    },
                                    Weighted {
                                        data: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert("berries".to_string(), "true".to_string());
                                            props.insert("age".to_string(), "0".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::CAVE_VINES,
                                                properties: Some(props),
                                            }
                                            .get_state()
                                        },
                                        weight: 1i32,
                                    },
                                ],
                            },
                        )),
                        property: "age".to_string(),
                        values: IntProvider::Object(NormalIntProvider::Uniform(
                            UniformIntProvider {
                                min_inclusive: 23i32,
                                max_inclusive: 25i32,
                            },
                        )),
                    }),
                },
            ],
            direction: BlockDirection::Down,
            allowed_placement: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                offset: OffsetBlocksBlockPredicate { offset: None },
                tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
            }),
            prioritize_tip: true,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::CaveVineInMoss,
        ConfiguredFeature::BlockColumn(BlockColumnFeature {
            layers: vec![
                Layer {
                    height: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Object(NormalIntProvider::Uniform(
                                        UniformIntProvider {
                                            min_inclusive: 0i32,
                                            max_inclusive: 3i32,
                                        },
                                    )),
                                    weight: 5i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Object(NormalIntProvider::Uniform(
                                        UniformIntProvider {
                                            min_inclusive: 1i32,
                                            max_inclusive: 7i32,
                                        },
                                    )),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                    provider: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                        entries: vec![
                            Weighted {
                                data: {
                                    let mut props = std::collections::HashMap::new();
                                    props.insert("berries".to_string(), "false".to_string());
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::CAVE_VINES_PLANT,
                                        properties: Some(props),
                                    }
                                    .get_state()
                                },
                                weight: 4i32,
                            },
                            Weighted {
                                data: {
                                    let mut props = std::collections::HashMap::new();
                                    props.insert("berries".to_string(), "true".to_string());
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::CAVE_VINES_PLANT,
                                        properties: Some(props),
                                    }
                                    .get_state()
                                },
                                weight: 1i32,
                            },
                        ],
                    }),
                },
                Layer {
                    height: IntProvider::Constant(1i32),
                    provider: BlockStateProvider::RandomizedInt(RandomizedIntBlockStateProvider {
                        source: Box::new(BlockStateProvider::Weighted(
                            WeightedBlockStateProvider {
                                entries: vec![
                                    Weighted {
                                        data: {
                                            let mut props = std::collections::HashMap::new();
                                            props
                                                .insert("berries".to_string(), "false".to_string());
                                            props.insert("age".to_string(), "0".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::CAVE_VINES,
                                                properties: Some(props),
                                            }
                                            .get_state()
                                        },
                                        weight: 4i32,
                                    },
                                    Weighted {
                                        data: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert("berries".to_string(), "true".to_string());
                                            props.insert("age".to_string(), "0".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::CAVE_VINES,
                                                properties: Some(props),
                                            }
                                            .get_state()
                                        },
                                        weight: 1i32,
                                    },
                                ],
                            },
                        )),
                        property: "age".to_string(),
                        values: IntProvider::Object(NormalIntProvider::Uniform(
                            UniformIntProvider {
                                min_inclusive: 23i32,
                                max_inclusive: 25i32,
                            },
                        )),
                    }),
                },
            ],
            direction: BlockDirection::Down,
            allowed_placement: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                offset: OffsetBlocksBlockPredicate { offset: None },
                tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
            }),
            prioritize_tip: true,
        }),
    );
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: Cherry , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: CHERRY_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 7u8 , height_rand_a : 1u8 , height_rand_b : 0u8 , r#type : TrunkType :: Cherry (CherryTrunkPlacer { count : IntProvider :: Object (NormalIntProvider :: WeightedList (WeightedListIntProvider { distribution : vec ! [WeightedEntry { data : IntProvider :: Constant (1i32) , weight : 1i32 } , WeightedEntry { data : IntProvider :: Constant (2i32) , weight : 1i32 } , WeightedEntry { data : IntProvider :: Constant (3i32) , weight : 1i32 }] })) , horizontal_length : IntProvider :: Object (NormalIntProvider :: Uniform (UniformIntProvider { min_inclusive : 2i32 , max_inclusive : 4i32 })) , start_offset_from_top : UniformIntProvider { min_inclusive : - 4i32 , max_inclusive : - 3i32 } , end_offset_from_top : IntProvider :: Object (NormalIntProvider :: Uniform (UniformIntProvider { min_inclusive : - 1i32 , max_inclusive : 0i32 })) , }) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: CHERRY_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (4i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Cherry (CherryFoliagePlacer { height : IntProvider :: Constant (5i32) , wide_bottom_layer_hole_chance : 0.25f32 , corner_hole_chance : 0.25f32 , hanging_leaves_chance : 0.16666667f32 , hanging_leaves_extension_chance : 0.33333334f32 , }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 2u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: CherryBees005 , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: CHERRY_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 7u8 , height_rand_a : 1u8 , height_rand_b : 0u8 , r#type : TrunkType :: Cherry (CherryTrunkPlacer { count : IntProvider :: Object (NormalIntProvider :: WeightedList (WeightedListIntProvider { distribution : vec ! [WeightedEntry { data : IntProvider :: Constant (1i32) , weight : 1i32 } , WeightedEntry { data : IntProvider :: Constant (2i32) , weight : 1i32 } , WeightedEntry { data : IntProvider :: Constant (3i32) , weight : 1i32 }] })) , horizontal_length : IntProvider :: Object (NormalIntProvider :: Uniform (UniformIntProvider { min_inclusive : 2i32 , max_inclusive : 4i32 })) , start_offset_from_top : UniformIntProvider { min_inclusive : - 4i32 , max_inclusive : - 3i32 } , end_offset_from_top : IntProvider :: Object (NormalIntProvider :: Uniform (UniformIntProvider { min_inclusive : - 1i32 , max_inclusive : 0i32 })) , }) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: CHERRY_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (4i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Cherry (CherryFoliagePlacer { height : IntProvider :: Constant (5i32) , wide_bottom_layer_hole_chance : 0.25f32 , corner_hole_chance : 0.25f32 , hanging_leaves_chance : 0.16666667f32 , hanging_leaves_extension_chance : 0.33333334f32 , }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 2u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: Beehive (BeehiveTreeDecorator { probability : 0.05f32 })] , root_placer : None , }))) ;
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::ChorusPlant,
        ConfiguredFeature::ChorusPlant(
            crate::generation::feature::features::chorus_plant::ChorusPlantFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::ClayPoolWithDripleaves,
        ConfiguredFeature::WaterloggedVegetationPatch(
            waterlogged_vegetation_patch::WaterloggedVegetationPatchFeature {
                base: vegetation_patch::VegetationPatchFeature {
                    replaceable: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_LUSH_GROUND_REPLACEABLE,
                    }),
                    ground_state: BlockStateProvider::Simple(SimpleStateProvider {
                        state: pumpkin_data::Block::CLAY.default_state,
                    }),
                    vegetation_feature: Box::new(PlacedFeature {
                        feature: Feature::Named(
                            pumpkin_data::configured_feature::ConfiguredFeature::Dripleaf,
                        ),
                        placement: vec![],
                    }),
                    surface: pumpkin_util::math::vertical_surface_type::VerticalSurfaceType::Floor,
                    depth: IntProvider::Constant(3i32),
                    extra_bottom_block_chance: 0.8f32,
                    vertical_range: 5i32,
                    vegetation_chance: 0.1f32,
                    xz_radius: IntProvider::Object(NormalIntProvider::Uniform(
                        UniformIntProvider {
                            min_inclusive: 4i32,
                            max_inclusive: 7i32,
                        },
                    )),
                    extra_edge_column_chance: 0.7f32,
                },
            },
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::ClayWithDripleaves,
        ConfiguredFeature::VegetationPatch(vegetation_patch::VegetationPatchFeature {
            replaceable: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                offset: OffsetBlocksBlockPredicate { offset: None },
                tag: pumpkin_data::tag::Block::MINECRAFT_LUSH_GROUND_REPLACEABLE,
            }),
            ground_state: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::CLAY.default_state,
            }),
            vegetation_feature: Box::new(PlacedFeature {
                feature: Feature::Named(
                    pumpkin_data::configured_feature::ConfiguredFeature::Dripleaf,
                ),
                placement: vec![],
            }),
            surface: pumpkin_util::math::vertical_surface_type::VerticalSurfaceType::Floor,
            depth: IntProvider::Constant(3i32),
            extra_bottom_block_chance: 0.8f32,
            vertical_range: 2i32,
            vegetation_chance: 0.05f32,
            xz_radius: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                min_inclusive: 4i32,
                max_inclusive: 7i32,
            })),
            extra_edge_column_chance: 0.7f32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::CrimsonForestVegetation,
        ConfiguredFeature::NetherForestVegetation(NetherForestVegetationFeature {
            state_provider: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: pumpkin_data::Block::CRIMSON_ROOTS.default_state,
                        weight: 87i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::CRIMSON_FUNGUS.default_state,
                        weight: 11i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::WARPED_FUNGUS.default_state,
                        weight: 1i32,
                    },
                ],
            }),
            spread_width: 8i32,
            spread_height: 4i32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::CrimsonForestVegetationBonemeal,
        ConfiguredFeature::NetherForestVegetation(NetherForestVegetationFeature {
            state_provider: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: pumpkin_data::Block::CRIMSON_ROOTS.default_state,
                        weight: 87i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::CRIMSON_FUNGUS.default_state,
                        weight: 11i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::WARPED_FUNGUS.default_state,
                        weight: 1i32,
                    },
                ],
            }),
            spread_width: 3i32,
            spread_height: 1i32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::CrimsonFungus,
        ConfiguredFeature::HugeFungus(
            crate::generation::feature::features::huge_fungus::HugeFungusFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::CrimsonFungusPlanted,
        ConfiguredFeature::HugeFungus(
            crate::generation::feature::features::huge_fungus::HugeFungusFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::CrimsonRoots,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::CRIMSON_ROOTS.default_state,
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::DarkForestVegetation,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Direct(PlacedFeature {
                        feature: Feature::Named(
                            pumpkin_data::configured_feature::ConfiguredFeature::HugeBrownMushroom,
                        ),
                        placement: vec![],
                    }),
                    chance: 0.025f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Direct(PlacedFeature {
                        feature: Feature::Named(
                            pumpkin_data::configured_feature::ConfiguredFeature::HugeRedMushroom,
                        ),
                        placement: vec![],
                    }),
                    chance: 0.05f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::DarkOakLeafLitter,
                    ),
                    chance: 0.6666667f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FallenBirchTree,
                    ),
                    chance: 0.0025f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::BirchLeafLitter,
                    ),
                    chance: 0.2f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FallenOakTree,
                    ),
                    chance: 0.0125f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FancyOakLeafLitter,
                    ),
                    chance: 0.1f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::OakLeafLitter,
            )),
        }),
    );
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: DarkOak , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: DARK_OAK_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 6u8 , height_rand_a : 2u8 , height_rand_b : 1u8 , r#type : TrunkType :: DarkOak (DarkOakTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: DARK_OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (0i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: DarkOak (DarkOakFoliagePlacer) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: ThreeLayersFeatureSize (ThreeLayersFeatureSize { limit : 1u8 , upper_limit : 1u8 , lower_size : 0u8 , middle_size : 1u8 , upper_size : 2u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: DarkOakLeafLitter , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: DARK_OAK_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 6u8 , height_rand_a : 2u8 , height_rand_b : 1u8 , r#type : TrunkType :: DarkOak (DarkOakTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: DARK_OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (0i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: DarkOak (DarkOakFoliagePlacer) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: ThreeLayersFeatureSize (ThreeLayersFeatureSize { limit : 1u8 , upper_limit : 1u8 , lower_size : 0u8 , middle_size : 1u8 , upper_size : 2u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: PlaceOnGround (PlaceOnGroundTreeDecorator { tries : 96i32 , radius : 4i32 , height : 2i32 , block_state_provider : BlockStateProvider :: Weighted (WeightedBlockStateProvider { entries : vec ! [Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 }] , }) , }) , TreeDecorator :: PlaceOnGround (PlaceOnGroundTreeDecorator { tries : 150i32 , radius : 1i32 , height : 2i32 , block_state_provider : BlockStateProvider :: Weighted (WeightedBlockStateProvider { entries : vec ! [Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 }] , }) , })] , root_placer : None , }))) ;
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::DeadBush,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::DEAD_BUSH.default_state,
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::Delta,
        ConfiguredFeature::DeltaFeature(
            crate::generation::feature::features::delta_feature::DeltaFeatureFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::DesertWell,
        ConfiguredFeature::DesertWell(
            crate::generation::feature::features::desert_well::DesertWellFeature,
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::DiskClay,
        ConfiguredFeature::Disk(crate::generation::feature::features::disk::DiskFeature {
            state_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::CLAY.default_state,
            }),
            target: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                offset: OffsetBlocksBlockPredicate { offset: None },
                blocks: MatchingBlocksWrapper::Multiple(vec![
                    "minecraft:dirt".to_string(),
                    "minecraft:clay".to_string(),
                ]),
            }),
            radius: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                min_inclusive: 2i32,
                max_inclusive: 3i32,
            })),
            half_height: 1i32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::DiskGrass,
        ConfiguredFeature::Disk(crate::generation::feature::features::disk::DiskFeature {
            state_provider: BlockStateProvider::Rule(RuleBasedBlockStateProvider {
                fallback: Some(Box::new(BlockStateProvider::Simple(SimpleStateProvider {
                    state: pumpkin_data::Block::DIRT.default_state,
                }))),
                rules: vec![BlockStateRule {
                    if_true: BlockPredicate::Not(NotBlockPredicate {
                        predicate: Box::new(BlockPredicate::AnyOf(AnyOfBlockPredicate {
                            predicates: vec![
                                BlockPredicate::Solid(SolidBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate {
                                        offset: Some(Vector3::new(0i32, 1i32, 0i32)),
                                    },
                                }),
                                BlockPredicate::MatchingFluids(MatchingFluidsBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate {
                                        offset: Some(Vector3::new(0i32, 1i32, 0i32)),
                                    },
                                    fluids: MatchingBlocksWrapper::Single(
                                        "minecraft:water".to_string(),
                                    ),
                                }),
                            ],
                        })),
                    }),
                    then: BlockStateProvider::Simple(SimpleStateProvider {
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("snowy".to_string(), "false".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::GRASS_BLOCK,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                    }),
                }],
            }),
            target: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                offset: OffsetBlocksBlockPredicate { offset: None },
                blocks: MatchingBlocksWrapper::Multiple(vec![
                    "minecraft:dirt".to_string(),
                    "minecraft:mud".to_string(),
                ]),
            }),
            radius: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                min_inclusive: 2i32,
                max_inclusive: 6i32,
            })),
            half_height: 2i32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::DiskGravel,
        ConfiguredFeature::Disk(crate::generation::feature::features::disk::DiskFeature {
            state_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::GRAVEL.default_state,
            }),
            target: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                offset: OffsetBlocksBlockPredicate { offset: None },
                blocks: MatchingBlocksWrapper::Multiple(vec![
                    "minecraft:dirt".to_string(),
                    "minecraft:grass_block".to_string(),
                ]),
            }),
            radius: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                min_inclusive: 2i32,
                max_inclusive: 5i32,
            })),
            half_height: 2i32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::DiskSand,
        ConfiguredFeature::Disk(crate::generation::feature::features::disk::DiskFeature {
            state_provider: BlockStateProvider::Rule(RuleBasedBlockStateProvider {
                fallback: Some(Box::new(BlockStateProvider::Simple(SimpleStateProvider {
                    state: pumpkin_data::Block::SAND.default_state,
                }))),
                rules: vec![BlockStateRule {
                    if_true: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                        offset: OffsetBlocksBlockPredicate {
                            offset: Some(Vector3::new(0i32, -1i32, 0i32)),
                        },
                        blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                    }),
                    then: BlockStateProvider::Simple(SimpleStateProvider {
                        state: pumpkin_data::Block::SANDSTONE.default_state,
                    }),
                }],
            }),
            target: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                offset: OffsetBlocksBlockPredicate { offset: None },
                blocks: MatchingBlocksWrapper::Multiple(vec![
                    "minecraft:dirt".to_string(),
                    "minecraft:grass_block".to_string(),
                ]),
            }),
            radius: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                min_inclusive: 2i32,
                max_inclusive: 6i32,
            })),
            half_height: 2i32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::Dripleaf,
        ConfiguredFeature::SimpleRandomSelector(SimpleRandomFeature {
            features: vec![
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                        SimpleBlockFeature {
                            to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                                entries: vec![
                                    Weighted {
                                        data: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            props.insert("half".to_string(), "lower".to_string());
                                            props.insert("facing".to_string(), "east".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::SMALL_DRIPLEAF,
                                                properties: Some(props),
                                            }
                                            .get_state()
                                        },
                                        weight: 1i32,
                                    },
                                    Weighted {
                                        data: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            props.insert("half".to_string(), "lower".to_string());
                                            props.insert("facing".to_string(), "west".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::SMALL_DRIPLEAF,
                                                properties: Some(props),
                                            }
                                            .get_state()
                                        },
                                        weight: 1i32,
                                    },
                                    Weighted {
                                        data: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            props.insert("half".to_string(), "lower".to_string());
                                            props.insert("facing".to_string(), "north".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::SMALL_DRIPLEAF,
                                                properties: Some(props),
                                            }
                                            .get_state()
                                        },
                                        weight: 1i32,
                                    },
                                    Weighted {
                                        data: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            props.insert("half".to_string(), "lower".to_string());
                                            props.insert("facing".to_string(), "south".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::SMALL_DRIPLEAF,
                                                properties: Some(props),
                                            }
                                            .get_state()
                                        },
                                        weight: 1i32,
                                    },
                                ],
                            }),
                            schedule_tick: None,
                        },
                    ))),
                    placement: vec![],
                },
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::BlockColumn(
                        BlockColumnFeature {
                            layers: vec![
                                Layer {
                                    height: IntProvider::Object(NormalIntProvider::WeightedList(
                                        WeightedListIntProvider {
                                            distribution: vec![
                                                WeightedEntry {
                                                    data: IntProvider::Object(
                                                        NormalIntProvider::Uniform(
                                                            UniformIntProvider {
                                                                min_inclusive: 0i32,
                                                                max_inclusive: 4i32,
                                                            },
                                                        ),
                                                    ),
                                                    weight: 2i32,
                                                },
                                                WeightedEntry {
                                                    data: IntProvider::Constant(0i32),
                                                    weight: 1i32,
                                                },
                                            ],
                                        },
                                    )),
                                    provider: BlockStateProvider::Simple(SimpleStateProvider {
                                        state: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            props.insert("facing".to_string(), "east".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::BIG_DRIPLEAF_STEM,
                                                properties: Some(props),
                                            }
                                            .get_state()
                                        },
                                    }),
                                },
                                Layer {
                                    height: IntProvider::Constant(1i32),
                                    provider: BlockStateProvider::Simple(SimpleStateProvider {
                                        state: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            props.insert("tilt".to_string(), "none".to_string());
                                            props.insert("facing".to_string(), "east".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::BIG_DRIPLEAF,
                                                properties: Some(props),
                                            }
                                            .get_state()
                                        },
                                    }),
                                },
                            ],
                            direction: BlockDirection::Up,
                            allowed_placement: BlockPredicate::AnyOf(AnyOfBlockPredicate {
                                predicates: vec![
                                    BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                                        offset: OffsetBlocksBlockPredicate { offset: None },
                                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                                    }),
                                    BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                        offset: OffsetBlocksBlockPredicate { offset: None },
                                        blocks: MatchingBlocksWrapper::Single(
                                            "minecraft:water".to_string(),
                                        ),
                                    }),
                                ],
                            }),
                            prioritize_tip: true,
                        },
                    ))),
                    placement: vec![],
                },
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::BlockColumn(
                        BlockColumnFeature {
                            layers: vec![
                                Layer {
                                    height: IntProvider::Object(NormalIntProvider::WeightedList(
                                        WeightedListIntProvider {
                                            distribution: vec![
                                                WeightedEntry {
                                                    data: IntProvider::Object(
                                                        NormalIntProvider::Uniform(
                                                            UniformIntProvider {
                                                                min_inclusive: 0i32,
                                                                max_inclusive: 4i32,
                                                            },
                                                        ),
                                                    ),
                                                    weight: 2i32,
                                                },
                                                WeightedEntry {
                                                    data: IntProvider::Constant(0i32),
                                                    weight: 1i32,
                                                },
                                            ],
                                        },
                                    )),
                                    provider: BlockStateProvider::Simple(SimpleStateProvider {
                                        state: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            props.insert("facing".to_string(), "west".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::BIG_DRIPLEAF_STEM,
                                                properties: Some(props),
                                            }
                                            .get_state()
                                        },
                                    }),
                                },
                                Layer {
                                    height: IntProvider::Constant(1i32),
                                    provider: BlockStateProvider::Simple(SimpleStateProvider {
                                        state: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            props.insert("tilt".to_string(), "none".to_string());
                                            props.insert("facing".to_string(), "west".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::BIG_DRIPLEAF,
                                                properties: Some(props),
                                            }
                                            .get_state()
                                        },
                                    }),
                                },
                            ],
                            direction: BlockDirection::Up,
                            allowed_placement: BlockPredicate::AnyOf(AnyOfBlockPredicate {
                                predicates: vec![
                                    BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                                        offset: OffsetBlocksBlockPredicate { offset: None },
                                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                                    }),
                                    BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                        offset: OffsetBlocksBlockPredicate { offset: None },
                                        blocks: MatchingBlocksWrapper::Single(
                                            "minecraft:water".to_string(),
                                        ),
                                    }),
                                ],
                            }),
                            prioritize_tip: true,
                        },
                    ))),
                    placement: vec![],
                },
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::BlockColumn(
                        BlockColumnFeature {
                            layers: vec![
                                Layer {
                                    height: IntProvider::Object(NormalIntProvider::WeightedList(
                                        WeightedListIntProvider {
                                            distribution: vec![
                                                WeightedEntry {
                                                    data: IntProvider::Object(
                                                        NormalIntProvider::Uniform(
                                                            UniformIntProvider {
                                                                min_inclusive: 0i32,
                                                                max_inclusive: 4i32,
                                                            },
                                                        ),
                                                    ),
                                                    weight: 2i32,
                                                },
                                                WeightedEntry {
                                                    data: IntProvider::Constant(0i32),
                                                    weight: 1i32,
                                                },
                                            ],
                                        },
                                    )),
                                    provider: BlockStateProvider::Simple(SimpleStateProvider {
                                        state: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            props.insert("facing".to_string(), "south".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::BIG_DRIPLEAF_STEM,
                                                properties: Some(props),
                                            }
                                            .get_state()
                                        },
                                    }),
                                },
                                Layer {
                                    height: IntProvider::Constant(1i32),
                                    provider: BlockStateProvider::Simple(SimpleStateProvider {
                                        state: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            props.insert("tilt".to_string(), "none".to_string());
                                            props.insert("facing".to_string(), "south".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::BIG_DRIPLEAF,
                                                properties: Some(props),
                                            }
                                            .get_state()
                                        },
                                    }),
                                },
                            ],
                            direction: BlockDirection::Up,
                            allowed_placement: BlockPredicate::AnyOf(AnyOfBlockPredicate {
                                predicates: vec![
                                    BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                                        offset: OffsetBlocksBlockPredicate { offset: None },
                                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                                    }),
                                    BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                        offset: OffsetBlocksBlockPredicate { offset: None },
                                        blocks: MatchingBlocksWrapper::Single(
                                            "minecraft:water".to_string(),
                                        ),
                                    }),
                                ],
                            }),
                            prioritize_tip: true,
                        },
                    ))),
                    placement: vec![],
                },
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::BlockColumn(
                        BlockColumnFeature {
                            layers: vec![
                                Layer {
                                    height: IntProvider::Object(NormalIntProvider::WeightedList(
                                        WeightedListIntProvider {
                                            distribution: vec![
                                                WeightedEntry {
                                                    data: IntProvider::Object(
                                                        NormalIntProvider::Uniform(
                                                            UniformIntProvider {
                                                                min_inclusive: 0i32,
                                                                max_inclusive: 4i32,
                                                            },
                                                        ),
                                                    ),
                                                    weight: 2i32,
                                                },
                                                WeightedEntry {
                                                    data: IntProvider::Constant(0i32),
                                                    weight: 1i32,
                                                },
                                            ],
                                        },
                                    )),
                                    provider: BlockStateProvider::Simple(SimpleStateProvider {
                                        state: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            props.insert("facing".to_string(), "north".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::BIG_DRIPLEAF_STEM,
                                                properties: Some(props),
                                            }
                                            .get_state()
                                        },
                                    }),
                                },
                                Layer {
                                    height: IntProvider::Constant(1i32),
                                    provider: BlockStateProvider::Simple(SimpleStateProvider {
                                        state: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            props.insert("tilt".to_string(), "none".to_string());
                                            props.insert("facing".to_string(), "north".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::BIG_DRIPLEAF,
                                                properties: Some(props),
                                            }
                                            .get_state()
                                        },
                                    }),
                                },
                            ],
                            direction: BlockDirection::Up,
                            allowed_placement: BlockPredicate::AnyOf(AnyOfBlockPredicate {
                                predicates: vec![
                                    BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                                        offset: OffsetBlocksBlockPredicate { offset: None },
                                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                                    }),
                                    BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                        offset: OffsetBlocksBlockPredicate { offset: None },
                                        blocks: MatchingBlocksWrapper::Single(
                                            "minecraft:water".to_string(),
                                        ),
                                    }),
                                ],
                            }),
                            prioritize_tip: true,
                        },
                    ))),
                    placement: vec![],
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::DripstoneCluster,
        ConfiguredFeature::DripstoneCluster(
            crate::generation::feature::features::drip_stone::cluster::DripstoneClusterFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::DryGrass,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: pumpkin_data::Block::SHORT_DRY_GRASS.default_state,
                        weight: 1i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::TALL_DRY_GRASS.default_state,
                        weight: 1i32,
                    },
                ],
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::EndGatewayDelayed,
        ConfiguredFeature::EndGateway(
            crate::generation::feature::features::end_gateway::EndGatewayFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::EndGatewayReturn,
        ConfiguredFeature::EndGateway(
            crate::generation::feature::features::end_gateway::EndGatewayFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::EndIsland,
        ConfiguredFeature::EndIsland(
            crate::generation::feature::features::end_island::EndIslandFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::EndPlatform,
        ConfiguredFeature::EndPlatform(
            crate::generation::feature::features::end_platform::EndPlatformFeature,
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::EndSpike,
        ConfiguredFeature::EndSpike(EndSpikeFeature {
            crystal_invulnerable: false,
            spikes: vec![],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::FallenBirchTree,
        ConfiguredFeature::FallenTree(FallenTreeFeature {
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LOG,
                        properties: Some(props),
                    }
                    .get_state()
                },
            }),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::FallenJungleTree,
        ConfiguredFeature::FallenTree(FallenTreeFeature {
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::JUNGLE_LOG,
                        properties: Some(props),
                    }
                    .get_state()
                },
            }),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::FallenOakTree,
        ConfiguredFeature::FallenTree(FallenTreeFeature {
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LOG,
                        properties: Some(props),
                    }
                    .get_state()
                },
            }),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::FallenSpruceTree,
        ConfiguredFeature::FallenTree(FallenTreeFeature {
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::SPRUCE_LOG,
                        properties: Some(props),
                    }
                    .get_state()
                },
            }),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::FallenSuperBirchTree,
        ConfiguredFeature::FallenTree(FallenTreeFeature {
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LOG,
                        properties: Some(props),
                    }
                    .get_state()
                },
            }),
        }),
    );
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: FancyOak , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 3u8 , height_rand_a : 11u8 , height_rand_b : 0u8 , r#type : TrunkType :: Fancy (FancyTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (4i32) , r#type : FoliageType :: Fancy (LargeOakFoliagePlacer { height : 4i32 }) } , minimum_size : FeatureSize { min_clipped_height : Some (4u8) , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 0u8 , lower_size : 0u8 , upper_size : 0u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: FancyOakBees , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 3u8 , height_rand_a : 11u8 , height_rand_b : 0u8 , r#type : TrunkType :: Fancy (FancyTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (4i32) , r#type : FoliageType :: Fancy (LargeOakFoliagePlacer { height : 4i32 }) } , minimum_size : FeatureSize { min_clipped_height : Some (4u8) , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 0u8 , lower_size : 0u8 , upper_size : 0u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: Beehive (BeehiveTreeDecorator { probability : 1f32 })] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: FancyOakBees0002LeafLitter , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 3u8 , height_rand_a : 11u8 , height_rand_b : 0u8 , r#type : TrunkType :: Fancy (FancyTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (4i32) , r#type : FoliageType :: Fancy (LargeOakFoliagePlacer { height : 4i32 }) } , minimum_size : FeatureSize { min_clipped_height : Some (4u8) , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 0u8 , lower_size : 0u8 , upper_size : 0u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: Beehive (BeehiveTreeDecorator { probability : 0.002f32 }) , TreeDecorator :: PlaceOnGround (PlaceOnGroundTreeDecorator { tries : 96i32 , radius : 4i32 , height : 2i32 , block_state_provider : BlockStateProvider :: Weighted (WeightedBlockStateProvider { entries : vec ! [Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 }] , }) , }) , TreeDecorator :: PlaceOnGround (PlaceOnGroundTreeDecorator { tries : 150i32 , radius : 1i32 , height : 2i32 , block_state_provider : BlockStateProvider :: Weighted (WeightedBlockStateProvider { entries : vec ! [Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 }] , }) , })] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: FancyOakBees002 , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 3u8 , height_rand_a : 11u8 , height_rand_b : 0u8 , r#type : TrunkType :: Fancy (FancyTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (4i32) , r#type : FoliageType :: Fancy (LargeOakFoliagePlacer { height : 4i32 }) } , minimum_size : FeatureSize { min_clipped_height : Some (4u8) , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 0u8 , lower_size : 0u8 , upper_size : 0u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: Beehive (BeehiveTreeDecorator { probability : 0.02f32 })] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: FancyOakBees005 , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 3u8 , height_rand_a : 11u8 , height_rand_b : 0u8 , r#type : TrunkType :: Fancy (FancyTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (4i32) , r#type : FoliageType :: Fancy (LargeOakFoliagePlacer { height : 4i32 }) } , minimum_size : FeatureSize { min_clipped_height : Some (4u8) , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 0u8 , lower_size : 0u8 , upper_size : 0u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: Beehive (BeehiveTreeDecorator { probability : 0.05f32 })] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: FancyOakLeafLitter , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 3u8 , height_rand_a : 11u8 , height_rand_b : 0u8 , r#type : TrunkType :: Fancy (FancyTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (4i32) , r#type : FoliageType :: Fancy (LargeOakFoliagePlacer { height : 4i32 }) } , minimum_size : FeatureSize { min_clipped_height : Some (4u8) , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 0u8 , lower_size : 0u8 , upper_size : 0u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: PlaceOnGround (PlaceOnGroundTreeDecorator { tries : 96i32 , radius : 4i32 , height : 2i32 , block_state_provider : BlockStateProvider :: Weighted (WeightedBlockStateProvider { entries : vec ! [Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 }] , }) , }) , TreeDecorator :: PlaceOnGround (PlaceOnGroundTreeDecorator { tries : 150i32 , radius : 1i32 , height : 2i32 , block_state_provider : BlockStateProvider :: Weighted (WeightedBlockStateProvider { entries : vec ! [Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 }] , }) , })] , root_placer : None , }))) ;
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::FireflyBush,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::FIREFLY_BUSH.default_state,
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::FlowerCherry,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "1".to_string());
                            props.insert("facing".to_string(), "north".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PINK_PETALS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "1".to_string());
                            props.insert("facing".to_string(), "east".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PINK_PETALS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "1".to_string());
                            props.insert("facing".to_string(), "south".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PINK_PETALS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "1".to_string());
                            props.insert("facing".to_string(), "west".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PINK_PETALS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "2".to_string());
                            props.insert("facing".to_string(), "north".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PINK_PETALS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "2".to_string());
                            props.insert("facing".to_string(), "east".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PINK_PETALS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "2".to_string());
                            props.insert("facing".to_string(), "south".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PINK_PETALS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "2".to_string());
                            props.insert("facing".to_string(), "west".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PINK_PETALS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "3".to_string());
                            props.insert("facing".to_string(), "north".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PINK_PETALS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "3".to_string());
                            props.insert("facing".to_string(), "east".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PINK_PETALS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "3".to_string());
                            props.insert("facing".to_string(), "south".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PINK_PETALS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "3".to_string());
                            props.insert("facing".to_string(), "west".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PINK_PETALS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "4".to_string());
                            props.insert("facing".to_string(), "north".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PINK_PETALS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "4".to_string());
                            props.insert("facing".to_string(), "east".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PINK_PETALS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "4".to_string());
                            props.insert("facing".to_string(), "south".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PINK_PETALS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "4".to_string());
                            props.insert("facing".to_string(), "west".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PINK_PETALS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                ],
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::FlowerDefault,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: pumpkin_data::Block::POPPY.default_state,
                        weight: 2i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::DANDELION.default_state,
                        weight: 1i32,
                    },
                ],
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::FlowerFlowerForest,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::NoiseProvider(NoiseBlockStateProvider {
                base: NoiseBlockStateProviderBase {
                    seed: 2345i64,
                    noise: DoublePerlinNoiseParametersCodec {
                        first_octave: 0i32,
                        amplitudes: vec![1f64],
                        amplitude: 0.8333333333333333f64,
                    },
                    scale: 0.020833334f32,
                },
                states: vec![
                    pumpkin_data::Block::DANDELION.default_state,
                    pumpkin_data::Block::POPPY.default_state,
                    pumpkin_data::Block::ALLIUM.default_state,
                    pumpkin_data::Block::AZURE_BLUET.default_state,
                    pumpkin_data::Block::RED_TULIP.default_state,
                    pumpkin_data::Block::ORANGE_TULIP.default_state,
                    pumpkin_data::Block::WHITE_TULIP.default_state,
                    pumpkin_data::Block::PINK_TULIP.default_state,
                    pumpkin_data::Block::OXEYE_DAISY.default_state,
                    pumpkin_data::Block::CORNFLOWER.default_state,
                    pumpkin_data::Block::LILY_OF_THE_VALLEY.default_state,
                ],
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::FlowerMeadow,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::DualNoise(DualNoiseBlockStateProvider {
                base: NoiseBlockStateProvider {
                    base: NoiseBlockStateProviderBase {
                        seed: 2345i64,
                        noise: DoublePerlinNoiseParametersCodec {
                            first_octave: -3i32,
                            amplitudes: vec![1f64],
                            amplitude: 0.8333333333333333f64,
                        },
                        scale: 1f32,
                    },
                    states: vec![
                        {
                            let mut props = std::collections::HashMap::new();
                            props.insert("half".to_string(), "lower".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::TALL_GRASS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        pumpkin_data::Block::ALLIUM.default_state,
                        pumpkin_data::Block::POPPY.default_state,
                        pumpkin_data::Block::AZURE_BLUET.default_state,
                        pumpkin_data::Block::DANDELION.default_state,
                        pumpkin_data::Block::CORNFLOWER.default_state,
                        pumpkin_data::Block::OXEYE_DAISY.default_state,
                        pumpkin_data::Block::SHORT_GRASS.default_state,
                    ],
                },
                variety: [1u32, 3u32],
                slow_noise: DoublePerlinNoiseParametersCodec {
                    first_octave: -10i32,
                    amplitudes: vec![1f64],
                    amplitude: 0.8333333333333333f64,
                },
                slow_scale: 1f64,
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::FlowerPaleGarden,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::CLOSED_EYEBLOSSOM.default_state,
            }),
            schedule_tick: Some(true),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::FlowerPlain,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::NoiseThreshold(NoiseThresholdBlockStateProvider {
                base: NoiseBlockStateProviderBase {
                    seed: 2345i64,
                    noise: DoublePerlinNoiseParametersCodec {
                        first_octave: 0i32,
                        amplitudes: vec![1f64],
                        amplitude: 0.8333333333333333f64,
                    },
                    scale: 0.005f32,
                },
                threshold: -0.8f32,
                high_chance: 0.33333334f32,
                default_state: pumpkin_data::Block::DANDELION.default_state,
                low_states: vec![
                    pumpkin_data::Block::ORANGE_TULIP.default_state,
                    pumpkin_data::Block::RED_TULIP.default_state,
                    pumpkin_data::Block::PINK_TULIP.default_state,
                    pumpkin_data::Block::WHITE_TULIP.default_state,
                ],
                high_states: vec![
                    pumpkin_data::Block::POPPY.default_state,
                    pumpkin_data::Block::AZURE_BLUET.default_state,
                    pumpkin_data::Block::OXEYE_DAISY.default_state,
                    pumpkin_data::Block::CORNFLOWER.default_state,
                ],
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::FlowerSwamp,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::BLUE_ORCHID.default_state,
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::ForestFlowers,
        ConfiguredFeature::SimpleRandomSelector(SimpleRandomFeature {
            features: vec![
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                        SimpleBlockFeature {
                            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                                state: {
                                    let mut props = std::collections::HashMap::new();
                                    props.insert("half".to_string(), "lower".to_string());
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::LILAC,
                                        properties: Some(props),
                                    }
                                    .get_state()
                                },
                            }),
                            schedule_tick: None,
                        },
                    ))),
                    placement: vec![
                        PlacementModifier::Count(CountPlacementModifier {
                            count: IntProvider::Constant(96i32),
                        }),
                        PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                            xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                                TrapezoidIntProvider {
                                    min_inclusive: -7i32,
                                    max_inclusive: 7i32,
                                    plateau: 0i32,
                                },
                            )),
                            y_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                                TrapezoidIntProvider {
                                    min_inclusive: -3i32,
                                    max_inclusive: 3i32,
                                    plateau: 0i32,
                                },
                            )),
                        }),
                        PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                            predicate: BlockPredicate::MatchingBlockTag(
                                MatchingBlockTagPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                                },
                            ),
                        }),
                    ],
                },
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                        SimpleBlockFeature {
                            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                                state: {
                                    let mut props = std::collections::HashMap::new();
                                    props.insert("half".to_string(), "lower".to_string());
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::ROSE_BUSH,
                                        properties: Some(props),
                                    }
                                    .get_state()
                                },
                            }),
                            schedule_tick: None,
                        },
                    ))),
                    placement: vec![
                        PlacementModifier::Count(CountPlacementModifier {
                            count: IntProvider::Constant(96i32),
                        }),
                        PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                            xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                                TrapezoidIntProvider {
                                    min_inclusive: -7i32,
                                    max_inclusive: 7i32,
                                    plateau: 0i32,
                                },
                            )),
                            y_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                                TrapezoidIntProvider {
                                    min_inclusive: -3i32,
                                    max_inclusive: 3i32,
                                    plateau: 0i32,
                                },
                            )),
                        }),
                        PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                            predicate: BlockPredicate::MatchingBlockTag(
                                MatchingBlockTagPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                                },
                            ),
                        }),
                    ],
                },
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                        SimpleBlockFeature {
                            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                                state: {
                                    let mut props = std::collections::HashMap::new();
                                    props.insert("half".to_string(), "lower".to_string());
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::PEONY,
                                        properties: Some(props),
                                    }
                                    .get_state()
                                },
                            }),
                            schedule_tick: None,
                        },
                    ))),
                    placement: vec![
                        PlacementModifier::Count(CountPlacementModifier {
                            count: IntProvider::Constant(96i32),
                        }),
                        PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                            xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                                TrapezoidIntProvider {
                                    min_inclusive: -7i32,
                                    max_inclusive: 7i32,
                                    plateau: 0i32,
                                },
                            )),
                            y_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                                TrapezoidIntProvider {
                                    min_inclusive: -3i32,
                                    max_inclusive: 3i32,
                                    plateau: 0i32,
                                },
                            )),
                        }),
                        PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                            predicate: BlockPredicate::MatchingBlockTag(
                                MatchingBlockTagPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                                },
                            ),
                        }),
                    ],
                },
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                        SimpleBlockFeature {
                            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                                state: pumpkin_data::Block::LILY_OF_THE_VALLEY.default_state,
                            }),
                            schedule_tick: None,
                        },
                    ))),
                    placement: vec![
                        PlacementModifier::Count(CountPlacementModifier {
                            count: IntProvider::Constant(96i32),
                        }),
                        PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                            xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                                TrapezoidIntProvider {
                                    min_inclusive: -7i32,
                                    max_inclusive: 7i32,
                                    plateau: 0i32,
                                },
                            )),
                            y_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                                TrapezoidIntProvider {
                                    min_inclusive: -3i32,
                                    max_inclusive: 3i32,
                                    plateau: 0i32,
                                },
                            )),
                        }),
                        PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                            predicate: BlockPredicate::MatchingBlockTag(
                                MatchingBlockTagPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                                },
                            ),
                        }),
                    ],
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::ForestRock,
        ConfiguredFeature::ForestRock(
            crate::generation::feature::features::forest_rock::ForestRockFeature {
                state: pumpkin_data::Block::MOSSY_COBBLESTONE.default_state,
            },
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::FossilCoal,
        ConfiguredFeature::Fossil(
            crate::generation::feature::features::fossil::FossilFeature {
                fossil_structures: vec![
                    "minecraft:fossil/spine_1",
                    "minecraft:fossil/spine_2",
                    "minecraft:fossil/spine_3",
                    "minecraft:fossil/spine_4",
                    "minecraft:fossil/skull_1",
                    "minecraft:fossil/skull_2",
                    "minecraft:fossil/skull_3",
                    "minecraft:fossil/skull_4",
                ],
                overlay_structures: vec![
                    "minecraft:fossil/spine_1_coal",
                    "minecraft:fossil/spine_2_coal",
                    "minecraft:fossil/spine_3_coal",
                    "minecraft:fossil/spine_4_coal",
                    "minecraft:fossil/skull_1_coal",
                    "minecraft:fossil/skull_2_coal",
                    "minecraft:fossil/skull_3_coal",
                    "minecraft:fossil/skull_4_coal",
                ],
                fossil_processor:
                    crate::generation::feature::features::fossil::FossilProcessor::FossilRot,
                overlay_processor:
                    crate::generation::feature::features::fossil::FossilProcessor::Coal,
                max_empty_corners_allowed: 4u8,
            },
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::FossilDiamonds,
        ConfiguredFeature::Fossil(
            crate::generation::feature::features::fossil::FossilFeature {
                fossil_structures: vec![
                    "minecraft:fossil/spine_1",
                    "minecraft:fossil/spine_2",
                    "minecraft:fossil/spine_3",
                    "minecraft:fossil/spine_4",
                    "minecraft:fossil/skull_1",
                    "minecraft:fossil/skull_2",
                    "minecraft:fossil/skull_3",
                    "minecraft:fossil/skull_4",
                ],
                overlay_structures: vec![
                    "minecraft:fossil/spine_1_coal",
                    "minecraft:fossil/spine_2_coal",
                    "minecraft:fossil/spine_3_coal",
                    "minecraft:fossil/spine_4_coal",
                    "minecraft:fossil/skull_1_coal",
                    "minecraft:fossil/skull_2_coal",
                    "minecraft:fossil/skull_3_coal",
                    "minecraft:fossil/skull_4_coal",
                ],
                fossil_processor:
                    crate::generation::feature::features::fossil::FossilProcessor::FossilRot,
                overlay_processor:
                    crate::generation::feature::features::fossil::FossilProcessor::Diamonds,
                max_empty_corners_allowed: 4u8,
            },
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::FreezeTopLayer,
        ConfiguredFeature::FreezeTopLayer(
            crate::generation::feature::features::freeze_top_layer::FreezeTopLayerFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::GlowLichen,
        ConfiguredFeature::MultifaceGrowth(
            crate::generation::feature::features::multiface_growth::MultifaceGrowthFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::GlowstoneExtra,
        ConfiguredFeature::GlowstoneBlob(
            crate::generation::feature::features::glowstone_blob::GlowstoneBlobFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::Grass,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::SHORT_GRASS.default_state,
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::GrassJungle,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: pumpkin_data::Block::SHORT_GRASS.default_state,
                        weight: 3i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::FERN.default_state,
                        weight: 1i32,
                    },
                ],
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::HugeBrownMushroom,
        ConfiguredFeature::HugeBrownMushroom(
            crate::generation::feature::features::huge_brown_mushroom::HugeBrownMushroomFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::HugeRedMushroom,
        ConfiguredFeature::HugeRedMushroom(
            crate::generation::feature::features::huge_red_mushroom::HugeRedMushroomFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::IcePatch,
        ConfiguredFeature::Disk(crate::generation::feature::features::disk::DiskFeature {
            state_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::PACKED_ICE.default_state,
            }),
            target: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                offset: OffsetBlocksBlockPredicate { offset: None },
                blocks: MatchingBlocksWrapper::Multiple(vec![
                    "minecraft:dirt".to_string(),
                    "minecraft:grass_block".to_string(),
                    "minecraft:podzol".to_string(),
                    "minecraft:coarse_dirt".to_string(),
                    "minecraft:mycelium".to_string(),
                    "minecraft:snow_block".to_string(),
                    "minecraft:ice".to_string(),
                ]),
            }),
            radius: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                min_inclusive: 2i32,
                max_inclusive: 3i32,
            })),
            half_height: 1i32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::IceSpike,
        ConfiguredFeature::IceSpike(
            crate::generation::feature::features::ice_spike::IceSpikeFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::IcebergBlue,
        ConfiguredFeature::Iceberg(
            crate::generation::feature::features::iceberg::IcebergFeature {
                main_block: BlockStateCodec {
                    name: &pumpkin_data::Block::BLUE_ICE,
                    properties: None,
                },
            },
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::IcebergPacked,
        ConfiguredFeature::Iceberg(
            crate::generation::feature::features::iceberg::IcebergFeature {
                main_block: BlockStateCodec {
                    name: &pumpkin_data::Block::PACKED_ICE,
                    properties: None,
                },
            },
        ),
    );
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: JungleBush , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: JUNGLE_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 1u8 , height_rand_a : 0u8 , height_rand_b : 0u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (1i32) , r#type : FoliageType :: Bush (BushFoliagePlacer { height : 2i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 0u8 , lower_size : 0u8 , upper_size : 0u8 , }) } , ignore_vines : false , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: JungleTree , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: JUNGLE_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 4u8 , height_rand_a : 8u8 , height_rand_b : 0u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: JUNGLE_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Blob (BlobFoliagePlacer { height : 3i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 1u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: Cocoa (CocoaTreeDecorator { }) , TreeDecorator :: TrunkVine (TrunkVineTreeDecorator) , TreeDecorator :: LeaveVine (LeavesVineTreeDecorator { probability : 0.25f32 })] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: JungleTreeNoVine , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: JUNGLE_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 4u8 , height_rand_a : 8u8 , height_rand_b : 0u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: JUNGLE_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Blob (BlobFoliagePlacer { height : 3i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 1u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [] , root_placer : None , }))) ;
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::Kelp,
        ConfiguredFeature::Kelp(crate::generation::feature::features::kelp::KelpFeature {}),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::LakeLava,
        ConfiguredFeature::Lake(crate::generation::feature::features::lake::LakeFeature {
            fluid: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("level".to_string(), "0".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::LAVA,
                        properties: Some(props),
                    }
                    .get_state()
                },
            }),
            barrier: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::STONE.default_state,
            }),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::LargeBasaltColumns,
        ConfiguredFeature::BasaltColumns(
            crate::generation::feature::features::basalt_columns::BasaltColumnsFeature {
                height: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                    min_inclusive: 5i32,
                    max_inclusive: 10i32,
                })),
                reach: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                    min_inclusive: 2i32,
                    max_inclusive: 3i32,
                })),
            },
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::LargeDripstone,
        ConfiguredFeature::LargeDripstone(
            crate::generation::feature::features::drip_stone::large::LargeDripstoneFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::LargeFern,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("half".to_string(), "lower".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::LARGE_FERN,
                        properties: Some(props),
                    }
                    .get_state()
                },
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::LeafLitter,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("segment_amount".to_string(), "1".to_string());
                            props.insert("facing".to_string(), "north".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::LEAF_LITTER,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("segment_amount".to_string(), "1".to_string());
                            props.insert("facing".to_string(), "east".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::LEAF_LITTER,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("segment_amount".to_string(), "1".to_string());
                            props.insert("facing".to_string(), "south".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::LEAF_LITTER,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("segment_amount".to_string(), "1".to_string());
                            props.insert("facing".to_string(), "west".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::LEAF_LITTER,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("segment_amount".to_string(), "2".to_string());
                            props.insert("facing".to_string(), "north".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::LEAF_LITTER,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("segment_amount".to_string(), "2".to_string());
                            props.insert("facing".to_string(), "east".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::LEAF_LITTER,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("segment_amount".to_string(), "2".to_string());
                            props.insert("facing".to_string(), "south".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::LEAF_LITTER,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("segment_amount".to_string(), "2".to_string());
                            props.insert("facing".to_string(), "west".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::LEAF_LITTER,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("segment_amount".to_string(), "3".to_string());
                            props.insert("facing".to_string(), "north".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::LEAF_LITTER,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("segment_amount".to_string(), "3".to_string());
                            props.insert("facing".to_string(), "east".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::LEAF_LITTER,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("segment_amount".to_string(), "3".to_string());
                            props.insert("facing".to_string(), "south".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::LEAF_LITTER,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("segment_amount".to_string(), "3".to_string());
                            props.insert("facing".to_string(), "west".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::LEAF_LITTER,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                ],
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::LushCavesClay,
        ConfiguredFeature::RandomBooleanSelector(RandomBooleanFeature {
            feature_true: Box::new(PlacedFeatureWrapper::Direct(PlacedFeature {
                feature: Feature::Named(
                    pumpkin_data::configured_feature::ConfiguredFeature::ClayWithDripleaves,
                ),
                placement: vec![],
            })),
            feature_false: Box::new(PlacedFeatureWrapper::Direct(PlacedFeature {
                feature: Feature::Named(
                    pumpkin_data::configured_feature::ConfiguredFeature::ClayPoolWithDripleaves,
                ),
                placement: vec![],
            })),
        }),
    );
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: Mangrove , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: MANGROVE_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 2u8 , height_rand_a : 1u8 , height_rand_b : 4u8 , r#type : TrunkType :: UpwardsBranching (UpwardsBranchingTrunkPlacer { extra_branch_steps : IntProvider :: Object (NormalIntProvider :: Uniform (UniformIntProvider { min_inclusive : 1i32 , max_inclusive : 4i32 })) , place_branch_per_log_probability : 0.5f32 , extra_branch_length : IntProvider :: Object (NormalIntProvider :: Uniform (UniformIntProvider { min_inclusive : 0i32 , max_inclusive : 1i32 })) , can_grow_through : & pumpkin_data :: tag :: Block :: MINECRAFT_MANGROVE_LOGS_CAN_GROW_THROUGH . 1 , }) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: MANGROVE_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (3i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: RandomSpread (RandomSpreadFoliagePlacer { foliage_height : IntProvider :: Constant (2i32) , leaf_placement_attempts : 70i32 , }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 2u8 , lower_size : 0u8 , upper_size : 2u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: LeaveVine (LeavesVineTreeDecorator { probability : 0.125f32 }) , TreeDecorator :: AttachedToLeaves (AttachedToLeavesTreeDecorator { }) , TreeDecorator :: Beehive (BeehiveTreeDecorator { probability : 0.01f32 })] , root_placer : Some (RootPlacer :: Mangrove (MangroveRootPlacer { trunk_offset_y : IntProvider :: Object (NormalIntProvider :: Uniform (UniformIntProvider { min_inclusive : 1i32 , max_inclusive : 3i32 })) , root_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: MANGROVE_ROOTS , properties : Some (props) , } . get_state () } }) , above_root_placement : Some (AboveRootPlacement { above_root_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: MOSS_CARPET . default_state }) , above_root_placement_chance : 0.5f32 , }) , mangrove_root_placement : MangroveRootPlacement { can_grow_through : & pumpkin_data :: tag :: Block :: MINECRAFT_MANGROVE_ROOTS_CAN_GROW_THROUGH . 1 , muddy_roots_in : const { & [pumpkin_data :: BlockId :: MUD . as_u16 () , pumpkin_data :: BlockId :: MUDDY_MANGROVE_ROOTS . as_u16 ()] } , muddy_roots_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: MUDDY_MANGROVE_ROOTS , properties : Some (props) , } . get_state () } }) , max_root_width : 8i32 , max_root_length : 15i32 , random_skew_chance : 0.2f32 , } , })) , }))) ;
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::MangroveVegetation,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![RandomFeatureEntry {
                feature: PlacedFeatureWrapper::Named(
                    pumpkin_data::placed_feature::PlacedFeature::TallMangroveChecked,
                ),
                chance: 0.85f32,
            }],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::MangroveChecked,
            )),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::MeadowTrees,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![RandomFeatureEntry {
                feature: PlacedFeatureWrapper::Named(
                    pumpkin_data::placed_feature::PlacedFeature::FancyOakBees,
                ),
                chance: 0.5f32,
            }],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::SuperBirchBees,
            )),
        }),
    );
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: MegaJungleTree , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: JUNGLE_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 10u8 , height_rand_a : 2u8 , height_rand_b : 19u8 , r#type : TrunkType :: MegaJungle (MegaJungleTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: JUNGLE_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Jungle (JungleFoliagePlacer { height : 2i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 1u8 , upper_size : 2u8 , }) } , ignore_vines : false , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: TrunkVine (TrunkVineTreeDecorator) , TreeDecorator :: LeaveVine (LeavesVineTreeDecorator { probability : 0.25f32 })] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: MegaPine , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: SPRUCE_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 13u8 , height_rand_a : 2u8 , height_rand_b : 14u8 , r#type : TrunkType :: Giant (GiantTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: SPRUCE_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (0i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: MegaPine (MegaPineFoliagePlacer { crown_height : IntProvider :: Object (NormalIntProvider :: Uniform (UniformIntProvider { min_inclusive : 3i32 , max_inclusive : 7i32 })) }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 1u8 , upper_size : 2u8 , }) } , ignore_vines : false , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: AlterGround (AlterGroundTreeDecorator { })] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: MegaSpruce , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: SPRUCE_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 13u8 , height_rand_a : 2u8 , height_rand_b : 14u8 , r#type : TrunkType :: Giant (GiantTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: SPRUCE_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (0i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: MegaPine (MegaPineFoliagePlacer { crown_height : IntProvider :: Object (NormalIntProvider :: Uniform (UniformIntProvider { min_inclusive : 13i32 , max_inclusive : 17i32 })) }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 1u8 , upper_size : 2u8 , }) } , ignore_vines : false , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: AlterGround (AlterGroundTreeDecorator { })] , root_placer : None , }))) ;
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::Melon,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::MELON.default_state,
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::MonsterRoom,
        ConfiguredFeature::MonsterRoom(
            crate::generation::feature::features::monster_room::DungeonFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::MossPatch,
        ConfiguredFeature::VegetationPatch(vegetation_patch::VegetationPatchFeature {
            replaceable: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                offset: OffsetBlocksBlockPredicate { offset: None },
                tag: pumpkin_data::tag::Block::MINECRAFT_MOSS_REPLACEABLE,
            }),
            ground_state: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::MOSS_BLOCK.default_state,
            }),
            vegetation_feature: Box::new(PlacedFeature {
                feature: Feature::Named(
                    pumpkin_data::configured_feature::ConfiguredFeature::MossVegetation,
                ),
                placement: vec![],
            }),
            surface: pumpkin_util::math::vertical_surface_type::VerticalSurfaceType::Floor,
            depth: IntProvider::Constant(1i32),
            extra_bottom_block_chance: 0f32,
            vertical_range: 5i32,
            vegetation_chance: 0.8f32,
            xz_radius: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                min_inclusive: 4i32,
                max_inclusive: 7i32,
            })),
            extra_edge_column_chance: 0.3f32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::MossPatchBonemeal,
        ConfiguredFeature::VegetationPatch(vegetation_patch::VegetationPatchFeature {
            replaceable: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                offset: OffsetBlocksBlockPredicate { offset: None },
                tag: pumpkin_data::tag::Block::MINECRAFT_MOSS_REPLACEABLE,
            }),
            ground_state: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::MOSS_BLOCK.default_state,
            }),
            vegetation_feature: Box::new(PlacedFeature {
                feature: Feature::Named(
                    pumpkin_data::configured_feature::ConfiguredFeature::MossVegetation,
                ),
                placement: vec![],
            }),
            surface: pumpkin_util::math::vertical_surface_type::VerticalSurfaceType::Floor,
            depth: IntProvider::Constant(1i32),
            extra_bottom_block_chance: 0f32,
            vertical_range: 5i32,
            vegetation_chance: 0.6f32,
            xz_radius: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                min_inclusive: 1i32,
                max_inclusive: 2i32,
            })),
            extra_edge_column_chance: 0.75f32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::MossPatchCeiling,
        ConfiguredFeature::VegetationPatch(vegetation_patch::VegetationPatchFeature {
            replaceable: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                offset: OffsetBlocksBlockPredicate { offset: None },
                tag: pumpkin_data::tag::Block::MINECRAFT_MOSS_REPLACEABLE,
            }),
            ground_state: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::MOSS_BLOCK.default_state,
            }),
            vegetation_feature: Box::new(PlacedFeature {
                feature: Feature::Named(
                    pumpkin_data::configured_feature::ConfiguredFeature::CaveVineInMoss,
                ),
                placement: vec![],
            }),
            surface: pumpkin_util::math::vertical_surface_type::VerticalSurfaceType::Ceiling,
            depth: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                min_inclusive: 1i32,
                max_inclusive: 2i32,
            })),
            extra_bottom_block_chance: 0f32,
            vertical_range: 5i32,
            vegetation_chance: 0.08f32,
            xz_radius: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                min_inclusive: 4i32,
                max_inclusive: 7i32,
            })),
            extra_edge_column_chance: 0.3f32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::MossVegetation,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: pumpkin_data::Block::FLOWERING_AZALEA.default_state,
                        weight: 4i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::AZALEA.default_state,
                        weight: 7i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::MOSS_CARPET.default_state,
                        weight: 25i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::SHORT_GRASS.default_state,
                        weight: 50i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("half".to_string(), "lower".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::TALL_GRASS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 10i32,
                    },
                ],
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::MushroomIslandVegetation,
        ConfiguredFeature::RandomBooleanSelector(RandomBooleanFeature {
            feature_true: Box::new(PlacedFeatureWrapper::Direct(PlacedFeature {
                feature: Feature::Named(
                    pumpkin_data::configured_feature::ConfiguredFeature::HugeRedMushroom,
                ),
                placement: vec![],
            })),
            feature_false: Box::new(PlacedFeatureWrapper::Direct(PlacedFeature {
                feature: Feature::Named(
                    pumpkin_data::configured_feature::ConfiguredFeature::HugeBrownMushroom,
                ),
                placement: vec![],
            })),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::NetherSprouts,
        ConfiguredFeature::NetherForestVegetation(NetherForestVegetationFeature {
            state_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::NETHER_SPROUTS.default_state,
            }),
            spread_width: 8i32,
            spread_height: 4i32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::NetherSproutsBonemeal,
        ConfiguredFeature::NetherForestVegetation(NetherForestVegetationFeature {
            state_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::NETHER_SPROUTS.default_state,
            }),
            spread_width: 3i32,
            spread_height: 1i32,
        }),
    );
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: Oak , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 4u8 , height_rand_a : 2u8 , height_rand_b : 0u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Blob (BlobFoliagePlacer { height : 3i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 1u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: OakBees0002LeafLitter , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 4u8 , height_rand_a : 2u8 , height_rand_b : 0u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Blob (BlobFoliagePlacer { height : 3i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 1u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: Beehive (BeehiveTreeDecorator { probability : 0.002f32 }) , TreeDecorator :: PlaceOnGround (PlaceOnGroundTreeDecorator { tries : 96i32 , radius : 4i32 , height : 2i32 , block_state_provider : BlockStateProvider :: Weighted (WeightedBlockStateProvider { entries : vec ! [Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 }] , }) , }) , TreeDecorator :: PlaceOnGround (PlaceOnGroundTreeDecorator { tries : 150i32 , radius : 1i32 , height : 2i32 , block_state_provider : BlockStateProvider :: Weighted (WeightedBlockStateProvider { entries : vec ! [Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 }] , }) , })] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: OakBees002 , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 4u8 , height_rand_a : 2u8 , height_rand_b : 0u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Blob (BlobFoliagePlacer { height : 3i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 1u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: Beehive (BeehiveTreeDecorator { probability : 0.02f32 })] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: OakBees005 , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 4u8 , height_rand_a : 2u8 , height_rand_b : 0u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Blob (BlobFoliagePlacer { height : 3i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 1u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: Beehive (BeehiveTreeDecorator { probability : 0.05f32 })] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: OakLeafLitter , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 4u8 , height_rand_a : 2u8 , height_rand_b : 0u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Blob (BlobFoliagePlacer { height : 3i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 1u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: PlaceOnGround (PlaceOnGroundTreeDecorator { tries : 96i32 , radius : 4i32 , height : 2i32 , block_state_provider : BlockStateProvider :: Weighted (WeightedBlockStateProvider { entries : vec ! [Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 }] , }) , }) , TreeDecorator :: PlaceOnGround (PlaceOnGroundTreeDecorator { tries : 150i32 , radius : 1i32 , height : 2i32 , block_state_provider : BlockStateProvider :: Weighted (WeightedBlockStateProvider { entries : vec ! [Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "1" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "2" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "3" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "north" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "east" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "south" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 } , Weighted { data : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("segment_amount" . to_string () , "4" . to_string ()) ; props . insert ("facing" . to_string () , "west" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: LEAF_LITTER , properties : Some (props) , } . get_state () } , weight : 1i32 }] , }) , })] , root_placer : None , }))) ;
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreAncientDebrisLarge,
        ConfiguredFeature::ScatteredOre(
            crate::generation::feature::features::scattered_ore::ScatteredOreFeature {
                size: 3i32,
                discard_chance_on_air_exposure: 1f32,
                targets: vec![OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_BASE_STONE_NETHER,
                    }),
                    state: pumpkin_data::Block::ANCIENT_DEBRIS.default_state,
                }],
            },
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreAncientDebrisSmall,
        ConfiguredFeature::ScatteredOre(
            crate::generation::feature::features::scattered_ore::ScatteredOreFeature {
                size: 2i32,
                discard_chance_on_air_exposure: 1f32,
                targets: vec![OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_BASE_STONE_NETHER,
                    }),
                    state: pumpkin_data::Block::ANCIENT_DEBRIS.default_state,
                }],
            },
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreAndesite,
        ConfiguredFeature::Ore(OreFeature {
            size: 64i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::TagMatch(TagMatchRuleTest {
                    tag: pumpkin_data::tag::Block::MINECRAFT_BASE_STONE_OVERWORLD,
                }),
                state: pumpkin_data::Block::ANDESITE.default_state,
            }],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreBlackstone,
        ConfiguredFeature::Ore(OreFeature {
            size: 33i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::BlockMatch(BlockMatchRuleTest {
                    block: pumpkin_data::BlockId::NETHERRACK,
                }),
                state: pumpkin_data::Block::BLACKSTONE.default_state,
            }],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreClay,
        ConfiguredFeature::Ore(OreFeature {
            size: 33i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::TagMatch(TagMatchRuleTest {
                    tag: pumpkin_data::tag::Block::MINECRAFT_BASE_STONE_OVERWORLD,
                }),
                state: pumpkin_data::Block::CLAY.default_state,
            }],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreCoal,
        ConfiguredFeature::Ore(OreFeature {
            size: 17i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_STONE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::COAL_ORE.default_state,
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_DEEPSLATE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DEEPSLATE_COAL_ORE.default_state,
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreCoalBuried,
        ConfiguredFeature::Ore(OreFeature {
            size: 17i32,
            discard_chance_on_air_exposure: 0.5f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_STONE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::COAL_ORE.default_state,
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_DEEPSLATE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DEEPSLATE_COAL_ORE.default_state,
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreCopperLarge,
        ConfiguredFeature::Ore(OreFeature {
            size: 20i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_STONE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::COPPER_ORE.default_state,
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_DEEPSLATE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DEEPSLATE_COPPER_ORE.default_state,
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreCopperSmall,
        ConfiguredFeature::Ore(OreFeature {
            size: 10i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_STONE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::COPPER_ORE.default_state,
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_DEEPSLATE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DEEPSLATE_COPPER_ORE.default_state,
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreDiamondBuried,
        ConfiguredFeature::Ore(OreFeature {
            size: 8i32,
            discard_chance_on_air_exposure: 1f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_STONE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DIAMOND_ORE.default_state,
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_DEEPSLATE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DEEPSLATE_DIAMOND_ORE.default_state,
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreDiamondLarge,
        ConfiguredFeature::Ore(OreFeature {
            size: 12i32,
            discard_chance_on_air_exposure: 0.7f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_STONE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DIAMOND_ORE.default_state,
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_DEEPSLATE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DEEPSLATE_DIAMOND_ORE.default_state,
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreDiamondMedium,
        ConfiguredFeature::Ore(OreFeature {
            size: 8i32,
            discard_chance_on_air_exposure: 0.5f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_STONE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DIAMOND_ORE.default_state,
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_DEEPSLATE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DEEPSLATE_DIAMOND_ORE.default_state,
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreDiamondSmall,
        ConfiguredFeature::Ore(OreFeature {
            size: 4i32,
            discard_chance_on_air_exposure: 0.5f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_STONE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DIAMOND_ORE.default_state,
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_DEEPSLATE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DEEPSLATE_DIAMOND_ORE.default_state,
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreDiorite,
        ConfiguredFeature::Ore(OreFeature {
            size: 64i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::TagMatch(TagMatchRuleTest {
                    tag: pumpkin_data::tag::Block::MINECRAFT_BASE_STONE_OVERWORLD,
                }),
                state: pumpkin_data::Block::DIORITE.default_state,
            }],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreDirt,
        ConfiguredFeature::Ore(OreFeature {
            size: 33i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::TagMatch(TagMatchRuleTest {
                    tag: pumpkin_data::tag::Block::MINECRAFT_BASE_STONE_OVERWORLD,
                }),
                state: pumpkin_data::Block::DIRT.default_state,
            }],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreEmerald,
        ConfiguredFeature::Ore(OreFeature {
            size: 3i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_STONE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::EMERALD_ORE.default_state,
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_DEEPSLATE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DEEPSLATE_EMERALD_ORE.default_state,
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreGold,
        ConfiguredFeature::Ore(OreFeature {
            size: 9i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_STONE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::GOLD_ORE.default_state,
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_DEEPSLATE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DEEPSLATE_GOLD_ORE.default_state,
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreGoldBuried,
        ConfiguredFeature::Ore(OreFeature {
            size: 9i32,
            discard_chance_on_air_exposure: 0.5f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_STONE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::GOLD_ORE.default_state,
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_DEEPSLATE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DEEPSLATE_GOLD_ORE.default_state,
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreGranite,
        ConfiguredFeature::Ore(OreFeature {
            size: 64i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::TagMatch(TagMatchRuleTest {
                    tag: pumpkin_data::tag::Block::MINECRAFT_BASE_STONE_OVERWORLD,
                }),
                state: pumpkin_data::Block::GRANITE.default_state,
            }],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreGravel,
        ConfiguredFeature::Ore(OreFeature {
            size: 33i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::TagMatch(TagMatchRuleTest {
                    tag: pumpkin_data::tag::Block::MINECRAFT_BASE_STONE_OVERWORLD,
                }),
                state: pumpkin_data::Block::GRAVEL.default_state,
            }],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreGravelNether,
        ConfiguredFeature::Ore(OreFeature {
            size: 33i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::BlockMatch(BlockMatchRuleTest {
                    block: pumpkin_data::BlockId::NETHERRACK,
                }),
                state: pumpkin_data::Block::GRAVEL.default_state,
            }],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreInfested,
        ConfiguredFeature::Ore(OreFeature {
            size: 9i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_STONE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::INFESTED_STONE.default_state,
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_DEEPSLATE_ORE_REPLACEABLES,
                    }),
                    state: {
                        let mut props = std::collections::HashMap::new();
                        props.insert("axis".to_string(), "y".to_string());
                        BlockStateCodec {
                            name: &pumpkin_data::Block::INFESTED_DEEPSLATE,
                            properties: Some(props),
                        }
                        .get_state()
                    },
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreIron,
        ConfiguredFeature::Ore(OreFeature {
            size: 9i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_STONE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::IRON_ORE.default_state,
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_DEEPSLATE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DEEPSLATE_IRON_ORE.default_state,
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreIronSmall,
        ConfiguredFeature::Ore(OreFeature {
            size: 4i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_STONE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::IRON_ORE.default_state,
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_DEEPSLATE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DEEPSLATE_IRON_ORE.default_state,
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreLapis,
        ConfiguredFeature::Ore(OreFeature {
            size: 7i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_STONE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::LAPIS_ORE.default_state,
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_DEEPSLATE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DEEPSLATE_LAPIS_ORE.default_state,
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreLapisBuried,
        ConfiguredFeature::Ore(OreFeature {
            size: 7i32,
            discard_chance_on_air_exposure: 1f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_STONE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::LAPIS_ORE.default_state,
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_DEEPSLATE_ORE_REPLACEABLES,
                    }),
                    state: pumpkin_data::Block::DEEPSLATE_LAPIS_ORE.default_state,
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreMagma,
        ConfiguredFeature::Ore(OreFeature {
            size: 33i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::BlockMatch(BlockMatchRuleTest {
                    block: pumpkin_data::BlockId::NETHERRACK,
                }),
                state: pumpkin_data::Block::MAGMA_BLOCK.default_state,
            }],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreNetherGold,
        ConfiguredFeature::Ore(OreFeature {
            size: 10i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::BlockMatch(BlockMatchRuleTest {
                    block: pumpkin_data::BlockId::NETHERRACK,
                }),
                state: pumpkin_data::Block::NETHER_GOLD_ORE.default_state,
            }],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreQuartz,
        ConfiguredFeature::Ore(OreFeature {
            size: 14i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::BlockMatch(BlockMatchRuleTest {
                    block: pumpkin_data::BlockId::NETHERRACK,
                }),
                state: pumpkin_data::Block::NETHER_QUARTZ_ORE.default_state,
            }],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreRedstone,
        ConfiguredFeature::Ore(OreFeature {
            size: 8i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_STONE_ORE_REPLACEABLES,
                    }),
                    state: {
                        let mut props = std::collections::HashMap::new();
                        props.insert("lit".to_string(), "false".to_string());
                        BlockStateCodec {
                            name: &pumpkin_data::Block::REDSTONE_ORE,
                            properties: Some(props),
                        }
                        .get_state()
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: pumpkin_data::tag::Block::MINECRAFT_DEEPSLATE_ORE_REPLACEABLES,
                    }),
                    state: {
                        let mut props = std::collections::HashMap::new();
                        props.insert("lit".to_string(), "false".to_string());
                        BlockStateCodec {
                            name: &pumpkin_data::Block::DEEPSLATE_REDSTONE_ORE,
                            properties: Some(props),
                        }
                        .get_state()
                    },
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreSoulSand,
        ConfiguredFeature::Ore(OreFeature {
            size: 12i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::BlockMatch(BlockMatchRuleTest {
                    block: pumpkin_data::BlockId::NETHERRACK,
                }),
                state: pumpkin_data::Block::SOUL_SAND.default_state,
            }],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::OreTuff,
        ConfiguredFeature::Ore(OreFeature {
            size: 64i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::TagMatch(TagMatchRuleTest {
                    tag: pumpkin_data::tag::Block::MINECRAFT_BASE_STONE_OVERWORLD,
                }),
                state: pumpkin_data::Block::TUFF.default_state,
            }],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::PaleForestFlower,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::CLOSED_EYEBLOSSOM.default_state,
            }),
            schedule_tick: Some(true),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::PaleGardenVegetation,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::PaleOakCreakingChecked,
                    ),
                    chance: 0.1f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::PaleOakChecked,
                    ),
                    chance: 0.9f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::PaleOakChecked,
            )),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::PaleMossPatch,
        ConfiguredFeature::VegetationPatch(vegetation_patch::VegetationPatchFeature {
            replaceable: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                offset: OffsetBlocksBlockPredicate { offset: None },
                tag: pumpkin_data::tag::Block::MINECRAFT_MOSS_REPLACEABLE,
            }),
            ground_state: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::PALE_MOSS_BLOCK.default_state,
            }),
            vegetation_feature: Box::new(PlacedFeature {
                feature: Feature::Named(
                    pumpkin_data::configured_feature::ConfiguredFeature::PaleMossVegetation,
                ),
                placement: vec![],
            }),
            surface: pumpkin_util::math::vertical_surface_type::VerticalSurfaceType::Floor,
            depth: IntProvider::Constant(1i32),
            extra_bottom_block_chance: 0f32,
            vertical_range: 5i32,
            vegetation_chance: 0.3f32,
            xz_radius: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                min_inclusive: 2i32,
                max_inclusive: 4i32,
            })),
            extra_edge_column_chance: 0.75f32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::PaleMossPatchBonemeal,
        ConfiguredFeature::VegetationPatch(vegetation_patch::VegetationPatchFeature {
            replaceable: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                offset: OffsetBlocksBlockPredicate { offset: None },
                tag: pumpkin_data::tag::Block::MINECRAFT_MOSS_REPLACEABLE,
            }),
            ground_state: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::PALE_MOSS_BLOCK.default_state,
            }),
            vegetation_feature: Box::new(PlacedFeature {
                feature: Feature::Named(
                    pumpkin_data::configured_feature::ConfiguredFeature::PaleMossVegetation,
                ),
                placement: vec![],
            }),
            surface: pumpkin_util::math::vertical_surface_type::VerticalSurfaceType::Floor,
            depth: IntProvider::Constant(1i32),
            extra_bottom_block_chance: 0f32,
            vertical_range: 5i32,
            vegetation_chance: 0.6f32,
            xz_radius: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                min_inclusive: 1i32,
                max_inclusive: 2i32,
            })),
            extra_edge_column_chance: 0.75f32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::PaleMossVegetation,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("west".to_string(), "none".to_string());
                            props.insert("south".to_string(), "none".to_string());
                            props.insert("north".to_string(), "none".to_string());
                            props.insert("east".to_string(), "none".to_string());
                            props.insert("bottom".to_string(), "true".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PALE_MOSS_CARPET,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 25i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::SHORT_GRASS.default_state,
                        weight: 25i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("half".to_string(), "lower".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::TALL_GRASS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 10i32,
                    },
                ],
            }),
            schedule_tick: None,
        }),
    );
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: PaleOak , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: PALE_OAK_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 6u8 , height_rand_a : 2u8 , height_rand_b : 1u8 , r#type : TrunkType :: DarkOak (DarkOakTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: PALE_OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (0i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: DarkOak (DarkOakFoliagePlacer) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: ThreeLayersFeatureSize (ThreeLayersFeatureSize { limit : 1u8 , upper_limit : 1u8 , lower_size : 0u8 , middle_size : 1u8 , upper_size : 2u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: PaleMoss (PaleMossTreeDecorator { })] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: PaleOakBonemeal , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: PALE_OAK_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 6u8 , height_rand_a : 2u8 , height_rand_b : 1u8 , r#type : TrunkType :: DarkOak (DarkOakTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: PALE_OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (0i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: DarkOak (DarkOakFoliagePlacer) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: ThreeLayersFeatureSize (ThreeLayersFeatureSize { limit : 1u8 , upper_limit : 1u8 , lower_size : 0u8 , middle_size : 1u8 , upper_size : 2u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: PaleOakCreaking , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: PALE_OAK_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 6u8 , height_rand_a : 2u8 , height_rand_b : 1u8 , r#type : TrunkType :: DarkOak (DarkOakTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: PALE_OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (0i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: DarkOak (DarkOakFoliagePlacer) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: ThreeLayersFeatureSize (ThreeLayersFeatureSize { limit : 1u8 , upper_limit : 1u8 , lower_size : 0u8 , middle_size : 1u8 , upper_size : 2u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: PaleMoss (PaleMossTreeDecorator { }) , TreeDecorator :: CreakingHeart (CreakingHeartTreeDecorator { })] , root_placer : None , }))) ;
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::PatchFire,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("west".to_string(), "false".to_string());
                    props.insert("up".to_string(), "false".to_string());
                    props.insert("south".to_string(), "false".to_string());
                    props.insert("north".to_string(), "false".to_string());
                    props.insert("east".to_string(), "false".to_string());
                    props.insert("age".to_string(), "0".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::FIRE,
                        properties: Some(props),
                    }
                    .get_state()
                },
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::PatchSoulFire,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::SOUL_FIRE.default_state,
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::PileHay,
        ConfiguredFeature::BlockPile(
            crate::generation::feature::features::block_pile::BlockPileFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::PileIce,
        ConfiguredFeature::BlockPile(
            crate::generation::feature::features::block_pile::BlockPileFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::PileMelon,
        ConfiguredFeature::BlockPile(
            crate::generation::feature::features::block_pile::BlockPileFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::PilePumpkin,
        ConfiguredFeature::BlockPile(
            crate::generation::feature::features::block_pile::BlockPileFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::PileSnow,
        ConfiguredFeature::BlockPile(
            crate::generation::feature::features::block_pile::BlockPileFeature {},
        ),
    );
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: Pine , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: SPRUCE_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 6u8 , height_rand_a : 4u8 , height_rand_b : 0u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: SPRUCE_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (1i32) , offset : IntProvider :: Constant (1i32) , r#type : FoliageType :: Pine (PineFoliagePlacer { height : IntProvider :: Object (NormalIntProvider :: Uniform (UniformIntProvider { min_inclusive : 3i32 , max_inclusive : 4i32 })) }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 2u8 , lower_size : 0u8 , upper_size : 2u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [] , root_placer : None , }))) ;
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::PointedDripstone,
        ConfiguredFeature::SimpleRandomSelector(SimpleRandomFeature {
            features: vec![
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::PointedDripstone(
                        SmallDripstoneFeature {
                            taller_dripstone: 0.2f32,
                            directional_spread: 0.7f32,
                            spread_radius2: 0.5f32,
                            spread_radius3: 0.5f32,
                        },
                    ))),
                    placement: vec![
                        PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier {
                            direction_of_search: BlockDirection::Down,
                            target_condition: BlockPredicate::Solid(SolidBlockPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                            }),
                            allowed_search_condition: Some(BlockPredicate::AnyOf(
                                AnyOfBlockPredicate {
                                    predicates: vec![
                                        BlockPredicate::MatchingBlockTag(
                                            MatchingBlockTagPredicate {
                                                offset: OffsetBlocksBlockPredicate { offset: None },
                                                tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                                            },
                                        ),
                                        BlockPredicate::MatchingBlocks(
                                            MatchingBlocksBlockPredicate {
                                                offset: OffsetBlocksBlockPredicate { offset: None },
                                                blocks: MatchingBlocksWrapper::Single(
                                                    "minecraft:water".to_string(),
                                                ),
                                            },
                                        ),
                                    ],
                                },
                            )),
                            max_steps: 12i32,
                        }),
                        PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                            xz_spread: IntProvider::Constant(0i32),
                            y_spread: IntProvider::Constant(1i32),
                        }),
                    ],
                },
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::PointedDripstone(
                        SmallDripstoneFeature {
                            taller_dripstone: 0.2f32,
                            directional_spread: 0.7f32,
                            spread_radius2: 0.5f32,
                            spread_radius3: 0.5f32,
                        },
                    ))),
                    placement: vec![
                        PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier {
                            direction_of_search: BlockDirection::Up,
                            target_condition: BlockPredicate::Solid(SolidBlockPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                            }),
                            allowed_search_condition: Some(BlockPredicate::AnyOf(
                                AnyOfBlockPredicate {
                                    predicates: vec![
                                        BlockPredicate::MatchingBlockTag(
                                            MatchingBlockTagPredicate {
                                                offset: OffsetBlocksBlockPredicate { offset: None },
                                                tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                                            },
                                        ),
                                        BlockPredicate::MatchingBlocks(
                                            MatchingBlocksBlockPredicate {
                                                offset: OffsetBlocksBlockPredicate { offset: None },
                                                blocks: MatchingBlocksWrapper::Single(
                                                    "minecraft:water".to_string(),
                                                ),
                                            },
                                        ),
                                    ],
                                },
                            )),
                            max_steps: 12i32,
                        }),
                        PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                            xz_spread: IntProvider::Constant(0i32),
                            y_spread: IntProvider::Constant(-1i32),
                        }),
                    ],
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::Pumpkin,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::PUMPKIN.default_state,
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::RedMushroom,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::RED_MUSHROOM.default_state,
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::RootedAzaleaTree,
        ConfiguredFeature::RootSystem(
            crate::generation::feature::features::root_system::RootSystemFeature {
                feature: Box::new(PlacedFeature {
                    feature: Feature::Named(
                        pumpkin_data::configured_feature::ConfiguredFeature::AzaleaTree,
                    ),
                    placement: vec![],
                }),
                required_vertical_space_for_tree: 3i32,
                root_radius: 3i32,
                root_replaceable: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                    offset: OffsetBlocksBlockPredicate { offset: None },
                    tag: pumpkin_data::tag::Block::MINECRAFT_AZALEA_ROOT_REPLACEABLE,
                }),
                root_state_provider: BlockStateProvider::Simple(SimpleStateProvider {
                    state: pumpkin_data::Block::ROOTED_DIRT.default_state,
                }),
                root_placement_attempts: 20i32,
                root_column_max_height: 100i32,
                hanging_root_radius: 3i32,
                hanging_roots_vertical_span: 2i32,
                hanging_root_state_provider: BlockStateProvider::Simple(SimpleStateProvider {
                    state: {
                        let mut props = std::collections::HashMap::new();
                        props.insert("waterlogged".to_string(), "false".to_string());
                        BlockStateCodec {
                            name: &pumpkin_data::Block::HANGING_ROOTS,
                            properties: Some(props),
                        }
                        .get_state()
                    },
                }),
                hanging_root_placement_attempts: 20i32,
                allowed_vertical_water_for_tree: 2i32,
                allowed_tree_position: BlockPredicate::AllOf(AllOfBlockPredicate {
                    predicates: vec![
                        BlockPredicate::AnyOf(AnyOfBlockPredicate {
                            predicates: vec![
                                BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                                }),
                                BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    tag: pumpkin_data::tag::Block::MINECRAFT_REPLACEABLE_BY_TREES,
                                }),
                            ],
                        }),
                        BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                            offset: OffsetBlocksBlockPredicate {
                                offset: Some(Vector3::new(0i32, -1i32, 0i32)),
                            },
                            tag: pumpkin_data::tag::Block::MINECRAFT_AZALEA_GROWS_ON,
                        }),
                    ],
                }),
            },
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::RootedSulfurSpring,
        ConfiguredFeature::RootSystem(
            crate::generation::feature::features::root_system::RootSystemFeature {
                feature: Box::new(PlacedFeature {
                    feature: Feature::Named(
                        pumpkin_data::configured_feature::ConfiguredFeature::SulfurSpring,
                    ),
                    placement: vec![],
                }),
                required_vertical_space_for_tree: 5i32,
                root_radius: 3i32,
                root_replaceable: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                    offset: OffsetBlocksBlockPredicate { offset: None },
                    tag: pumpkin_data::tag::Block::MINECRAFT_AZALEA_ROOT_REPLACEABLE,
                }),
                root_state_provider: BlockStateProvider::Simple(SimpleStateProvider {
                    state: pumpkin_data::Block::SULFUR.default_state,
                }),
                root_placement_attempts: 20i32,
                root_column_max_height: 184i32,
                hanging_root_radius: 1i32,
                hanging_roots_vertical_span: 1i32,
                hanging_root_state_provider: BlockStateProvider::Simple(SimpleStateProvider {
                    state: pumpkin_data::Block::SULFUR.default_state,
                }),
                hanging_root_placement_attempts: 1i32,
                allowed_vertical_water_for_tree: 1i32,
                allowed_tree_position: BlockPredicate::MatchingBlockTag(
                    MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    },
                ),
            },
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SculkPatchAncientCity,
        ConfiguredFeature::SculkPatch(
            crate::generation::feature::features::sculk_patch::SculkPatchFeature {
                charge_count: 10i32,
                amount_per_charge: 32i32,
                spread_attempts: 64i32,
                growth_rounds: 0i32,
                spread_rounds: 1i32,
                extra_rare_growths: IntProvider::Object(NormalIntProvider::Uniform(
                    UniformIntProvider {
                        min_inclusive: 1i32,
                        max_inclusive: 3i32,
                    },
                )),
                catalyst_chance: 0.5f32,
            },
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SculkPatchDeepDark,
        ConfiguredFeature::SculkPatch(
            crate::generation::feature::features::sculk_patch::SculkPatchFeature {
                charge_count: 10i32,
                amount_per_charge: 32i32,
                spread_attempts: 64i32,
                growth_rounds: 0i32,
                spread_rounds: 1i32,
                extra_rare_growths: IntProvider::Constant(0i32),
                catalyst_chance: 0.5f32,
            },
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SculkVein,
        ConfiguredFeature::MultifaceGrowth(
            crate::generation::feature::features::multiface_growth::MultifaceGrowthFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SeaPickle,
        ConfiguredFeature::SeaPickle(SeaPickleFeature {
            count: IntProvider::Constant(20i32),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SeagrassMid,
        ConfiguredFeature::Seagrass(SeagrassFeature {
            probability: 0.6f32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SeagrassShort,
        ConfiguredFeature::Seagrass(SeagrassFeature {
            probability: 0.3f32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SeagrassSlightlyLessShort,
        ConfiguredFeature::Seagrass(SeagrassFeature {
            probability: 0.4f32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SeagrassTall,
        ConfiguredFeature::Seagrass(SeagrassFeature {
            probability: 0.8f32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SmallBasaltColumns,
        ConfiguredFeature::BasaltColumns(
            crate::generation::feature::features::basalt_columns::BasaltColumnsFeature {
                height: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                    min_inclusive: 1i32,
                    max_inclusive: 4i32,
                })),
                reach: IntProvider::Constant(1i32),
            },
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SporeBlossom,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::SPORE_BLOSSOM.default_state,
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SpringLavaFrozen,
        ConfiguredFeature::SpringFeature(SpringFeatureFeature {
            state: {
                let mut props = std::collections::HashMap::new();
                props.insert("falling".to_string(), "true".to_string());
                BlockStateCodec {
                    name: &pumpkin_data::Block::LAVA,
                    properties: Some(props),
                }
                .get_state()
            },
            requires_block_below: true,
            rock_count: 4i32,
            hole_count: 1i32,
            valid_blocks: BlockWrapper::Multi(vec![
                "minecraft:snow_block".to_string(),
                "minecraft:powder_snow".to_string(),
                "minecraft:packed_ice".to_string(),
            ]),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SpringLavaNether,
        ConfiguredFeature::SpringFeature(SpringFeatureFeature {
            state: {
                let mut props = std::collections::HashMap::new();
                props.insert("falling".to_string(), "true".to_string());
                BlockStateCodec {
                    name: &pumpkin_data::Block::LAVA,
                    properties: Some(props),
                }
                .get_state()
            },
            requires_block_below: true,
            rock_count: 4i32,
            hole_count: 1i32,
            valid_blocks: BlockWrapper::Multi(vec![
                "minecraft:netherrack".to_string(),
                "minecraft:soul_sand".to_string(),
                "minecraft:gravel".to_string(),
                "minecraft:magma_block".to_string(),
                "minecraft:blackstone".to_string(),
            ]),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SpringLavaOverworld,
        ConfiguredFeature::SpringFeature(SpringFeatureFeature {
            state: {
                let mut props = std::collections::HashMap::new();
                props.insert("falling".to_string(), "true".to_string());
                BlockStateCodec {
                    name: &pumpkin_data::Block::LAVA,
                    properties: Some(props),
                }
                .get_state()
            },
            requires_block_below: true,
            rock_count: 4i32,
            hole_count: 1i32,
            valid_blocks: BlockWrapper::Multi(vec![
                "minecraft:stone".to_string(),
                "minecraft:granite".to_string(),
                "minecraft:diorite".to_string(),
                "minecraft:andesite".to_string(),
                "minecraft:deepslate".to_string(),
                "minecraft:tuff".to_string(),
                "minecraft:calcite".to_string(),
                "minecraft:dirt".to_string(),
            ]),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SpringNetherClosed,
        ConfiguredFeature::SpringFeature(SpringFeatureFeature {
            state: {
                let mut props = std::collections::HashMap::new();
                props.insert("falling".to_string(), "true".to_string());
                BlockStateCodec {
                    name: &pumpkin_data::Block::LAVA,
                    properties: Some(props),
                }
                .get_state()
            },
            requires_block_below: false,
            rock_count: 5i32,
            hole_count: 0i32,
            valid_blocks: BlockWrapper::Single("minecraft:netherrack".to_string()),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SpringNetherOpen,
        ConfiguredFeature::SpringFeature(SpringFeatureFeature {
            state: {
                let mut props = std::collections::HashMap::new();
                props.insert("falling".to_string(), "true".to_string());
                BlockStateCodec {
                    name: &pumpkin_data::Block::LAVA,
                    properties: Some(props),
                }
                .get_state()
            },
            requires_block_below: false,
            rock_count: 4i32,
            hole_count: 1i32,
            valid_blocks: BlockWrapper::Single("minecraft:netherrack".to_string()),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SpringWater,
        ConfiguredFeature::SpringFeature(SpringFeatureFeature {
            state: {
                let mut props = std::collections::HashMap::new();
                props.insert("falling".to_string(), "true".to_string());
                BlockStateCodec {
                    name: &pumpkin_data::Block::WATER,
                    properties: Some(props),
                }
                .get_state()
            },
            requires_block_below: true,
            rock_count: 4i32,
            hole_count: 1i32,
            valid_blocks: BlockWrapper::Multi(vec![
                "minecraft:stone".to_string(),
                "minecraft:granite".to_string(),
                "minecraft:diorite".to_string(),
                "minecraft:andesite".to_string(),
                "minecraft:deepslate".to_string(),
                "minecraft:tuff".to_string(),
                "minecraft:calcite".to_string(),
                "minecraft:dirt".to_string(),
                "minecraft:snow_block".to_string(),
                "minecraft:powder_snow".to_string(),
                "minecraft:packed_ice".to_string(),
            ]),
        }),
    );
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: Spruce , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: SPRUCE_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 5u8 , height_rand_a : 2u8 , height_rand_b : 1u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: SPRUCE_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Object (NormalIntProvider :: Uniform (UniformIntProvider { min_inclusive : 2i32 , max_inclusive : 3i32 })) , offset : IntProvider :: Object (NormalIntProvider :: Uniform (UniformIntProvider { min_inclusive : 0i32 , max_inclusive : 2i32 })) , r#type : FoliageType :: Spruce (SpruceFoliagePlacer { trunk_height : IntProvider :: Object (NormalIntProvider :: Uniform (UniformIntProvider { min_inclusive : 1i32 , max_inclusive : 2i32 })) }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 2u8 , lower_size : 0u8 , upper_size : 2u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [] , root_placer : None , }))) ;
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SugarCane,
        ConfiguredFeature::BlockColumn(BlockColumnFeature {
            layers: vec![Layer {
                height: IntProvider::Object(NormalIntProvider::BiasedToBottom(
                    BiasedToBottomIntProvider {
                        min_inclusive: 2i32,
                        max_inclusive: 4i32,
                    },
                )),
                provider: BlockStateProvider::Simple(SimpleStateProvider {
                    state: {
                        let mut props = std::collections::HashMap::new();
                        props.insert("age".to_string(), "0".to_string());
                        BlockStateCodec {
                            name: &pumpkin_data::Block::SUGAR_CANE,
                            properties: Some(props),
                        }
                        .get_state()
                    },
                }),
            }],
            direction: BlockDirection::Up,
            allowed_placement: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                offset: OffsetBlocksBlockPredicate { offset: None },
                tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
            }),
            prioritize_tip: false,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SulfurPool,
        ConfiguredFeature::NoOp,
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SulfurSpike,
        ConfiguredFeature::SimpleRandomSelector(SimpleRandomFeature {
            features: vec![
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::PointedDripstone(
                        SmallDripstoneFeature {
                            taller_dripstone: 0.2f32,
                            directional_spread: 0.7f32,
                            spread_radius2: 0.5f32,
                            spread_radius3: 0.5f32,
                        },
                    ))),
                    placement: vec![
                        PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier {
                            direction_of_search: BlockDirection::Down,
                            target_condition: BlockPredicate::Solid(SolidBlockPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                            }),
                            allowed_search_condition: Some(BlockPredicate::AnyOf(
                                AnyOfBlockPredicate {
                                    predicates: vec![
                                        BlockPredicate::MatchingBlockTag(
                                            MatchingBlockTagPredicate {
                                                offset: OffsetBlocksBlockPredicate { offset: None },
                                                tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                                            },
                                        ),
                                        BlockPredicate::MatchingBlocks(
                                            MatchingBlocksBlockPredicate {
                                                offset: OffsetBlocksBlockPredicate { offset: None },
                                                blocks: MatchingBlocksWrapper::Single(
                                                    "minecraft:water".to_string(),
                                                ),
                                            },
                                        ),
                                    ],
                                },
                            )),
                            max_steps: 12i32,
                        }),
                        PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                            xz_spread: IntProvider::Constant(0i32),
                            y_spread: IntProvider::Constant(1i32),
                        }),
                    ],
                },
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::PointedDripstone(
                        SmallDripstoneFeature {
                            taller_dripstone: 0.2f32,
                            directional_spread: 0.7f32,
                            spread_radius2: 0.5f32,
                            spread_radius3: 0.5f32,
                        },
                    ))),
                    placement: vec![
                        PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier {
                            direction_of_search: BlockDirection::Up,
                            target_condition: BlockPredicate::Solid(SolidBlockPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                            }),
                            allowed_search_condition: Some(BlockPredicate::AnyOf(
                                AnyOfBlockPredicate {
                                    predicates: vec![
                                        BlockPredicate::MatchingBlockTag(
                                            MatchingBlockTagPredicate {
                                                offset: OffsetBlocksBlockPredicate { offset: None },
                                                tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                                            },
                                        ),
                                        BlockPredicate::MatchingBlocks(
                                            MatchingBlocksBlockPredicate {
                                                offset: OffsetBlocksBlockPredicate { offset: None },
                                                blocks: MatchingBlocksWrapper::Single(
                                                    "minecraft:water".to_string(),
                                                ),
                                            },
                                        ),
                                    ],
                                },
                            )),
                            max_steps: 12i32,
                        }),
                        PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                            xz_spread: IntProvider::Constant(0i32),
                            y_spread: IntProvider::Constant(-1i32),
                        }),
                    ],
                },
            ],
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SulfurSpikeCluster,
        ConfiguredFeature::DripstoneCluster(
            crate::generation::feature::features::drip_stone::cluster::DripstoneClusterFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::SulfurSpring,
        ConfiguredFeature::NoOp,
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::Sunflower,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("half".to_string(), "lower".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::SUNFLOWER,
                        properties: Some(props),
                    }
                    .get_state()
                },
            }),
            schedule_tick: None,
        }),
    );
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: SuperBirchBees , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: BIRCH_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 5u8 , height_rand_a : 2u8 , height_rand_b : 6u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: BIRCH_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Blob (BlobFoliagePlacer { height : 3i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 1u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: Beehive (BeehiveTreeDecorator { probability : 1f32 })] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: SuperBirchBees0002 , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: BIRCH_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 5u8 , height_rand_a : 2u8 , height_rand_b : 6u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: BIRCH_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (2i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Blob (BlobFoliagePlacer { height : 3i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 1u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: Beehive (BeehiveTreeDecorator { probability : 0.002f32 })] , root_placer : None , }))) ;
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: SwampOak , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 5u8 , height_rand_a : 3u8 , height_rand_b : 0u8 , r#type : TrunkType :: Straight (StraightTrunkPlacer) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: OAK_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (3i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: Blob (BlobFoliagePlacer { height : 3i32 }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 1u8 , lower_size : 0u8 , upper_size : 1u8 , }) } , ignore_vines : false , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: LeaveVine (LeavesVineTreeDecorator { probability : 0.25f32 })] , root_placer : None , }))) ;
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TaigaGrass,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: pumpkin_data::Block::SHORT_GRASS.default_state,
                        weight: 1i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::FERN.default_state,
                        weight: 4i32,
                    },
                ],
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TallGrass,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("half".to_string(), "lower".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::TALL_GRASS,
                        properties: Some(props),
                    }
                    .get_state()
                },
            }),
            schedule_tick: None,
        }),
    );
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: TallMangrove , ConfiguredFeature :: Tree (Box :: new (TreeFeature { trunk_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: MANGROVE_LOG , properties : Some (props) , } . get_state () } }) , trunk_placer : TrunkPlacer { base_height : 4u8 , height_rand_a : 1u8 , height_rand_b : 9u8 , r#type : TrunkType :: UpwardsBranching (UpwardsBranchingTrunkPlacer { extra_branch_steps : IntProvider :: Object (NormalIntProvider :: Uniform (UniformIntProvider { min_inclusive : 1i32 , max_inclusive : 6i32 })) , place_branch_per_log_probability : 0.5f32 , extra_branch_length : IntProvider :: Object (NormalIntProvider :: Uniform (UniformIntProvider { min_inclusive : 0i32 , max_inclusive : 1i32 })) , can_grow_through : & pumpkin_data :: tag :: Block :: MINECRAFT_MANGROVE_LOGS_CAN_GROW_THROUGH . 1 , }) , } , foliage_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; props . insert ("persistent" . to_string () , "false" . to_string ()) ; props . insert ("distance" . to_string () , "7" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: MANGROVE_LEAVES , properties : Some (props) , } . get_state () } }) , foliage_placer : FoliagePlacer { radius : IntProvider :: Constant (3i32) , offset : IntProvider :: Constant (0i32) , r#type : FoliageType :: RandomSpread (RandomSpreadFoliagePlacer { foliage_height : IntProvider :: Constant (2i32) , leaf_placement_attempts : 70i32 , }) } , minimum_size : FeatureSize { min_clipped_height : None , r#type : FeatureSizeType :: TwoLayersFeatureSize (TwoLayersFeatureSize { limit : 3u8 , lower_size : 0u8 , upper_size : 2u8 , }) } , ignore_vines : true , below_trunk_provider : BlockStateProvider :: Rule (RuleBasedBlockStateProvider { fallback : None , rules : vec ! [BlockStateRule { if_true : BlockPredicate :: Not (NotBlockPredicate { predicate : Box :: new (BlockPredicate :: MatchingBlockTag (MatchingBlockTagPredicate { offset : OffsetBlocksBlockPredicate { offset : None } , tag : pumpkin_data :: tag :: Block :: MINECRAFT_CANNOT_REPLACE_BELOW_TREE_TRUNK , })) , }) , then : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: DIRT . default_state }) }] , }) , decorators : vec ! [TreeDecorator :: LeaveVine (LeavesVineTreeDecorator { probability : 0.125f32 }) , TreeDecorator :: AttachedToLeaves (AttachedToLeavesTreeDecorator { }) , TreeDecorator :: Beehive (BeehiveTreeDecorator { probability : 0.01f32 })] , root_placer : Some (RootPlacer :: Mangrove (MangroveRootPlacer { trunk_offset_y : IntProvider :: Object (NormalIntProvider :: Uniform (UniformIntProvider { min_inclusive : 3i32 , max_inclusive : 7i32 })) , root_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("waterlogged" . to_string () , "false" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: MANGROVE_ROOTS , properties : Some (props) , } . get_state () } }) , above_root_placement : Some (AboveRootPlacement { above_root_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : pumpkin_data :: Block :: MOSS_CARPET . default_state }) , above_root_placement_chance : 0.5f32 , }) , mangrove_root_placement : MangroveRootPlacement { can_grow_through : & pumpkin_data :: tag :: Block :: MINECRAFT_MANGROVE_ROOTS_CAN_GROW_THROUGH . 1 , muddy_roots_in : const { & [pumpkin_data :: BlockId :: MUD . as_u16 () , pumpkin_data :: BlockId :: MUDDY_MANGROVE_ROOTS . as_u16 ()] } , muddy_roots_provider : BlockStateProvider :: Simple (SimpleStateProvider { state : { let mut props = std :: collections :: HashMap :: new () ; props . insert ("axis" . to_string () , "y" . to_string ()) ; BlockStateCodec { name : & pumpkin_data :: Block :: MUDDY_MANGROVE_ROOTS , properties : Some (props) , } . get_state () } }) , max_root_width : 8i32 , max_root_length : 15i32 , random_skew_chance : 0.2f32 , } , })) , }))) ;
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TreesBadlands,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![RandomFeatureEntry {
                feature: PlacedFeatureWrapper::Named(
                    pumpkin_data::placed_feature::PlacedFeature::FallenOakTree,
                ),
                chance: 0.0125f32,
            }],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::OakLeafLitter,
            )),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TreesBirch,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![RandomFeatureEntry {
                feature: PlacedFeatureWrapper::Named(
                    pumpkin_data::placed_feature::PlacedFeature::FallenBirchTree,
                ),
                chance: 0.0125f32,
            }],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::BirchBees0002,
            )),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TreesBirchAndOakLeafLitter,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FallenBirchTree,
                    ),
                    chance: 0.0025f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::BirchBees0002LeafLitter,
                    ),
                    chance: 0.2f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FancyOakBees0002LeafLitter,
                    ),
                    chance: 0.1f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FallenOakTree,
                    ),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::OakBees0002LeafLitter,
            )),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TreesFlowerForest,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FallenBirchTree,
                    ),
                    chance: 0.0025f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::BirchBees002,
                    ),
                    chance: 0.2f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FancyOakBees002,
                    ),
                    chance: 0.1f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::OakBees002,
            )),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TreesGrove,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![RandomFeatureEntry {
                feature: PlacedFeatureWrapper::Named(
                    pumpkin_data::placed_feature::PlacedFeature::PineOnSnow,
                ),
                chance: 0.33333334f32,
            }],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::SpruceOnSnow,
            )),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TreesJungle,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FancyOakChecked,
                    ),
                    chance: 0.1f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::JungleBush,
                    ),
                    chance: 0.5f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::MegaJungleTreeChecked,
                    ),
                    chance: 0.33333334f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FallenJungleTree,
                    ),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::JungleTree,
            )),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TreesOldGrowthPineTaiga,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::MegaSpruceChecked,
                    ),
                    chance: 0.025641026f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::MegaPineChecked,
                    ),
                    chance: 0.30769232f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::PineChecked,
                    ),
                    chance: 0.33333334f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FallenSpruceTree,
                    ),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::SpruceChecked,
            )),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TreesOldGrowthSpruceTaiga,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::MegaSpruceChecked,
                    ),
                    chance: 0.33333334f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::PineChecked,
                    ),
                    chance: 0.33333334f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FallenSpruceTree,
                    ),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::SpruceChecked,
            )),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TreesPlains,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Direct(PlacedFeature {
                        feature: Feature::Named(
                            pumpkin_data::configured_feature::ConfiguredFeature::FancyOakBees005,
                        ),
                        placement: vec![],
                    }),
                    chance: 0.33333334f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FallenOakTree,
                    ),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Direct(PlacedFeature {
                feature: Feature::Named(
                    pumpkin_data::configured_feature::ConfiguredFeature::OakBees005,
                ),
                placement: vec![],
            })),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TreesSavanna,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::AcaciaChecked,
                    ),
                    chance: 0.8f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FallenOakTree,
                    ),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::OakChecked,
            )),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TreesSnowy,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![RandomFeatureEntry {
                feature: PlacedFeatureWrapper::Named(
                    pumpkin_data::placed_feature::PlacedFeature::FallenSpruceTree,
                ),
                chance: 0.0125f32,
            }],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::SpruceChecked,
            )),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TreesSparseJungle,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FancyOakChecked,
                    ),
                    chance: 0.1f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::JungleBush,
                    ),
                    chance: 0.5f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FallenJungleTree,
                    ),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::JungleTree,
            )),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TreesTaiga,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::PineChecked,
                    ),
                    chance: 0.33333334f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FallenSpruceTree,
                    ),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::SpruceChecked,
            )),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TreesWater,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![RandomFeatureEntry {
                feature: PlacedFeatureWrapper::Named(
                    pumpkin_data::placed_feature::PlacedFeature::FancyOakChecked,
                ),
                chance: 0.1f32,
            }],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::OakChecked,
            )),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TreesWindsweptHills,
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FallenSpruceTree,
                    ),
                    chance: 0.008325f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::SpruceChecked,
                    ),
                    chance: 0.666f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FancyOakChecked,
                    ),
                    chance: 0.1f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        pumpkin_data::placed_feature::PlacedFeature::FallenOakTree,
                    ),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named(
                pumpkin_data::placed_feature::PlacedFeature::OakChecked,
            )),
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TwistingVines,
        ConfiguredFeature::TwistingVines(
            crate::generation::feature::features::twisting_vines::TwistingVinesFeature {
                spread_width: 8i32,
                spread_height: 4i32,
                max_height: 8i32,
            },
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::TwistingVinesBonemeal,
        ConfiguredFeature::TwistingVines(
            crate::generation::feature::features::twisting_vines::TwistingVinesFeature {
                spread_width: 3i32,
                spread_height: 1i32,
                max_height: 2i32,
            },
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::UnderwaterMagma,
        ConfiguredFeature::UnderwaterMagma(
            crate::generation::feature::features::underwater_magma::UnderwaterMagmaFeature {
                floor_search_range: 5i32,
                placement_radius: 1i32,
                placement_probability: 0.5f32,
            },
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::Vines,
        ConfiguredFeature::Vines(crate::generation::feature::features::vines::VinesFeature),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::VoidStartPlatform,
        ConfiguredFeature::VoidStartPlatform(
            crate::generation::feature::features::void_start_platform::VoidStartPlatformFeature {},
        ),
    );
    map . insert (pumpkin_data :: configured_feature :: ConfiguredFeature :: WarmOceanVegetation , ConfiguredFeature :: SimpleRandomSelector (SimpleRandomFeature { features : vec ! [PlacedFeature { feature : Feature :: Inlined (Box :: new (ConfiguredFeature :: CoralTree (crate :: generation :: feature :: features :: coral :: coral_tree :: CoralTreeFeature))) , placement : vec ! [] , } , PlacedFeature { feature : Feature :: Inlined (Box :: new (ConfiguredFeature :: CoralClaw (crate :: generation :: feature :: features :: coral :: coral_claw :: CoralClawFeature))) , placement : vec ! [] , } , PlacedFeature { feature : Feature :: Inlined (Box :: new (ConfiguredFeature :: CoralMushroom (crate :: generation :: feature :: features :: coral :: coral_mushroom :: CoralMushroomFeature))) , placement : vec ! [] , }] , })) ;
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::WarpedForestVegetation,
        ConfiguredFeature::NetherForestVegetation(NetherForestVegetationFeature {
            state_provider: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: pumpkin_data::Block::WARPED_ROOTS.default_state,
                        weight: 85i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::CRIMSON_ROOTS.default_state,
                        weight: 1i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::WARPED_FUNGUS.default_state,
                        weight: 13i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::CRIMSON_FUNGUS.default_state,
                        weight: 1i32,
                    },
                ],
            }),
            spread_width: 8i32,
            spread_height: 4i32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::WarpedForestVegetationBonemeal,
        ConfiguredFeature::NetherForestVegetation(NetherForestVegetationFeature {
            state_provider: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: pumpkin_data::Block::WARPED_ROOTS.default_state,
                        weight: 85i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::CRIMSON_ROOTS.default_state,
                        weight: 1i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::WARPED_FUNGUS.default_state,
                        weight: 13i32,
                    },
                    Weighted {
                        data: pumpkin_data::Block::CRIMSON_FUNGUS.default_state,
                        weight: 1i32,
                    },
                ],
            }),
            spread_width: 3i32,
            spread_height: 1i32,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::WarpedFungus,
        ConfiguredFeature::HugeFungus(
            crate::generation::feature::features::huge_fungus::HugeFungusFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::WarpedFungusPlanted,
        ConfiguredFeature::HugeFungus(
            crate::generation::feature::features::huge_fungus::HugeFungusFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::Waterlily,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: pumpkin_data::Block::LILY_PAD.default_state,
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::WeepingVines,
        ConfiguredFeature::WeepingVines(
            crate::generation::feature::features::weeping_vines::WeepingVinesFeature {},
        ),
    );
    map.insert(
        pumpkin_data::configured_feature::ConfiguredFeature::Wildflower,
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "1".to_string());
                            props.insert("facing".to_string(), "north".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::WILDFLOWERS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "1".to_string());
                            props.insert("facing".to_string(), "east".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::WILDFLOWERS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "1".to_string());
                            props.insert("facing".to_string(), "south".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::WILDFLOWERS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "1".to_string());
                            props.insert("facing".to_string(), "west".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::WILDFLOWERS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "2".to_string());
                            props.insert("facing".to_string(), "north".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::WILDFLOWERS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "2".to_string());
                            props.insert("facing".to_string(), "east".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::WILDFLOWERS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "2".to_string());
                            props.insert("facing".to_string(), "south".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::WILDFLOWERS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "2".to_string());
                            props.insert("facing".to_string(), "west".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::WILDFLOWERS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "3".to_string());
                            props.insert("facing".to_string(), "north".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::WILDFLOWERS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "3".to_string());
                            props.insert("facing".to_string(), "east".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::WILDFLOWERS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "3".to_string());
                            props.insert("facing".to_string(), "south".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::WILDFLOWERS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "3".to_string());
                            props.insert("facing".to_string(), "west".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::WILDFLOWERS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "4".to_string());
                            props.insert("facing".to_string(), "north".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::WILDFLOWERS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "4".to_string());
                            props.insert("facing".to_string(), "east".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::WILDFLOWERS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "4".to_string());
                            props.insert("facing".to_string(), "south".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::WILDFLOWERS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("flower_amount".to_string(), "4".to_string());
                            props.insert("facing".to_string(), "west".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::WILDFLOWERS,
                                properties: Some(props),
                            }
                            .get_state()
                        },
                        weight: 1i32,
                    },
                ],
            }),
            schedule_tick: None,
        }),
    );
    map
}
