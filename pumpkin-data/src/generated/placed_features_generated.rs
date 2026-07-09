/* This file is generated. Do not edit manually. */
#[allow(clippy::all, unused_imports, dead_code, clippy::too_many_lines)]
fn build_placed_features()
-> std::collections::HashMap<pumpkin_data::placed_feature::PlacedFeature, PlacedFeature> {
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
        ConstantIntProvider, IntProvider, NormalIntProvider, TrapezoidIntProvider,
        UniformIntProvider, WeightedEntry, WeightedListIntProvider,
    };
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_util::y_offset::{AboveBottom, Absolute, BelowTop, YOffset};
    let mut map = std::collections::HashMap::new();
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::Acacia,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Acacia),
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
        pumpkin_data::placed_feature::PlacedFeature::AcaciaChecked,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Acacia),
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
        pumpkin_data::placed_feature::PlacedFeature::AmethystGeode,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::AmethystGeode,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::Bamboo,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::BambooSomePodzol,
            ),
            placement: vec![
                PlacementModifier::NoiseBasedCount(NoiseBasedCountPlacementModifier {
                    to_count_ratio: 160i32,
                    factor: 80f64,
                    offset: 0.3f64,
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
        pumpkin_data::placed_feature::PlacedFeature::BambooLight,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::BambooNoPodzol,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::BambooVegetation,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::BambooVegetation,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::BasaltBlobs,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::BasaltBlobs,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::BasaltPillar,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::BasaltPillar,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::BirchBees0002,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::BirchBees0002,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::BirchBees0002LeafLitter,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::BirchBees0002LeafLitter,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::BirchBees002,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::BirchBees002,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::BirchChecked,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Birch),
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
        pumpkin_data::placed_feature::PlacedFeature::BirchLeafLitter,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::BirchLeafLitter,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::BirchTall,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::BirchTall),
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
        pumpkin_data::placed_feature::PlacedFeature::BlackstoneBlobs,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::BlackstoneBlobs,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::BlueIce,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::BlueIce),
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
        pumpkin_data::placed_feature::PlacedFeature::BrownMushroomNether,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::BrownMushroom,
            ),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::BrownMushroomNormal,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::BrownMushroom,
            ),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 256u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::BrownMushroomOldGrowth,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::BrownMushroom,
            ),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::BrownMushroomSwamp,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::BrownMushroom,
            ),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::BrownMushroomTaiga,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::BrownMushroom,
            ),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 4u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::CaveVines,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::CaveVine),
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
                    allowed_search_condition: Some(BlockPredicate::MatchingBlockTag(
                        MatchingBlockTagPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
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
        pumpkin_data::placed_feature::PlacedFeature::CherryBees005,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::CherryBees005,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::CherryChecked,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Cherry),
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
        pumpkin_data::placed_feature::PlacedFeature::ChorusPlant,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::ChorusPlant,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::ClassicVinesCaveFeature,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Vines),
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
        pumpkin_data::placed_feature::PlacedFeature::CrimsonForestVegetation,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::CrimsonForestVegetation,
            ),
            placement: vec![
                PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier {
                    count: IntProvider::Constant(6i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::CrimsonFungi,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::CrimsonFungus,
            ),
            placement: vec![
                PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier {
                    count: IntProvider::Constant(8i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::DarkForestVegetation,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::DarkForestVegetation,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::DarkOakChecked,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::DarkOak),
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
        pumpkin_data::placed_feature::PlacedFeature::DarkOakLeafLitter,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::DarkOakLeafLitter,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::Delta,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Delta),
            placement: vec![
                PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier {
                    count: IntProvider::Constant(40i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::DesertWell,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::DesertWell,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::DiskClay,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::DiskClay),
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
        pumpkin_data::placed_feature::PlacedFeature::DiskGrass,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::DiskGrass),
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
        pumpkin_data::placed_feature::PlacedFeature::DiskGravel,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::DiskGravel,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::DiskSand,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::DiskSand),
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
        pumpkin_data::placed_feature::PlacedFeature::DripstoneCluster,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::DripstoneCluster,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::EndGatewayReturn,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::EndGatewayReturn,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::EndIslandDecorated,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::EndIsland),
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
        pumpkin_data::placed_feature::PlacedFeature::EndPlatform,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::EndPlatform,
            ),
            placement: vec![
                PlacementModifier::FixedPlacement(vec![BlockPos::new(100i32, 49i32, 0i32)]),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::EndSpike,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::EndSpike),
            placement: vec![PlacementModifier::Biome(BiomePlacementModifier)],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::FallenBirchTree,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FallenBirchTree,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::FallenJungleTree,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FallenJungleTree,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::FallenOakTree,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FallenOakTree,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::FallenSpruceTree,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FallenSpruceTree,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::FallenSuperBirchTree,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FallenSuperBirchTree,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::FancyOakBees,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FancyOakBees,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::FancyOakBees0002LeafLitter,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FancyOakBees0002LeafLitter,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::FancyOakBees002,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FancyOakBees002,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::FancyOakChecked,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::FancyOak),
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
        pumpkin_data::placed_feature::PlacedFeature::FancyOakLeafLitter,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FancyOakLeafLitter,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::FlowerCherry,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FlowerCherry,
            ),
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
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(96i32),
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -6i32,
                            max_inclusive: 6i32,
                            plateau: 0i32,
                        },
                    )),
                    y_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -2i32,
                            max_inclusive: 2i32,
                            plateau: 0i32,
                        },
                    )),
                }),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::FlowerDefault,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FlowerDefault,
            ),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 32u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(64i32),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::FlowerFlowerForest,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FlowerFlowerForest,
            ),
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
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(96i32),
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -6i32,
                            max_inclusive: 6i32,
                            plateau: 0i32,
                        },
                    )),
                    y_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -2i32,
                            max_inclusive: 2i32,
                            plateau: 0i32,
                        },
                    )),
                }),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::FlowerForestFlowers,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::ForestFlowers,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::FlowerMeadow,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FlowerMeadow,
            ),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(96i32),
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -6i32,
                            max_inclusive: 6i32,
                            plateau: 0i32,
                        },
                    )),
                    y_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -2i32,
                            max_inclusive: 2i32,
                            plateau: 0i32,
                        },
                    )),
                }),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::FlowerPaleGarden,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FlowerPaleGarden,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::FlowerPlain,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FlowerPlain,
            ),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(64i32),
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -6i32,
                            max_inclusive: 6i32,
                            plateau: 0i32,
                        },
                    )),
                    y_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -2i32,
                            max_inclusive: 2i32,
                            plateau: 0i32,
                        },
                    )),
                }),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::FlowerPlains,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FlowerPlain,
            ),
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
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(64i32),
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -6i32,
                            max_inclusive: 6i32,
                            plateau: 0i32,
                        },
                    )),
                    y_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -2i32,
                            max_inclusive: 2i32,
                            plateau: 0i32,
                        },
                    )),
                }),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::FlowerSwamp,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FlowerSwamp,
            ),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 32u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(64i32),
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -6i32,
                            max_inclusive: 6i32,
                            plateau: 0i32,
                        },
                    )),
                    y_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -2i32,
                            max_inclusive: 2i32,
                            plateau: 0i32,
                        },
                    )),
                }),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::FlowerWarm,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FlowerDefault,
            ),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 16u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(64i32),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::ForestFlowers,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::ForestFlowers,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::ForestRock,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::ForestRock,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::FossilLower,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FossilDiamonds,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::FossilUpper,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FossilCoal,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::FreezeTopLayer,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FreezeTopLayer,
            ),
            placement: vec![PlacementModifier::Biome(BiomePlacementModifier)],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::GlowLichen,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::GlowLichen,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::Glowstone,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::GlowstoneExtra,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::GlowstoneExtra,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::GlowstoneExtra,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::GrassBonemeal,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Grass),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                },
            )],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::IcePatch,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::IcePatch),
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
        pumpkin_data::placed_feature::PlacedFeature::IceSpike,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::IceSpike),
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
        pumpkin_data::placed_feature::PlacedFeature::IcebergBlue,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::IcebergBlue,
            ),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 200u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::IcebergPacked,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::IcebergPacked,
            ),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 16u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::JungleBush,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::JungleBush,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::JungleTree,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::JungleTree,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::KelpCold,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Kelp),
            placement: vec![
                PlacementModifier::NoiseBasedCount(NoiseBasedCountPlacementModifier {
                    to_count_ratio: 120i32,
                    factor: 80f64,
                    offset: 0f64,
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
        pumpkin_data::placed_feature::PlacedFeature::KelpWarm,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Kelp),
            placement: vec![
                PlacementModifier::NoiseBasedCount(NoiseBasedCountPlacementModifier {
                    to_count_ratio: 80i32,
                    factor: 80f64,
                    offset: 0f64,
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
        pumpkin_data::placed_feature::PlacedFeature::LakeLavaSurface,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::LakeLava),
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
        pumpkin_data::placed_feature::PlacedFeature::LakeLavaUnderground,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::LakeLava),
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
                                predicate: Box::new(BlockPredicate::MatchingBlockTag(
                                    MatchingBlockTagPredicate {
                                        offset: OffsetBlocksBlockPredicate { offset: None },
                                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
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
        pumpkin_data::placed_feature::PlacedFeature::LargeBasaltColumns,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::LargeBasaltColumns,
            ),
            placement: vec![
                PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::LargeDripstone,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::LargeDripstone,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::LushCavesCeilingVegetation,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::MossPatchCeiling,
            ),
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
                    allowed_search_condition: Some(BlockPredicate::MatchingBlockTag(
                        MatchingBlockTagPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
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
        pumpkin_data::placed_feature::PlacedFeature::LushCavesClay,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::LushCavesClay,
            ),
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
                    allowed_search_condition: Some(BlockPredicate::MatchingBlockTag(
                        MatchingBlockTagPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
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
        pumpkin_data::placed_feature::PlacedFeature::LushCavesVegetation,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::MossPatch),
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
                    allowed_search_condition: Some(BlockPredicate::MatchingBlockTag(
                        MatchingBlockTagPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
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
        pumpkin_data::placed_feature::PlacedFeature::MangroveChecked,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Mangrove),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("waterlogged".to_string(), "false".to_string());
                            props.insert("stage".to_string(), "0".to_string());
                            props.insert("hanging".to_string(), "false".to_string());
                            props.insert("age".to_string(), "0".to_string());
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
        pumpkin_data::placed_feature::PlacedFeature::MegaJungleTreeChecked,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::MegaJungleTree,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::MegaPineChecked,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::MegaPine),
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
        pumpkin_data::placed_feature::PlacedFeature::MegaSpruceChecked,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::MegaSpruce,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::MonsterRoom,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::MonsterRoom,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::MonsterRoomDeep,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::MonsterRoom,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::MushroomIslandVegetation,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::MushroomIslandVegetation,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::NetherSprouts,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::NetherSprouts,
            ),
            placement: vec![
                PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier {
                    count: IntProvider::Constant(4i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::Oak,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Oak),
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
        pumpkin_data::placed_feature::PlacedFeature::OakBees0002LeafLitter,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OakBees0002LeafLitter,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OakBees002,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OakBees002,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OakChecked,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Oak),
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
        pumpkin_data::placed_feature::PlacedFeature::OakLeafLitter,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OakLeafLitter,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreAncientDebrisLarge,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreAncientDebrisLarge,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreAndesiteLower,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreAndesite,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreAndesiteUpper,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreAndesite,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreBlackstone,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreBlackstone,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreClay,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::OreClay),
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
        pumpkin_data::placed_feature::PlacedFeature::OreCoalLower,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreCoalBuried,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreCoalUpper,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::OreCoal),
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
        pumpkin_data::placed_feature::PlacedFeature::OreCopper,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreCopperSmall,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreCopperLarge,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreCopperLarge,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreDebrisSmall,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreAncientDebrisSmall,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreDiamond,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreDiamondSmall,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreDiamondBuried,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreDiamondBuried,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreDiamondLarge,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreDiamondLarge,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreDiamondMedium,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreDiamondMedium,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreDioriteLower,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreDiorite,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreDioriteUpper,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreDiorite,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreDirt,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::OreDirt),
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
        pumpkin_data::placed_feature::PlacedFeature::OreEmerald,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreEmerald,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreGold,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreGoldBuried,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreGoldDeltas,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreNetherGold,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreGoldExtra,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::OreGold),
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
        pumpkin_data::placed_feature::PlacedFeature::OreGoldLower,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreGoldBuried,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreGoldNether,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreNetherGold,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreGraniteLower,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreGranite,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreGraniteUpper,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreGranite,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreGravel,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::OreGravel),
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
        pumpkin_data::placed_feature::PlacedFeature::OreGravelNether,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreGravelNether,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreInfested,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreInfested,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreIronMiddle,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::OreIron),
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
        pumpkin_data::placed_feature::PlacedFeature::OreIronSmall,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreIronSmall,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreIronUpper,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::OreIron),
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
        pumpkin_data::placed_feature::PlacedFeature::OreLapis,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::OreLapis),
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
        pumpkin_data::placed_feature::PlacedFeature::OreLapisBuried,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreLapisBuried,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreMagma,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::OreMagma),
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
        pumpkin_data::placed_feature::PlacedFeature::OreQuartzDeltas,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::OreQuartz),
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
        pumpkin_data::placed_feature::PlacedFeature::OreQuartzNether,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::OreQuartz),
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
        pumpkin_data::placed_feature::PlacedFeature::OreRedstone,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreRedstone,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreRedstoneLower,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreRedstone,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreSoulSand,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::OreSoulSand,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::OreTuff,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::OreTuff),
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
        pumpkin_data::placed_feature::PlacedFeature::PaleGardenFlowers,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::PaleForestFlower,
            ),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 8u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlockingNoLeaves,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PaleGardenVegetation,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::PaleGardenVegetation,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::PaleMossPatch,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::PaleMossPatch,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::PaleOakChecked,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::PaleOak),
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
        pumpkin_data::placed_feature::PlacedFeature::PaleOakCreakingChecked,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::PaleOakCreaking,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::PatchBerryBush,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::BerryBush),
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
                    predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                        predicates: vec![
                            BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                                tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
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
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchBerryCommon,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::BerryBush),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 32u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                        predicates: vec![
                            BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                                tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
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
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchBerryRare,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::BerryBush),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 384u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                        predicates: vec![
                            BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                                tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
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
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchBush,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Bush),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 4u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(24i32),
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -5i32,
                            max_inclusive: 5i32,
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchCactus,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Cactus),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(10i32),
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
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchCactusDecorated,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Cactus),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 13u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(10i32),
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
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchCactusDesert,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Cactus),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 6u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(10i32),
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
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchCrimsonRoots,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::CrimsonRoots,
            ),
            placement: vec![
                PlacementModifier::HeightRange(HeightRangePlacementModifier {
                    height: HeightProvider::Uniform(UniformHeightProvider {
                        min_inclusive: YOffset::AboveBottom(AboveBottom { above_bottom: 0i8 }),
                        max_inclusive: YOffset::BelowTop(BelowTop { below_top: 0i8 }),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchDeadBush,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::DeadBush),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(4i32),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchDeadBush2,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::DeadBush),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(4i32),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchDeadBushBadlands,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::DeadBush),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(20i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(4i32),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchDryGrassBadlands,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::DryGrass),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 6u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(64i32),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchDryGrassDesert,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::DryGrass),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 3u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(64i32),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchFire,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::PatchFire),
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
                    predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                        predicates: vec![
                            BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                                tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
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
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchFireflyBushNearWater,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FireflyBush,
            ),
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
                            BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                                tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
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
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(20i32),
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -4i32,
                            max_inclusive: 4i32,
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchFireflyBushNearWaterSwamp,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FireflyBush,
            ),
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
                            BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                                tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
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
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(20i32),
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -4i32,
                            max_inclusive: 4i32,
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchFireflyBushSwamp,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::FireflyBush,
            ),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 8u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(20i32),
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -4i32,
                            max_inclusive: 4i32,
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchGrassBadlands,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Grass),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchGrassForest,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Grass),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchGrassJungle,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::GrassJungle,
            ),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(25i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchGrassMeadow,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Grass),
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
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(16i32),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchGrassNormal,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Grass),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(5i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchGrassPlain,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Grass),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchGrassSavanna,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Grass),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(20i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchGrassTaiga,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TaigaGrass,
            ),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(7i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchGrassTaiga2,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TaigaGrass,
            ),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchLargeFern,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::LargeFern),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 5u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchLeafLitter,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::LeafLitter,
            ),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(2i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchMelon,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Melon),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 6u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(64i32),
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
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchMelonSparse,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Melon),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 64u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(64i32),
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
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchPumpkin,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Pumpkin),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 300u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                        predicates: vec![
                            BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                                tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
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
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchSoulFire,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::PatchSoulFire,
            ),
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
                    predicate: BlockPredicate::AllOf(AllOfBlockPredicate {
                        predicates: vec![
                            BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                                offset: OffsetBlocksBlockPredicate { offset: None },
                                tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
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
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchSugarCane,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::SugarCane),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 6u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(20i32),
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -4i32,
                            max_inclusive: 4i32,
                            plateau: 0i32,
                        },
                    )),
                    y_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: 0i32,
                            max_inclusive: 0i32,
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
        pumpkin_data::placed_feature::PlacedFeature::PatchSugarCaneBadlands,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::SugarCane),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 5u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(20i32),
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -4i32,
                            max_inclusive: 4i32,
                            plateau: 0i32,
                        },
                    )),
                    y_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: 0i32,
                            max_inclusive: 0i32,
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
        pumpkin_data::placed_feature::PlacedFeature::PatchSugarCaneDesert,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::SugarCane),
            placement: vec![
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(20i32),
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -4i32,
                            max_inclusive: 4i32,
                            plateau: 0i32,
                        },
                    )),
                    y_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: 0i32,
                            max_inclusive: 0i32,
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
        pumpkin_data::placed_feature::PlacedFeature::PatchSugarCaneSwamp,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::SugarCane),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 3u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(20i32),
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -4i32,
                            max_inclusive: 4i32,
                            plateau: 0i32,
                        },
                    )),
                    y_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: 0i32,
                            max_inclusive: 0i32,
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
        pumpkin_data::placed_feature::PlacedFeature::PatchSunflower,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Sunflower),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 3u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchTaigaGrass,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TaigaGrass,
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchTallGrass,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::TallGrass),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 5u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchTallGrass2,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::TallGrass),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PatchWaterlily,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Waterlily),
            placement: vec![
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(4i32),
                }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::WorldSurfaceWg,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(10i32),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PileHay,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::PileHay),
            placement: vec![],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PileIce,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::PileIce),
            placement: vec![],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PileMelon,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::PileMelon),
            placement: vec![],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PilePumpkin,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::PilePumpkin,
            ),
            placement: vec![],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::PileSnow,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::PileSnow),
            placement: vec![],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::Pine,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Pine),
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
        pumpkin_data::placed_feature::PlacedFeature::PineChecked,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Pine),
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
        pumpkin_data::placed_feature::PlacedFeature::PineOnSnow,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Pine),
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
        pumpkin_data::placed_feature::PlacedFeature::PointedDripstone,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::PointedDripstone,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::RedMushroomNether,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::RedMushroom,
            ),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::RedMushroomNormal,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::RedMushroom,
            ),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 512u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::RedMushroomOldGrowth,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::RedMushroom,
            ),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 171u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::RedMushroomSwamp,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::RedMushroom,
            ),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 64u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::RedMushroomTaiga,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::RedMushroom,
            ),
            placement: vec![
                PlacementModifier::RarityFilter(RarityFilterPlacementModifier { chance: 256u32 }),
                PlacementModifier::InSquare(SquarePlacementModifier),
                PlacementModifier::Heightmap(HeightmapPlacementModifier {
                    heightmap: HeightMap::MotionBlocking,
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
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
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::RootedAzaleaTree,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::RootedAzaleaTree,
            ),
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
                    allowed_search_condition: Some(BlockPredicate::MatchingBlockTag(
                        MatchingBlockTagPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
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
        pumpkin_data::placed_feature::PlacedFeature::RootedSulfurSpring,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::RootedSulfurSpring,
            ),
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
                    allowed_search_condition: Some(BlockPredicate::MatchingBlockTag(
                        MatchingBlockTagPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
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
        pumpkin_data::placed_feature::PlacedFeature::SculkPatchAncientCity,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SculkPatchAncientCity,
            ),
            placement: vec![],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::SculkPatchDeepDark,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SculkPatchDeepDark,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SculkVein,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::SculkVein),
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
        pumpkin_data::placed_feature::PlacedFeature::SeaPickle,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::SeaPickle),
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
        pumpkin_data::placed_feature::PlacedFeature::SeagrassCold,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SeagrassShort,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SeagrassDeep,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SeagrassTall,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SeagrassDeepCold,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SeagrassTall,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SeagrassDeepWarm,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SeagrassTall,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SeagrassNormal,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SeagrassShort,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SeagrassRiver,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SeagrassSlightlyLessShort,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SeagrassSwamp,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SeagrassMid,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SeagrassWarm,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SeagrassShort,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SmallBasaltColumns,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SmallBasaltColumns,
            ),
            placement: vec![
                PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier {
                    count: IntProvider::Constant(4i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::SporeBlossom,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SporeBlossom,
            ),
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
                    allowed_search_condition: Some(BlockPredicate::MatchingBlockTag(
                        MatchingBlockTagPredicate {
                            offset: OffsetBlocksBlockPredicate { offset: None },
                            tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
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
        pumpkin_data::placed_feature::PlacedFeature::SpringClosed,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SpringNetherClosed,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SpringClosedDouble,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SpringNetherClosed,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SpringDelta,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SpringLavaNether,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SpringLava,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SpringLavaOverworld,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SpringLavaFrozen,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SpringLavaFrozen,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SpringOpen,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SpringNetherOpen,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SpringWater,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SpringWater,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::Spruce,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Spruce),
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
        pumpkin_data::placed_feature::PlacedFeature::SpruceChecked,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Spruce),
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
        pumpkin_data::placed_feature::PlacedFeature::SpruceOnSnow,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Spruce),
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
        pumpkin_data::placed_feature::PlacedFeature::SulfurPool,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SulfurPool,
            ),
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
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::Solid(SolidBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                    }),
                }),
                PlacementModifier::EnvironmentScan(EnvironmentScanPlacementModifier {
                    direction_of_search: BlockDirection::Up,
                    target_condition: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                    allowed_search_condition: None,
                    max_steps: 32i32,
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Constant(0i32),
                    y_spread: IntProvider::Constant(-1i32),
                }),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingBlocks(MatchingBlocksBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        blocks: MatchingBlocksWrapper::Single("minecraft:sulfur".to_string()),
                    }),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::SulfurSpike,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SulfurSpike,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SulfurSpikeCluster,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SulfurSpikeCluster,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SuperBirchBees,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SuperBirchBees,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::SuperBirchBees0002,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::SuperBirchBees0002,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TallMangroveChecked,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TallMangrove,
            ),
            placement: vec![PlacementModifier::BlockPredicateFilter(
                BlockFilterPlacementModifier {
                    predicate: BlockPredicate::WouldSurvive(WouldSurviveBlockPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        state: {
                            let mut props = std::collections::HashMap::new();
                            props.insert("waterlogged".to_string(), "false".to_string());
                            props.insert("stage".to_string(), "0".to_string());
                            props.insert("hanging".to_string(), "false".to_string());
                            props.insert("age".to_string(), "0".to_string());
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
        pumpkin_data::placed_feature::PlacedFeature::TreesBadlands,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TreesBadlands,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesBirch,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TreesBirch,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesBirchAndOakLeafLitter,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TreesBirchAndOakLeafLitter,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesCherry,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::CherryBees005,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesFlowerForest,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TreesFlowerForest,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesGrove,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TreesGrove,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesJungle,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TreesJungle,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesMangrove,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::MangroveVegetation,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesMeadow,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::MeadowTrees,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesOldGrowthPineTaiga,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TreesOldGrowthPineTaiga,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesOldGrowthSpruceTaiga,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TreesOldGrowthSpruceTaiga,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesPlains,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TreesPlains,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesSavanna,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TreesSavanna,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesSnowy,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TreesSnowy,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesSparseJungle,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TreesSparseJungle,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesSwamp,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::SwampOak),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesTaiga,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TreesTaiga,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesWater,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TreesWater,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesWindsweptForest,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TreesWindsweptHills,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesWindsweptHills,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TreesWindsweptHills,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TreesWindsweptSavanna,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TreesSavanna,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::TwistingVines,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::TwistingVines,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::UnderwaterMagma,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::UnderwaterMagma,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::Vines,
        PlacedFeature {
            feature: Feature::Named(pumpkin_data::configured_feature::ConfiguredFeature::Vines),
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
        pumpkin_data::placed_feature::PlacedFeature::VoidStartPlatform,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::VoidStartPlatform,
            ),
            placement: vec![PlacementModifier::Biome(BiomePlacementModifier)],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::WarmOceanVegetation,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::WarmOceanVegetation,
            ),
            placement: vec![
                PlacementModifier::NoiseBasedCount(NoiseBasedCountPlacementModifier {
                    to_count_ratio: 20i32,
                    factor: 400f64,
                    offset: 0f64,
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
        pumpkin_data::placed_feature::PlacedFeature::WarpedForestVegetation,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::WarpedForestVegetation,
            ),
            placement: vec![
                PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier {
                    count: IntProvider::Constant(5i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::WarpedFungi,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::WarpedFungus,
            ),
            placement: vec![
                PlacementModifier::CountOnEveryLayer(CountOnEveryLayerPlacementModifier {
                    count: IntProvider::Constant(8i32),
                }),
                PlacementModifier::Biome(BiomePlacementModifier),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::WeepingVines,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::WeepingVines,
            ),
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
        pumpkin_data::placed_feature::PlacedFeature::WildflowersBirchForest,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::Wildflower,
            ),
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
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(64i32),
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -6i32,
                            max_inclusive: 6i32,
                            plateau: 0i32,
                        },
                    )),
                    y_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -2i32,
                            max_inclusive: 2i32,
                            plateau: 0i32,
                        },
                    )),
                }),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map.insert(
        pumpkin_data::placed_feature::PlacedFeature::WildflowersMeadow,
        PlacedFeature {
            feature: Feature::Named(
                pumpkin_data::configured_feature::ConfiguredFeature::Wildflower,
            ),
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
                PlacementModifier::Count(CountPlacementModifier {
                    count: IntProvider::Constant(8i32),
                }),
                PlacementModifier::RandomOffset(RandomOffsetPlacementModifier {
                    xz_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -6i32,
                            max_inclusive: 6i32,
                            plateau: 0i32,
                        },
                    )),
                    y_spread: IntProvider::Object(NormalIntProvider::Trapezoid(
                        TrapezoidIntProvider {
                            min_inclusive: -2i32,
                            max_inclusive: 2i32,
                            plateau: 0i32,
                        },
                    )),
                }),
                PlacementModifier::BlockPredicateFilter(BlockFilterPlacementModifier {
                    predicate: BlockPredicate::MatchingBlockTag(MatchingBlockTagPredicate {
                        offset: OffsetBlocksBlockPredicate { offset: None },
                        tag: pumpkin_data::tag::Block::MINECRAFT_AIR,
                    }),
                }),
            ],
        },
    );
    map
}
