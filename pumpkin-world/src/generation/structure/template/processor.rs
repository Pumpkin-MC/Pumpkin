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
    input: RulePredicate,
    /// Vanilla `location_predicate`: tested against the block already in the
    /// world at the target position (e.g. streets turn to planks over water).
    location: RulePredicate,
    output_state: &'static BlockState,
    /// Vanilla `block_entity_modifier` of type `minecraft:append_loot`
    /// (AppendLoot.java:34-40): the matched block entity gains this loot table
    /// plus a seed drawn from the rule random (trail-ruins archaeology).
    append_loot: Option<Arc<str>>,
}

/// Result of running a processor over one template block.
pub struct ProcessedBlock {
    pub state: &'static BlockState,
    /// `append_loot` payload: loot table id + `LootTableSeed` drawn from the
    /// per-position rule random after the predicate draws.
    pub loot: Option<(Arc<str>, i64)>,
}

/// Vanilla `RuleTest` subset used by the shipped processor lists.
#[derive(Clone, Copy)]
pub enum RulePredicate {
    AlwaysTrue,
    Block(BlockId),
    /// `random_block_match`: matches the block with the given probability,
    /// consuming one random draw like vanilla `RandomBlockMatchTest`.
    RandomBlock(BlockId, f32),
    /// `blockstate_match`: exact block state comparison (vanilla
    /// `BlockStateMatchTest`), e.g. directional glass panes in zombie villages.
    BlockState(&'static BlockState),
    Tag(BlockTag),
}

impl RulePredicate {
    fn matches(self, state: &'static BlockState, random: &mut LegacyRand) -> bool {
        match self {
            Self::AlwaysTrue => true,
            Self::Block(id) => id == state.id.to_block_id(),
            Self::RandomBlock(id, probability) => {
                id == state.id.to_block_id() && random.next_f32() < probability
            }
            Self::BlockState(expected) => expected.id == state.id,
            Self::Tag(tag) => tag.contains(state.id.to_block_id()),
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
    ) -> Option<ProcessedBlock> {
        let input_block = state.id.to_block_id();
        match self {
            Self::BlockRot { integrity, blocks } => {
                if !blocks.contains(input_block) {
                    return Some(ProcessedBlock { state, loot: None });
                }
                let mut random = LegacyRand::from_seed(hash_block_pos(pos.x, pos.y, pos.z) as u64);
                (random.next_f32() <= *integrity).then_some(ProcessedBlock { state, loot: None })
            }
            Self::Rules(rules) => {
                // Vanilla RuleProcessor: one random per block position, the
                // template block as input and the current world block as the
                // location; the first matching rule wins.
                let mut random = LegacyRand::from_seed(hash_block_pos(pos.x, pos.y, pos.z) as u64);
                let world_state = BlockState::from_id(chunk.get_block_state(&pos));
                rules
                    .iter()
                    .find(|rule| {
                        rule.input.matches(state, &mut random)
                            && rule.location.matches(world_state, &mut random)
                    })
                    .map_or(Some(ProcessedBlock { state, loot: None }), |rule| {
                        // AppendLoot.apply draws the seed from the same rule
                        // random after the predicate draws (AppendLoot.java:38).
                        let loot = rule
                            .append_loot
                            .as_ref()
                            .map(|table| (Arc::clone(table), random.next_i64()));
                        Some(ProcessedBlock {
                            state: rule.output_state,
                            loot,
                        })
                    })
            }
            Self::ProtectedBlocks(blocks) => {
                let existing = chunk.get_block_state(&pos).to_block_id();
                (!blocks.contains(existing)).then_some(ProcessedBlock { state, loot: None })
            }
            // Vanilla CappedProcessor has no per-block behavior (default
            // StructureProcessor.processBlock passes through); its limited
            // random selection runs in the finalize pass of template
            // placement (CappedProcessor.java:54-80).
            Self::Capped { .. } => Some(ProcessedBlock { state, loot: None }),
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
    input_predicate: RawPredicate,
    location_predicate: Option<RawPredicate>,
    output_state: RawOutputState,
    block_entity_modifier: Option<RawBlockEntityModifier>,
}

#[derive(Deserialize)]
struct RawBlockEntityModifier {
    #[serde(rename = "type")]
    kind: String,
    loot_table: Option<String>,
}

#[derive(Deserialize)]
struct RawPredicate {
    predicate_type: Option<String>,
    block: Option<String>,
    block_state: Option<RawOutputState>,
    tag: Option<String>,
    probability: Option<f32>,
}

#[derive(Deserialize)]
struct RawOutputState {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Properties", default)]
    properties: std::collections::BTreeMap<String, String>,
}

/// Resolves a `Name` + `Properties` pair to a concrete block state.
fn resolve_output_state(raw: &RawOutputState) -> Option<&'static BlockState> {
    let name = raw.name.strip_prefix("minecraft:").unwrap_or(&raw.name);
    let block = Block::from_name(name)?;
    if raw.properties.is_empty() {
        return Some(block.default_state);
    }
    let properties = raw
        .properties
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect::<Vec<_>>();
    Some(BlockState::from_id(
        block.from_properties(&properties).to_state_id(block),
    ))
}

fn convert_predicate(raw: &RawPredicate) -> Option<RulePredicate> {
    match raw.predicate_type.as_deref() {
        Some("minecraft:always_true") | None => Some(RulePredicate::AlwaysTrue),
        Some("minecraft:block_match") => {
            let name = raw.block.as_deref()?;
            let block = Block::from_name(name.strip_prefix("minecraft:").unwrap_or(name))?;
            Some(RulePredicate::Block(block.id))
        }
        Some("minecraft:blockstate_match") => {
            // Data carries the state under `block_state` with Name+Properties.
            let raw_state = raw.block_state.as_ref()?;
            resolve_output_state(raw_state).map(RulePredicate::BlockState)
        }
        Some("minecraft:random_block_match") => {
            let name = raw.block.as_deref()?;
            let block = Block::from_name(name.strip_prefix("minecraft:").unwrap_or(name))?;
            Some(RulePredicate::RandomBlock(
                block.id,
                raw.probability.unwrap_or(1.0),
            ))
        }
        Some("minecraft:tag_match") => {
            BlockTag::from_name(raw.tag.as_deref()?).map(RulePredicate::Tag)
        }
        Some(other) => {
            tracing::warn!("Unsupported structure rule predicate: {other}");
            None
        }
    }
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
                    let output_state = resolve_output_state(&rule.output_state)?;
                    let append_loot = match &rule.block_entity_modifier {
                        Some(modifier) if modifier.kind == "minecraft:append_loot" => {
                            Some(Arc::from(modifier.loot_table.as_deref()?))
                        }
                        Some(modifier) if modifier.kind != "minecraft:passthrough" => {
                            tracing::warn!("Unsupported block entity modifier: {}", modifier.kind);
                            None
                        }
                        _ => None,
                    };

                    Some(ProcessorRule {
                        input: convert_predicate(&rule.input_predicate)?,
                        location: rule
                            .location_predicate
                            .as_ref()
                            .map_or(Some(RulePredicate::AlwaysTrue), convert_predicate)?,
                        output_state,
                        append_loot,
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

    #[test]
    fn trail_ruins_archaeology_carries_append_loot() {
        let list = load_processor_list("minecraft:trail_ruins_houses_archaeology");
        let capped_with_loot = list.iter().any(|processor| {
            matches!(processor, StructureProcessor::Capped { delegate, .. }
                if matches!(&**delegate, StructureProcessor::Rules(rules)
                    if rules.iter().any(|rule| rule.append_loot.is_some())))
        });
        assert!(capped_with_loot, "capped delegates must parse append_loot");
    }
}
