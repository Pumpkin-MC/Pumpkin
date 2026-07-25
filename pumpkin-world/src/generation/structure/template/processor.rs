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
    Capped { limit: i32, delegate: Box<Self> },
}

#[derive(Clone)]
pub struct ProcessorRule {
    input: RuleInput,
    probability: f32,
    output_state: &'static BlockState,
}

#[derive(Clone, Copy)]
pub enum RuleInput {
    Block(BlockId),
    Tag(BlockTag),
}

impl RuleInput {
    fn matches(self, block_id: BlockId) -> bool {
        match self {
            Self::Block(id) => id == block_id,
            Self::Tag(tag) => tag.contains(block_id),
        }
    }
}

#[derive(Clone, Copy)]
pub enum BlockTag {
    AncientCityReplaceable,
    FeaturesCannotReplace,
    TrailRuinsReplaceable,
    Doors,
}

impl BlockTag {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "#minecraft:ancient_city_replaceable" | "minecraft:ancient_city_replaceable" => {
                Some(Self::AncientCityReplaceable)
            }
            "#minecraft:features_cannot_replace" | "minecraft:features_cannot_replace" => {
                Some(Self::FeaturesCannotReplace)
            }
            "#minecraft:trail_ruins_replaceable" | "minecraft:trail_ruins_replaceable" => {
                Some(Self::TrailRuinsReplaceable)
            }
            "#minecraft:doors" | "minecraft:doors" => Some(Self::Doors),
            _ => None,
        }
    }

    fn contains(self, block_id: BlockId) -> bool {
        block_id.has_tag(match self {
            Self::AncientCityReplaceable => tag::Block::MINECRAFT_ANCIENT_CITY_REPLACEABLE,
            Self::FeaturesCannotReplace => tag::Block::MINECRAFT_FEATURES_CANNOT_REPLACE,
            Self::TrailRuinsReplaceable => tag::Block::MINECRAFT_TRAIL_RUINS_REPLACEABLE,
            Self::Doors => tag::Block::MINECRAFT_DOORS,
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
                        rule.input.matches(input_block) && random.next_f32() < rule.probability
                    })
                    .map_or(Some(state), |rule| Some(rule.output_state))
            }
            Self::ProtectedBlocks(blocks) => {
                let existing = chunk.get_block_state(&pos).to_block_id();
                (!blocks.contains(existing)).then_some(state)
            }
            Self::Capped { limit: _, delegate } => delegate.process(chunk, pos, state),
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
    #[serde(rename = "minecraft:capped")]
    Capped { limit: i32, delegate: Box<Self> },
}

#[derive(Deserialize)]
struct RawRule {
    input_predicate: RawInputPredicate,
    output_state: RawOutputState,
}

#[derive(Deserialize)]
struct RawInputPredicate {
    block: Option<String>,
    tag: Option<String>,
    probability: Option<f32>,
}

#[derive(Deserialize)]
struct RawOutputState {
    #[serde(rename = "Name")]
    name: String,
}

fn convert_raw_processor(raw: RawProcessor) -> Option<StructureProcessor> {
    match raw {
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
                    let output_name = rule
                        .output_state
                        .name
                        .strip_prefix("minecraft:")
                        .unwrap_or(&rule.output_state.name);
                    let output_block = Block::from_name(output_name)?;

                    let input = if let Some(ref block_name) = rule.input_predicate.block {
                        let input_name =
                            block_name.strip_prefix("minecraft:").unwrap_or(block_name);
                        let input_block = Block::from_name(input_name)?;
                        RuleInput::Block(input_block.id)
                    } else {
                        let tag_name = rule.input_predicate.tag.as_ref()?;
                        let tag = BlockTag::from_name(tag_name)?;
                        RuleInput::Tag(tag)
                    };

                    Some(ProcessorRule {
                        input,
                        probability: rule.input_predicate.probability.unwrap_or(1.0),
                        output_state: output_block.default_state,
                    })
                })
                .collect(),
        )),
        RawProcessor::Capped { limit, delegate } => {
            convert_raw_processor(*delegate).map(|proc| StructureProcessor::Capped {
                limit,
                delegate: Box::new(proc),
            })
        }
    }
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
        .filter_map(convert_raw_processor)
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

    #[test]
    fn parses_street_processor_lists() {
        assert_eq!(load_processor_list("minecraft:street_plains").len(), 1);
        assert_eq!(load_processor_list("minecraft:street_savanna").len(), 1);
    }

    #[test]
    fn parses_trail_ruins_processor_lists() {
        assert_eq!(
            load_processor_list("minecraft:trail_ruins_houses_archaeology").len(),
            3
        );
    }
}
