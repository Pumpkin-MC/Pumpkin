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
            rules: &[],
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
            rules: &[],
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
            rules: &[],
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
            rules: &[],
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
            rules: &[],
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
            rules: &[],
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
            rules: &[],
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
            rules: &[],
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
            rules: &[],
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
        rules: &[],
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
