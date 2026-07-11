use pumpkin_data::{Block, BlockId, BlockState, tag};
use pumpkin_util::{
    math::vector3::Vector3,
    random::{RandomImpl, hash_block_pos, legacy_rand::LegacyRand},
};
use serde::Deserialize;
use std::sync::{Arc, LazyLock};

use crate::ProtoChunk;

#[derive(Clone)]
pub enum StructureProcessor {
    BlockRot { integrity: f32, blocks: BlockTag },
    Rules(Vec<ProcessorRule>),
    ProtectedBlocks(BlockTag),
}

#[derive(Clone)]
pub struct ProcessorRule {
    input_block: BlockId,
    probability: f32,
    output_state: &'static BlockState,
}

#[derive(Clone, Copy)]
pub enum BlockTag {
    AncientCityReplaceable,
    FeaturesCannotReplace,
}

impl BlockTag {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "#minecraft:ancient_city_replaceable" => Some(Self::AncientCityReplaceable),
            "#minecraft:features_cannot_replace" => Some(Self::FeaturesCannotReplace),
            _ => None,
        }
    }

    fn contains(self, block_id: BlockId) -> bool {
        block_id.has_tag(match self {
            Self::AncientCityReplaceable => tag::Block::MINECRAFT_ANCIENT_CITY_REPLACEABLE,
            Self::FeaturesCannotReplace => tag::Block::MINECRAFT_FEATURES_CANNOT_REPLACE,
        })
    }
}

impl StructureProcessor {
    #[must_use]
    pub fn process(
        &self,
        chunk: &ProtoChunk,
        pos: Vector3<i32>,
        state: &'static BlockState,
    ) -> Option<&'static BlockState> {
        let input_block = state.id.to_block_id();
        match self {
            Self::BlockRot { integrity, blocks } => {
                if !blocks.contains(input_block) {
                    return Some(state);
                }
                let mut random = LegacyRand::from_seed(hash_block_pos(pos.x, pos.y, pos.z) as u64);
                (random.next_f32() <= *integrity).then_some(state)
            }
            Self::Rules(rules) => {
                let mut random = LegacyRand::from_seed(hash_block_pos(pos.x, pos.y, pos.z) as u64);
                rules
                    .iter()
                    .find(|rule| {
                        input_block == rule.input_block && random.next_f32() < rule.probability
                    })
                    .map_or(Some(state), |rule| Some(rule.output_state))
            }
            Self::ProtectedBlocks(blocks) => {
                let existing = chunk.get_block_state(&pos).to_block_id();
                (!blocks.contains(existing)).then_some(state)
            }
        }
    }
}

#[derive(Deserialize)]
struct RawProcessorList {
    processors: Vec<RawProcessor>,
}

#[derive(Deserialize)]
#[serde(tag = "processor_type")]
enum RawProcessor {
    #[serde(rename = "minecraft:block_rot")]
    BlockRot {
        integrity: f32,
        rottable_blocks: String,
    },
    #[serde(rename = "minecraft:rule")]
    Rule { rules: Vec<RawRule> },
    #[serde(rename = "minecraft:protected_blocks")]
    ProtectedBlocks { value: String },
}

#[derive(Deserialize)]
struct RawRule {
    input_predicate: RawInputPredicate,
    output_state: RawOutputState,
}

#[derive(Deserialize)]
struct RawInputPredicate {
    block: String,
    probability: f32,
}

#[derive(Deserialize)]
struct RawOutputState {
    #[serde(rename = "Name")]
    name: String,
}

#[must_use]
pub fn load_processor_list(name: &str) -> Arc<[StructureProcessor]> {
    static CACHE: LazyLock<dashmap::DashMap<String, Arc<[StructureProcessor]>>> =
        LazyLock::new(dashmap::DashMap::new);

    if let Some(processors) = CACHE.get(name) {
        return Arc::clone(&processors);
    }

    let Some(json) = super::cache::get_processor_list_json(name) else {
        tracing::warn!("Unknown structure processor list: {name}");
        return Arc::from([]);
    };
    let raw: RawProcessorList = match serde_json::from_str(json) {
        Ok(raw) => raw,
        Err(error) => {
            tracing::error!("Failed to parse structure processor list {name}: {error}");
            return Arc::from([]);
        }
    };

    let processors = raw
        .processors
        .into_iter()
        .filter_map(|processor| match processor {
            RawProcessor::BlockRot {
                integrity,
                rottable_blocks,
            } => BlockTag::from_name(&rottable_blocks)
                .map(|blocks| StructureProcessor::BlockRot { integrity, blocks }),
            RawProcessor::ProtectedBlocks { value } => {
                BlockTag::from_name(&value).map(StructureProcessor::ProtectedBlocks)
            }
            RawProcessor::Rule { rules } => Some(StructureProcessor::Rules(
                rules
                    .into_iter()
                    .filter_map(|rule| {
                        let input_name = rule
                            .input_predicate
                            .block
                            .strip_prefix("minecraft:")
                            .unwrap_or(&rule.input_predicate.block);
                        let output_name = rule
                            .output_state
                            .name
                            .strip_prefix("minecraft:")
                            .unwrap_or(&rule.output_state.name);
                        let input_block = Block::from_name(input_name)?;
                        let output_block = Block::from_name(output_name)?;
                        Some(ProcessorRule {
                            input_block: input_block.id,
                            probability: rule.input_predicate.probability,
                            output_state: output_block.default_state,
                        })
                    })
                    .collect(),
            )),
        })
        .collect::<Arc<[_]>>();
    CACHE.insert(name.to_owned(), Arc::clone(&processors));
    processors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ancient_city_processor_lists() {
        assert_eq!(
            load_processor_list("minecraft:ancient_city_generic_degradation").len(),
            3
        );
        assert_eq!(
            load_processor_list("minecraft:ancient_city_start_degradation").len(),
            2
        );
        assert_eq!(
            load_processor_list("minecraft:ancient_city_walls_degradation").len(),
            3
        );
    }
}
