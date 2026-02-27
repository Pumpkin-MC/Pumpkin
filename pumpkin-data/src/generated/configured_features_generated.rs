/* This file is generated. Do not edit manually. */
#[allow(clippy::all, unused_imports, dead_code)]
fn build_configured_features() -> std::collections::HashMap<String, ConfiguredFeature> {
    use crate::block::BlockStateCodec;
    use crate::generation::block_predicate::{
        AllOfBlockPredicate, AnyOfBlockPredicate, BlockPredicate, HasSturdyFacePredicate,
        InsideWorldBoundsBlockPredicate, MatchingBlockTagPredicate, MatchingBlocksBlockPredicate,
        MatchingBlocksWrapper, MatchingFluidsBlockPredicate, NotBlockPredicate,
        OffsetBlocksBlockPredicate, ReplaceableBlockPredicate, SolidBlockPredicate,
        WouldSurviveBlockPredicate,
    };
    use crate::generation::block_state_provider::{
        BlockStateProvider, DualNoiseBlockStateProvider, NoiseBlockStateProvider,
        NoiseBlockStateProviderBase, NoiseThresholdBlockStateProvider, PillarBlockStateProvider,
        RandomizedIntBlockStateProvider, SimpleStateProvider, WeightedBlockStateProvider,
    };
    use crate::generation::feature::features::drip_stone::small::SmallDripstoneFeature;
    use crate::generation::feature::features::{
        bamboo::BambooFeature,
        block_column::{BlockColumnFeature, Layer},
        end_spike::{EndSpikeFeature, Spike},
        fallen_tree::FallenTreeFeature,
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
        tree::trunk::{
            TrunkPlacer, TrunkType, bending::BendingTrunkPlacer, cherry::CherryTrunkPlacer,
            dark_oak::DarkOakTrunkPlacer, fancy::FancyTrunkPlacer, forking::ForkingTrunkPlacer,
            giant::GiantTrunkPlacer, mega_jungle::MegaJungleTrunkPlacer,
            straight::StraightTrunkPlacer, upwards_branching::UpwardsBranchingTrunkPlacer,
        },
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
        ConstantIntProvider, IntProvider, NormalIntProvider, UniformIntProvider, WeightedEntry,
        WeightedListIntProvider,
    };
    use pumpkin_util::math::pool::Weighted;
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_util::y_offset::{AboveBottom, Absolute, BelowTop, YOffset};
    let mut map = std::collections::HashMap::new();
    map.insert(
        "acacia".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::ACACIA_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 5u8,
                height_rand_a: 2u8,
                height_rand_b: 2u8,
                r#type: TrunkType::Forking(ForkingTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::ACACIA_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Acacia(AcaciaFoliagePlacer),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 2u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![],
        })),
    );
    map.insert(
        "amethyst_geode".to_string(),
        ConfiguredFeature::Geode(crate::generation::feature::features::geode::GeodeFeature {}),
    );
    map.insert(
        "azalea_tree".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::ROOTED_DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LOG,
                        properties: Some(props),
                    }
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
                            props.insert("distance".to_string(), "7".to_string());
                            props.insert("persistent".to_string(), "false".to_string());
                            props.insert("waterlogged".to_string(), "false".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::AZALEA_LEAVES,
                                properties: Some(props),
                            }
                        },
                        weight: 3i32,
                    },
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("distance".to_string(), "7".to_string());
                            props.insert("persistent".to_string(), "false".to_string());
                            props.insert("waterlogged".to_string(), "false".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::FLOWERING_AZALEA_LEAVES,
                                properties: Some(props),
                            }
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
            force_dirt: true,
            decorators: vec![],
        })),
    );
    map.insert(
        "bamboo_no_podzol".to_string(),
        ConfiguredFeature::Bamboo(BambooFeature { probability: 0f32 }),
    );
    map.insert(
        "bamboo_some_podzol".to_string(),
        ConfiguredFeature::Bamboo(BambooFeature {
            probability: 0.2f32,
        }),
    );
    map.insert(
        "bamboo_vegetation".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fancy_oak_checked".to_string()),
                    chance: 0.05f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("jungle_bush".to_string()),
                    chance: 0.15f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("mega_jungle_tree_checked".to_string()),
                    chance: 0.7f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Direct(PlacedFeature {
                feature: Feature::Named("patch_grass_jungle".to_string()),
                placement: vec![],
            })),
        }),
    );
    map.insert(
        "basalt_blobs".to_string(),
        ConfiguredFeature::NetherrackReplaceBlobs(ReplaceBlobsFeature {
            target: BlockStateCodec {
                name: &pumpkin_data::Block::NETHERRACK,
                properties: None,
            },
            state: {
                let mut props = std::collections::HashMap::new();
                props.insert("axis".to_string(), "y".to_string());
                BlockStateCodec {
                    name: &pumpkin_data::Block::BASALT,
                    properties: Some(props),
                }
            },
            radius: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                min_inclusive: 3i32,
                max_inclusive: 7i32,
            })),
        }),
    );
    map.insert(
        "basalt_pillar".to_string(),
        ConfiguredFeature::BasaltPillar(
            crate::generation::feature::features::basalt_pillar::BasaltPillarFeature {},
        ),
    );
    map.insert(
        "birch".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 5u8,
                height_rand_a: 2u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Blob(BlobFoliagePlacer { height: 3i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 1u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![],
        })),
    );
    map.insert(
        "birch_bees_0002".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 5u8,
                height_rand_a: 2u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Blob(BlobFoliagePlacer { height: 3i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 1u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![TreeDecorator::Beehive(BeehiveTreeDecorator {
                probability: 0.002f32,
            })],
        })),
    );
    map.insert(
        "birch_bees_0002_leaf_litter".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 5u8,
                height_rand_a: 2u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Blob(BlobFoliagePlacer { height: 3i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 1u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![
                TreeDecorator::Beehive(BeehiveTreeDecorator {
                    probability: 0.002f32,
                }),
                TreeDecorator::PlaceOnGround(PlaceOnGroundTreeDecorator {
                    tries: 96i32,
                    radius: 4i32,
                    height: 2i32,
                    block_state_provider: BlockStateProvider::Weighted(
                        WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        },
                    ),
                }),
                TreeDecorator::PlaceOnGround(PlaceOnGroundTreeDecorator {
                    tries: 150i32,
                    radius: 2i32,
                    height: 2i32,
                    block_state_provider: BlockStateProvider::Weighted(
                        WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        },
                    ),
                }),
            ],
        })),
    );
    map.insert(
        "birch_bees_002".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 5u8,
                height_rand_a: 2u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Blob(BlobFoliagePlacer { height: 3i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 1u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![TreeDecorator::Beehive(BeehiveTreeDecorator {
                probability: 0.02f32,
            })],
        })),
    );
    map.insert(
        "birch_bees_005".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 5u8,
                height_rand_a: 2u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Blob(BlobFoliagePlacer { height: 3i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 1u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![TreeDecorator::Beehive(BeehiveTreeDecorator {
                probability: 0.05f32,
            })],
        })),
    );
    map.insert(
        "birch_leaf_litter".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 5u8,
                height_rand_a: 2u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Blob(BlobFoliagePlacer { height: 3i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 1u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![
                TreeDecorator::PlaceOnGround(PlaceOnGroundTreeDecorator {
                    tries: 96i32,
                    radius: 4i32,
                    height: 2i32,
                    block_state_provider: BlockStateProvider::Weighted(
                        WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        },
                    ),
                }),
                TreeDecorator::PlaceOnGround(PlaceOnGroundTreeDecorator {
                    tries: 150i32,
                    radius: 2i32,
                    height: 2i32,
                    block_state_provider: BlockStateProvider::Weighted(
                        WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        },
                    ),
                }),
            ],
        })),
    );
    map.insert(
        "birch_tall".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fallen_super_birch_tree".to_string()),
                    chance: 0.00625f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("super_birch_bees_0002".to_string()),
                    chance: 0.5f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fallen_birch_tree".to_string()),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named("birch_bees_0002".to_string())),
        }),
    );
    map.insert(
        "blackstone_blobs".to_string(),
        ConfiguredFeature::NetherrackReplaceBlobs(ReplaceBlobsFeature {
            target: BlockStateCodec {
                name: &pumpkin_data::Block::NETHERRACK,
                properties: None,
            },
            state: BlockStateCodec {
                name: &pumpkin_data::Block::BLACKSTONE,
                properties: None,
            },
            radius: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                min_inclusive: 3i32,
                max_inclusive: 7i32,
            })),
        }),
    );
    map.insert(
        "blue_ice".to_string(),
        ConfiguredFeature::BlueIce(
            crate::generation::feature::features::blue_ice::BlueIceFeature {},
        ),
    );
    map.insert(
        "bonus_chest".to_string(),
        ConfiguredFeature::BonusChest(
            crate::generation::feature::features::bonus_chest::BonusChestFeature {},
        ),
    );
    map.insert(
        "cave_vine".to_string(),
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
                                            props.insert("age".to_string(), "0".to_string());
                                            props
                                                .insert("berries".to_string(), "false".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::CAVE_VINES,
                                                properties: Some(props),
                                            }
                                        },
                                        weight: 4i32,
                                    },
                                    Weighted {
                                        data: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert("age".to_string(), "0".to_string());
                                            props.insert("berries".to_string(), "true".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::CAVE_VINES,
                                                properties: Some(props),
                                            }
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
            allowed_placement: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                offset: OffsetBlocksBlockPredicate { offset: None },
                blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
            }),
            prioritize_tip: true,
        }),
    );
    map.insert(
        "cave_vine_in_moss".to_string(),
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
                                            props.insert("age".to_string(), "0".to_string());
                                            props
                                                .insert("berries".to_string(), "false".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::CAVE_VINES,
                                                properties: Some(props),
                                            }
                                        },
                                        weight: 4i32,
                                    },
                                    Weighted {
                                        data: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert("age".to_string(), "0".to_string());
                                            props.insert("berries".to_string(), "true".to_string());
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::CAVE_VINES,
                                                properties: Some(props),
                                            }
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
            allowed_placement: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                offset: OffsetBlocksBlockPredicate { offset: None },
                blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
            }),
            prioritize_tip: true,
        }),
    );
    map.insert(
        "cherry".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::CHERRY_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 7u8,
                height_rand_a: 1u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Cherry(CherryTrunkPlacer {}),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::CHERRY_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(4i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Cherry(CherryFoliagePlacer {
                    height: IntProvider::Constant(5i32),
                    wide_bottom_layer_hole_chance: 0.25f32,
                    corner_hole_chance: 0.25f32,
                    hanging_leaves_chance: 0.16666667f32,
                    hanging_leaves_extension_chance: 0.33333334f32,
                }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 2u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![],
        })),
    );
    map.insert(
        "cherry_bees_005".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::CHERRY_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 7u8,
                height_rand_a: 1u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Cherry(CherryTrunkPlacer {}),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::CHERRY_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(4i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Cherry(CherryFoliagePlacer {
                    height: IntProvider::Constant(5i32),
                    wide_bottom_layer_hole_chance: 0.25f32,
                    corner_hole_chance: 0.25f32,
                    hanging_leaves_chance: 0.16666667f32,
                    hanging_leaves_extension_chance: 0.33333334f32,
                }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 2u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![TreeDecorator::Beehive(BeehiveTreeDecorator {
                probability: 0.05f32,
            })],
        })),
    );
    map.insert(
        "chorus_plant".to_string(),
        ConfiguredFeature::ChorusPlant(
            crate::generation::feature::features::chorus_plant::ChorusPlantFeature {},
        ),
    );
    map . insert ("clay_pool_with_dripleaves" . to_string () , ConfiguredFeature :: WaterloggedVegetationPatch (crate :: generation :: feature :: features :: waterlogged_vegetation_patch :: WaterloggedVegetationPatchFeature { })) ;
    map.insert(
        "clay_with_dripleaves".to_string(),
        ConfiguredFeature::VegetationPatch(
            crate::generation::feature::features::vegetation_patch::VegetationPatchFeature {},
        ),
    );
    map.insert(
        "crimson_forest_vegetation".to_string(),
        ConfiguredFeature::NetherForestVegetation(NetherForestVegetationFeature {
            state_provider: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::CRIMSON_ROOTS,
                            properties: None,
                        },
                        weight: 87i32,
                    },
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::CRIMSON_FUNGUS,
                            properties: None,
                        },
                        weight: 11i32,
                    },
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::WARPED_FUNGUS,
                            properties: None,
                        },
                        weight: 1i32,
                    },
                ],
            }),
            spread_width: 8i32,
            spread_height: 4i32,
        }),
    );
    map.insert(
        "crimson_forest_vegetation_bonemeal".to_string(),
        ConfiguredFeature::NetherForestVegetation(NetherForestVegetationFeature {
            state_provider: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::CRIMSON_ROOTS,
                            properties: None,
                        },
                        weight: 87i32,
                    },
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::CRIMSON_FUNGUS,
                            properties: None,
                        },
                        weight: 11i32,
                    },
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::WARPED_FUNGUS,
                            properties: None,
                        },
                        weight: 1i32,
                    },
                ],
            }),
            spread_width: 3i32,
            spread_height: 1i32,
        }),
    );
    map.insert(
        "crimson_fungus".to_string(),
        ConfiguredFeature::HugeFungus(
            crate::generation::feature::features::huge_fungus::HugeFungusFeature {},
        ),
    );
    map.insert(
        "crimson_fungus_planted".to_string(),
        ConfiguredFeature::HugeFungus(
            crate::generation::feature::features::huge_fungus::HugeFungusFeature {},
        ),
    );
    map.insert(
        "dark_forest_vegetation".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Direct(PlacedFeature {
                        feature: Feature::Named("huge_brown_mushroom".to_string()),
                        placement: vec![],
                    }),
                    chance: 0.025f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Direct(PlacedFeature {
                        feature: Feature::Named("huge_red_mushroom".to_string()),
                        placement: vec![],
                    }),
                    chance: 0.05f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("dark_oak_leaf_litter".to_string()),
                    chance: 0.6666667f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fallen_birch_tree".to_string()),
                    chance: 0.0025f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("birch_leaf_litter".to_string()),
                    chance: 0.2f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fallen_oak_tree".to_string()),
                    chance: 0.0125f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fancy_oak_leaf_litter".to_string()),
                    chance: 0.1f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named("oak_leaf_litter".to_string())),
        }),
    );
    map.insert(
        "dark_oak".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::DARK_OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 6u8,
                height_rand_a: 2u8,
                height_rand_b: 1u8,
                r#type: TrunkType::DarkOak(DarkOakTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::DARK_OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(0i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::DarkOak(DarkOakFoliagePlacer),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::ThreeLayersFeatureSize(ThreeLayersFeatureSize {
                    limit: 1u8,
                    upper_limit: 1u8,
                    lower_size: 0u8,
                    middle_size: 1u8,
                    upper_size: 2u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![],
        })),
    );
    map.insert(
        "dark_oak_leaf_litter".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::DARK_OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 6u8,
                height_rand_a: 2u8,
                height_rand_b: 1u8,
                r#type: TrunkType::DarkOak(DarkOakTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::DARK_OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(0i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::DarkOak(DarkOakFoliagePlacer),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::ThreeLayersFeatureSize(ThreeLayersFeatureSize {
                    limit: 1u8,
                    upper_limit: 1u8,
                    lower_size: 0u8,
                    middle_size: 1u8,
                    upper_size: 2u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![
                TreeDecorator::PlaceOnGround(PlaceOnGroundTreeDecorator {
                    tries: 96i32,
                    radius: 4i32,
                    height: 2i32,
                    block_state_provider: BlockStateProvider::Weighted(
                        WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        },
                    ),
                }),
                TreeDecorator::PlaceOnGround(PlaceOnGroundTreeDecorator {
                    tries: 150i32,
                    radius: 2i32,
                    height: 2i32,
                    block_state_provider: BlockStateProvider::Weighted(
                        WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        },
                    ),
                }),
            ],
        })),
    );
    map.insert(
        "delta".to_string(),
        ConfiguredFeature::DeltaFeature(
            crate::generation::feature::features::delta_feature::DeltaFeatureFeature {},
        ),
    );
    map.insert(
        "desert_well".to_string(),
        ConfiguredFeature::DesertWell(
            crate::generation::feature::features::desert_well::DesertWellFeature,
        ),
    );
    map.insert(
        "disk_clay".to_string(),
        ConfiguredFeature::Disk(crate::generation::feature::features::disk::DiskFeature {}),
    );
    map.insert(
        "disk_grass".to_string(),
        ConfiguredFeature::Disk(crate::generation::feature::features::disk::DiskFeature {}),
    );
    map.insert(
        "disk_gravel".to_string(),
        ConfiguredFeature::Disk(crate::generation::feature::features::disk::DiskFeature {}),
    );
    map.insert(
        "disk_sand".to_string(),
        ConfiguredFeature::Disk(crate::generation::feature::features::disk::DiskFeature {}),
    );
    map.insert(
        "dripleaf".to_string(),
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
                                            props.insert("facing".to_string(), "east".to_string());
                                            props.insert("half".to_string(), "lower".to_string());
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::SMALL_DRIPLEAF,
                                                properties: Some(props),
                                            }
                                        },
                                        weight: 1i32,
                                    },
                                    Weighted {
                                        data: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert("facing".to_string(), "west".to_string());
                                            props.insert("half".to_string(), "lower".to_string());
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::SMALL_DRIPLEAF,
                                                properties: Some(props),
                                            }
                                        },
                                        weight: 1i32,
                                    },
                                    Weighted {
                                        data: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert("facing".to_string(), "north".to_string());
                                            props.insert("half".to_string(), "lower".to_string());
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::SMALL_DRIPLEAF,
                                                properties: Some(props),
                                            }
                                        },
                                        weight: 1i32,
                                    },
                                    Weighted {
                                        data: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert("facing".to_string(), "south".to_string());
                                            props.insert("half".to_string(), "lower".to_string());
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::SMALL_DRIPLEAF,
                                                properties: Some(props),
                                            }
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
                                            props.insert("facing".to_string(), "east".to_string());
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::BIG_DRIPLEAF_STEM,
                                                properties: Some(props),
                                            }
                                        },
                                    }),
                                },
                                Layer {
                                    height: IntProvider::Constant(1i32),
                                    provider: BlockStateProvider::Simple(SimpleStateProvider {
                                        state: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert("facing".to_string(), "east".to_string());
                                            props.insert("tilt".to_string(), "none".to_string());
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::BIG_DRIPLEAF,
                                                properties: Some(props),
                                            }
                                        },
                                    }),
                                },
                            ],
                            direction: BlockDirection::Up,
                            allowed_placement: BlockPredicate::MatchingBlocks(
                                MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    blocks: MatchingBlocksWrapper::Multiple(vec![
                                        "minecraft:air".to_string(),
                                        "minecraft:water".to_string(),
                                    ]),
                                },
                            ),
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
                                            props.insert("facing".to_string(), "west".to_string());
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::BIG_DRIPLEAF_STEM,
                                                properties: Some(props),
                                            }
                                        },
                                    }),
                                },
                                Layer {
                                    height: IntProvider::Constant(1i32),
                                    provider: BlockStateProvider::Simple(SimpleStateProvider {
                                        state: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert("facing".to_string(), "west".to_string());
                                            props.insert("tilt".to_string(), "none".to_string());
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::BIG_DRIPLEAF,
                                                properties: Some(props),
                                            }
                                        },
                                    }),
                                },
                            ],
                            direction: BlockDirection::Up,
                            allowed_placement: BlockPredicate::MatchingBlocks(
                                MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    blocks: MatchingBlocksWrapper::Multiple(vec![
                                        "minecraft:air".to_string(),
                                        "minecraft:water".to_string(),
                                    ]),
                                },
                            ),
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
                                            props.insert("facing".to_string(), "south".to_string());
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::BIG_DRIPLEAF_STEM,
                                                properties: Some(props),
                                            }
                                        },
                                    }),
                                },
                                Layer {
                                    height: IntProvider::Constant(1i32),
                                    provider: BlockStateProvider::Simple(SimpleStateProvider {
                                        state: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert("facing".to_string(), "south".to_string());
                                            props.insert("tilt".to_string(), "none".to_string());
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::BIG_DRIPLEAF,
                                                properties: Some(props),
                                            }
                                        },
                                    }),
                                },
                            ],
                            direction: BlockDirection::Up,
                            allowed_placement: BlockPredicate::MatchingBlocks(
                                MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    blocks: MatchingBlocksWrapper::Multiple(vec![
                                        "minecraft:air".to_string(),
                                        "minecraft:water".to_string(),
                                    ]),
                                },
                            ),
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
                                            props.insert("facing".to_string(), "north".to_string());
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::BIG_DRIPLEAF_STEM,
                                                properties: Some(props),
                                            }
                                        },
                                    }),
                                },
                                Layer {
                                    height: IntProvider::Constant(1i32),
                                    provider: BlockStateProvider::Simple(SimpleStateProvider {
                                        state: {
                                            let mut props = std::collections::HashMap::new();
                                            props.insert("facing".to_string(), "north".to_string());
                                            props.insert("tilt".to_string(), "none".to_string());
                                            props.insert(
                                                "waterlogged".to_string(),
                                                "false".to_string(),
                                            );
                                            BlockStateCodec {
                                                name: &pumpkin_data::Block::BIG_DRIPLEAF,
                                                properties: Some(props),
                                            }
                                        },
                                    }),
                                },
                            ],
                            direction: BlockDirection::Up,
                            allowed_placement: BlockPredicate::MatchingBlocks(
                                MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    blocks: MatchingBlocksWrapper::Multiple(vec![
                                        "minecraft:air".to_string(),
                                        "minecraft:water".to_string(),
                                    ]),
                                },
                            ),
                            prioritize_tip: true,
                        },
                    ))),
                    placement: vec![],
                },
            ],
        }),
    );
    map.insert(
        "dripstone_cluster".to_string(),
        ConfiguredFeature::DripstoneCluster(
            crate::generation::feature::features::drip_stone::cluster::DripstoneClusterFeature {},
        ),
    );
    map.insert(
        "end_gateway_delayed".to_string(),
        ConfiguredFeature::EndGateway(
            crate::generation::feature::features::end_gateway::EndGatewayFeature {},
        ),
    );
    map.insert(
        "end_gateway_return".to_string(),
        ConfiguredFeature::EndGateway(
            crate::generation::feature::features::end_gateway::EndGatewayFeature {},
        ),
    );
    map.insert(
        "end_island".to_string(),
        ConfiguredFeature::EndIsland(
            crate::generation::feature::features::end_island::EndIslandFeature {},
        ),
    );
    map.insert(
        "end_platform".to_string(),
        ConfiguredFeature::EndPlatform(
            crate::generation::feature::features::end_platform::EndPlatformFeature,
        ),
    );
    map.insert(
        "end_spike".to_string(),
        ConfiguredFeature::EndSpike(EndSpikeFeature {
            crystal_invulnerable: false,
            spikes: vec![],
        }),
    );
    map.insert(
        "fallen_birch_tree".to_string(),
        ConfiguredFeature::FallenTree(FallenTreeFeature {
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LOG,
                        properties: Some(props),
                    }
                },
            }),
        }),
    );
    map.insert(
        "fallen_jungle_tree".to_string(),
        ConfiguredFeature::FallenTree(FallenTreeFeature {
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::JUNGLE_LOG,
                        properties: Some(props),
                    }
                },
            }),
        }),
    );
    map.insert(
        "fallen_oak_tree".to_string(),
        ConfiguredFeature::FallenTree(FallenTreeFeature {
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
        }),
    );
    map.insert(
        "fallen_spruce_tree".to_string(),
        ConfiguredFeature::FallenTree(FallenTreeFeature {
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::SPRUCE_LOG,
                        properties: Some(props),
                    }
                },
            }),
        }),
    );
    map.insert(
        "fallen_super_birch_tree".to_string(),
        ConfiguredFeature::FallenTree(FallenTreeFeature {
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LOG,
                        properties: Some(props),
                    }
                },
            }),
        }),
    );
    map.insert(
        "fancy_oak".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 3u8,
                height_rand_a: 11u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Fancy(FancyTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(4i32),
                r#type: FoliageType::Fancy(LargeOakFoliagePlacer { height: 4i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: Some(4u8),
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 0u8,
                    lower_size: 0u8,
                    upper_size: 0u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![],
        })),
    );
    map.insert(
        "fancy_oak_bees".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 3u8,
                height_rand_a: 11u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Fancy(FancyTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(4i32),
                r#type: FoliageType::Fancy(LargeOakFoliagePlacer { height: 4i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: Some(4u8),
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 0u8,
                    lower_size: 0u8,
                    upper_size: 0u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![TreeDecorator::Beehive(BeehiveTreeDecorator {
                probability: 1f32,
            })],
        })),
    );
    map.insert(
        "fancy_oak_bees_0002_leaf_litter".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 3u8,
                height_rand_a: 11u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Fancy(FancyTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(4i32),
                r#type: FoliageType::Fancy(LargeOakFoliagePlacer { height: 4i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: Some(4u8),
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 0u8,
                    lower_size: 0u8,
                    upper_size: 0u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![
                TreeDecorator::Beehive(BeehiveTreeDecorator {
                    probability: 0.002f32,
                }),
                TreeDecorator::PlaceOnGround(PlaceOnGroundTreeDecorator {
                    tries: 96i32,
                    radius: 4i32,
                    height: 2i32,
                    block_state_provider: BlockStateProvider::Weighted(
                        WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        },
                    ),
                }),
                TreeDecorator::PlaceOnGround(PlaceOnGroundTreeDecorator {
                    tries: 150i32,
                    radius: 2i32,
                    height: 2i32,
                    block_state_provider: BlockStateProvider::Weighted(
                        WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        },
                    ),
                }),
            ],
        })),
    );
    map.insert(
        "fancy_oak_bees_002".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 3u8,
                height_rand_a: 11u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Fancy(FancyTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(4i32),
                r#type: FoliageType::Fancy(LargeOakFoliagePlacer { height: 4i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: Some(4u8),
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 0u8,
                    lower_size: 0u8,
                    upper_size: 0u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![TreeDecorator::Beehive(BeehiveTreeDecorator {
                probability: 0.02f32,
            })],
        })),
    );
    map.insert(
        "fancy_oak_bees_005".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 3u8,
                height_rand_a: 11u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Fancy(FancyTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(4i32),
                r#type: FoliageType::Fancy(LargeOakFoliagePlacer { height: 4i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: Some(4u8),
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 0u8,
                    lower_size: 0u8,
                    upper_size: 0u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![TreeDecorator::Beehive(BeehiveTreeDecorator {
                probability: 0.05f32,
            })],
        })),
    );
    map.insert(
        "fancy_oak_leaf_litter".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 3u8,
                height_rand_a: 11u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Fancy(FancyTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(4i32),
                r#type: FoliageType::Fancy(LargeOakFoliagePlacer { height: 4i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: Some(4u8),
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 0u8,
                    lower_size: 0u8,
                    upper_size: 0u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![
                TreeDecorator::PlaceOnGround(PlaceOnGroundTreeDecorator {
                    tries: 96i32,
                    radius: 4i32,
                    height: 2i32,
                    block_state_provider: BlockStateProvider::Weighted(
                        WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        },
                    ),
                }),
                TreeDecorator::PlaceOnGround(PlaceOnGroundTreeDecorator {
                    tries: 150i32,
                    radius: 2i32,
                    height: 2i32,
                    block_state_provider: BlockStateProvider::Weighted(
                        WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        },
                    ),
                }),
            ],
        })),
    );
    map.insert(
        "flower_cherry".to_string(),
        ConfiguredFeature::Flower(RandomPatchFeature {
            tries: 96u8,
            xz_spread: 6u8,
            y_spread: 2u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("flower_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::PINK_PETALS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("flower_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::PINK_PETALS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("flower_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::PINK_PETALS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("flower_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::PINK_PETALS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("flower_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::PINK_PETALS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("flower_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::PINK_PETALS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("flower_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::PINK_PETALS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("flower_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::PINK_PETALS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("flower_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::PINK_PETALS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("flower_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::PINK_PETALS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("flower_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::PINK_PETALS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("flower_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::PINK_PETALS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("flower_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::PINK_PETALS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("flower_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::PINK_PETALS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("flower_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::PINK_PETALS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("flower_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::PINK_PETALS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "flower_default".to_string(),
        ConfiguredFeature::Flower(RandomPatchFeature {
            tries: 64u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: BlockStateCodec {
                                        name: &pumpkin_data::Block::POPPY,
                                        properties: None,
                                    },
                                    weight: 2i32,
                                },
                                Weighted {
                                    data: BlockStateCodec {
                                        name: &pumpkin_data::Block::DANDELION,
                                        properties: None,
                                    },
                                    weight: 1i32,
                                },
                            ],
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "flower_flower_forest".to_string(),
        ConfiguredFeature::Flower(RandomPatchFeature {
            tries: 96u8,
            xz_spread: 6u8,
            y_spread: 2u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::NoiseProvider(NoiseBlockStateProvider {
                            base: NoiseBlockStateProviderBase {
                                seed: 2345i64,
                                noise: DoublePerlinNoiseParametersCodec {
                                    first_octave: 0i32,
                                    amplitudes: vec![1f64],
                                },
                                scale: 0.020833334f32,
                            },
                            states: vec![
                                BlockStateCodec {
                                    name: &pumpkin_data::Block::DANDELION,
                                    properties: None,
                                },
                                BlockStateCodec {
                                    name: &pumpkin_data::Block::POPPY,
                                    properties: None,
                                },
                                BlockStateCodec {
                                    name: &pumpkin_data::Block::ALLIUM,
                                    properties: None,
                                },
                                BlockStateCodec {
                                    name: &pumpkin_data::Block::AZURE_BLUET,
                                    properties: None,
                                },
                                BlockStateCodec {
                                    name: &pumpkin_data::Block::RED_TULIP,
                                    properties: None,
                                },
                                BlockStateCodec {
                                    name: &pumpkin_data::Block::ORANGE_TULIP,
                                    properties: None,
                                },
                                BlockStateCodec {
                                    name: &pumpkin_data::Block::WHITE_TULIP,
                                    properties: None,
                                },
                                BlockStateCodec {
                                    name: &pumpkin_data::Block::PINK_TULIP,
                                    properties: None,
                                },
                                BlockStateCodec {
                                    name: &pumpkin_data::Block::OXEYE_DAISY,
                                    properties: None,
                                },
                                BlockStateCodec {
                                    name: &pumpkin_data::Block::CORNFLOWER,
                                    properties: None,
                                },
                                BlockStateCodec {
                                    name: &pumpkin_data::Block::LILY_OF_THE_VALLEY,
                                    properties: None,
                                },
                            ],
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "flower_meadow".to_string(),
        ConfiguredFeature::Flower(RandomPatchFeature {
            tries: 96u8,
            xz_spread: 6u8,
            y_spread: 2u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::DualNoise(DualNoiseBlockStateProvider {
                            base: NoiseBlockStateProvider {
                                base: NoiseBlockStateProviderBase {
                                    seed: 2345i64,
                                    noise: DoublePerlinNoiseParametersCodec {
                                        first_octave: -3i32,
                                        amplitudes: vec![1f64],
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
                                    },
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::ALLIUM,
                                        properties: None,
                                    },
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::POPPY,
                                        properties: None,
                                    },
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::AZURE_BLUET,
                                        properties: None,
                                    },
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::DANDELION,
                                        properties: None,
                                    },
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::CORNFLOWER,
                                        properties: None,
                                    },
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::OXEYE_DAISY,
                                        properties: None,
                                    },
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::SHORT_GRASS,
                                        properties: None,
                                    },
                                ],
                            },
                            variety: [1u32, 3u32],
                            slow_noise: DoublePerlinNoiseParametersCodec {
                                first_octave: -10i32,
                                amplitudes: vec![1f64],
                            },
                            slow_scale: 1f64,
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "flower_pale_garden".to_string(),
        ConfiguredFeature::Flower(RandomPatchFeature {
            tries: 1u8,
            xz_spread: 0u8,
            y_spread: 0u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: BlockStateCodec {
                                name: &pumpkin_data::Block::CLOSED_EYEBLOSSOM,
                                properties: None,
                            },
                        }),
                        schedule_tick: Some(true),
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "flower_plain".to_string(),
        ConfiguredFeature::Flower(RandomPatchFeature {
            tries: 64u8,
            xz_spread: 6u8,
            y_spread: 2u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::NoiseThreshold(
                            NoiseThresholdBlockStateProvider {
                                base: NoiseBlockStateProviderBase {
                                    seed: 2345i64,
                                    noise: DoublePerlinNoiseParametersCodec {
                                        first_octave: 0i32,
                                        amplitudes: vec![1f64],
                                    },
                                    scale: 0.005f32,
                                },
                                threshold: -0.8f32,
                                high_chance: 0.33333334f32,
                                default_state: BlockStateCodec {
                                    name: &pumpkin_data::Block::DANDELION,
                                    properties: None,
                                },
                                low_states: vec![
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::ORANGE_TULIP,
                                        properties: None,
                                    },
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::RED_TULIP,
                                        properties: None,
                                    },
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::PINK_TULIP,
                                        properties: None,
                                    },
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::WHITE_TULIP,
                                        properties: None,
                                    },
                                ],
                                high_states: vec![
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::POPPY,
                                        properties: None,
                                    },
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::AZURE_BLUET,
                                        properties: None,
                                    },
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::OXEYE_DAISY,
                                        properties: None,
                                    },
                                    BlockStateCodec {
                                        name: &pumpkin_data::Block::CORNFLOWER,
                                        properties: None,
                                    },
                                ],
                            },
                        ),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "flower_swamp".to_string(),
        ConfiguredFeature::Flower(RandomPatchFeature {
            tries: 64u8,
            xz_spread: 6u8,
            y_spread: 2u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: BlockStateCodec {
                                name: &pumpkin_data::Block::BLUE_ORCHID,
                                properties: None,
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "forest_flowers".to_string(),
        ConfiguredFeature::SimpleRandomSelector(SimpleRandomFeature {
            features: vec![
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::RandomPatch(
                        RandomPatchFeature {
                            tries: 96u8,
                            xz_spread: 7u8,
                            y_spread: 3u8,
                            feature: Box::new(PlacedFeature {
                                feature: Feature::Inlined(Box::new(
                                    ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
                                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                                            state: {
                                                let mut props = std::collections::HashMap::new();
                                                props.insert(
                                                    "half".to_string(),
                                                    "lower".to_string(),
                                                );
                                                BlockStateCodec {
                                                    name: &pumpkin_data::Block::LILAC,
                                                    properties: Some(props),
                                                }
                                            },
                                        }),
                                        schedule_tick: None,
                                    }),
                                )),
                                placement: vec![PlacementModifier::BlockPredicateFilter(
                                    BlockFilterPlacementModifier {
                                        predicate: BlockPredicate::MatchingBlocks(
                                            MatchingBlocksBlockPredicate {
                                                offset: OffsetBlocksBlockPredicate { offset: None },
                                                blocks: MatchingBlocksWrapper::Single(
                                                    "minecraft:air".to_string(),
                                                ),
                                            },
                                        ),
                                    },
                                )],
                            }),
                        },
                    ))),
                    placement: vec![],
                },
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::RandomPatch(
                        RandomPatchFeature {
                            tries: 96u8,
                            xz_spread: 7u8,
                            y_spread: 3u8,
                            feature: Box::new(PlacedFeature {
                                feature: Feature::Inlined(Box::new(
                                    ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
                                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                                            state: {
                                                let mut props = std::collections::HashMap::new();
                                                props.insert(
                                                    "half".to_string(),
                                                    "lower".to_string(),
                                                );
                                                BlockStateCodec {
                                                    name: &pumpkin_data::Block::ROSE_BUSH,
                                                    properties: Some(props),
                                                }
                                            },
                                        }),
                                        schedule_tick: None,
                                    }),
                                )),
                                placement: vec![PlacementModifier::BlockPredicateFilter(
                                    BlockFilterPlacementModifier {
                                        predicate: BlockPredicate::MatchingBlocks(
                                            MatchingBlocksBlockPredicate {
                                                offset: OffsetBlocksBlockPredicate { offset: None },
                                                blocks: MatchingBlocksWrapper::Single(
                                                    "minecraft:air".to_string(),
                                                ),
                                            },
                                        ),
                                    },
                                )],
                            }),
                        },
                    ))),
                    placement: vec![],
                },
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::RandomPatch(
                        RandomPatchFeature {
                            tries: 96u8,
                            xz_spread: 7u8,
                            y_spread: 3u8,
                            feature: Box::new(PlacedFeature {
                                feature: Feature::Inlined(Box::new(
                                    ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
                                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                                            state: {
                                                let mut props = std::collections::HashMap::new();
                                                props.insert(
                                                    "half".to_string(),
                                                    "lower".to_string(),
                                                );
                                                BlockStateCodec {
                                                    name: &pumpkin_data::Block::PEONY,
                                                    properties: Some(props),
                                                }
                                            },
                                        }),
                                        schedule_tick: None,
                                    }),
                                )),
                                placement: vec![PlacementModifier::BlockPredicateFilter(
                                    BlockFilterPlacementModifier {
                                        predicate: BlockPredicate::MatchingBlocks(
                                            MatchingBlocksBlockPredicate {
                                                offset: OffsetBlocksBlockPredicate { offset: None },
                                                blocks: MatchingBlocksWrapper::Single(
                                                    "minecraft:air".to_string(),
                                                ),
                                            },
                                        ),
                                    },
                                )],
                            }),
                        },
                    ))),
                    placement: vec![],
                },
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::NoBonemealFlower(
                        RandomPatchFeature {
                            tries: 96u8,
                            xz_spread: 7u8,
                            y_spread: 3u8,
                            feature: Box::new(PlacedFeature {
                                feature: Feature::Inlined(Box::new(
                                    ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
                                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                                            state: BlockStateCodec {
                                                name: &pumpkin_data::Block::LILY_OF_THE_VALLEY,
                                                properties: None,
                                            },
                                        }),
                                        schedule_tick: None,
                                    }),
                                )),
                                placement: vec![PlacementModifier::BlockPredicateFilter(
                                    BlockFilterPlacementModifier {
                                        predicate: BlockPredicate::MatchingBlocks(
                                            MatchingBlocksBlockPredicate {
                                                offset: OffsetBlocksBlockPredicate { offset: None },
                                                blocks: MatchingBlocksWrapper::Single(
                                                    "minecraft:air".to_string(),
                                                ),
                                            },
                                        ),
                                    },
                                )],
                            }),
                        },
                    ))),
                    placement: vec![],
                },
            ],
        }),
    );
    map.insert(
        "forest_rock".to_string(),
        ConfiguredFeature::ForestRock(
            crate::generation::feature::features::forest_rock::ForestRockFeature {},
        ),
    );
    map.insert(
        "fossil_coal".to_string(),
        ConfiguredFeature::Fossil(crate::generation::feature::features::fossil::FossilFeature {}),
    );
    map.insert(
        "fossil_diamonds".to_string(),
        ConfiguredFeature::Fossil(crate::generation::feature::features::fossil::FossilFeature {}),
    );
    map.insert(
        "freeze_top_layer".to_string(),
        ConfiguredFeature::FreezeTopLayer(
            crate::generation::feature::features::freeze_top_layer::FreezeTopLayerFeature {},
        ),
    );
    map.insert(
        "glow_lichen".to_string(),
        ConfiguredFeature::MultifaceGrowth(
            crate::generation::feature::features::multiface_growth::MultifaceGrowthFeature {},
        ),
    );
    map.insert(
        "glowstone_extra".to_string(),
        ConfiguredFeature::GlowstoneBlob(
            crate::generation::feature::features::glowstone_blob::GlowstoneBlobFeature {},
        ),
    );
    map.insert(
        "huge_brown_mushroom".to_string(),
        ConfiguredFeature::HugeBrownMushroom(
            crate::generation::feature::features::huge_brown_mushroom::HugeBrownMushroomFeature {},
        ),
    );
    map.insert(
        "huge_red_mushroom".to_string(),
        ConfiguredFeature::HugeRedMushroom(
            crate::generation::feature::features::huge_red_mushroom::HugeRedMushroomFeature {},
        ),
    );
    map.insert(
        "ice_patch".to_string(),
        ConfiguredFeature::Disk(crate::generation::feature::features::disk::DiskFeature {}),
    );
    map.insert(
        "ice_spike".to_string(),
        ConfiguredFeature::IceSpike(
            crate::generation::feature::features::ice_spike::IceSpikeFeature {},
        ),
    );
    map.insert(
        "iceberg_blue".to_string(),
        ConfiguredFeature::Iceberg(
            crate::generation::feature::features::iceberg::IcebergFeature {},
        ),
    );
    map.insert(
        "iceberg_packed".to_string(),
        ConfiguredFeature::Iceberg(
            crate::generation::feature::features::iceberg::IcebergFeature {},
        ),
    );
    map.insert(
        "jungle_bush".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::JUNGLE_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 1u8,
                height_rand_a: 0u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(1i32),
                r#type: FoliageType::Bush(BushFoliagePlacer { height: 2i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 0u8,
                    lower_size: 0u8,
                    upper_size: 0u8,
                }),
            },
            ignore_vines: false,
            force_dirt: false,
            decorators: vec![],
        })),
    );
    map.insert(
        "jungle_tree".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::JUNGLE_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 4u8,
                height_rand_a: 8u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::JUNGLE_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Blob(BlobFoliagePlacer { height: 3i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 1u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![
                TreeDecorator::Cocoa(CocoaTreeDecorator {}),
                TreeDecorator::TrunkVine(TrunkVineTreeDecorator),
                TreeDecorator::LeaveVine(LeavesVineTreeDecorator {}),
            ],
        })),
    );
    map.insert(
        "jungle_tree_no_vine".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::JUNGLE_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 4u8,
                height_rand_a: 8u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::JUNGLE_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Blob(BlobFoliagePlacer { height: 3i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 1u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![],
        })),
    );
    map.insert(
        "kelp".to_string(),
        ConfiguredFeature::Kelp(crate::generation::feature::features::kelp::KelpFeature {}),
    );
    map.insert(
        "lake_lava".to_string(),
        ConfiguredFeature::Lake(crate::generation::feature::features::lake::LakeFeature {}),
    );
    map.insert(
        "large_basalt_columns".to_string(),
        ConfiguredFeature::BasaltColumns(
            crate::generation::feature::features::basalt_columns::BasaltColumnsFeature {},
        ),
    );
    map.insert(
        "large_dripstone".to_string(),
        ConfiguredFeature::LargeDripstone(
            crate::generation::feature::features::drip_stone::large::LargeDripstoneFeature {},
        ),
    );
    map.insert(
        "lush_caves_clay".to_string(),
        ConfiguredFeature::RandomBooleanSelector(RandomBooleanFeature {
            feature_true: Box::new(PlacedFeatureWrapper::Direct(PlacedFeature {
                feature: Feature::Named("clay_with_dripleaves".to_string()),
                placement: vec![],
            })),
            feature_false: Box::new(PlacedFeatureWrapper::Direct(PlacedFeature {
                feature: Feature::Named("clay_pool_with_dripleaves".to_string()),
                placement: vec![],
            })),
        }),
    );
    map.insert(
        "mangrove".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::MANGROVE_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 2u8,
                height_rand_a: 1u8,
                height_rand_b: 4u8,
                r#type: TrunkType::UpwardsBranching(UpwardsBranchingTrunkPlacer {}),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::MANGROVE_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(3i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::RandomSpread(RandomSpreadFoliagePlacer {
                    foliage_height: IntProvider::Constant(2i32),
                    leaf_placement_attempts: 70i32,
                }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 2u8,
                    lower_size: 0u8,
                    upper_size: 2u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![
                TreeDecorator::LeaveVine(LeavesVineTreeDecorator {}),
                TreeDecorator::AttachedToLeaves(AttachedToLeavesTreeDecorator {}),
                TreeDecorator::Beehive(BeehiveTreeDecorator {
                    probability: 0.01f32,
                }),
            ],
        })),
    );
    map.insert(
        "mangrove_vegetation".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![RandomFeatureEntry {
                feature: PlacedFeatureWrapper::Named("tall_mangrove_checked".to_string()),
                chance: 0.85f32,
            }],
            default: Box::new(PlacedFeatureWrapper::Named("mangrove_checked".to_string())),
        }),
    );
    map.insert(
        "meadow_trees".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![RandomFeatureEntry {
                feature: PlacedFeatureWrapper::Named("fancy_oak_bees".to_string()),
                chance: 0.5f32,
            }],
            default: Box::new(PlacedFeatureWrapper::Named("super_birch_bees".to_string())),
        }),
    );
    map.insert(
        "mega_jungle_tree".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::JUNGLE_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 10u8,
                height_rand_a: 2u8,
                height_rand_b: 19u8,
                r#type: TrunkType::MegaJungle(MegaJungleTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::JUNGLE_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Jungle(JungleFoliagePlacer { height: 2i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 1u8,
                    upper_size: 2u8,
                }),
            },
            ignore_vines: false,
            force_dirt: false,
            decorators: vec![
                TreeDecorator::TrunkVine(TrunkVineTreeDecorator),
                TreeDecorator::LeaveVine(LeavesVineTreeDecorator {}),
            ],
        })),
    );
    map.insert(
        "mega_pine".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::SPRUCE_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 13u8,
                height_rand_a: 2u8,
                height_rand_b: 14u8,
                r#type: TrunkType::Giant(GiantTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::SPRUCE_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(0i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::MegaPine(MegaPineFoliagePlacer {
                    crown_height: IntProvider::Object(NormalIntProvider::Uniform(
                        UniformIntProvider {
                            min_inclusive: 3i32,
                            max_inclusive: 7i32,
                        },
                    )),
                }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 1u8,
                    upper_size: 2u8,
                }),
            },
            ignore_vines: false,
            force_dirt: false,
            decorators: vec![TreeDecorator::AlterGround(AlterGroundTreeDecorator {})],
        })),
    );
    map.insert(
        "mega_spruce".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::SPRUCE_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 13u8,
                height_rand_a: 2u8,
                height_rand_b: 14u8,
                r#type: TrunkType::Giant(GiantTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::SPRUCE_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(0i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::MegaPine(MegaPineFoliagePlacer {
                    crown_height: IntProvider::Object(NormalIntProvider::Uniform(
                        UniformIntProvider {
                            min_inclusive: 13i32,
                            max_inclusive: 17i32,
                        },
                    )),
                }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 1u8,
                    upper_size: 2u8,
                }),
            },
            ignore_vines: false,
            force_dirt: false,
            decorators: vec![TreeDecorator::AlterGround(AlterGroundTreeDecorator {})],
        })),
    );
    map.insert(
        "monster_room".to_string(),
        ConfiguredFeature::MonsterRoom(
            crate::generation::feature::features::monster_room::DungeonFeature {},
        ),
    );
    map.insert(
        "moss_patch".to_string(),
        ConfiguredFeature::VegetationPatch(
            crate::generation::feature::features::vegetation_patch::VegetationPatchFeature {},
        ),
    );
    map.insert(
        "moss_patch_bonemeal".to_string(),
        ConfiguredFeature::VegetationPatch(
            crate::generation::feature::features::vegetation_patch::VegetationPatchFeature {},
        ),
    );
    map.insert(
        "moss_patch_ceiling".to_string(),
        ConfiguredFeature::VegetationPatch(
            crate::generation::feature::features::vegetation_patch::VegetationPatchFeature {},
        ),
    );
    map.insert(
        "moss_vegetation".to_string(),
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::FLOWERING_AZALEA,
                            properties: None,
                        },
                        weight: 4i32,
                    },
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::AZALEA,
                            properties: None,
                        },
                        weight: 7i32,
                    },
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::MOSS_CARPET,
                            properties: None,
                        },
                        weight: 25i32,
                    },
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::SHORT_GRASS,
                            properties: None,
                        },
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
                        },
                        weight: 10i32,
                    },
                ],
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        "mushroom_island_vegetation".to_string(),
        ConfiguredFeature::RandomBooleanSelector(RandomBooleanFeature {
            feature_true: Box::new(PlacedFeatureWrapper::Direct(PlacedFeature {
                feature: Feature::Named("huge_red_mushroom".to_string()),
                placement: vec![],
            })),
            feature_false: Box::new(PlacedFeatureWrapper::Direct(PlacedFeature {
                feature: Feature::Named("huge_brown_mushroom".to_string()),
                placement: vec![],
            })),
        }),
    );
    map.insert(
        "nether_sprouts".to_string(),
        ConfiguredFeature::NetherForestVegetation(NetherForestVegetationFeature {
            state_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::NETHER_SPROUTS,
                    properties: None,
                },
            }),
            spread_width: 8i32,
            spread_height: 4i32,
        }),
    );
    map.insert(
        "nether_sprouts_bonemeal".to_string(),
        ConfiguredFeature::NetherForestVegetation(NetherForestVegetationFeature {
            state_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::NETHER_SPROUTS,
                    properties: None,
                },
            }),
            spread_width: 3i32,
            spread_height: 1i32,
        }),
    );
    map.insert(
        "oak".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 4u8,
                height_rand_a: 2u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Blob(BlobFoliagePlacer { height: 3i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 1u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![],
        })),
    );
    map.insert(
        "oak_bees_0002_leaf_litter".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 4u8,
                height_rand_a: 2u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Blob(BlobFoliagePlacer { height: 3i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 1u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![
                TreeDecorator::Beehive(BeehiveTreeDecorator {
                    probability: 0.002f32,
                }),
                TreeDecorator::PlaceOnGround(PlaceOnGroundTreeDecorator {
                    tries: 96i32,
                    radius: 4i32,
                    height: 2i32,
                    block_state_provider: BlockStateProvider::Weighted(
                        WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        },
                    ),
                }),
                TreeDecorator::PlaceOnGround(PlaceOnGroundTreeDecorator {
                    tries: 150i32,
                    radius: 2i32,
                    height: 2i32,
                    block_state_provider: BlockStateProvider::Weighted(
                        WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        },
                    ),
                }),
            ],
        })),
    );
    map.insert(
        "oak_bees_002".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 4u8,
                height_rand_a: 2u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Blob(BlobFoliagePlacer { height: 3i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 1u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![TreeDecorator::Beehive(BeehiveTreeDecorator {
                probability: 0.02f32,
            })],
        })),
    );
    map.insert(
        "oak_bees_005".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 4u8,
                height_rand_a: 2u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Blob(BlobFoliagePlacer { height: 3i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 1u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![TreeDecorator::Beehive(BeehiveTreeDecorator {
                probability: 0.05f32,
            })],
        })),
    );
    map.insert(
        "oak_leaf_litter".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 4u8,
                height_rand_a: 2u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Blob(BlobFoliagePlacer { height: 3i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 1u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![
                TreeDecorator::PlaceOnGround(PlaceOnGroundTreeDecorator {
                    tries: 96i32,
                    radius: 4i32,
                    height: 2i32,
                    block_state_provider: BlockStateProvider::Weighted(
                        WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        },
                    ),
                }),
                TreeDecorator::PlaceOnGround(PlaceOnGroundTreeDecorator {
                    tries: 150i32,
                    radius: 2i32,
                    height: 2i32,
                    block_state_provider: BlockStateProvider::Weighted(
                        WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        },
                    ),
                }),
            ],
        })),
    );
    map.insert(
        "ore_ancient_debris_large".to_string(),
        ConfiguredFeature::ScatteredOre(
            crate::generation::feature::features::scattered_ore::ScatteredOreFeature {},
        ),
    );
    map.insert(
        "ore_ancient_debris_small".to_string(),
        ConfiguredFeature::ScatteredOre(
            crate::generation::feature::features::scattered_ore::ScatteredOreFeature {},
        ),
    );
    map.insert(
        "ore_andesite".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 64i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::TagMatch(TagMatchRuleTest {
                    tag: "minecraft:base_stone_overworld".to_string(),
                }),
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::ANDESITE,
                    properties: None,
                },
            }],
        }),
    );
    map.insert(
        "ore_blackstone".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 33i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::BlockMatch(BlockMatchRuleTest {
                    block: "minecraft:netherrack".to_string(),
                }),
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::BLACKSTONE,
                    properties: None,
                },
            }],
        }),
    );
    map.insert(
        "ore_clay".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 33i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::TagMatch(TagMatchRuleTest {
                    tag: "minecraft:base_stone_overworld".to_string(),
                }),
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::CLAY,
                    properties: None,
                },
            }],
        }),
    );
    map.insert(
        "ore_coal".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 17i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:stone_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::COAL_ORE,
                        properties: None,
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:deepslate_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DEEPSLATE_COAL_ORE,
                        properties: None,
                    },
                },
            ],
        }),
    );
    map.insert(
        "ore_coal_buried".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 17i32,
            discard_chance_on_air_exposure: 0.5f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:stone_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::COAL_ORE,
                        properties: None,
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:deepslate_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DEEPSLATE_COAL_ORE,
                        properties: None,
                    },
                },
            ],
        }),
    );
    map.insert(
        "ore_copper_large".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 20i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:stone_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::COPPER_ORE,
                        properties: None,
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:deepslate_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DEEPSLATE_COPPER_ORE,
                        properties: None,
                    },
                },
            ],
        }),
    );
    map.insert(
        "ore_copper_small".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 10i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:stone_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::COPPER_ORE,
                        properties: None,
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:deepslate_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DEEPSLATE_COPPER_ORE,
                        properties: None,
                    },
                },
            ],
        }),
    );
    map.insert(
        "ore_diamond_buried".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 8i32,
            discard_chance_on_air_exposure: 1f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:stone_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DIAMOND_ORE,
                        properties: None,
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:deepslate_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DEEPSLATE_DIAMOND_ORE,
                        properties: None,
                    },
                },
            ],
        }),
    );
    map.insert(
        "ore_diamond_large".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 12i32,
            discard_chance_on_air_exposure: 0.7f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:stone_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DIAMOND_ORE,
                        properties: None,
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:deepslate_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DEEPSLATE_DIAMOND_ORE,
                        properties: None,
                    },
                },
            ],
        }),
    );
    map.insert(
        "ore_diamond_medium".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 8i32,
            discard_chance_on_air_exposure: 0.5f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:stone_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DIAMOND_ORE,
                        properties: None,
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:deepslate_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DEEPSLATE_DIAMOND_ORE,
                        properties: None,
                    },
                },
            ],
        }),
    );
    map.insert(
        "ore_diamond_small".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 4i32,
            discard_chance_on_air_exposure: 0.5f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:stone_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DIAMOND_ORE,
                        properties: None,
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:deepslate_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DEEPSLATE_DIAMOND_ORE,
                        properties: None,
                    },
                },
            ],
        }),
    );
    map.insert(
        "ore_diorite".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 64i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::TagMatch(TagMatchRuleTest {
                    tag: "minecraft:base_stone_overworld".to_string(),
                }),
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIORITE,
                    properties: None,
                },
            }],
        }),
    );
    map.insert(
        "ore_dirt".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 33i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::TagMatch(TagMatchRuleTest {
                    tag: "minecraft:base_stone_overworld".to_string(),
                }),
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }],
        }),
    );
    map.insert(
        "ore_emerald".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 3i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:stone_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::EMERALD_ORE,
                        properties: None,
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:deepslate_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DEEPSLATE_EMERALD_ORE,
                        properties: None,
                    },
                },
            ],
        }),
    );
    map.insert(
        "ore_gold".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 9i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:stone_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::GOLD_ORE,
                        properties: None,
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:deepslate_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DEEPSLATE_GOLD_ORE,
                        properties: None,
                    },
                },
            ],
        }),
    );
    map.insert(
        "ore_gold_buried".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 9i32,
            discard_chance_on_air_exposure: 0.5f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:stone_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::GOLD_ORE,
                        properties: None,
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:deepslate_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DEEPSLATE_GOLD_ORE,
                        properties: None,
                    },
                },
            ],
        }),
    );
    map.insert(
        "ore_granite".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 64i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::TagMatch(TagMatchRuleTest {
                    tag: "minecraft:base_stone_overworld".to_string(),
                }),
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::GRANITE,
                    properties: None,
                },
            }],
        }),
    );
    map.insert(
        "ore_gravel".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 33i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::TagMatch(TagMatchRuleTest {
                    tag: "minecraft:base_stone_overworld".to_string(),
                }),
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::GRAVEL,
                    properties: None,
                },
            }],
        }),
    );
    map.insert(
        "ore_gravel_nether".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 33i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::BlockMatch(BlockMatchRuleTest {
                    block: "minecraft:netherrack".to_string(),
                }),
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::GRAVEL,
                    properties: None,
                },
            }],
        }),
    );
    map.insert(
        "ore_infested".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 9i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:stone_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::INFESTED_STONE,
                        properties: None,
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:deepslate_ore_replaceables".to_string(),
                    }),
                    state: {
                        let mut props = std::collections::HashMap::new();
                        props.insert("axis".to_string(), "y".to_string());
                        BlockStateCodec {
                            name: &pumpkin_data::Block::INFESTED_DEEPSLATE,
                            properties: Some(props),
                        }
                    },
                },
            ],
        }),
    );
    map.insert(
        "ore_iron".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 9i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:stone_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::IRON_ORE,
                        properties: None,
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:deepslate_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DEEPSLATE_IRON_ORE,
                        properties: None,
                    },
                },
            ],
        }),
    );
    map.insert(
        "ore_iron_small".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 4i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:stone_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::IRON_ORE,
                        properties: None,
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:deepslate_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DEEPSLATE_IRON_ORE,
                        properties: None,
                    },
                },
            ],
        }),
    );
    map.insert(
        "ore_lapis".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 7i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:stone_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::LAPIS_ORE,
                        properties: None,
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:deepslate_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DEEPSLATE_LAPIS_ORE,
                        properties: None,
                    },
                },
            ],
        }),
    );
    map.insert(
        "ore_lapis_buried".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 7i32,
            discard_chance_on_air_exposure: 1f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:stone_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::LAPIS_ORE,
                        properties: None,
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:deepslate_ore_replaceables".to_string(),
                    }),
                    state: BlockStateCodec {
                        name: &pumpkin_data::Block::DEEPSLATE_LAPIS_ORE,
                        properties: None,
                    },
                },
            ],
        }),
    );
    map.insert(
        "ore_magma".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 33i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::BlockMatch(BlockMatchRuleTest {
                    block: "minecraft:netherrack".to_string(),
                }),
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::MAGMA_BLOCK,
                    properties: None,
                },
            }],
        }),
    );
    map.insert(
        "ore_nether_gold".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 10i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::BlockMatch(BlockMatchRuleTest {
                    block: "minecraft:netherrack".to_string(),
                }),
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::NETHER_GOLD_ORE,
                    properties: None,
                },
            }],
        }),
    );
    map.insert(
        "ore_quartz".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 14i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::BlockMatch(BlockMatchRuleTest {
                    block: "minecraft:netherrack".to_string(),
                }),
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::NETHER_QUARTZ_ORE,
                    properties: None,
                },
            }],
        }),
    );
    map.insert(
        "ore_redstone".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 8i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:stone_ore_replaceables".to_string(),
                    }),
                    state: {
                        let mut props = std::collections::HashMap::new();
                        props.insert("lit".to_string(), "false".to_string());
                        BlockStateCodec {
                            name: &pumpkin_data::Block::REDSTONE_ORE,
                            properties: Some(props),
                        }
                    },
                },
                OreTarget {
                    target: RuleTest::TagMatch(TagMatchRuleTest {
                        tag: "minecraft:deepslate_ore_replaceables".to_string(),
                    }),
                    state: {
                        let mut props = std::collections::HashMap::new();
                        props.insert("lit".to_string(), "false".to_string());
                        BlockStateCodec {
                            name: &pumpkin_data::Block::DEEPSLATE_REDSTONE_ORE,
                            properties: Some(props),
                        }
                    },
                },
            ],
        }),
    );
    map.insert(
        "ore_soul_sand".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 12i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::BlockMatch(BlockMatchRuleTest {
                    block: "minecraft:netherrack".to_string(),
                }),
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::SOUL_SAND,
                    properties: None,
                },
            }],
        }),
    );
    map.insert(
        "ore_tuff".to_string(),
        ConfiguredFeature::Ore(OreFeature {
            size: 64i32,
            discard_chance_on_air_exposure: 0f32,
            targets: vec![OreTarget {
                target: RuleTest::TagMatch(TagMatchRuleTest {
                    tag: "minecraft:base_stone_overworld".to_string(),
                }),
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::TUFF,
                    properties: None,
                },
            }],
        }),
    );
    map.insert(
        "pale_forest_flowers".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 96u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: BlockStateCodec {
                                name: &pumpkin_data::Block::CLOSED_EYEBLOSSOM,
                                properties: None,
                            },
                        }),
                        schedule_tick: Some(true),
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "pale_garden_vegetation".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("pale_oak_creaking_checked".to_string()),
                    chance: 0.1f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("pale_oak_checked".to_string()),
                    chance: 0.9f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named("pale_oak_checked".to_string())),
        }),
    );
    map.insert(
        "pale_moss_patch".to_string(),
        ConfiguredFeature::VegetationPatch(
            crate::generation::feature::features::vegetation_patch::VegetationPatchFeature {},
        ),
    );
    map.insert(
        "pale_moss_patch_bonemeal".to_string(),
        ConfiguredFeature::VegetationPatch(
            crate::generation::feature::features::vegetation_patch::VegetationPatchFeature {},
        ),
    );
    map.insert(
        "pale_moss_vegetation".to_string(),
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("bottom".to_string(), "true".to_string());
                            props.insert("east".to_string(), "none".to_string());
                            props.insert("north".to_string(), "none".to_string());
                            props.insert("south".to_string(), "none".to_string());
                            props.insert("west".to_string(), "none".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PALE_MOSS_CARPET,
                                properties: Some(props),
                            }
                        },
                        weight: 25i32,
                    },
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::SHORT_GRASS,
                            properties: None,
                        },
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
                        },
                        weight: 10i32,
                    },
                ],
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        "pale_oak".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::PALE_OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 6u8,
                height_rand_a: 2u8,
                height_rand_b: 1u8,
                r#type: TrunkType::DarkOak(DarkOakTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::PALE_OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(0i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::DarkOak(DarkOakFoliagePlacer),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::ThreeLayersFeatureSize(ThreeLayersFeatureSize {
                    limit: 1u8,
                    upper_limit: 1u8,
                    lower_size: 0u8,
                    middle_size: 1u8,
                    upper_size: 2u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![TreeDecorator::PaleMoss(PaleMossTreeDecorator {})],
        })),
    );
    map.insert(
        "pale_oak_bonemeal".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::PALE_OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 6u8,
                height_rand_a: 2u8,
                height_rand_b: 1u8,
                r#type: TrunkType::DarkOak(DarkOakTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::PALE_OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(0i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::DarkOak(DarkOakFoliagePlacer),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::ThreeLayersFeatureSize(ThreeLayersFeatureSize {
                    limit: 1u8,
                    upper_limit: 1u8,
                    lower_size: 0u8,
                    middle_size: 1u8,
                    upper_size: 2u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![],
        })),
    );
    map.insert(
        "pale_oak_creaking".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::PALE_OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 6u8,
                height_rand_a: 2u8,
                height_rand_b: 1u8,
                r#type: TrunkType::DarkOak(DarkOakTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::PALE_OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(0i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::DarkOak(DarkOakFoliagePlacer),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::ThreeLayersFeatureSize(ThreeLayersFeatureSize {
                    limit: 1u8,
                    upper_limit: 1u8,
                    lower_size: 0u8,
                    middle_size: 1u8,
                    upper_size: 2u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![
                TreeDecorator::PaleMoss(PaleMossTreeDecorator {}),
                TreeDecorator::CreakingHeart(CreakingHeartTreeDecorator {}),
            ],
        })),
    );
    map.insert(
        "patch_berry_bush".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 96u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: {
                                let mut props = std::collections::HashMap::new();
                                props.insert("age".to_string(), "3".to_string());
                                BlockStateCodec {
                                    name: &pumpkin_data::Block::SWEET_BERRY_BUSH,
                                    properties: Some(props),
                                }
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                            predicates: vec![
                                BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    blocks: MatchingBlocksWrapper::Single(
                                        "minecraft:air".to_string(),
                                    ),
                                }),
                                BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate {
                                        offset: Some(Vector3::new(0i32, -1i32, 0i32)),
                                    },
                                    blocks: MatchingBlocksWrapper::Single(
                                        "minecraft:grass_block".to_string(),
                                    ),
                                }),
                            ],
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_brown_mushroom".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 96u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: BlockStateCodec {
                                name: &pumpkin_data::Block::BROWN_MUSHROOM,
                                properties: None,
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_bush".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 24u8,
            xz_spread: 5u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: BlockStateCodec {
                                name: &pumpkin_data::Block::BUSH,
                                properties: None,
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_cactus".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 10u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::BlockColumn(
                    BlockColumnFeature {
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
                                    state: BlockStateCodec {
                                        name: &pumpkin_data::Block::CACTUS_FLOWER,
                                        properties: None,
                                    },
                                }),
                            },
                        ],
                        direction: BlockDirection::Up,
                        allowed_placement: BlockPredicate::MatchingBlocks(
                            MatchingBlocksBlockPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                                blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                            },
                        ),
                        prioritize_tip: false,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                            predicates: vec![
                                BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    blocks: MatchingBlocksWrapper::Single(
                                        "minecraft:air".to_string(),
                                    ),
                                }),
                                BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    state: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("age".to_string(), "0".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::CACTUS,
                                            properties: Some(props),
                                        }
                                    },
                                }),
                            ],
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_crimson_roots".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 96u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: BlockStateCodec {
                                name: &pumpkin_data::Block::CRIMSON_ROOTS,
                                properties: None,
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_dead_bush".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 4u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: BlockStateCodec {
                                name: &pumpkin_data::Block::DEAD_BUSH,
                                properties: None,
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_dry_grass".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 64u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: BlockStateCodec {
                                        name: &pumpkin_data::Block::SHORT_DRY_GRASS,
                                        properties: None,
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: BlockStateCodec {
                                        name: &pumpkin_data::Block::TALL_DRY_GRASS,
                                        properties: None,
                                    },
                                    weight: 1i32,
                                },
                            ],
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_fire".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 96u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: {
                                let mut props = std::collections::HashMap::new();
                                props.insert("age".to_string(), "0".to_string());
                                props.insert("east".to_string(), "false".to_string());
                                props.insert("north".to_string(), "false".to_string());
                                props.insert("south".to_string(), "false".to_string());
                                props.insert("up".to_string(), "false".to_string());
                                props.insert("west".to_string(), "false".to_string());
                                BlockStateCodec {
                                    name: &pumpkin_data::Block::FIRE,
                                    properties: Some(props),
                                }
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                            predicates: vec![
                                BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    blocks: MatchingBlocksWrapper::Single(
                                        "minecraft:air".to_string(),
                                    ),
                                }),
                                BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate {
                                        offset: Some(Vector3::new(0i32, -1i32, 0i32)),
                                    },
                                    blocks: MatchingBlocksWrapper::Single(
                                        "minecraft:netherrack".to_string(),
                                    ),
                                }),
                            ],
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_firefly_bush".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 20u8,
            xz_spread: 4u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: BlockStateCodec {
                                name: &pumpkin_data::Block::FIREFLY_BUSH,
                                properties: None,
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_grass".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 32u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: BlockStateCodec {
                                name: &pumpkin_data::Block::SHORT_GRASS,
                                properties: None,
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_grass_jungle".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 32u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: BlockStateCodec {
                                        name: &pumpkin_data::Block::SHORT_GRASS,
                                        properties: None,
                                    },
                                    weight: 3i32,
                                },
                                Weighted {
                                    data: BlockStateCodec {
                                        name: &pumpkin_data::Block::FERN,
                                        properties: None,
                                    },
                                    weight: 1i32,
                                },
                            ],
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                            predicates: vec![
                                BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    blocks: MatchingBlocksWrapper::Single(
                                        "minecraft:air".to_string(),
                                    ),
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
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_grass_meadow".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 16u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: BlockStateCodec {
                                name: &pumpkin_data::Block::SHORT_GRASS,
                                properties: None,
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_large_fern".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 96u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: {
                                let mut props = std::collections::HashMap::new();
                                props.insert("half".to_string(), "lower".to_string());
                                BlockStateCodec {
                                    name: &pumpkin_data::Block::LARGE_FERN,
                                    properties: Some(props),
                                }
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_leaf_litter".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 32u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("segment_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::LEAF_LITTER,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                            predicates: vec![
                                BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    blocks: MatchingBlocksWrapper::Single(
                                        "minecraft:air".to_string(),
                                    ),
                                }),
                                BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate {
                                        offset: Some(Vector3::new(0i32, -1i32, 0i32)),
                                    },
                                    blocks: MatchingBlocksWrapper::Single(
                                        "minecraft:grass_block".to_string(),
                                    ),
                                }),
                            ],
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_melon".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 64u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: BlockStateCodec {
                                name: &pumpkin_data::Block::MELON,
                                properties: None,
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                            predicates: vec![
                                BlockPredicate::Replaceable(ReplaceableBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                }),
                                BlockPredicate::MatchingFluids(MatchingFluidsBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    fluids: MatchingBlocksWrapper::Single(
                                        "minecraft:empty".to_string(),
                                    ),
                                }),
                                BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate {
                                        offset: Some(Vector3::new(0i32, -1i32, 0i32)),
                                    },
                                    blocks: MatchingBlocksWrapper::Single(
                                        "minecraft:grass_block".to_string(),
                                    ),
                                }),
                            ],
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_pumpkin".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 96u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: BlockStateCodec {
                                name: &pumpkin_data::Block::PUMPKIN,
                                properties: None,
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                            predicates: vec![
                                BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    blocks: MatchingBlocksWrapper::Single(
                                        "minecraft:air".to_string(),
                                    ),
                                }),
                                BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate {
                                        offset: Some(Vector3::new(0i32, -1i32, 0i32)),
                                    },
                                    blocks: MatchingBlocksWrapper::Single(
                                        "minecraft:grass_block".to_string(),
                                    ),
                                }),
                            ],
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_red_mushroom".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 96u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: BlockStateCodec {
                                name: &pumpkin_data::Block::RED_MUSHROOM,
                                properties: None,
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_soul_fire".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 96u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: BlockStateCodec {
                                name: &pumpkin_data::Block::SOUL_FIRE,
                                properties: None,
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                            predicates: vec![
                                BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    blocks: MatchingBlocksWrapper::Single(
                                        "minecraft:air".to_string(),
                                    ),
                                }),
                                BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate {
                                        offset: Some(Vector3::new(0i32, -1i32, 0i32)),
                                    },
                                    blocks: MatchingBlocksWrapper::Single(
                                        "minecraft:soul_soil".to_string(),
                                    ),
                                }),
                            ],
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_sugar_cane".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 20u8,
            xz_spread: 4u8,
            y_spread: 0u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::BlockColumn(
                    BlockColumnFeature {
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
                                },
                            }),
                        }],
                        direction: BlockDirection::Up,
                        allowed_placement: BlockPredicate::MatchingBlocks(
                            MatchingBlocksBlockPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                                blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                            },
                        ),
                        prioritize_tip: false,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                            predicates: vec![
                                BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    blocks: MatchingBlocksWrapper::Single(
                                        "minecraft:air".to_string(),
                                    ),
                                }),
                                BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    state: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("age".to_string(), "0".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::SUGAR_CANE,
                                            properties: Some(props),
                                        }
                                    },
                                }),
                                BlockPredicate::AnyOf(AnyOfBlockPredicate {
                                    predicates: vec![
                                        BlockPredicate::MatchingFluids(
                                            MatchingFluidsBlockPredicate {
                                                offset: OffsetBlocksBlockPredicate {
                                                    offset: Some(Vector3::new(1i32, -1i32, 0i32)),
                                                },
                                                fluids: MatchingBlocksWrapper::Multiple(vec![
                                                    "minecraft:water".to_string(),
                                                    "minecraft:flowing_water".to_string(),
                                                ]),
                                            },
                                        ),
                                        BlockPredicate::MatchingFluids(
                                            MatchingFluidsBlockPredicate {
                                                offset: OffsetBlocksBlockPredicate {
                                                    offset: Some(Vector3::new(-1i32, -1i32, 0i32)),
                                                },
                                                fluids: MatchingBlocksWrapper::Multiple(vec![
                                                    "minecraft:water".to_string(),
                                                    "minecraft:flowing_water".to_string(),
                                                ]),
                                            },
                                        ),
                                        BlockPredicate::MatchingFluids(
                                            MatchingFluidsBlockPredicate {
                                                offset: OffsetBlocksBlockPredicate {
                                                    offset: Some(Vector3::new(0i32, -1i32, 1i32)),
                                                },
                                                fluids: MatchingBlocksWrapper::Multiple(vec![
                                                    "minecraft:water".to_string(),
                                                    "minecraft:flowing_water".to_string(),
                                                ]),
                                            },
                                        ),
                                        BlockPredicate::MatchingFluids(
                                            MatchingFluidsBlockPredicate {
                                                offset: OffsetBlocksBlockPredicate {
                                                    offset: Some(Vector3::new(0i32, -1i32, -1i32)),
                                                },
                                                fluids: MatchingBlocksWrapper::Multiple(vec![
                                                    "minecraft:water".to_string(),
                                                    "minecraft:flowing_water".to_string(),
                                                ]),
                                            },
                                        ),
                                    ],
                                }),
                            ],
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_sunflower".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 96u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: {
                                let mut props = std::collections::HashMap::new();
                                props.insert("half".to_string(), "lower".to_string());
                                BlockStateCodec {
                                    name: &pumpkin_data::Block::SUNFLOWER,
                                    properties: Some(props),
                                }
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_taiga_grass".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 32u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: BlockStateCodec {
                                        name: &pumpkin_data::Block::SHORT_GRASS,
                                        properties: None,
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: BlockStateCodec {
                                        name: &pumpkin_data::Block::FERN,
                                        properties: None,
                                    },
                                    weight: 4i32,
                                },
                            ],
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_tall_grass".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 96u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: {
                                let mut props = std::collections::HashMap::new();
                                props.insert("half".to_string(), "lower".to_string());
                                BlockStateCodec {
                                    name: &pumpkin_data::Block::TALL_GRASS,
                                    properties: Some(props),
                                }
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "patch_waterlily".to_string(),
        ConfiguredFeature::RandomPatch(RandomPatchFeature {
            tries: 10u8,
            xz_spread: 7u8,
            y_spread: 3u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Simple(SimpleStateProvider {
                            state: BlockStateCodec {
                                name: &pumpkin_data::Block::LILY_PAD,
                                properties: None,
                            },
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "pile_hay".to_string(),
        ConfiguredFeature::BlockPile(
            crate::generation::feature::features::block_pile::BlockPileFeature {},
        ),
    );
    map.insert(
        "pile_ice".to_string(),
        ConfiguredFeature::BlockPile(
            crate::generation::feature::features::block_pile::BlockPileFeature {},
        ),
    );
    map.insert(
        "pile_melon".to_string(),
        ConfiguredFeature::BlockPile(
            crate::generation::feature::features::block_pile::BlockPileFeature {},
        ),
    );
    map.insert(
        "pile_pumpkin".to_string(),
        ConfiguredFeature::BlockPile(
            crate::generation::feature::features::block_pile::BlockPileFeature {},
        ),
    );
    map.insert(
        "pile_snow".to_string(),
        ConfiguredFeature::BlockPile(
            crate::generation::feature::features::block_pile::BlockPileFeature {},
        ),
    );
    map.insert(
        "pine".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::SPRUCE_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 6u8,
                height_rand_a: 4u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::SPRUCE_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(1i32),
                offset: IntProvider::Constant(1i32),
                r#type: FoliageType::Pine(PineFoliagePlacer {
                    height: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                        min_inclusive: 3i32,
                        max_inclusive: 4i32,
                    })),
                }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 2u8,
                    lower_size: 0u8,
                    upper_size: 2u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![],
        })),
    );
    map.insert(
        "pointed_dripstone".to_string(),
        ConfiguredFeature::SimpleRandomSelector(SimpleRandomFeature {
            features: vec![
                PlacedFeature {
                    feature: Feature::Inlined(Box::new(ConfiguredFeature::PointedDripstone(
                        SmallDripstoneFeature {
                            chance_of_taller_dripstone: 0.2f32,
                            chance_of_directional_spread: 0.7f32,
                            chance_of_spread_radius2: 0.5f32,
                            chance_of_spread_radius3: 0.5f32,
                        },
                    ))),
                    placement: vec![
                        PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier {
                            direction_of_search: BlockDirection::Down,
                            target_condition: BlockPredicate::Solid(SolidBlockPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                            }),
                            allowed_search_condition: Some(BlockPredicate::MatchingBlocks(
                                MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    blocks: MatchingBlocksWrapper::Multiple(vec![
                                        "minecraft:air".to_string(),
                                        "minecraft:water".to_string(),
                                    ]),
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
                            chance_of_taller_dripstone: 0.2f32,
                            chance_of_directional_spread: 0.7f32,
                            chance_of_spread_radius2: 0.5f32,
                            chance_of_spread_radius3: 0.5f32,
                        },
                    ))),
                    placement: vec![
                        PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier {
                            direction_of_search: BlockDirection::Up,
                            target_condition: BlockPredicate::Solid(SolidBlockPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                            }),
                            allowed_search_condition: Some(BlockPredicate::MatchingBlocks(
                                MatchingBlocksBlockPredicate {
                                    offset: OffsetBlocksBlockPredicate { offset: None },
                                    blocks: MatchingBlocksWrapper::Multiple(vec![
                                        "minecraft:air".to_string(),
                                        "minecraft:water".to_string(),
                                    ]),
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
        "rooted_azalea_tree".to_string(),
        ConfiguredFeature::RootSystem(
            crate::generation::feature::features::root_system::RootSystemFeature {},
        ),
    );
    map.insert(
        "sculk_patch_ancient_city".to_string(),
        ConfiguredFeature::SculkPatch(
            crate::generation::feature::features::sculk_patch::SculkPatchFeature {},
        ),
    );
    map.insert(
        "sculk_patch_deep_dark".to_string(),
        ConfiguredFeature::SculkPatch(
            crate::generation::feature::features::sculk_patch::SculkPatchFeature {},
        ),
    );
    map.insert(
        "sculk_vein".to_string(),
        ConfiguredFeature::MultifaceGrowth(
            crate::generation::feature::features::multiface_growth::MultifaceGrowthFeature {},
        ),
    );
    map.insert(
        "sea_pickle".to_string(),
        ConfiguredFeature::SeaPickle(SeaPickleFeature {
            count: IntProvider::Constant(20i32),
        }),
    );
    map.insert(
        "seagrass_mid".to_string(),
        ConfiguredFeature::Seagrass(SeagrassFeature {
            probability: 0.6f32,
        }),
    );
    map.insert(
        "seagrass_short".to_string(),
        ConfiguredFeature::Seagrass(SeagrassFeature {
            probability: 0.3f32,
        }),
    );
    map.insert(
        "seagrass_slightly_less_short".to_string(),
        ConfiguredFeature::Seagrass(SeagrassFeature {
            probability: 0.4f32,
        }),
    );
    map.insert(
        "seagrass_tall".to_string(),
        ConfiguredFeature::Seagrass(SeagrassFeature {
            probability: 0.8f32,
        }),
    );
    map.insert(
        "single_piece_of_grass".to_string(),
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::SHORT_GRASS,
                    properties: None,
                },
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        "small_basalt_columns".to_string(),
        ConfiguredFeature::BasaltColumns(
            crate::generation::feature::features::basalt_columns::BasaltColumnsFeature {},
        ),
    );
    map.insert(
        "spore_blossom".to_string(),
        ConfiguredFeature::SimpleBlock(SimpleBlockFeature {
            to_place: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::SPORE_BLOSSOM,
                    properties: None,
                },
            }),
            schedule_tick: None,
        }),
    );
    map.insert(
        "spring_lava_frozen".to_string(),
        ConfiguredFeature::SpringFeature(SpringFeatureFeature {
            state: {
                let mut props = std::collections::HashMap::new();
                props.insert("falling".to_string(), "true".to_string());
                BlockStateCodec {
                    name: &pumpkin_data::Block::LAVA,
                    properties: Some(props),
                }
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
        "spring_lava_nether".to_string(),
        ConfiguredFeature::SpringFeature(SpringFeatureFeature {
            state: {
                let mut props = std::collections::HashMap::new();
                props.insert("falling".to_string(), "true".to_string());
                BlockStateCodec {
                    name: &pumpkin_data::Block::LAVA,
                    properties: Some(props),
                }
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
        "spring_lava_overworld".to_string(),
        ConfiguredFeature::SpringFeature(SpringFeatureFeature {
            state: {
                let mut props = std::collections::HashMap::new();
                props.insert("falling".to_string(), "true".to_string());
                BlockStateCodec {
                    name: &pumpkin_data::Block::LAVA,
                    properties: Some(props),
                }
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
        "spring_nether_closed".to_string(),
        ConfiguredFeature::SpringFeature(SpringFeatureFeature {
            state: {
                let mut props = std::collections::HashMap::new();
                props.insert("falling".to_string(), "true".to_string());
                BlockStateCodec {
                    name: &pumpkin_data::Block::LAVA,
                    properties: Some(props),
                }
            },
            requires_block_below: false,
            rock_count: 5i32,
            hole_count: 0i32,
            valid_blocks: BlockWrapper::Single("minecraft:netherrack".to_string()),
        }),
    );
    map.insert(
        "spring_nether_open".to_string(),
        ConfiguredFeature::SpringFeature(SpringFeatureFeature {
            state: {
                let mut props = std::collections::HashMap::new();
                props.insert("falling".to_string(), "true".to_string());
                BlockStateCodec {
                    name: &pumpkin_data::Block::LAVA,
                    properties: Some(props),
                }
            },
            requires_block_below: false,
            rock_count: 4i32,
            hole_count: 1i32,
            valid_blocks: BlockWrapper::Single("minecraft:netherrack".to_string()),
        }),
    );
    map.insert(
        "spring_water".to_string(),
        ConfiguredFeature::SpringFeature(SpringFeatureFeature {
            state: {
                let mut props = std::collections::HashMap::new();
                props.insert("falling".to_string(), "true".to_string());
                BlockStateCodec {
                    name: &pumpkin_data::Block::WATER,
                    properties: Some(props),
                }
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
    map.insert(
        "spruce".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::SPRUCE_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 5u8,
                height_rand_a: 2u8,
                height_rand_b: 1u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::SPRUCE_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                    min_inclusive: 2i32,
                    max_inclusive: 3i32,
                })),
                offset: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                    min_inclusive: 0i32,
                    max_inclusive: 2i32,
                })),
                r#type: FoliageType::Spruce(SpruceFoliagePlacer {
                    trunk_height: IntProvider::Object(NormalIntProvider::Uniform(
                        UniformIntProvider {
                            min_inclusive: 1i32,
                            max_inclusive: 2i32,
                        },
                    )),
                }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 2u8,
                    lower_size: 0u8,
                    upper_size: 2u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![],
        })),
    );
    map.insert(
        "super_birch_bees".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 5u8,
                height_rand_a: 2u8,
                height_rand_b: 6u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Blob(BlobFoliagePlacer { height: 3i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 1u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![TreeDecorator::Beehive(BeehiveTreeDecorator {
                probability: 1f32,
            })],
        })),
    );
    map.insert(
        "super_birch_bees_0002".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 5u8,
                height_rand_a: 2u8,
                height_rand_b: 6u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::BIRCH_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(2i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Blob(BlobFoliagePlacer { height: 3i32 }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 1u8,
                    lower_size: 0u8,
                    upper_size: 1u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![TreeDecorator::Beehive(BeehiveTreeDecorator {
                probability: 0.002f32,
            })],
        })),
    );
    map.insert(
        "swamp_oak".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 5u8,
                height_rand_a: 3u8,
                height_rand_b: 0u8,
                r#type: TrunkType::Straight(StraightTrunkPlacer),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::OAK_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(3i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::Blob(BlobFoliagePlacer { height: 3i32 }),
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
            force_dirt: false,
            decorators: vec![TreeDecorator::LeaveVine(LeavesVineTreeDecorator {})],
        })),
    );
    map.insert(
        "tall_mangrove".to_string(),
        ConfiguredFeature::Tree(Box::new(TreeFeature {
            dirt_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: BlockStateCodec {
                    name: &pumpkin_data::Block::DIRT,
                    properties: None,
                },
            }),
            trunk_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("axis".to_string(), "y".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::MANGROVE_LOG,
                        properties: Some(props),
                    }
                },
            }),
            trunk_placer: TrunkPlacer {
                base_height: 4u8,
                height_rand_a: 1u8,
                height_rand_b: 9u8,
                r#type: TrunkType::UpwardsBranching(UpwardsBranchingTrunkPlacer {}),
            },
            foliage_provider: BlockStateProvider::Simple(SimpleStateProvider {
                state: {
                    let mut props = std::collections::HashMap::new();
                    props.insert("distance".to_string(), "7".to_string());
                    props.insert("persistent".to_string(), "false".to_string());
                    props.insert("waterlogged".to_string(), "false".to_string());
                    BlockStateCodec {
                        name: &pumpkin_data::Block::MANGROVE_LEAVES,
                        properties: Some(props),
                    }
                },
            }),
            foliage_placer: FoliagePlacer {
                radius: IntProvider::Constant(3i32),
                offset: IntProvider::Constant(0i32),
                r#type: FoliageType::RandomSpread(RandomSpreadFoliagePlacer {
                    foliage_height: IntProvider::Constant(2i32),
                    leaf_placement_attempts: 70i32,
                }),
            },
            minimum_size: FeatureSize {
                min_clipped_height: None,
                r#type: FeatureSizeType::TwoLayersFeatureSize(TwoLayersFeatureSize {
                    limit: 3u8,
                    lower_size: 0u8,
                    upper_size: 2u8,
                }),
            },
            ignore_vines: true,
            force_dirt: false,
            decorators: vec![
                TreeDecorator::LeaveVine(LeavesVineTreeDecorator {}),
                TreeDecorator::AttachedToLeaves(AttachedToLeavesTreeDecorator {}),
                TreeDecorator::Beehive(BeehiveTreeDecorator {
                    probability: 0.01f32,
                }),
            ],
        })),
    );
    map.insert(
        "trees_badlands".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![RandomFeatureEntry {
                feature: PlacedFeatureWrapper::Named("fallen_oak_tree".to_string()),
                chance: 0.0125f32,
            }],
            default: Box::new(PlacedFeatureWrapper::Named("oak_leaf_litter".to_string())),
        }),
    );
    map.insert(
        "trees_birch".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![RandomFeatureEntry {
                feature: PlacedFeatureWrapper::Named("fallen_birch_tree".to_string()),
                chance: 0.0125f32,
            }],
            default: Box::new(PlacedFeatureWrapper::Named("birch_bees_0002".to_string())),
        }),
    );
    map.insert(
        "trees_birch_and_oak_leaf_litter".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fallen_birch_tree".to_string()),
                    chance: 0.0025f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("birch_bees_0002_leaf_litter".to_string()),
                    chance: 0.2f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named(
                        "fancy_oak_bees_0002_leaf_litter".to_string(),
                    ),
                    chance: 0.1f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fallen_oak_tree".to_string()),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named(
                "oak_bees_0002_leaf_litter".to_string(),
            )),
        }),
    );
    map.insert(
        "trees_flower_forest".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fallen_birch_tree".to_string()),
                    chance: 0.0025f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("birch_bees_002".to_string()),
                    chance: 0.2f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fancy_oak_bees_002".to_string()),
                    chance: 0.1f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named("oak_bees_002".to_string())),
        }),
    );
    map.insert(
        "trees_grove".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![RandomFeatureEntry {
                feature: PlacedFeatureWrapper::Named("pine_on_snow".to_string()),
                chance: 0.33333334f32,
            }],
            default: Box::new(PlacedFeatureWrapper::Named("spruce_on_snow".to_string())),
        }),
    );
    map.insert(
        "trees_jungle".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fancy_oak_checked".to_string()),
                    chance: 0.1f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("jungle_bush".to_string()),
                    chance: 0.5f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("mega_jungle_tree_checked".to_string()),
                    chance: 0.33333334f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fallen_jungle_tree".to_string()),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named("jungle_tree".to_string())),
        }),
    );
    map.insert(
        "trees_old_growth_pine_taiga".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("mega_spruce_checked".to_string()),
                    chance: 0.025641026f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("mega_pine_checked".to_string()),
                    chance: 0.30769232f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("pine_checked".to_string()),
                    chance: 0.33333334f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fallen_spruce_tree".to_string()),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named("spruce_checked".to_string())),
        }),
    );
    map.insert(
        "trees_old_growth_spruce_taiga".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("mega_spruce_checked".to_string()),
                    chance: 0.33333334f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("pine_checked".to_string()),
                    chance: 0.33333334f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fallen_spruce_tree".to_string()),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named("spruce_checked".to_string())),
        }),
    );
    map.insert(
        "trees_plains".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Direct(PlacedFeature {
                        feature: Feature::Named("fancy_oak_bees_005".to_string()),
                        placement: vec![],
                    }),
                    chance: 0.33333334f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fallen_oak_tree".to_string()),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Direct(PlacedFeature {
                feature: Feature::Named("oak_bees_005".to_string()),
                placement: vec![],
            })),
        }),
    );
    map.insert(
        "trees_savanna".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("acacia_checked".to_string()),
                    chance: 0.8f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fallen_oak_tree".to_string()),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named("oak_checked".to_string())),
        }),
    );
    map.insert(
        "trees_snowy".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![RandomFeatureEntry {
                feature: PlacedFeatureWrapper::Named("fallen_spruce_tree".to_string()),
                chance: 0.0125f32,
            }],
            default: Box::new(PlacedFeatureWrapper::Named("spruce_checked".to_string())),
        }),
    );
    map.insert(
        "trees_sparse_jungle".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fancy_oak_checked".to_string()),
                    chance: 0.1f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("jungle_bush".to_string()),
                    chance: 0.5f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fallen_jungle_tree".to_string()),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named("jungle_tree".to_string())),
        }),
    );
    map.insert(
        "trees_taiga".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("pine_checked".to_string()),
                    chance: 0.33333334f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fallen_spruce_tree".to_string()),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named("spruce_checked".to_string())),
        }),
    );
    map.insert(
        "trees_water".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![RandomFeatureEntry {
                feature: PlacedFeatureWrapper::Named("fancy_oak_checked".to_string()),
                chance: 0.1f32,
            }],
            default: Box::new(PlacedFeatureWrapper::Named("oak_checked".to_string())),
        }),
    );
    map.insert(
        "trees_windswept_hills".to_string(),
        ConfiguredFeature::RandomSelector(RandomFeature {
            features: vec![
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fallen_spruce_tree".to_string()),
                    chance: 0.008325f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("spruce_checked".to_string()),
                    chance: 0.666f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fancy_oak_checked".to_string()),
                    chance: 0.1f32,
                },
                RandomFeatureEntry {
                    feature: PlacedFeatureWrapper::Named("fallen_oak_tree".to_string()),
                    chance: 0.0125f32,
                },
            ],
            default: Box::new(PlacedFeatureWrapper::Named("oak_checked".to_string())),
        }),
    );
    map.insert(
        "twisting_vines".to_string(),
        ConfiguredFeature::TwistingVines(
            crate::generation::feature::features::twisting_vines::TwistingVinesFeature {},
        ),
    );
    map.insert(
        "twisting_vines_bonemeal".to_string(),
        ConfiguredFeature::TwistingVines(
            crate::generation::feature::features::twisting_vines::TwistingVinesFeature {},
        ),
    );
    map.insert(
        "underwater_magma".to_string(),
        ConfiguredFeature::UnderwaterMagma(
            crate::generation::feature::features::underwater_magma::UnderwaterMagmaFeature {
                floor_search_range: 5i32,
                placement_radius: 1i32,
                placement_probability: 0.5f32,
            },
        ),
    );
    map.insert(
        "vines".to_string(),
        ConfiguredFeature::Vines(crate::generation::feature::features::vines::VinesFeature),
    );
    map.insert(
        "void_start_platform".to_string(),
        ConfiguredFeature::VoidStartPlatform(
            crate::generation::feature::features::void_start_platform::VoidStartPlatformFeature {},
        ),
    );
    map . insert ("warm_ocean_vegetation" . to_string () , ConfiguredFeature :: SimpleRandomSelector (SimpleRandomFeature { features : vec ! [PlacedFeature { feature : Feature :: Inlined (Box :: new (ConfiguredFeature :: CoralTree (crate :: generation :: feature :: features :: coral :: coral_tree :: CoralTreeFeature))) , placement : vec ! [] , } , PlacedFeature { feature : Feature :: Inlined (Box :: new (ConfiguredFeature :: CoralClaw (crate :: generation :: feature :: features :: coral :: coral_claw :: CoralClawFeature))) , placement : vec ! [] , } , PlacedFeature { feature : Feature :: Inlined (Box :: new (ConfiguredFeature :: CoralMushroom (crate :: generation :: feature :: features :: coral :: coral_mushroom :: CoralMushroomFeature))) , placement : vec ! [] , }] , })) ;
    map.insert(
        "warped_forest_vegetation".to_string(),
        ConfiguredFeature::NetherForestVegetation(NetherForestVegetationFeature {
            state_provider: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::WARPED_ROOTS,
                            properties: None,
                        },
                        weight: 85i32,
                    },
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::CRIMSON_ROOTS,
                            properties: None,
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::WARPED_FUNGUS,
                            properties: None,
                        },
                        weight: 13i32,
                    },
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::CRIMSON_FUNGUS,
                            properties: None,
                        },
                        weight: 1i32,
                    },
                ],
            }),
            spread_width: 8i32,
            spread_height: 4i32,
        }),
    );
    map.insert(
        "warped_forest_vegetation_bonemeal".to_string(),
        ConfiguredFeature::NetherForestVegetation(NetherForestVegetationFeature {
            state_provider: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                entries: vec![
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::WARPED_ROOTS,
                            properties: None,
                        },
                        weight: 85i32,
                    },
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::CRIMSON_ROOTS,
                            properties: None,
                        },
                        weight: 1i32,
                    },
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::WARPED_FUNGUS,
                            properties: None,
                        },
                        weight: 13i32,
                    },
                    Weighted {
                        data: BlockStateCodec {
                            name: &pumpkin_data::Block::CRIMSON_FUNGUS,
                            properties: None,
                        },
                        weight: 1i32,
                    },
                ],
            }),
            spread_width: 3i32,
            spread_height: 1i32,
        }),
    );
    map.insert(
        "warped_fungus".to_string(),
        ConfiguredFeature::HugeFungus(
            crate::generation::feature::features::huge_fungus::HugeFungusFeature {},
        ),
    );
    map.insert(
        "warped_fungus_planted".to_string(),
        ConfiguredFeature::HugeFungus(
            crate::generation::feature::features::huge_fungus::HugeFungusFeature {},
        ),
    );
    map.insert(
        "weeping_vines".to_string(),
        ConfiguredFeature::WeepingVines(
            crate::generation::feature::features::weeping_vines::WeepingVinesFeature {},
        ),
    );
    map.insert(
        "wildflowers_birch_forest".to_string(),
        ConfiguredFeature::Flower(RandomPatchFeature {
            tries: 64u8,
            xz_spread: 6u8,
            y_spread: 2u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("flower_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("flower_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("flower_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("flower_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("flower_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("flower_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("flower_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("flower_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("flower_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("flower_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("flower_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("flower_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("flower_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("flower_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("flower_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("flower_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map.insert(
        "wildflowers_meadow".to_string(),
        ConfiguredFeature::Flower(RandomPatchFeature {
            tries: 8u8,
            xz_spread: 6u8,
            y_spread: 2u8,
            feature: Box::new(PlacedFeature {
                feature: Feature::Inlined(Box::new(ConfiguredFeature::SimpleBlock(
                    SimpleBlockFeature {
                        to_place: BlockStateProvider::Weighted(WeightedBlockStateProvider {
                            entries: vec![
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("flower_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("flower_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("flower_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("flower_amount".to_string(), "1".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("flower_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("flower_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("flower_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("flower_amount".to_string(), "2".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("flower_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("flower_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("flower_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("flower_amount".to_string(), "3".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "north".to_string());
                                        props.insert("flower_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "east".to_string());
                                        props.insert("flower_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "south".to_string());
                                        props.insert("flower_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                                Weighted {
                                    data: {
                                        let mut props = std::collections::HashMap::new();
                                        props.insert("facing".to_string(), "west".to_string());
                                        props.insert("flower_amount".to_string(), "4".to_string());
                                        BlockStateCodec {
                                            name: &pumpkin_data::Block::WILDFLOWERS,
                                            properties: Some(props),
                                        }
                                    },
                                    weight: 1i32,
                                },
                            ],
                        }),
                        schedule_tick: None,
                    },
                ))),
                placement: vec![PlacementModifier::BlockPredicateFilter(
                    BlockFilterPlacementModifier {
                        predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        }),
                    },
                )],
            }),
        }),
    );
    map
}
