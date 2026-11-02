/* This file is generated. Do not edit manually. */
use crate::{Block, BlockDirection, BlockId, BlockStateId, tag};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropStrategy {
    ClickedFace,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformType {
    CopperChest,
}
#[derive(Debug, Clone, Copy)]
pub enum BlockPredicate {
    MatchingBlocks {
        blocks: &'static [BlockId],
        offset: (i8, i8, i8),
    },
    MatchingBlockTag {
        tag: tag::Tag,
        offset: (i8, i8, i8),
    },
    AllOf(&'static [BlockPredicate]),
}
impl BlockPredicate {
    #[must_use]
    pub fn matches<F>(&self, get_block: &F) -> bool
    where
        F: Fn(i8, i8, i8) -> &'static Block,
    {
        match self {
            Self::MatchingBlocks { blocks, offset } => {
                let block = get_block(offset.0, offset.1, offset.2);
                blocks.contains(&block.id)
            }
            Self::MatchingBlockTag { tag, offset } => {
                let block = get_block(offset.0, offset.1, offset.2);
                block.id.has_tag(*tag)
            }
            Self::AllOf(predicates) => predicates.iter().all(|p| p.matches(get_block)),
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum BlockTransformerStateProvider {
    SimpleState(BlockId),
    CopyProperties(BlockId),
}
#[derive(Debug, Clone, Copy)]
pub struct BlockTransformerRule {
    pub predicate: BlockPredicate,
    pub provider: BlockTransformerStateProvider,
}
#[derive(Debug, Clone, Copy)]
pub struct BlockTransformerEntry {
    pub rules: &'static [BlockTransformerRule],
    pub disallowed_faces: &'static [BlockDirection],
    pub item_damage_per_use: u16,
    pub sound: Option<crate::sound::Sound>,
    pub particle: Option<crate::world::WorldEvent>,
    pub loot: Option<&'static str>,
    pub drop_strategy: Option<DropStrategy>,
    pub transform_type: Option<TransformType>,
    pub update_from_neighbors: bool,
}
#[derive(Debug, Clone, Copy)]
pub struct BlockTransformer {
    pub entries: &'static [BlockTransformerEntry],
}
#[derive(Debug, Clone, Copy)]
pub struct TransformResult {
    pub new_state_id: BlockStateId,
    pub target_block: &'static Block,
    pub entry: &'static BlockTransformerEntry,
}
impl BlockTransformer {
    #[must_use]
    pub fn transform<F>(
        &self,
        current_block: &Block,
        current_state_id: BlockStateId,
        face: BlockDirection,
        get_block: &F,
    ) -> Option<TransformResult>
    where
        F: Fn(i8, i8, i8) -> &'static Block,
    {
        for entry in self.entries {
            if entry.disallowed_faces.contains(&face) {
                continue;
            }
            for rule in entry.rules {
                if rule.predicate.matches(get_block) {
                    let (new_state_id, target_block) = match rule.provider {
                        BlockTransformerStateProvider::SimpleState(target_id) => {
                            let target_block = target_id.to_block();
                            (target_block.default_state.id, target_block)
                        }
                        BlockTransformerStateProvider::CopyProperties(target_id) => {
                            let target_block = target_id.to_block();
                            let new_state_id = if target_block.states.len() <= 1 {
                                target_block.default_state.id
                            } else if let Some(source_props) =
                                current_block.properties(current_state_id)
                            {
                                let props = source_props.to_props();
                                target_block
                                    .from_properties(&props)
                                    .to_state_id(target_block)
                            } else {
                                target_block.default_state.id
                            };
                            (new_state_id, target_block)
                        }
                    };
                    return Some(TransformResult {
                        new_state_id,
                        target_block,
                        entry,
                    });
                }
            }
        }
        None
    }
}
pub static AXE: BlockTransformer = BlockTransformer {
    entries: &[
        BlockTransformerEntry {
            rules: &[
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::OAK_WOOD],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_OAK_WOOD,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::OAK_LOG],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_OAK_LOG,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::DARK_OAK_WOOD],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_DARK_OAK_WOOD,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::DARK_OAK_LOG],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_DARK_OAK_LOG,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::PALE_OAK_WOOD],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_PALE_OAK_WOOD,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::PALE_OAK_LOG],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_PALE_OAK_LOG,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::POPLAR_WOOD],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_POPLAR_WOOD,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::POPLAR_LOG],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_POPLAR_LOG,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::ACACIA_WOOD],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_ACACIA_WOOD,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::ACACIA_LOG],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_ACACIA_LOG,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::CHERRY_WOOD],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_CHERRY_WOOD,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::CHERRY_LOG],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_CHERRY_LOG,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::BIRCH_WOOD],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_BIRCH_WOOD,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::BIRCH_LOG],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_BIRCH_LOG,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::JUNGLE_WOOD],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_JUNGLE_WOOD,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::JUNGLE_LOG],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_JUNGLE_LOG,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::SPRUCE_WOOD],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_SPRUCE_WOOD,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::SPRUCE_LOG],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_SPRUCE_LOG,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WARPED_STEM],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_WARPED_STEM,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WARPED_HYPHAE],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_WARPED_HYPHAE,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::CRIMSON_STEM],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_CRIMSON_STEM,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::CRIMSON_HYPHAE],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_CRIMSON_HYPHAE,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::MANGROVE_WOOD],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_MANGROVE_WOOD,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::MANGROVE_LOG],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_MANGROVE_LOG,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::BAMBOO_BLOCK],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::STRIPPED_BAMBOO_BLOCK,
                    ),
                },
            ],
            disallowed_faces: &[],
            item_damage_per_use: 1u16,
            sound: Some(crate::sound::Sound::ItemAxeStrip),
            particle: None,
            loot: None,
            drop_strategy: None,
            transform_type: None,
            update_from_neighbors: true,
        },
        BlockTransformerEntry {
            rules: &[
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::EXPOSED_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::COPPER_BLOCK),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WEATHERED_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::OXIDIZED_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::EXPOSED_CUT_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::CUT_COPPER),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WEATHERED_CUT_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_CUT_COPPER,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::OXIDIZED_CUT_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_CUT_COPPER,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::EXPOSED_CHISELED_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::CHISELED_COPPER,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WEATHERED_CHISELED_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_CHISELED_COPPER,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::OXIDIZED_CHISELED_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_CHISELED_COPPER,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::EXPOSED_CUT_COPPER_SLAB],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::CUT_COPPER_SLAB,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WEATHERED_CUT_COPPER_SLAB],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_CUT_COPPER_SLAB,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::OXIDIZED_CUT_COPPER_SLAB],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_CUT_COPPER_SLAB,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::EXPOSED_CUT_COPPER_STAIRS],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::CUT_COPPER_STAIRS,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WEATHERED_CUT_COPPER_STAIRS],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_CUT_COPPER_STAIRS,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::OXIDIZED_CUT_COPPER_STAIRS],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_CUT_COPPER_STAIRS,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::EXPOSED_COPPER_TRAPDOOR],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::COPPER_TRAPDOOR,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WEATHERED_COPPER_TRAPDOOR],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_TRAPDOOR,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::OXIDIZED_COPPER_TRAPDOOR],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_TRAPDOOR,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::EXPOSED_COPPER_BARS],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::COPPER_BARS),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WEATHERED_COPPER_BARS],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_BARS,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::OXIDIZED_COPPER_BARS],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_BARS,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::EXPOSED_COPPER_GRATE],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::COPPER_GRATE),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WEATHERED_COPPER_GRATE],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_GRATE,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::OXIDIZED_COPPER_GRATE],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_GRATE,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::EXPOSED_COPPER_BULB],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::COPPER_BULB),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WEATHERED_COPPER_BULB],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_BULB,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::OXIDIZED_COPPER_BULB],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_BULB,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::EXPOSED_COPPER_LANTERN],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::COPPER_LANTERN,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WEATHERED_COPPER_LANTERN],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_LANTERN,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::OXIDIZED_COPPER_LANTERN],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_LANTERN,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::EXPOSED_COPPER_GOLEM_STATUE],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::COPPER_GOLEM_STATUE,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WEATHERED_COPPER_GOLEM_STATUE],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_GOLEM_STATUE,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::OXIDIZED_COPPER_GOLEM_STATUE],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_GOLEM_STATUE,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::EXPOSED_LIGHTNING_ROD],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::LIGHTNING_ROD),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WEATHERED_LIGHTNING_ROD],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_LIGHTNING_ROD,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::OXIDIZED_LIGHTNING_ROD],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_LIGHTNING_ROD,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::EXPOSED_COPPER_CHAIN],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::COPPER_CHAIN),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WEATHERED_COPPER_CHAIN],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_CHAIN,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::OXIDIZED_COPPER_CHAIN],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_CHAIN,
                    ),
                },
            ],
            disallowed_faces: &[],
            item_damage_per_use: 1u16,
            sound: Some(crate::sound::Sound::ItemAxeScrape),
            particle: Some(crate::world::WorldEvent::ParticlesScrape),
            loot: None,
            drop_strategy: None,
            transform_type: None,
            update_from_neighbors: true,
        },
        BlockTransformerEntry {
            rules: &[
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::EXPOSED_COPPER_CHEST],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::COPPER_CHEST),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WEATHERED_COPPER_CHEST],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_CHEST,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::OXIDIZED_COPPER_CHEST],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_CHEST,
                    ),
                },
            ],
            disallowed_faces: &[],
            item_damage_per_use: 1u16,
            sound: Some(crate::sound::Sound::ItemAxeScrape),
            particle: Some(crate::world::WorldEvent::ParticlesScrape),
            loot: None,
            drop_strategy: None,
            transform_type: Some(TransformType::CopperChest),
            update_from_neighbors: false,
        },
        BlockTransformerEntry {
            rules: &[
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::EXPOSED_COPPER_DOOR],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::COPPER_DOOR),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WEATHERED_COPPER_DOOR],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_DOOR,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::OXIDIZED_COPPER_DOOR],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_DOOR,
                    ),
                },
            ],
            disallowed_faces: &[],
            item_damage_per_use: 1u16,
            sound: Some(crate::sound::Sound::ItemAxeScrape),
            particle: Some(crate::world::WorldEvent::ParticlesScrape),
            loot: None,
            drop_strategy: None,
            transform_type: None,
            update_from_neighbors: false,
        },
        BlockTransformerEntry {
            rules: &[
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_COPPER_BLOCK],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::COPPER_BLOCK),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_EXPOSED_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_WEATHERED_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_OXIDIZED_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::OXIDIZED_COPPER,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_CUT_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::CUT_COPPER),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_EXPOSED_CUT_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_CUT_COPPER,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_WEATHERED_CUT_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_CUT_COPPER,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_OXIDIZED_CUT_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::OXIDIZED_CUT_COPPER,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_CUT_COPPER_SLAB],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::CUT_COPPER_SLAB,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_EXPOSED_CUT_COPPER_SLAB],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_CUT_COPPER_SLAB,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_WEATHERED_CUT_COPPER_SLAB],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_CUT_COPPER_SLAB,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_OXIDIZED_CUT_COPPER_SLAB],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::OXIDIZED_CUT_COPPER_SLAB,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_CUT_COPPER_STAIRS],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::CUT_COPPER_STAIRS,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_EXPOSED_CUT_COPPER_STAIRS],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_CUT_COPPER_STAIRS,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_WEATHERED_CUT_COPPER_STAIRS],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_CUT_COPPER_STAIRS,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_OXIDIZED_CUT_COPPER_STAIRS],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::OXIDIZED_CUT_COPPER_STAIRS,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_CHISELED_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::CHISELED_COPPER,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_EXPOSED_CHISELED_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_CHISELED_COPPER,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_WEATHERED_CHISELED_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_CHISELED_COPPER,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_OXIDIZED_CHISELED_COPPER],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::OXIDIZED_CHISELED_COPPER,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_COPPER_TRAPDOOR],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::COPPER_TRAPDOOR,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_EXPOSED_COPPER_TRAPDOOR],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_TRAPDOOR,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_WEATHERED_COPPER_TRAPDOOR],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_TRAPDOOR,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_OXIDIZED_COPPER_TRAPDOOR],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::OXIDIZED_COPPER_TRAPDOOR,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_COPPER_BARS],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::COPPER_BARS),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_EXPOSED_COPPER_BARS],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_BARS,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_WEATHERED_COPPER_BARS],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_BARS,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_OXIDIZED_COPPER_BARS],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::OXIDIZED_COPPER_BARS,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_COPPER_GRATE],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::COPPER_GRATE),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_EXPOSED_COPPER_GRATE],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_GRATE,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_WEATHERED_COPPER_GRATE],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_GRATE,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_OXIDIZED_COPPER_GRATE],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::OXIDIZED_COPPER_GRATE,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_COPPER_BULB],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::COPPER_BULB),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_EXPOSED_COPPER_BULB],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_BULB,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_WEATHERED_COPPER_BULB],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_BULB,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_OXIDIZED_COPPER_BULB],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::OXIDIZED_COPPER_BULB,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_COPPER_GOLEM_STATUE],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::COPPER_GOLEM_STATUE,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_EXPOSED_COPPER_GOLEM_STATUE],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_GOLEM_STATUE,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_WEATHERED_COPPER_GOLEM_STATUE],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_GOLEM_STATUE,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_OXIDIZED_COPPER_GOLEM_STATUE],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::OXIDIZED_COPPER_GOLEM_STATUE,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_LIGHTNING_ROD],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::LIGHTNING_ROD),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_EXPOSED_LIGHTNING_ROD],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_LIGHTNING_ROD,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_WEATHERED_LIGHTNING_ROD],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_LIGHTNING_ROD,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_OXIDIZED_LIGHTNING_ROD],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::OXIDIZED_LIGHTNING_ROD,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_COPPER_LANTERN],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::COPPER_LANTERN,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_EXPOSED_COPPER_LANTERN],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_LANTERN,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_WEATHERED_COPPER_LANTERN],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_LANTERN,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_OXIDIZED_COPPER_LANTERN],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::OXIDIZED_COPPER_LANTERN,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_COPPER_CHAIN],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::COPPER_CHAIN),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_EXPOSED_COPPER_CHAIN],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_CHAIN,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_WEATHERED_COPPER_CHAIN],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_CHAIN,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_OXIDIZED_COPPER_CHAIN],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::OXIDIZED_COPPER_CHAIN,
                    ),
                },
            ],
            disallowed_faces: &[],
            item_damage_per_use: 1u16,
            sound: Some(crate::sound::Sound::ItemAxeWaxOff),
            particle: Some(crate::world::WorldEvent::ParticlesWaxOff),
            loot: None,
            drop_strategy: None,
            transform_type: None,
            update_from_neighbors: true,
        },
        BlockTransformerEntry {
            rules: &[
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_COPPER_CHEST],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::COPPER_CHEST),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_EXPOSED_COPPER_CHEST],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_CHEST,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_WEATHERED_COPPER_CHEST],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_CHEST,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_OXIDIZED_COPPER_CHEST],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::OXIDIZED_COPPER_CHEST,
                    ),
                },
            ],
            disallowed_faces: &[],
            item_damage_per_use: 1u16,
            sound: Some(crate::sound::Sound::ItemAxeWaxOff),
            particle: Some(crate::world::WorldEvent::ParticlesWaxOff),
            loot: None,
            drop_strategy: None,
            transform_type: Some(TransformType::CopperChest),
            update_from_neighbors: false,
        },
        BlockTransformerEntry {
            rules: &[
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_COPPER_DOOR],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(BlockId::COPPER_DOOR),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_EXPOSED_COPPER_DOOR],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::EXPOSED_COPPER_DOOR,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_WEATHERED_COPPER_DOOR],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::WEATHERED_COPPER_DOOR,
                    ),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::MatchingBlocks {
                        blocks: &[BlockId::WAXED_OXIDIZED_COPPER_DOOR],
                        offset: (0i8, 0i8, 0i8),
                    },
                    provider: BlockTransformerStateProvider::CopyProperties(
                        BlockId::OXIDIZED_COPPER_DOOR,
                    ),
                },
            ],
            disallowed_faces: &[],
            item_damage_per_use: 1u16,
            sound: Some(crate::sound::Sound::ItemAxeWaxOff),
            particle: Some(crate::world::WorldEvent::ParticlesWaxOff),
            loot: None,
            drop_strategy: None,
            transform_type: None,
            update_from_neighbors: false,
        },
    ],
};
pub static HOE: BlockTransformer = BlockTransformer {
    entries: &[
        BlockTransformerEntry {
            rules: &[
                BlockTransformerRule {
                    predicate: BlockPredicate::AllOf(&[
                        BlockPredicate::MatchingBlockTag {
                            tag: tag::Block::MINECRAFT_TURNS_INTO_FARMLAND,
                            offset: (0i8, 0i8, 0i8),
                        },
                        BlockPredicate::MatchingBlockTag {
                            tag: tag::Block::MINECRAFT_AIR,
                            offset: (0i8, 1i8, 0i8),
                        },
                    ]),
                    provider: BlockTransformerStateProvider::SimpleState(BlockId::FARMLAND),
                },
                BlockTransformerRule {
                    predicate: BlockPredicate::AllOf(&[
                        BlockPredicate::MatchingBlocks {
                            blocks: &[BlockId::COARSE_DIRT],
                            offset: (0i8, 0i8, 0i8),
                        },
                        BlockPredicate::MatchingBlockTag {
                            tag: tag::Block::MINECRAFT_AIR,
                            offset: (0i8, 1i8, 0i8),
                        },
                    ]),
                    provider: BlockTransformerStateProvider::SimpleState(BlockId::DIRT),
                },
            ],
            disallowed_faces: &[BlockDirection::Down],
            item_damage_per_use: 1u16,
            sound: Some(crate::sound::Sound::ItemHoeTill),
            particle: None,
            loot: None,
            drop_strategy: None,
            transform_type: None,
            update_from_neighbors: true,
        },
        BlockTransformerEntry {
            rules: &[BlockTransformerRule {
                predicate: BlockPredicate::MatchingBlocks {
                    blocks: &[BlockId::ROOTED_DIRT],
                    offset: (0i8, 0i8, 0i8),
                },
                provider: BlockTransformerStateProvider::SimpleState(BlockId::DIRT),
            }],
            disallowed_faces: &[],
            item_damage_per_use: 1u16,
            sound: Some(crate::sound::Sound::ItemHoeTill),
            particle: None,
            loot: Some("minecraft:till/rooted_dirt"),
            drop_strategy: Some(DropStrategy::ClickedFace),
            transform_type: None,
            update_from_neighbors: true,
        },
    ],
};
pub static SHOVEL: BlockTransformer = BlockTransformer {
    entries: &[BlockTransformerEntry {
        rules: &[BlockTransformerRule {
            predicate: BlockPredicate::AllOf(&[
                BlockPredicate::MatchingBlockTag {
                    tag: tag::Block::MINECRAFT_TURNS_INTO_DIRT_PATH,
                    offset: (0i8, 0i8, 0i8),
                },
                BlockPredicate::MatchingBlockTag {
                    tag: tag::Block::MINECRAFT_AIR,
                    offset: (0i8, 1i8, 0i8),
                },
            ]),
            provider: BlockTransformerStateProvider::SimpleState(BlockId::DIRT_PATH),
        }],
        disallowed_faces: &[BlockDirection::Down],
        item_damage_per_use: 1u16,
        sound: Some(crate::sound::Sound::ItemShovelFlatten),
        particle: None,
        loot: None,
        drop_strategy: None,
        transform_type: None,
        update_from_neighbors: true,
    }],
};
#[must_use]
pub fn get_block_transformer(key: &str) -> Option<&'static BlockTransformer> {
    match key {
        "minecraft:axe" | "axe" => Some(&AXE),
        "minecraft:hoe" | "hoe" => Some(&HOE),
        "minecraft:shovel" | "shovel" => Some(&SHOVEL),
        _ => None,
    }
}
