/* This file is generated. Do not edit manually. */
#[allow(clippy::all, unused_imports, dead_code)]
fn build_placed_features() -> std::collections::HashMap<String, PlacedFeature> {
    use crate::block::BlockStateCodec;
    use crate::generation::block_predicate::{
        AllOfBlockPredicate, AnyOfBlockPredicate, BlockPredicate, HasSturdyFacePredicate,
        InsideWorldBoundsBlockPredicate, MatchingBlockTagPredicate, MatchingBlocksBlockPredicate,
        MatchingBlocksWrapper, MatchingFluidsBlockPredicate, NotBlockPredicate,
        OffsetBlocksBlockPredicate, ReplaceableBlockPredicate, SolidBlockPredicate,
        WouldSurviveBlockPredicate,
    };
    use crate::generation::height_provider::{
        HeightProvider, TrapezoidHeightProvider, UniformHeightProvider,
        VeryBiasedToBottomHeightProvider,
    };
    use pumpkin_data::{Block, BlockDirection};
    use pumpkin_util::HeightMap;
    use pumpkin_util::math::int_provider::{
        BiasedToBottomIntProvider, ClampedIntProvider, ClampedNormalIntProvider,
        ConstantIntProvider, IntProvider, NormalIntProvider, UniformIntProvider, WeightedEntry,
        WeightedListIntProvider,
    };
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_util::y_offset::{AboveBottom, Absolute, BelowTop, YOffset};
    let mut map = std::collections::HashMap::new();
    map.insert(
        "acacia".to_string(),
        PlacedFeature {
            feature: Feature::Named("acacia".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::ACACIA_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "acacia_checked".to_string(),
        PlacedFeature {
            feature: Feature::Named("acacia".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::ACACIA_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "amethyst_geode".to_string(),
        PlacedFeature {
            feature: Feature::Named("amethyst_geode".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 24u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 6i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 30i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "bamboo".to_string(),
        PlacedFeature {
            feature: Feature::Named("bamboo_some_podzol".to_string()),
            placement: vec![
                PlacementModifier::NoiseBasedCount(NoiseBasedCountPlacementModifier {
                    noise_to_count_ratio: 160i32,
                    noise_factor: 80f64,
                    noise_offset: 0.3f64,
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "bamboo_light".to_string(),
        PlacedFeature {
            feature: Feature::Named("bamboo_no_podzol".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 4u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "bamboo_vegetation".to_string(),
        PlacedFeature {
            feature: Feature::Named("bamboo_vegetation".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(30i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(31i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "basalt_blobs".to_string(),
        PlacedFeature {
            feature: Feature::Named("basalt_blobs".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(75i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 0i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "basalt_pillar".to_string(),
        PlacedFeature {
            feature: Feature::Named("basalt_pillar".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(10i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 0i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "birch_bees_0002".to_string(),
        PlacedFeature {
            feature: Feature::Named("birch_bees_0002".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::BIRCH_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "birch_bees_0002_leaf_litter".to_string(),
        PlacedFeature {
            feature: Feature::Named("birch_bees_0002_leaf_litter".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::BIRCH_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "birch_bees_002".to_string(),
        PlacedFeature {
            feature: Feature::Named("birch_bees_002".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::BIRCH_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "birch_checked".to_string(),
        PlacedFeature {
            feature: Feature::Named("birch".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::BIRCH_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "birch_leaf_litter".to_string(),
        PlacedFeature {
            feature: Feature::Named("birch_leaf_litter".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::BIRCH_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "birch_tall".to_string(),
        PlacedFeature {
            feature: Feature::Named("birch_tall".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(10i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(11i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "blackstone_blobs".to_string(),
        PlacedFeature {
            feature: Feature::Named("blackstone_blobs".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(25i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 0i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "blue_ice".to_string(),
        PlacedFeature {
            feature: Feature::Named("blue_ice".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                        min_inclusive: 0i32,
                        max_inclusive: 19i32,
                    })),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 30i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 61i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "brown_mushroom_nether".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_brown_mushroom".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 2u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 0i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "brown_mushroom_normal".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_brown_mushroom".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 256u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "brown_mushroom_old_growth".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_brown_mushroom".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(3i32),
                }),
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 4u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "brown_mushroom_swamp".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_brown_mushroom".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "brown_mushroom_taiga".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_brown_mushroom".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 4u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "cave_vines".to_string(),
        PlacedFeature {
            feature: Feature::Named("cave_vine".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(188i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 256i16 }),
                    }),
                }),
                PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier {
                    direction_of_search: BlockDirection::Up,
                    target_condition: BlockPredicate::HasSturdyFace(HasSturdyFacePredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        direction: BlockDirection::Down,
                    }),
                    allowed_search_condition: Some(BlockPredicate::MatchingBlocks(
                        MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        },
                    )),
                    max_steps: 12i32,
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Constant(0i32),
                    y_spread: IntProvider::Constant(-1i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "cherry_bees_005".to_string(),
        PlacedFeature {
            feature: Feature::Named("cherry_bees_005".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::CHERRY_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "cherry_checked".to_string(),
        PlacedFeature {
            feature: Feature::Named("cherry".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::CHERRY_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "chorus_plant".to_string(),
        PlacedFeature {
            feature: Feature::Named("chorus_plant".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                        min_inclusive: 0i32,
                        max_inclusive: 4i32,
                    })),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "classic_vines_cave_feature".to_string(),
        PlacedFeature {
            feature: Feature::Named("vines".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(256i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 256i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "crimson_forest_vegetation".to_string(),
        PlacedFeature {
            feature: Feature::Named("crimson_forest_vegetation".to_string()),
            placement: vec![
                PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier {
                    count: IntProvider::Constant(6i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "crimson_fungi".to_string(),
        PlacedFeature {
            feature: Feature::Named("crimson_fungus".to_string()),
            placement: vec![
                PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier {
                    count: IntProvider::Constant(8i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "dark_forest_vegetation".to_string(),
        PlacedFeature {
            feature: Feature::Named("dark_forest_vegetation".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(16i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "dark_oak_checked".to_string(),
        PlacedFeature {
            feature: Feature::Named("dark_oak".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::DARK_OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "dark_oak_leaf_litter".to_string(),
        PlacedFeature {
            feature: Feature::Named("dark_oak_leaf_litter".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::DARK_OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "delta".to_string(),
        PlacedFeature {
            feature: Feature::Named("delta".to_string()),
            placement: vec![
                PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier {
                    count: IntProvider::Constant(40i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "desert_well".to_string(),
        PlacedFeature {
            feature: Feature::Named("desert_well".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 1000u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "disk_clay".to_string(),
        PlacedFeature {
            feature: Feature::Named("disk_clay".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloorWg,
                }),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingFluids(MatchingFluidsBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        fluids: MatchingBlocksWrapper::Single("minecraft:water".to_string()),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "disk_grass".to_string(),
        PlacedFeature {
            feature: Feature::Named("disk_grass".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(1i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloorWg,
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Constant(0i32),
                    y_spread: IntProvider::Constant(-1i32),
                }),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        blocks: MatchingBlocksWrapper::Single("minecraft:mud".to_string()),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "disk_gravel".to_string(),
        PlacedFeature {
            feature: Feature::Named("disk_gravel".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloorWg,
                }),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingFluids(MatchingFluidsBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        fluids: MatchingBlocksWrapper::Single("minecraft:water".to_string()),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "disk_sand".to_string(),
        PlacedFeature {
            feature: Feature::Named("disk_sand".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(3i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloorWg,
                }),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingFluids(MatchingFluidsBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        fluids: MatchingBlocksWrapper::Single("minecraft:water".to_string()),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "dripstone_cluster".to_string(),
        PlacedFeature {
            feature: Feature::Named("dripstone_cluster".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                        min_inclusive: 48i32,
                        max_inclusive: 96i32,
                    })),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 256i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "end_gateway_return".to_string(),
        PlacedFeature {
            feature: Feature::Named("end_gateway_return".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 700u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Constant(0i32),
                    y_spread: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                        min_inclusive: 3i32,
                        max_inclusive: 9i32,
                    })),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "end_island_decorated".to_string(),
        PlacedFeature {
            feature: Feature::Named("end_island".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 14u32 }),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(1i32),
                                    weight: 3i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(2i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 55i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 70i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "end_platform".to_string(),
        PlacedFeature {
            feature: Feature::Named("end_platform".to_string()),
            placement: vec![
                PlacementModifier::FixedPlacement,
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "end_spike".to_string(),
        PlacedFeature {
            feature: Feature::Named("end_spike".to_string()),
            placement: vec![PlacementModifier::Biome(BiomePlacementModifier)],
        },
    );
    map.insert(
        "fallen_birch_tree".to_string(),
        PlacedFeature {
            feature: Feature::Named("fallen_birch_tree".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::BIRCH_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "fallen_jungle_tree".to_string(),
        PlacedFeature {
            feature: Feature::Named("fallen_jungle_tree".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::JUNGLE_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "fallen_oak_tree".to_string(),
        PlacedFeature {
            feature: Feature::Named("fallen_oak_tree".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "fallen_spruce_tree".to_string(),
        PlacedFeature {
            feature: Feature::Named("fallen_spruce_tree".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::SPRUCE_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "fallen_super_birch_tree".to_string(),
        PlacedFeature {
            feature: Feature::Named("fallen_super_birch_tree".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::BIRCH_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "fancy_oak_bees".to_string(),
        PlacedFeature {
            feature: Feature::Named("fancy_oak_bees".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "fancy_oak_bees_0002_leaf_litter".to_string(),
        PlacedFeature {
            feature: Feature::Named("fancy_oak_bees_0002_leaf_litter".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "fancy_oak_bees_002".to_string(),
        PlacedFeature {
            feature: Feature::Named("fancy_oak_bees_002".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "fancy_oak_checked".to_string(),
        PlacedFeature {
            feature: Feature::Named("fancy_oak".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "fancy_oak_leaf_litter".to_string(),
        PlacedFeature {
            feature: Feature::Named("fancy_oak_leaf_litter".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "flower_cherry".to_string(),
        PlacedFeature {
            feature: Feature::Named("flower_cherry".to_string()),
            placement: vec![
                PlacementModifier::NoiseThresholdCount(NoiseThresholdCountPlacementModifier {
                    noise_level: -0.8f64,
                    below_noise: 5i32,
                    above_noise: 10i32,
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "flower_default".to_string(),
        PlacedFeature {
            feature: Feature::Named("flower_default".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 32u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "flower_flower_forest".to_string(),
        PlacedFeature {
            feature: Feature::Named("flower_flower_forest".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(3i32),
                }),
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 2u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "flower_forest_flowers".to_string(),
        PlacedFeature {
            feature: Feature::Named("forest_flowers".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 7u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::Clamped(ClampedIntProvider {
                        source: Box::new(IntProvider::Object(NormalIntProvider::Uniform(
                            UniformIntProvider {
                                min_inclusive: -1i32,
                                max_inclusive: 3i32,
                            },
                        ))),
                        min_inclusive: 0i32,
                        max_inclusive: 3i32,
                    })),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "flower_meadow".to_string(),
        PlacedFeature {
            feature: Feature::Named("flower_meadow".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "flower_pale_garden".to_string(),
        PlacedFeature {
            feature: Feature::Named("flower_pale_garden".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 32u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "flower_plain".to_string(),
        PlacedFeature {
            feature: Feature::Named("flower_plain".to_string()),
            placement: vec![],
        },
    );
    map.insert(
        "flower_plains".to_string(),
        PlacedFeature {
            feature: Feature::Named("flower_plain".to_string()),
            placement: vec![
                PlacementModifier::NoiseThresholdCount(NoiseThresholdCountPlacementModifier {
                    noise_level: -0.8f64,
                    below_noise: 15i32,
                    above_noise: 4i32,
                }),
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 32u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "flower_swamp".to_string(),
        PlacedFeature {
            feature: Feature::Named("flower_swamp".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 32u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "flower_warm".to_string(),
        PlacedFeature {
            feature: Feature::Named("flower_default".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 16u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "forest_flowers".to_string(),
        PlacedFeature {
            feature: Feature::Named("forest_flowers".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 7u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::Clamped(ClampedIntProvider {
                        source: Box::new(IntProvider::Object(NormalIntProvider::Uniform(
                            UniformIntProvider {
                                min_inclusive: -3i32,
                                max_inclusive: 1i32,
                            },
                        ))),
                        min_inclusive: 0i32,
                        max_inclusive: 1i32,
                    })),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "forest_rock".to_string(),
        PlacedFeature {
            feature: Feature::Named("forest_rock".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "fossil_lower".to_string(),
        PlacedFeature {
            feature: Feature::Named("fossil_diamonds".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 64u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: -8i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "fossil_upper".to_string(),
        PlacedFeature {
            feature: Feature::Named("fossil_coal".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 64u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 0i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "freeze_top_layer".to_string(),
        PlacedFeature {
            feature: Feature::Named("freeze_top_layer".to_string()),
            placement: vec![PlacementModifier::Biome(BiomePlacementModifier)],
        },
    );
    map.insert(
        "glow_lichen".to_string(),
        PlacedFeature {
            feature: Feature::Named("glow_lichen".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                        min_inclusive: 104i32,
                        max_inclusive: 157i32,
                    })),
                }),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 256i16 }),
                    }),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceRelativeThresholdFilter(
                    SurfaceThresholdFilterPlacementModifier {
                        heightmap: HeightMap::OceanFloorWg,
                        min_inclusive: None,
                        max_inclusive: Some(-13i32),
                    },
                ),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "glowstone".to_string(),
        PlacedFeature {
            feature: Feature::Named("glowstone_extra".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(10i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 0i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "glowstone_extra".to_string(),
        PlacedFeature {
            feature: Feature::Named("glowstone_extra".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::BiasedToBottom(
                        BiasedToBottomIntProvider {
                            min_inclusive: 0i32,
                            max_inclusive: 9i32,
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 4i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 4i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "grass_bonemeal".to_string(),
        PlacedFeature {
            feature: Feature::Named("single_piece_of_grass".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                    }),
                },
            )],
        },
    );
    map.insert(
        "ice_patch".to_string(),
        PlacedFeature {
            feature: Feature::Named("ice_patch".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Constant(0i32),
                    y_spread: IntProvider::Constant(-1i32),
                }),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        blocks: MatchingBlocksWrapper::Single("minecraft:snow_block".to_string()),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ice_spike".to_string(),
        PlacedFeature {
            feature: Feature::Named("ice_spike".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(3i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "iceberg_blue".to_string(),
        PlacedFeature {
            feature: Feature::Named("iceberg_blue".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 200u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "iceberg_packed".to_string(),
        PlacedFeature {
            feature: Feature::Named("iceberg_packed".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 16u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "jungle_bush".to_string(),
        PlacedFeature {
            feature: Feature::Named("jungle_bush".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "jungle_tree".to_string(),
        PlacedFeature {
            feature: Feature::Named("jungle_tree".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::JUNGLE_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "kelp_cold".to_string(),
        PlacedFeature {
            feature: Feature::Named("kelp".to_string()),
            placement: vec![
                PlacementModifier::NoiseBasedCount(NoiseBasedCountPlacementModifier {
                    noise_to_count_ratio: 120i32,
                    noise_factor: 80f64,
                    noise_offset: 0f64,
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloorWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "kelp_warm".to_string(),
        PlacedFeature {
            feature: Feature::Named("kelp".to_string()),
            placement: vec![
                PlacementModifier::NoiseBasedCount(NoiseBasedCountPlacementModifier {
                    noise_to_count_ratio: 80i32,
                    noise_factor: 80f64,
                    noise_offset: 0f64,
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloorWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "lake_lava_surface".to_string(),
        PlacedFeature {
            feature: Feature::Named("lake_lava".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 200u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "lake_lava_underground".to_string(),
        PlacedFeature {
            feature: Feature::Named("lake_lava".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 9u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 0i8 }),
                    }),
                }),
                PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier {
                    direction_of_search: BlockDirection::Down,
                    target_condition: BlockPredicate::AllOf(AllOfBlockPredicate {
                        predicates: vec![
                            BlockPredicate::Not(NotBlockPredicate {
                                predicate: Box::new(BlockPredicate::MatchingBlocks(
                                    MatchingBlocksBlockPredicate {
                                        offset: OffsetBlocksBlockPredicate { offset: None },
                                        blocks: MatchingBlocksWrapper::Single(
                                            "minecraft:air".to_string(),
                                        ),
                                    },
                                )),
                            }),
                            BlockPredicate::InsideWorldBounds(InsideWorldBoundsBlockPredicate {
                                offset: Vector3::new(0i32, -5i32, 0i32),
                            }),
                        ],
                    }),
                    allowed_search_condition: None,
                    max_steps: 32i32,
                }),
                PlacementModifier::SurfaceRelativeThresholdFilter(
                    SurfaceThresholdFilterPlacementModifier {
                        heightmap: HeightMap::OceanFloorWg,
                        min_inclusive: None,
                        max_inclusive: Some(-5i32),
                    },
                ),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "large_basalt_columns".to_string(),
        PlacedFeature {
            feature: Feature::Named("large_basalt_columns".to_string()),
            placement: vec![
                PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "large_dripstone".to_string(),
        PlacedFeature {
            feature: Feature::Named("large_dripstone".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                        min_inclusive: 10i32,
                        max_inclusive: 48i32,
                    })),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 256i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "lush_caves_ceiling_vegetation".to_string(),
        PlacedFeature {
            feature: Feature::Named("moss_patch_ceiling".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(125i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 256i16 }),
                    }),
                }),
                PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier {
                    direction_of_search: BlockDirection::Up,
                    target_condition: BlockPredicate::Solid(SolidBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                    }),
                    allowed_search_condition: Some(BlockPredicate::MatchingBlocks(
                        MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        },
                    )),
                    max_steps: 12i32,
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Constant(0i32),
                    y_spread: IntProvider::Constant(-1i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "lush_caves_clay".to_string(),
        PlacedFeature {
            feature: Feature::Named("lush_caves_clay".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(62i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 256i16 }),
                    }),
                }),
                PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier {
                    direction_of_search: BlockDirection::Down,
                    target_condition: BlockPredicate::Solid(SolidBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                    }),
                    allowed_search_condition: Some(BlockPredicate::MatchingBlocks(
                        MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        },
                    )),
                    max_steps: 12i32,
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Constant(0i32),
                    y_spread: IntProvider::Constant(1i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "lush_caves_vegetation".to_string(),
        PlacedFeature {
            feature: Feature::Named("moss_patch".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(125i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 256i16 }),
                    }),
                }),
                PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier {
                    direction_of_search: BlockDirection::Down,
                    target_condition: BlockPredicate::Solid(SolidBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                    }),
                    allowed_search_condition: Some(BlockPredicate::MatchingBlocks(
                        MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        },
                    )),
                    max_steps: 12i32,
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Constant(0i32),
                    y_spread: IntProvider::Constant(1i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "mangrove_checked".to_string(),
        PlacedFeature {
            feature: Feature::Named("mangrove".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("age".to_string(), "0".to_string());
                            props.insert("hanging".to_string(), "false".to_string());
                            props.insert("stage".to_string(), "0".to_string());
                            props.insert("waterlogged".to_string(), "false".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::MANGROVE_PROPAGULE,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "mega_jungle_tree_checked".to_string(),
        PlacedFeature {
            feature: Feature::Named("mega_jungle_tree".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::JUNGLE_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "mega_pine_checked".to_string(),
        PlacedFeature {
            feature: Feature::Named("mega_pine".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::SPRUCE_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "mega_spruce_checked".to_string(),
        PlacedFeature {
            feature: Feature::Named("mega_spruce".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::SPRUCE_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "monster_room".to_string(),
        PlacedFeature {
            feature: Feature::Named("monster_room".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(10i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 0i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "monster_room_deep".to_string(),
        PlacedFeature {
            feature: Feature::Named("monster_room".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(4i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 6i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: -1i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "mushroom_island_vegetation".to_string(),
        PlacedFeature {
            feature: Feature::Named("mushroom_island_vegetation".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "nether_sprouts".to_string(),
        PlacedFeature {
            feature: Feature::Named("nether_sprouts".to_string()),
            placement: vec![
                PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier {
                    count: IntProvider::Constant(4i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "oak".to_string(),
        PlacedFeature {
            feature: Feature::Named("oak".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "oak_bees_0002_leaf_litter".to_string(),
        PlacedFeature {
            feature: Feature::Named("oak_bees_0002_leaf_litter".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "oak_bees_002".to_string(),
        PlacedFeature {
            feature: Feature::Named("oak_bees_002".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "oak_checked".to_string(),
        PlacedFeature {
            feature: Feature::Named("oak".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "oak_leaf_litter".to_string(),
        PlacedFeature {
            feature: Feature::Named("oak_leaf_litter".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "ore_ancient_debris_large".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_ancient_debris_large".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Trapezoid(TrapezoidHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 8i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 24i16 }),
                        plateau: None,
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_andesite_lower".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_andesite".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 60i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_andesite_upper".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_andesite".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 6u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 64i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 128i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_blackstone".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_blackstone".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 5i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 31i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_clay".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_clay".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(46i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 256i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_coal_lower".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_coal_buried".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(20i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Trapezoid(TrapezoidHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 192i16 }),
                        plateau: None,
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_coal_upper".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_coal".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(30i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 136i16 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 0i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_copper".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_copper_small".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(16i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Trapezoid(TrapezoidHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: -16i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 112i16 }),
                        plateau: None,
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_copper_large".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_copper_large".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(16i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Trapezoid(TrapezoidHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: -16i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 112i16 }),
                        plateau: None,
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_debris_small".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_ancient_debris_small".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 8i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 8i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_diamond".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_diamond_small".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(7i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Trapezoid(TrapezoidHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom {
                            above_bottom: -80i8,
                        }),
                        max_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 80i8 }),
                        plateau: None,
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_diamond_buried".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_diamond_buried".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(4i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Trapezoid(TrapezoidHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom {
                            above_bottom: -80i8,
                        }),
                        max_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 80i8 }),
                        plateau: None,
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_diamond_large".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_diamond_large".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 9u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Trapezoid(TrapezoidHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom {
                            above_bottom: -80i8,
                        }),
                        max_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 80i8 }),
                        plateau: None,
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_diamond_medium".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_diamond_medium".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: -64i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: -4i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_diorite_lower".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_diorite".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 60i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_diorite_upper".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_diorite".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 6u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 64i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 128i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_dirt".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_dirt".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(7i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 160i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_emerald".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_emerald".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(100i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Trapezoid(TrapezoidHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: -16i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 480i16 }),
                        plateau: None,
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_gold".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_gold_buried".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(4i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Trapezoid(TrapezoidHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: -64i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 32i16 }),
                        plateau: None,
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_gold_deltas".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_nether_gold".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(20i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 10i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 10i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_gold_extra".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_gold".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(50i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 32i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 256i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_gold_lower".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_gold_buried".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                        min_inclusive: 0i32,
                        max_inclusive: 1i32,
                    })),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: -64i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: -48i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_gold_nether".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_nether_gold".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(10i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 10i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 10i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_granite_lower".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_granite".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 60i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_granite_upper".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_granite".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 6u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 64i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 128i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_gravel".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_gravel".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(14i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 0i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_gravel_nether".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_gravel_nether".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 5i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 41i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_infested".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_infested".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(14i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 63i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_iron_middle".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_iron".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(10i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Trapezoid(TrapezoidHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: -24i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 56i16 }),
                        plateau: None,
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_iron_small".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_iron_small".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(10i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 72i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_iron_upper".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_iron".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(90i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Trapezoid(TrapezoidHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 80i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 384i16 }),
                        plateau: None,
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_lapis".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_lapis".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Trapezoid(TrapezoidHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: -32i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 32i16 }),
                        plateau: None,
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_lapis_buried".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_lapis_buried".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(4i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 64i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_magma".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_magma".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(4i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 27i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 36i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_quartz_deltas".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_quartz".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(32i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 10i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 10i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_quartz_nether".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_quartz".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(16i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 10i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 10i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_redstone".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_redstone".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(4i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 15i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_redstone_lower".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_redstone".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(8i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Trapezoid(TrapezoidHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom {
                            above_bottom: -32i8,
                        }),
                        max_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 32i8 }),
                        plateau: None,
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_soul_sand".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_soul_sand".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(12i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 31i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "ore_tuff".to_string(),
        PlacedFeature {
            feature: Feature::Named("ore_tuff".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 0i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "pale_garden_flowers".to_string(),
        PlacedFeature {
            feature: Feature::Named("pale_forest_flowers".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 8u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlockingNoLeaves,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "pale_garden_vegetation".to_string(),
        PlacedFeature {
            feature: Feature::Named("pale_garden_vegetation".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(16i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "pale_moss_patch".to_string(),
        PlacedFeature {
            feature: Feature::Named("pale_moss_patch".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(1i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlockingNoLeaves,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "pale_oak_checked".to_string(),
        PlacedFeature {
            feature: Feature::Named("pale_oak".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PALE_OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "pale_oak_creaking_checked".to_string(),
        PlacedFeature {
            feature: Feature::Named("pale_oak_creaking".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::PALE_OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "patch_berry_bush".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_berry_bush".to_string()),
            placement: vec![],
        },
    );
    map.insert(
        "patch_berry_common".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_berry_bush".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 32u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_berry_rare".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_berry_bush".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 384u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_bush".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_bush".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 4u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_cactus".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_cactus".to_string()),
            placement: vec![],
        },
    );
    map.insert(
        "patch_cactus_decorated".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_cactus".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 13u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_cactus_desert".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_cactus".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 6u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_crimson_roots".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_crimson_roots".to_string()),
            placement: vec![
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 0i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_dead_bush".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_dead_bush".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_dead_bush_2".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_dead_bush".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_dead_bush_badlands".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_dead_bush".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(20i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_dry_grass_badlands".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_dry_grass".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 6u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_dry_grass_desert".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_dry_grass".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 3u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_fire".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_fire".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                        min_inclusive: 0i32,
                        max_inclusive: 5i32,
                    })),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 4i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 4i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_firefly_bush_near_water".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_firefly_bush".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlockingNoLeaves,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                        predicates: vec![
                            BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                                blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                            }),
                            BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                                state: BlockStateCodec {
                                    name: &pumpkin_data::Block::FIREFLY_BUSH,
                                    properties: None,
                                },
                            }),
                            BlockPredicate::AnyOf(AnyOfBlockPredicate {
                                predicates: vec![
                                    BlockPredicate::MatchingFluids(MatchingFluidsBlockPredicate {
                                        offset: OffsetBlocksBlockPredicate {
                                            offset: Some(Vector3::new(1i32, -1i32, 0i32)),
                                        },
                                        fluids: MatchingBlocksWrapper::Multiple(vec![
                                            "minecraft:water".to_string(),
                                            "minecraft:flowing_water".to_string(),
                                        ]),
                                    }),
                                    BlockPredicate::MatchingFluids(MatchingFluidsBlockPredicate {
                                        offset: OffsetBlocksBlockPredicate {
                                            offset: Some(Vector3::new(-1i32, -1i32, 0i32)),
                                        },
                                        fluids: MatchingBlocksWrapper::Multiple(vec![
                                            "minecraft:water".to_string(),
                                            "minecraft:flowing_water".to_string(),
                                        ]),
                                    }),
                                    BlockPredicate::MatchingFluids(MatchingFluidsBlockPredicate {
                                        offset: OffsetBlocksBlockPredicate {
                                            offset: Some(Vector3::new(0i32, -1i32, 1i32)),
                                        },
                                        fluids: MatchingBlocksWrapper::Multiple(vec![
                                            "minecraft:water".to_string(),
                                            "minecraft:flowing_water".to_string(),
                                        ]),
                                    }),
                                    BlockPredicate::MatchingFluids(MatchingFluidsBlockPredicate {
                                        offset: OffsetBlocksBlockPredicate {
                                            offset: Some(Vector3::new(0i32, -1i32, -1i32)),
                                        },
                                        fluids: MatchingBlocksWrapper::Multiple(vec![
                                            "minecraft:water".to_string(),
                                            "minecraft:flowing_water".to_string(),
                                        ]),
                                    }),
                                ],
                            }),
                        ],
                    }),
                }),
            ],
        },
    );
    map.insert(
        "patch_firefly_bush_near_water_swamp".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_firefly_bush".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(3i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                        predicates: vec![
                            BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                                blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                            }),
                            BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                                state: BlockStateCodec {
                                    name: &pumpkin_data::Block::FIREFLY_BUSH,
                                    properties: None,
                                },
                            }),
                            BlockPredicate::AnyOf(AnyOfBlockPredicate {
                                predicates: vec![
                                    BlockPredicate::MatchingFluids(MatchingFluidsBlockPredicate {
                                        offset: OffsetBlocksBlockPredicate {
                                            offset: Some(Vector3::new(1i32, -1i32, 0i32)),
                                        },
                                        fluids: MatchingBlocksWrapper::Multiple(vec![
                                            "minecraft:water".to_string(),
                                            "minecraft:flowing_water".to_string(),
                                        ]),
                                    }),
                                    BlockPredicate::MatchingFluids(MatchingFluidsBlockPredicate {
                                        offset: OffsetBlocksBlockPredicate {
                                            offset: Some(Vector3::new(-1i32, -1i32, 0i32)),
                                        },
                                        fluids: MatchingBlocksWrapper::Multiple(vec![
                                            "minecraft:water".to_string(),
                                            "minecraft:flowing_water".to_string(),
                                        ]),
                                    }),
                                    BlockPredicate::MatchingFluids(MatchingFluidsBlockPredicate {
                                        offset: OffsetBlocksBlockPredicate {
                                            offset: Some(Vector3::new(0i32, -1i32, 1i32)),
                                        },
                                        fluids: MatchingBlocksWrapper::Multiple(vec![
                                            "minecraft:water".to_string(),
                                            "minecraft:flowing_water".to_string(),
                                        ]),
                                    }),
                                    BlockPredicate::MatchingFluids(MatchingFluidsBlockPredicate {
                                        offset: OffsetBlocksBlockPredicate {
                                            offset: Some(Vector3::new(0i32, -1i32, -1i32)),
                                        },
                                        fluids: MatchingBlocksWrapper::Multiple(vec![
                                            "minecraft:water".to_string(),
                                            "minecraft:flowing_water".to_string(),
                                        ]),
                                    }),
                                ],
                            }),
                        ],
                    }),
                }),
            ],
        },
    );
    map.insert(
        "patch_firefly_bush_swamp".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_firefly_bush".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 8u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_grass_badlands".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_grass".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_grass_forest".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_grass".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_grass_jungle".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_grass_jungle".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(25i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_grass_meadow".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_grass_meadow".to_string()),
            placement: vec![
                PlacementModifier::NoiseThresholdCount(NoiseThresholdCountPlacementModifier {
                    noise_level: -0.8f64,
                    below_noise: 5i32,
                    above_noise: 10i32,
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_grass_normal".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_grass".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(5i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_grass_plain".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_grass".to_string()),
            placement: vec![
                PlacementModifier::NoiseThresholdCount(NoiseThresholdCountPlacementModifier {
                    noise_level: -0.8f64,
                    below_noise: 5i32,
                    above_noise: 10i32,
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_grass_savanna".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_grass".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(20i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_grass_taiga".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_taiga_grass".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(7i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_grass_taiga_2".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_taiga_grass".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_large_fern".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_large_fern".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 5u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_leaf_litter".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_leaf_litter".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_melon".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_melon".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 6u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_melon_sparse".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_melon".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 64u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_pumpkin".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_pumpkin".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 300u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_soul_fire".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_soul_fire".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                        min_inclusive: 0i32,
                        max_inclusive: 5i32,
                    })),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 4i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 4i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_sugar_cane".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_sugar_cane".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 6u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_sugar_cane_badlands".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_sugar_cane".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 5u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_sugar_cane_desert".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_sugar_cane".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_sugar_cane_swamp".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_sugar_cane".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 3u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_sunflower".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_sunflower".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 3u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_taiga_grass".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_taiga_grass".to_string()),
            placement: vec![],
        },
    );
    map.insert(
        "patch_tall_grass".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_tall_grass".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 5u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_tall_grass_2".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_tall_grass".to_string()),
            placement: vec![
                PlacementModifier::NoiseThresholdCount(NoiseThresholdCountPlacementModifier {
                    noise_level: -0.8f64,
                    below_noise: 0i32,
                    above_noise: 7i32,
                }),
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 32u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "patch_waterlily".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_waterlily".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(4i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "pile_hay".to_string(),
        PlacedFeature {
            feature: Feature::Named("pile_hay".to_string()),
            placement: vec![],
        },
    );
    map.insert(
        "pile_ice".to_string(),
        PlacedFeature {
            feature: Feature::Named("pile_ice".to_string()),
            placement: vec![],
        },
    );
    map.insert(
        "pile_melon".to_string(),
        PlacedFeature {
            feature: Feature::Named("pile_melon".to_string()),
            placement: vec![],
        },
    );
    map.insert(
        "pile_pumpkin".to_string(),
        PlacedFeature {
            feature: Feature::Named("pile_pumpkin".to_string()),
            placement: vec![],
        },
    );
    map.insert(
        "pile_snow".to_string(),
        PlacedFeature {
            feature: Feature::Named("pile_snow".to_string()),
            placement: vec![],
        },
    );
    map.insert(
        "pine".to_string(),
        PlacedFeature {
            feature: Feature::Named("pine".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::SPRUCE_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "pine_checked".to_string(),
        PlacedFeature {
            feature: Feature::Named("pine".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::SPRUCE_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "pine_on_snow".to_string(),
        PlacedFeature {
            feature: Feature::Named("pine".to_string()),
            placement: vec![
                PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier {
                    direction_of_search: BlockDirection::Up,
                    target_condition: BlockPredicate::Not(NotBlockPredicate {
                        predicate: Box::new(BlockPredicate::MatchingBlocks(
                            MatchingBlocksBlockPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                                blocks: MatchingBlocksWrapper::Single(
                                    "minecraft:powder_snow".to_string(),
                                ),
                            },
                        )),
                    }),
                    allowed_search_condition: None,
                    max_steps: 8i32,
                }),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                        offset: OffsetBlocksBlockPredicate {
                            offset: Some(Vector3::new(0i32, -1i32, 0i32)),
                        },
                        blocks: MatchingBlocksWrapper::Multiple(vec![
                            "minecraft:snow_block".to_string(),
                            "minecraft:powder_snow".to_string(),
                        ]),
                    }),
                }),
            ],
        },
    );
    map.insert(
        "pointed_dripstone".to_string(),
        PlacedFeature {
            feature: Feature::Named("pointed_dripstone".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                        min_inclusive: 192i32,
                        max_inclusive: 256i32,
                    })),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 256i16 }),
                    }),
                }),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                        min_inclusive: 1i32,
                        max_inclusive: 5i32,
                    })),
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Object(NormalIntProvider::ClampedNormal(
                        ClampedNormalIntProvider {
                            mean: 0f32,
                            deviation: 3f32,
                            min_inclusive: -10i32,
                            max_inclusive: 10i32,
                        },
                    )),
                    y_spread: IntProvider::Object(NormalIntProvider::ClampedNormal(
                        ClampedNormalIntProvider {
                            mean: 0f32,
                            deviation: 0.6f32,
                            min_inclusive: -2i32,
                            max_inclusive: 2i32,
                        },
                    )),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "red_mushroom_nether".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_red_mushroom".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 2u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 0i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "red_mushroom_normal".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_red_mushroom".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 512u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "red_mushroom_old_growth".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_red_mushroom".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 171u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "red_mushroom_swamp".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_red_mushroom".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 64u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "red_mushroom_taiga".to_string(),
        PlacedFeature {
            feature: Feature::Named("patch_red_mushroom".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 256u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "rooted_azalea_tree".to_string(),
        PlacedFeature {
            feature: Feature::Named("rooted_azalea_tree".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                        min_inclusive: 1i32,
                        max_inclusive: 2i32,
                    })),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 256i16 }),
                    }),
                }),
                PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier {
                    direction_of_search: BlockDirection::Up,
                    target_condition: BlockPredicate::Solid(SolidBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                    }),
                    allowed_search_condition: Some(BlockPredicate::MatchingBlocks(
                        MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        },
                    )),
                    max_steps: 12i32,
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Constant(0i32),
                    y_spread: IntProvider::Constant(-1i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "sculk_patch_ancient_city".to_string(),
        PlacedFeature {
            feature: Feature::Named("sculk_patch_ancient_city".to_string()),
            placement: vec![],
        },
    );
    map.insert(
        "sculk_patch_deep_dark".to_string(),
        PlacedFeature {
            feature: Feature::Named("sculk_patch_deep_dark".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(256i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 256i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "sculk_vein".to_string(),
        PlacedFeature {
            feature: Feature::Named("sculk_vein".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                        min_inclusive: 204i32,
                        max_inclusive: 250i32,
                    })),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 256i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "sea_pickle".to_string(),
        PlacedFeature {
            feature: Feature::Named("sea_pickle".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 16u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloorWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "seagrass_cold".to_string(),
        PlacedFeature {
            feature: Feature::Named("seagrass_short".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloorWg,
                }),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(32i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "seagrass_deep".to_string(),
        PlacedFeature {
            feature: Feature::Named("seagrass_tall".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloorWg,
                }),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(48i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "seagrass_deep_cold".to_string(),
        PlacedFeature {
            feature: Feature::Named("seagrass_tall".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloorWg,
                }),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(40i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "seagrass_deep_warm".to_string(),
        PlacedFeature {
            feature: Feature::Named("seagrass_tall".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloorWg,
                }),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(80i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "seagrass_normal".to_string(),
        PlacedFeature {
            feature: Feature::Named("seagrass_short".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloorWg,
                }),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(48i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "seagrass_river".to_string(),
        PlacedFeature {
            feature: Feature::Named("seagrass_slightly_less_short".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloorWg,
                }),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(48i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "seagrass_swamp".to_string(),
        PlacedFeature {
            feature: Feature::Named("seagrass_mid".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloorWg,
                }),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(64i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "seagrass_warm".to_string(),
        PlacedFeature {
            feature: Feature::Named("seagrass_short".to_string()),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloorWg,
                }),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(80i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "small_basalt_columns".to_string(),
        PlacedFeature {
            feature: Feature::Named("small_basalt_columns".to_string()),
            placement: vec![
                PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier {
                    count: IntProvider::Constant(4i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "spore_blossom".to_string(),
        PlacedFeature {
            feature: Feature::Named("spore_blossom".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(25i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 256i16 }),
                    }),
                }),
                PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier {
                    direction_of_search: BlockDirection::Up,
                    target_condition: BlockPredicate::Solid(SolidBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                    }),
                    allowed_search_condition: Some(BlockPredicate::MatchingBlocks(
                        MatchingBlocksBlockPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            blocks: MatchingBlocksWrapper::Single("minecraft:air".to_string()),
                        },
                    )),
                    max_steps: 12i32,
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Constant(0i32),
                    y_spread: IntProvider::Constant(-1i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "spring_closed".to_string(),
        PlacedFeature {
            feature: Feature::Named("spring_nether_closed".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(16i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 10i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 10i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "spring_closed_double".to_string(),
        PlacedFeature {
            feature: Feature::Named("spring_nether_closed".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(32i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 10i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 10i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "spring_delta".to_string(),
        PlacedFeature {
            feature: Feature::Named("spring_lava_nether".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(16i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 4i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 4i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "spring_lava".to_string(),
        PlacedFeature {
            feature: Feature::Named("spring_lava_overworld".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(20i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::VeryBiasedToBottom(VeryBiasedToBottomHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 8i8 }),
                        inner: std::num::NonZeroU32::new(8u32),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "spring_lava_frozen".to_string(),
        PlacedFeature {
            feature: Feature::Named("spring_lava_frozen".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(20i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::VeryBiasedToBottom(VeryBiasedToBottomHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 8i8 }),
                        inner: std::num::NonZeroU32::new(8u32),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "spring_open".to_string(),
        PlacedFeature {
            feature: Feature::Named("spring_nether_open".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(8i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 4i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 4i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "spring_water".to_string(),
        PlacedFeature {
            feature: Feature::Named("spring_water".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(25i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 192i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "spruce".to_string(),
        PlacedFeature {
            feature: Feature::Named("spruce".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::SPRUCE_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "spruce_checked".to_string(),
        PlacedFeature {
            feature: Feature::Named("spruce".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::SPRUCE_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "spruce_on_snow".to_string(),
        PlacedFeature {
            feature: Feature::Named("spruce".to_string()),
            placement: vec![
                PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier {
                    direction_of_search: BlockDirection::Up,
                    target_condition: BlockPredicate::Not(NotBlockPredicate {
                        predicate: Box::new(BlockPredicate::MatchingBlocks(
                            MatchingBlocksBlockPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                                blocks: MatchingBlocksWrapper::Single(
                                    "minecraft:powder_snow".to_string(),
                                ),
                            },
                        )),
                    }),
                    allowed_search_condition: None,
                    max_steps: 8i32,
                }),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                        offset: OffsetBlocksBlockPredicate {
                            offset: Some(Vector3::new(0i32, -1i32, 0i32)),
                        },
                        blocks: MatchingBlocksWrapper::Multiple(vec![
                            "minecraft:snow_block".to_string(),
                            "minecraft:powder_snow".to_string(),
                        ]),
                    }),
                }),
            ],
        },
    );
    map.insert(
        "super_birch_bees".to_string(),
        PlacedFeature {
            feature: Feature::Named("super_birch_bees".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::BIRCH_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "super_birch_bees_0002".to_string(),
        PlacedFeature {
            feature: Feature::Named("super_birch_bees_0002".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::BIRCH_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "tall_mangrove_checked".to_string(),
        PlacedFeature {
            feature: Feature::Named("tall_mangrove".to_string()),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("age".to_string(), "0".to_string());
                            props.insert("hanging".to_string(), "false".to_string());
                            props.insert("stage".to_string(), "0".to_string());
                            props.insert("waterlogged".to_string(), "false".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::MANGROVE_PROPAGULE,
                                properties: Some(props),
                            }
                        },
                    }),
                },
            )],
        },
    );
    map.insert(
        "trees_badlands".to_string(),
        PlacedFeature {
            feature: Feature::Named("trees_badlands".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(5i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(6i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                }),
            ],
        },
    );
    map.insert(
        "trees_birch".to_string(),
        PlacedFeature {
            feature: Feature::Named("trees_birch".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(10i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(11i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::BIRCH_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                }),
            ],
        },
    );
    map.insert(
        "trees_birch_and_oak_leaf_litter".to_string(),
        PlacedFeature {
            feature: Feature::Named("trees_birch_and_oak_leaf_litter".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(10i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(11i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "trees_cherry".to_string(),
        PlacedFeature {
            feature: Feature::Named("cherry_bees_005".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(10i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(11i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::CHERRY_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                }),
            ],
        },
    );
    map.insert(
        "trees_flower_forest".to_string(),
        PlacedFeature {
            feature: Feature::Named("trees_flower_forest".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(6i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(7i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "trees_grove".to_string(),
        PlacedFeature {
            feature: Feature::Named("trees_grove".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(10i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(11i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "trees_jungle".to_string(),
        PlacedFeature {
            feature: Feature::Named("trees_jungle".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(50i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(51i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "trees_mangrove".to_string(),
        PlacedFeature {
            feature: Feature::Named("mangrove_vegetation".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(25i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 5i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "trees_meadow".to_string(),
        PlacedFeature {
            feature: Feature::Named("meadow_trees".to_string()),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 100u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "trees_old_growth_pine_taiga".to_string(),
        PlacedFeature {
            feature: Feature::Named("trees_old_growth_pine_taiga".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(10i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(11i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "trees_old_growth_spruce_taiga".to_string(),
        PlacedFeature {
            feature: Feature::Named("trees_old_growth_spruce_taiga".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(10i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(11i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "trees_plains".to_string(),
        PlacedFeature {
            feature: Feature::Named("trees_plains".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(0i32),
                                    weight: 19i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(1i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "trees_savanna".to_string(),
        PlacedFeature {
            feature: Feature::Named("trees_savanna".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(1i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(2i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "trees_snowy".to_string(),
        PlacedFeature {
            feature: Feature::Named("trees_snowy".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(0i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(1i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::SPRUCE_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                }),
            ],
        },
    );
    map.insert(
        "trees_sparse_jungle".to_string(),
        PlacedFeature {
            feature: Feature::Named("trees_sparse_jungle".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(2i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(3i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "trees_swamp".to_string(),
        PlacedFeature {
            feature: Feature::Named("swamp_oak".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(2i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(3i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 2i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("stage".to_string(), "0".to_string());
                            BlockStateCodec {
                                name: &pumpkin_data::Block::OAK_SAPLING,
                                properties: Some(props),
                            }
                        },
                    }),
                }),
            ],
        },
    );
    map.insert(
        "trees_taiga".to_string(),
        PlacedFeature {
            feature: Feature::Named("trees_taiga".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(10i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(11i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "trees_water".to_string(),
        PlacedFeature {
            feature: Feature::Named("trees_water".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(0i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(1i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "trees_windswept_forest".to_string(),
        PlacedFeature {
            feature: Feature::Named("trees_windswept_hills".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(3i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(4i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "trees_windswept_hills".to_string(),
        PlacedFeature {
            feature: Feature::Named("trees_windswept_hills".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(0i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(1i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "trees_windswept_savanna".to_string(),
        PlacedFeature {
            feature: Feature::Named("trees_savanna".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::WeightedList(
                        WeightedListIntProvider {
                            distribution: vec![
                                WeightedEntry {
                                    data: IntProvider::Constant(2i32),
                                    weight: 9i32,
                                },
                                WeightedEntry {
                                    data: IntProvider::Constant(3i32),
                                    weight: 1i32,
                                },
                            ],
                        },
                    )),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::SurfaceWaterDepthFilter(
                    SurfaceWaterDepthFilterPlacementModifier {
                        max_water_depth: 0i32,
                    },
                ),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloor,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "twisting_vines".to_string(),
        PlacedFeature {
            feature: Feature::Named("twisting_vines".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(10i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 0i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "underwater_magma".to_string(),
        PlacedFeature {
            feature: Feature::Named("underwater_magma".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Object(NormalIntProvider::Uniform(UniformIntProvider {
                        min_inclusive: 44i32,
                        max_inclusive: 52i32,
                    })),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 256i16 }),
                    }),
                }),
                PlacementModifier::SurfaceRelativeThresholdFilter(
                    SurfaceThresholdFilterPlacementModifier {
                        heightmap: HeightMap::OceanFloorWg,
                        min_inclusive: None,
                        max_inclusive: Some(-2i32),
                    },
                ),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "vines".to_string(),
        PlacedFeature {
            feature: Feature::Named("vines".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(127i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::Absolute(Absolute { absolute: 64i16 }),
                        max_inclusive: YOffset::Absolute(Absolute { absolute: 100i16 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "void_start_platform".to_string(),
        PlacedFeature {
            feature: Feature::Named("void_start_platform".to_string()),
            placement: vec![PlacementModifier::Biome(BiomePlacementModifier)],
        },
    );
    map.insert(
        "warm_ocean_vegetation".to_string(),
        PlacedFeature {
            feature: Feature::Named("warm_ocean_vegetation".to_string()),
            placement: vec![
                PlacementModifier::NoiseBasedCount(NoiseBasedCountPlacementModifier {
                    noise_to_count_ratio: 20i32,
                    noise_factor: 400f64,
                    noise_offset: 0f64,
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::OceanFloorWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "warped_forest_vegetation".to_string(),
        PlacedFeature {
            feature: Feature::Named("warped_forest_vegetation".to_string()),
            placement: vec![
                PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier {
                    count: IntProvider::Constant(5i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "warped_fungi".to_string(),
        PlacedFeature {
            feature: Feature::Named("warped_fungus".to_string()),
            placement: vec![
                PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier {
                    count: IntProvider::Constant(8i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "weeping_vines".to_string(),
        PlacedFeature {
            feature: Feature::Named("weeping_vines".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(10i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 0i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "wildflowers_birch_forest".to_string(),
        PlacedFeature {
            feature: Feature::Named("wildflowers_birch_forest".to_string()),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(3i32),
                }),
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 2u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        "wildflowers_meadow".to_string(),
        PlacedFeature {
            feature: Feature::Named("wildflowers_meadow".to_string()),
            placement: vec![
                PlacementModifier::NoiseThresholdCount(NoiseThresholdCountPlacementModifier {
                    noise_level: -0.8f64,
                    below_noise: 5i32,
                    above_noise: 10i32,
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map
}
