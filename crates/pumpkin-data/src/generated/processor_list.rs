/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaticHeightmapType {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaticAxis {
    X,
    Y,
    Z,
}
#[derive(Clone, Debug, PartialEq)]
pub enum StaticPosRuleTest {
    AlwaysTrue,
    LinearPos {
        min_dist: i32,
        max_dist: i32,
        min_chance: f32,
        max_chance: f32,
    },
    AxisAlignedLinearPos {
        axis: StaticAxis,
        min_dist: i32,
        max_dist: i32,
        min_chance: f32,
        max_chance: f32,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub enum StaticRuleTest {
    AlwaysTrue,
    BlockMatch(crate::BlockId),
    BlockStateMatch {
        block_id: crate::BlockId,
        properties: &'static [(&'static str, &'static str)],
    },
    RandomBlockMatch {
        block: crate::BlockId,
        probability: f32,
    },
    RandomBlockStateMatch {
        block_id: crate::BlockId,
        properties: &'static [(&'static str, &'static str)],
        probability: f32,
    },
    TagMatch(&'static str),
}
#[derive(Clone, Debug, PartialEq)]
pub struct StaticIgnoredBlock {
    pub block_id: crate::BlockId,
    pub properties: Option<&'static [(&'static str, &'static str)]>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum StaticRottableBlocks {
    Tag(&'static str),
    Block(crate::BlockId),
    List(&'static [crate::BlockId]),
}
#[derive(Clone, Debug, PartialEq)]
pub enum StaticBlockEntityModifier {
    AppendLoot { loot_table: &'static str },
    Clear,
    Passthrough,
}
#[derive(Clone, Debug, PartialEq)]
pub struct StaticOutputState {
    pub default_state: &'static crate::BlockState,
    pub properties: &'static [(&'static str, &'static str)],
}
#[derive(Clone, Debug, PartialEq)]
pub struct StaticProcessorRule {
    pub position_predicate: StaticPosRuleTest,
    pub input_predicate: StaticRuleTest,
    pub location_predicate: StaticRuleTest,
    pub output_state: StaticOutputState,
    pub block_entity_modifier: Option<StaticBlockEntityModifier>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum StaticProcessor {
    Rule(&'static [StaticProcessorRule]),
    BlockRot {
        integrity: f32,
        rottable_blocks: Option<StaticRottableBlocks>,
    },
    BlockAge {
        mossiness: f32,
    },
    BlockIgnore(&'static [StaticIgnoredBlock]),
    Gravity {
        heightmap: StaticHeightmapType,
        offset: i32,
    },
    ProtectedBlocks(&'static str),
    BlackstoneReplace,
    JigsawReplacement,
    LavaSubmergedBlock,
    Capped {
        limit: i32,
        delegate: &'static StaticProcessor,
    },
    Nop,
}
pub struct StaticProcessorList;
impl StaticProcessorList {
    pub const ANCIENT_CITY_GENERIC_DEGRADATION: &'static [StaticProcessor] = &[
        StaticProcessor::BlockRot {
            integrity: 0.95f32,
            rottable_blocks: Some(StaticRottableBlocks::Tag("ancient_city_replaceable")),
        },
        StaticProcessor::Rule(&[
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::DEEPSLATE_BRICKS.id,
                    probability: 0.3f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::CRACKED_DEEPSLATE_BRICKS.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::DEEPSLATE_TILES.id,
                    probability: 0.3f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::CRACKED_DEEPSLATE_TILES.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::SOUL_LANTERN.id,
                    probability: 0.05f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::AIR.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
        ]),
        StaticProcessor::ProtectedBlocks("features_cannot_replace"),
    ];
    pub const ANCIENT_CITY_START_DEGRADATION: &'static [StaticProcessor] = &[
        StaticProcessor::Rule(&[
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::DEEPSLATE_BRICKS.id,
                    probability: 0.3f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::CRACKED_DEEPSLATE_BRICKS.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::DEEPSLATE_TILES.id,
                    probability: 0.3f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::CRACKED_DEEPSLATE_TILES.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::SOUL_LANTERN.id,
                    probability: 0.05f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::AIR.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
        ]),
        StaticProcessor::ProtectedBlocks("features_cannot_replace"),
    ];
    pub const ANCIENT_CITY_WALLS_DEGRADATION: &'static [StaticProcessor] = &[
        StaticProcessor::BlockRot {
            integrity: 0.95f32,
            rottable_blocks: Some(StaticRottableBlocks::Tag("ancient_city_replaceable")),
        },
        StaticProcessor::Rule(&[
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::DEEPSLATE_BRICKS.id,
                    probability: 0.3f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::CRACKED_DEEPSLATE_BRICKS.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::DEEPSLATE_TILES.id,
                    probability: 0.3f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::CRACKED_DEEPSLATE_TILES.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::DEEPSLATE_TILE_SLAB.id,
                    probability: 0.3f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::AIR.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::SOUL_LANTERN.id,
                    probability: 0.05f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::AIR.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
        ]),
        StaticProcessor::ProtectedBlocks("features_cannot_replace"),
    ];
    pub const BASTION_GENERIC_DEGRADATION: &'static [StaticProcessor] =
        &[StaticProcessor::Rule(&[
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::POLISHED_BLACKSTONE_BRICKS.id,
                    probability: 0.3f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::BLACKSTONE.id,
                    probability: 0.0001f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::AIR.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::GOLD_BLOCK.id,
                    probability: 0.3f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::GILDED_BLACKSTONE.id,
                    probability: 0.5f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::BLACKSTONE.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::BLACKSTONE.id,
                    probability: 0.01f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::GILDED_BLACKSTONE.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
        ])];
    pub const BOTTOM_RAMPART: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::MAGMA_BLOCK.id,
                probability: 0.75f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.id,
                probability: 0.15f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::POLISHED_BLACKSTONE_BRICKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::GILDED_BLACKSTONE.id,
                probability: 0.5f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::BLACKSTONE.id,
                probability: 0.01f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::GILDED_BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const BRIDGE: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::POLISHED_BLACKSTONE_BRICKS.id,
                probability: 0.3f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::BLACKSTONE.id,
                probability: 0.0001f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const EMPTY: &'static [StaticProcessor] = &[];
    pub const ENTRANCE_REPLACEMENT: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::CHISELED_POLISHED_BLACKSTONE.id,
                probability: 0.5f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::GOLD_BLOCK.id,
                probability: 0.6f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::GILDED_BLACKSTONE.id,
                probability: 0.5f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::BLACKSTONE.id,
                probability: 0.01f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::GILDED_BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const FARM_DESERT: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.2f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BEETROOTS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.1f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::MELON_STEM.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const FARM_PLAINS: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.3f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CARROTS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.2f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::POTATOES.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.1f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BEETROOTS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const FARM_SAVANNA: &'static [StaticProcessor] =
        &[StaticProcessor::Rule(&[StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.1f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::MELON_STEM.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        }])];
    pub const FARM_SNOWY: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.1f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CARROTS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.8f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::POTATOES.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const FARM_TAIGA: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.3f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::PUMPKIN_STEM.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.2f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::POTATOES.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const FOSSIL_COAL: &'static [StaticProcessor] = &[
        StaticProcessor::BlockRot {
            integrity: 0.1f32,
            rottable_blocks: None,
        },
        StaticProcessor::ProtectedBlocks("features_cannot_replace"),
    ];
    pub const FOSSIL_DIAMONDS: &'static [StaticProcessor] = &[
        StaticProcessor::BlockRot {
            integrity: 0.1f32,
            rottable_blocks: None,
        },
        StaticProcessor::Rule(&[StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::COAL_ORE.id),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::DEEPSLATE_DIAMOND_ORE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        }]),
        StaticProcessor::ProtectedBlocks("features_cannot_replace"),
    ];
    pub const FOSSIL_ROT: &'static [StaticProcessor] = &[
        StaticProcessor::BlockRot {
            integrity: 0.9f32,
            rottable_blocks: None,
        },
        StaticProcessor::ProtectedBlocks("features_cannot_replace"),
    ];
    pub const HIGH_RAMPART: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::GOLD_BLOCK.id,
                probability: 0.3f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AxisAlignedLinearPos {
                axis: StaticAxis::Y,
                min_dist: 0i32,
                max_dist: 100i32,
                min_chance: 0f32,
                max_chance: 0.05f32,
            },
            input_predicate: StaticRuleTest::AlwaysTrue,
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::GILDED_BLACKSTONE.id,
                probability: 0.5f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const HIGH_WALL: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::POLISHED_BLACKSTONE_BRICKS.id,
                probability: 0.01f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::POLISHED_BLACKSTONE_BRICKS.id,
                probability: 0.5f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::POLISHED_BLACKSTONE_BRICKS.id,
                probability: 0.3f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::GILDED_BLACKSTONE.id,
                probability: 0.5f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const HOUSING: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::POLISHED_BLACKSTONE_BRICKS.id,
                probability: 0.3f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::BLACKSTONE.id,
                probability: 0.0001f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::GILDED_BLACKSTONE.id,
                probability: 0.5f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::BLACKSTONE.id,
                probability: 0.01f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::GILDED_BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const MOSSIFY_10_PERCENT: &'static [StaticProcessor] =
        &[StaticProcessor::Rule(&[StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::COBBLESTONE.id,
                probability: 0.1f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::MOSSY_COBBLESTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        }])];
    pub const MOSSIFY_20_PERCENT: &'static [StaticProcessor] =
        &[StaticProcessor::Rule(&[StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::COBBLESTONE.id,
                probability: 0.2f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::MOSSY_COBBLESTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        }])];
    pub const MOSSIFY_70_PERCENT: &'static [StaticProcessor] =
        &[StaticProcessor::Rule(&[StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::COBBLESTONE.id,
                probability: 0.7f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::MOSSY_COBBLESTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        }])];
    pub const OUTPOST_ROT: &'static [StaticProcessor] = &[StaticProcessor::BlockRot {
        integrity: 0.05f32,
        rottable_blocks: None,
    }];
    pub const RAMPART_DEGRADATION: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::POLISHED_BLACKSTONE_BRICKS.id,
                probability: 0.4f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::BLACKSTONE.id,
                probability: 0.01f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::POLISHED_BLACKSTONE_BRICKS.id,
                probability: 0.0001f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::BLACKSTONE.id,
                probability: 0.0001f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::GOLD_BLOCK.id,
                probability: 0.3f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::GILDED_BLACKSTONE.id,
                probability: 0.5f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::BLACKSTONE.id,
                probability: 0.01f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::GILDED_BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const ROOF: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::POLISHED_BLACKSTONE_BRICKS.id,
                probability: 0.3f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::POLISHED_BLACKSTONE_BRICKS.id,
                probability: 0.15f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::POLISHED_BLACKSTONE_BRICKS.id,
                probability: 0.3f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const SIDE_WALL_DEGRADATION: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::CHISELED_POLISHED_BLACKSTONE.id,
                probability: 0.5f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::GOLD_BLOCK.id,
                probability: 0.1f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::GILDED_BLACKSTONE.id,
                probability: 0.5f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::BLACKSTONE.id,
                probability: 0.01f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::GILDED_BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const STABLE_DEGRADATION: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::POLISHED_BLACKSTONE_BRICKS.id,
                probability: 0.1f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::BLACKSTONE.id,
                probability: 0.0001f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::GILDED_BLACKSTONE.id,
                probability: 0.5f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::BLACKSTONE.id,
                probability: 0.01f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::GILDED_BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const STREET_PLAINS: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::DIRT_PATH.id),
            location_predicate: StaticRuleTest::BlockMatch(crate::Block::WATER.id),
            output_state: StaticOutputState {
                default_state: crate::Block::OAK_PLANKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::DIRT_PATH.id,
                probability: 0.1f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::GRASS_BLOCK.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::GRASS_BLOCK.id),
            location_predicate: StaticRuleTest::BlockMatch(crate::Block::WATER.id),
            output_state: StaticOutputState {
                default_state: crate::Block::WATER.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::DIRT.id),
            location_predicate: StaticRuleTest::BlockMatch(crate::Block::WATER.id),
            output_state: StaticOutputState {
                default_state: crate::Block::WATER.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const STREET_SAVANNA: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::DIRT_PATH.id),
            location_predicate: StaticRuleTest::BlockMatch(crate::Block::WATER.id),
            output_state: StaticOutputState {
                default_state: crate::Block::ACACIA_PLANKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::DIRT_PATH.id,
                probability: 0.2f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::GRASS_BLOCK.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::GRASS_BLOCK.id),
            location_predicate: StaticRuleTest::BlockMatch(crate::Block::WATER.id),
            output_state: StaticOutputState {
                default_state: crate::Block::WATER.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::DIRT.id),
            location_predicate: StaticRuleTest::BlockMatch(crate::Block::WATER.id),
            output_state: StaticOutputState {
                default_state: crate::Block::WATER.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const STREET_SNOWY_OR_TAIGA: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::DIRT_PATH.id),
            location_predicate: StaticRuleTest::BlockMatch(crate::Block::WATER.id),
            output_state: StaticOutputState {
                default_state: crate::Block::SPRUCE_PLANKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::DIRT_PATH.id),
            location_predicate: StaticRuleTest::BlockMatch(crate::Block::ICE.id),
            output_state: StaticOutputState {
                default_state: crate::Block::SPRUCE_PLANKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::DIRT_PATH.id,
                probability: 0.2f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::GRASS_BLOCK.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::GRASS_BLOCK.id),
            location_predicate: StaticRuleTest::BlockMatch(crate::Block::WATER.id),
            output_state: StaticOutputState {
                default_state: crate::Block::WATER.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::DIRT.id),
            location_predicate: StaticRuleTest::BlockMatch(crate::Block::WATER.id),
            output_state: StaticOutputState {
                default_state: crate::Block::WATER.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const TRAIL_RUINS_HOUSES_ARCHAEOLOGY: &'static [StaticProcessor] = &[
        StaticProcessor::Rule(&[
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::GRAVEL.id,
                    probability: 0.2f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::DIRT.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::GRAVEL.id,
                    probability: 0.1f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::COARSE_DIRT.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::MUD_BRICKS.id,
                    probability: 0.1f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::PACKED_MUD.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
        ]),
        StaticProcessor::Capped {
            limit: 6i32,
            delegate: &StaticProcessor::Rule(&[StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::TagMatch("trail_ruins_replaceable"),
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::SUSPICIOUS_GRAVEL.default_state,
                    properties: &[],
                },
                block_entity_modifier: Some(StaticBlockEntityModifier::AppendLoot {
                    loot_table: "minecraft:archaeology/trail_ruins_common",
                }),
            }]),
        },
        StaticProcessor::Capped {
            limit: 3i32,
            delegate: &StaticProcessor::Rule(&[StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::TagMatch("trail_ruins_replaceable"),
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::SUSPICIOUS_GRAVEL.default_state,
                    properties: &[],
                },
                block_entity_modifier: Some(StaticBlockEntityModifier::AppendLoot {
                    loot_table: "minecraft:archaeology/trail_ruins_rare",
                }),
            }]),
        },
    ];
    pub const TRAIL_RUINS_ROADS_ARCHAEOLOGY: &'static [StaticProcessor] = &[
        StaticProcessor::Rule(&[
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::GRAVEL.id,
                    probability: 0.2f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::DIRT.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::GRAVEL.id,
                    probability: 0.1f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::COARSE_DIRT.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::MUD_BRICKS.id,
                    probability: 0.1f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::PACKED_MUD.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
        ]),
        StaticProcessor::Capped {
            limit: 2i32,
            delegate: &StaticProcessor::Rule(&[StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::TagMatch("trail_ruins_replaceable"),
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::SUSPICIOUS_GRAVEL.default_state,
                    properties: &[],
                },
                block_entity_modifier: Some(StaticBlockEntityModifier::AppendLoot {
                    loot_table: "minecraft:archaeology/trail_ruins_common",
                }),
            }]),
        },
    ];
    pub const TRAIL_RUINS_TOWER_TOP_ARCHAEOLOGY: &'static [StaticProcessor] =
        &[StaticProcessor::Capped {
            limit: 2i32,
            delegate: &StaticProcessor::Rule(&[StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::TagMatch("trail_ruins_replaceable"),
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::SUSPICIOUS_GRAVEL.default_state,
                    properties: &[],
                },
                block_entity_modifier: Some(StaticBlockEntityModifier::AppendLoot {
                    loot_table: "minecraft:archaeology/trail_ruins_common",
                }),
            }]),
        }];
    pub const TREASURE_ROOMS: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::POLISHED_BLACKSTONE_BRICKS.id,
                probability: 0.35f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::CHISELED_POLISHED_BLACKSTONE.id,
                probability: 0.1f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::GILDED_BLACKSTONE.id,
                probability: 0.5f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::BLACKSTONE.id,
                probability: 0.01f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::GILDED_BLACKSTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const TRIAL_CHAMBERS_COPPER_BULB_DEGRADATION: &'static [StaticProcessor] = &[
        StaticProcessor::Rule(&[
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::WAXED_COPPER_BULB.id,
                    probability: 0.1f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::WAXED_OXIDIZED_COPPER_BULB.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::WAXED_COPPER_BULB.id,
                    probability: 0.33333334f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::WAXED_WEATHERED_COPPER_BULB.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
            StaticProcessorRule {
                position_predicate: StaticPosRuleTest::AlwaysTrue,
                input_predicate: StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::WAXED_COPPER_BULB.id,
                    probability: 0.5f32,
                },
                location_predicate: StaticRuleTest::AlwaysTrue,
                output_state: StaticOutputState {
                    default_state: crate::Block::WAXED_EXPOSED_COPPER_BULB.default_state,
                    properties: &[],
                },
                block_entity_modifier: None,
            },
        ]),
        StaticProcessor::ProtectedBlocks("features_cannot_replace"),
    ];
    pub const ZOMBIE_DESERT: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::TagMatch("doors"),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::TORCH.id),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::WALL_TORCH.id),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::SMOOTH_SANDSTONE.id,
                probability: 0.08f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::CUT_SANDSTONE.id,
                probability: 0.1f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::TERRACOTTA.id,
                probability: 0.08f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::SMOOTH_SANDSTONE_STAIRS.id,
                probability: 0.08f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::SMOOTH_SANDSTONE_SLAB.id,
                probability: 0.08f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.2f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BEETROOTS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.1f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::MELON_STEM.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const ZOMBIE_PLAINS: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::COBBLESTONE.id,
                probability: 0.8f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::MOSSY_COBBLESTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::TagMatch("doors"),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::TORCH.id),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::WALL_TORCH.id),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::COBBLESTONE.id,
                probability: 0.07f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::MOSSY_COBBLESTONE.id,
                probability: 0.07f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHITE_TERRACOTTA.id,
                probability: 0.07f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::OAK_LOG.id,
                probability: 0.05f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::OAK_PLANKS.id,
                probability: 0.1f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::OAK_STAIRS.id,
                probability: 0.1f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::STRIPPED_OAK_LOG.id,
                probability: 0.02f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::GLASS_PANE.id,
                probability: 0.5f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockStateMatch {
                block_id: crate::Block::GLASS_PANE.id,
                properties: &[],
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BROWN_STAINED_GLASS_PANE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockStateMatch {
                block_id: crate::Block::GLASS_PANE.id,
                properties: &[],
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BROWN_STAINED_GLASS_PANE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.3f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CARROTS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.2f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::POTATOES.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.1f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BEETROOTS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const ZOMBIE_SAVANNA: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::TagMatch("doors"),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::TORCH.id),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::WALL_TORCH.id),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::ACACIA_PLANKS.id,
                probability: 0.2f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::ACACIA_STAIRS.id,
                probability: 0.2f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::ACACIA_LOG.id,
                probability: 0.05f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::ACACIA_WOOD.id,
                probability: 0.05f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::ORANGE_TERRACOTTA.id,
                probability: 0.05f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::YELLOW_TERRACOTTA.id,
                probability: 0.05f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::RED_TERRACOTTA.id,
                probability: 0.05f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::GLASS_PANE.id,
                probability: 0.5f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockStateMatch {
                block_id: crate::Block::GLASS_PANE.id,
                properties: &[],
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BROWN_STAINED_GLASS_PANE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockStateMatch {
                block_id: crate::Block::GLASS_PANE.id,
                properties: &[],
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BROWN_STAINED_GLASS_PANE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.1f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::MELON_STEM.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const ZOMBIE_SNOWY: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::TagMatch("doors"),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::TORCH.id),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::WALL_TORCH.id),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::LANTERN.id),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::SPRUCE_PLANKS.id,
                probability: 0.2f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::SPRUCE_SLAB.id,
                probability: 0.4f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::STRIPPED_SPRUCE_LOG.id,
                probability: 0.05f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::STRIPPED_SPRUCE_WOOD.id,
                probability: 0.05f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::GLASS_PANE.id,
                probability: 0.5f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockStateMatch {
                block_id: crate::Block::GLASS_PANE.id,
                properties: &[],
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BROWN_STAINED_GLASS_PANE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockStateMatch {
                block_id: crate::Block::GLASS_PANE.id,
                properties: &[],
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BROWN_STAINED_GLASS_PANE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.1f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CARROTS.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.8f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::POTATOES.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    pub const ZOMBIE_TAIGA: &'static [StaticProcessor] = &[StaticProcessor::Rule(&[
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::COBBLESTONE.id,
                probability: 0.8f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::MOSSY_COBBLESTONE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::TagMatch("doors"),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::TORCH.id),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::WALL_TORCH.id),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::AIR.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockMatch(crate::Block::CAMPFIRE.id),
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::CAMPFIRE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::COBBLESTONE.id,
                probability: 0.08f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::SPRUCE_LOG.id,
                probability: 0.08f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::GLASS_PANE.id,
                probability: 0.5f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::COBWEB.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockStateMatch {
                block_id: crate::Block::GLASS_PANE.id,
                properties: &[],
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BROWN_STAINED_GLASS_PANE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::BlockStateMatch {
                block_id: crate::Block::GLASS_PANE.id,
                properties: &[],
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::BROWN_STAINED_GLASS_PANE.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.3f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::PUMPKIN_STEM.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
        StaticProcessorRule {
            position_predicate: StaticPosRuleTest::AlwaysTrue,
            input_predicate: StaticRuleTest::RandomBlockMatch {
                block: crate::Block::WHEAT.id,
                probability: 0.2f32,
            },
            location_predicate: StaticRuleTest::AlwaysTrue,
            output_state: StaticOutputState {
                default_state: crate::Block::POTATOES.default_state,
                properties: &[],
            },
            block_entity_modifier: None,
        },
    ])];
    #[must_use]
    pub fn get(id: &str) -> Option<&'static [StaticProcessor]> {
        let trimmed = id.strip_prefix("minecraft:").unwrap_or(id);
        match trimmed {
            "ancient_city_generic_degradation" | "minecraft:ancient_city_generic_degradation" => {
                Some(Self::ANCIENT_CITY_GENERIC_DEGRADATION)
            }
            "ancient_city_start_degradation" | "minecraft:ancient_city_start_degradation" => {
                Some(Self::ANCIENT_CITY_START_DEGRADATION)
            }
            "ancient_city_walls_degradation" | "minecraft:ancient_city_walls_degradation" => {
                Some(Self::ANCIENT_CITY_WALLS_DEGRADATION)
            }
            "bastion_generic_degradation" | "minecraft:bastion_generic_degradation" => {
                Some(Self::BASTION_GENERIC_DEGRADATION)
            }
            "bottom_rampart" | "minecraft:bottom_rampart" => Some(Self::BOTTOM_RAMPART),
            "bridge" | "minecraft:bridge" => Some(Self::BRIDGE),
            "empty" | "minecraft:empty" => Some(Self::EMPTY),
            "entrance_replacement" | "minecraft:entrance_replacement" => {
                Some(Self::ENTRANCE_REPLACEMENT)
            }
            "farm_desert" | "minecraft:farm_desert" => Some(Self::FARM_DESERT),
            "farm_plains" | "minecraft:farm_plains" => Some(Self::FARM_PLAINS),
            "farm_savanna" | "minecraft:farm_savanna" => Some(Self::FARM_SAVANNA),
            "farm_snowy" | "minecraft:farm_snowy" => Some(Self::FARM_SNOWY),
            "farm_taiga" | "minecraft:farm_taiga" => Some(Self::FARM_TAIGA),
            "fossil_coal" | "minecraft:fossil_coal" => Some(Self::FOSSIL_COAL),
            "fossil_diamonds" | "minecraft:fossil_diamonds" => Some(Self::FOSSIL_DIAMONDS),
            "fossil_rot" | "minecraft:fossil_rot" => Some(Self::FOSSIL_ROT),
            "high_rampart" | "minecraft:high_rampart" => Some(Self::HIGH_RAMPART),
            "high_wall" | "minecraft:high_wall" => Some(Self::HIGH_WALL),
            "housing" | "minecraft:housing" => Some(Self::HOUSING),
            "mossify_10_percent" | "minecraft:mossify_10_percent" => Some(Self::MOSSIFY_10_PERCENT),
            "mossify_20_percent" | "minecraft:mossify_20_percent" => Some(Self::MOSSIFY_20_PERCENT),
            "mossify_70_percent" | "minecraft:mossify_70_percent" => Some(Self::MOSSIFY_70_PERCENT),
            "outpost_rot" | "minecraft:outpost_rot" => Some(Self::OUTPOST_ROT),
            "rampart_degradation" | "minecraft:rampart_degradation" => {
                Some(Self::RAMPART_DEGRADATION)
            }
            "roof" | "minecraft:roof" => Some(Self::ROOF),
            "side_wall_degradation" | "minecraft:side_wall_degradation" => {
                Some(Self::SIDE_WALL_DEGRADATION)
            }
            "stable_degradation" | "minecraft:stable_degradation" => Some(Self::STABLE_DEGRADATION),
            "street_plains" | "minecraft:street_plains" => Some(Self::STREET_PLAINS),
            "street_savanna" | "minecraft:street_savanna" => Some(Self::STREET_SAVANNA),
            "street_snowy_or_taiga" | "minecraft:street_snowy_or_taiga" => {
                Some(Self::STREET_SNOWY_OR_TAIGA)
            }
            "trail_ruins_houses_archaeology" | "minecraft:trail_ruins_houses_archaeology" => {
                Some(Self::TRAIL_RUINS_HOUSES_ARCHAEOLOGY)
            }
            "trail_ruins_roads_archaeology" | "minecraft:trail_ruins_roads_archaeology" => {
                Some(Self::TRAIL_RUINS_ROADS_ARCHAEOLOGY)
            }
            "trail_ruins_tower_top_archaeology" | "minecraft:trail_ruins_tower_top_archaeology" => {
                Some(Self::TRAIL_RUINS_TOWER_TOP_ARCHAEOLOGY)
            }
            "treasure_rooms" | "minecraft:treasure_rooms" => Some(Self::TREASURE_ROOMS),
            "trial_chambers_copper_bulb_degradation"
            | "minecraft:trial_chambers_copper_bulb_degradation" => {
                Some(Self::TRIAL_CHAMBERS_COPPER_BULB_DEGRADATION)
            }
            "zombie_desert" | "minecraft:zombie_desert" => Some(Self::ZOMBIE_DESERT),
            "zombie_plains" | "minecraft:zombie_plains" => Some(Self::ZOMBIE_PLAINS),
            "zombie_savanna" | "minecraft:zombie_savanna" => Some(Self::ZOMBIE_SAVANNA),
            "zombie_snowy" | "minecraft:zombie_snowy" => Some(Self::ZOMBIE_SNOWY),
            "zombie_taiga" | "minecraft:zombie_taiga" => Some(Self::ZOMBIE_TAIGA),
            _ => None,
        }
    }
    #[must_use]
    pub const fn all_names() -> &'static [&'static str] {
        &[
            "minecraft:ancient_city_generic_degradation",
            "minecraft:ancient_city_start_degradation",
            "minecraft:ancient_city_walls_degradation",
            "minecraft:bastion_generic_degradation",
            "minecraft:bottom_rampart",
            "minecraft:bridge",
            "minecraft:empty",
            "minecraft:entrance_replacement",
            "minecraft:farm_desert",
            "minecraft:farm_plains",
            "minecraft:farm_savanna",
            "minecraft:farm_snowy",
            "minecraft:farm_taiga",
            "minecraft:fossil_coal",
            "minecraft:fossil_diamonds",
            "minecraft:fossil_rot",
            "minecraft:high_rampart",
            "minecraft:high_wall",
            "minecraft:housing",
            "minecraft:mossify_10_percent",
            "minecraft:mossify_20_percent",
            "minecraft:mossify_70_percent",
            "minecraft:outpost_rot",
            "minecraft:rampart_degradation",
            "minecraft:roof",
            "minecraft:side_wall_degradation",
            "minecraft:stable_degradation",
            "minecraft:street_plains",
            "minecraft:street_savanna",
            "minecraft:street_snowy_or_taiga",
            "minecraft:trail_ruins_houses_archaeology",
            "minecraft:trail_ruins_roads_archaeology",
            "minecraft:trail_ruins_tower_top_archaeology",
            "minecraft:treasure_rooms",
            "minecraft:trial_chambers_copper_bulb_degradation",
            "minecraft:zombie_desert",
            "minecraft:zombie_plains",
            "minecraft:zombie_savanna",
            "minecraft:zombie_snowy",
            "minecraft:zombie_taiga",
        ]
    }
}
