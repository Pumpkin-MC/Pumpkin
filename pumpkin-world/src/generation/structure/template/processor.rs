use pumpkin_data::{Block, BlockId, BlockState, tag};
use pumpkin_util::{
    HeightMap,
    math::vector3::Vector3,
    random::{RandomImpl, hash_block_pos, legacy_rand::LegacyRand},
};
use serde::Deserialize;
use std::sync::{Arc, LazyLock};

use crate::ProtoChunk;

#[derive(Clone)]
pub enum StructureProcessor {
    BlockRot {
        integrity: f32,
        blocks: BlockTag,
    },
    Rules(Vec<ProcessorRule>),
    ProtectedBlocks(BlockTag),
    Capped {
        limit: i32,
        delegate: Box<Self>,
    },
    /// Vanilla `GravityProcessor` (GravityProcessor.java:40-47): snaps each
    /// block to `heightmap(x, z) + offset + templateLocalY`.
    Gravity {
        heightmap: HeightMap,
        offset: i32,
    },
    /// Vanilla `BlockIgnoreProcessor` (BlockIgnoreProcessor.java:47-52):
    /// drops template blocks whose block is in the list.
    BlockIgnore(Vec<BlockId>),
    /// Vanilla `JigsawReplacementProcessor` (JigsawReplacementProcessor.java:39-63).
    /// The placement pipeline already replaces jigsaw blocks with their
    /// `final_state` before list processors run — matching vanilla's processor
    /// order (SinglePoolElement.java:159-165: `STRUCTURE_BLOCK` ignore, then
    /// jigsaw replacement, then the data-driven list) — so this parses to a
    /// pass-through here; see `place_template_blocks` in `template/mod.rs`.
    JigsawReplacement,
    /// Vanilla `BlockAgeProcessor` (BlockAgeProcessor.java): mossifies
    /// stone-family blocks with the given probability.
    BlockAge {
        mossiness: f32,
    },
    /// Vanilla `LavaSubmergedBlockProcessor` (LavaSubmergedBlockProcessor.java:26-33).
    LavaSubmergedBlock,
    /// Vanilla `BlackstoneReplaceProcessor` (BlackstoneReplaceProcessor.java:31-55).
    BlackstoneReplace,
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
    /// World position of the block. Only `gravity` moves it
    /// (GravityProcessor.java:45-46); every other processor passes it through.
    pub pos: Vector3<i32>,
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
    /// One-block pass-through result (vanilla default `processBlock`).
    const fn pass(pos: Vector3<i32>, state: &'static BlockState) -> Option<ProcessedBlock> {
        Some(ProcessedBlock {
            pos,
            state,
            loot: None,
        })
    }

    /// Runs this processor on one template block.
    ///
    /// Mirrors vanilla `StructureProcessor.processBlock` as invoked from
    /// `StructureTemplate.processBlockInfos` (StructureTemplate.java:380-391):
    /// `pos` is the transformed world position and `template_pos` the raw
    /// template-local position (`blockInfo.pos`, the `templateRelativePos`
    /// argument), while `state` is the untransformed template state.
    #[must_use]
    pub fn process(
        &self,
        chunk: &ProtoChunk,
        pos: Vector3<i32>,
        template_pos: Vector3<i32>,
        state: &'static BlockState,
    ) -> Option<ProcessedBlock> {
        let input_block = state.id.to_block_id();
        match self {
            Self::BlockRot { integrity, blocks } => {
                if !blocks.contains(input_block) {
                    return Self::pass(pos, state);
                }
                let mut random = LegacyRand::from_seed(hash_block_pos(pos.x, pos.y, pos.z) as u64);
                (random.next_f32() <= *integrity).then_some(ProcessedBlock {
                    pos,
                    state,
                    loot: None,
                })
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
                    .map_or(Self::pass(pos, state), |rule| {
                        // AppendLoot.apply draws the seed from the same rule
                        // random after the predicate draws (AppendLoot.java:38).
                        let loot = rule
                            .append_loot
                            .as_ref()
                            .map(|table| (Arc::clone(table), random.next_i64()));
                        Some(ProcessedBlock {
                            pos,
                            state: rule.output_state,
                            loot,
                        })
                    })
            }
            Self::ProtectedBlocks(blocks) => {
                let existing = chunk.get_block_state(&pos).to_block_id();
                (!blocks.contains(existing)).then_some(ProcessedBlock {
                    pos,
                    state,
                    loot: None,
                })
            }
            // Vanilla CappedProcessor has no per-block behavior (default
            // StructureProcessor.processBlock passes through); its limited
            // random selection runs in the finalize pass of template
            // placement (CappedProcessor.java:54-80).
            Self::Capped { .. } => Self::pass(pos, state),
            Self::Gravity { heightmap, offset } => {
                // GravityProcessor.java:41-46: during worldgen the `_WG`
                // heightmap types are used as-is (the `ServerLevel` remap only
                // applies outside generation); the new Y is
                // `level.getHeight(heightmap, x, z) + offset + templateRelativePos.getY()`.
                // `ProtoChunk::get_top_y` returns highest occupied + 1, which
                // is exactly `LevelReader.getHeight` (first free block).
                let surface = chunk.get_top_y(heightmap, pos.x, pos.z) + offset;
                Some(ProcessedBlock {
                    pos: Vector3::new(pos.x, surface + template_pos.y, pos.z),
                    state,
                    loot: None,
                })
            }
            Self::BlockIgnore(blocks) => {
                // BlockIgnoreProcessor.java:48-50: listed blocks are dropped.
                (!blocks.contains(&input_block)).then_some(ProcessedBlock {
                    pos,
                    state,
                    loot: None,
                })
            }
            // Jigsaw blocks are already replaced with their `final_state`
            // before list processors run (see `place_template_blocks`),
            // matching vanilla's processor order (SinglePoolElement.java:159-165),
            // so a data-driven `jigsaw_replacement` entry is a pass-through.
            Self::JigsawReplacement => Self::pass(pos, state),
            Self::BlockAge { mossiness } => {
                // BlockAgeProcessor.java:50: `settings.getRandom(pos)` with no
                // explicit random is `RandomSource.create(Mth.getSeed(pos))`
                // (StructurePlaceSettings.getRandom), i.e. a legacy random
                // seeded with the position hash.
                let mut random = LegacyRand::from_seed(hash_block_pos(pos.x, pos.y, pos.z) as u64);
                let aged = apply_block_age(*mossiness, state, &mut random).unwrap_or(state);
                Self::pass(pos, aged)
            }
            Self::LavaSubmergedBlock => {
                // LavaSubmergedBlockProcessor.java:27-32: a non-full-cube block
                // placed where the world already has lava becomes lava.
                let world_is_lava = chunk.get_block_state(&pos).to_block_id() == Block::LAVA.id;
                if world_is_lava && !state.is_full_cube() {
                    Self::pass(pos, Block::LAVA.default_state)
                } else {
                    Self::pass(pos, state)
                }
            }
            Self::BlackstoneReplace => {
                Self::pass(pos, blackstone_replacement(state).unwrap_or(state))
            }
        }
    }
}

/// `Direction.Plane.HORIZONTAL` iteration order (Direction.java:661:
/// `{NORTH, EAST, SOUTH, WEST}`), indexed by `random.nextInt(4)`
/// (`Util.getRandom`).
const HORIZONTAL_FACINGS: [&str; 4] = ["north", "east", "south", "west"];
/// `Half.values()` order (Half.java:10-11: `TOP, BOTTOM`), indexed by
/// `random.nextInt(2)` (`Util.getRandom`).
const HALVES: [&str; 2] = ["top", "bottom"];

/// Builds a state from `block`'s default state with the given properties
/// overridden. Override keys the block does not have are ignored, mirroring
/// vanilla's `hasProperty` guards.
fn state_with_props(
    block: &'static Block,
    overrides: &[(&'static str, &'static str)],
) -> &'static BlockState {
    let Some(properties) = block.properties(block.default_state.id) else {
        return block.default_state;
    };
    let mut props = properties.to_props();
    for (name, value) in &mut props {
        if let Some(&(_, replacement)) = overrides.iter().find(|(key, _)| key == name) {
            *value = replacement;
        }
    }
    BlockState::from_id(block.from_properties(&props).to_state_id(block))
}

/// Vanilla `BlockStateBase.withPropertiesOf`: copies every property the two
/// blocks share from `source` onto `block`'s default state.
fn with_properties_of(block: &'static Block, source: &'static BlockState) -> &'static BlockState {
    let source_block = Block::from_state_id(source.id);
    source_block.properties(source.id).map_or_else(
        || block.default_state,
        |source_props| state_with_props(block, &source_props.to_props()),
    )
}

/// Vanilla `BlockAgeProcessor.getRandomFacingStairs` (BlockAgeProcessor.java:109-111):
/// draws a horizontal facing (`nextInt(4)`) then a half (`nextInt(2)`).
fn random_facing_stairs(random: &mut LegacyRand, stairs: &'static Block) -> &'static BlockState {
    let facing = HORIZONTAL_FACINGS[random.next_bounded_i32(4) as usize];
    let half = HALVES[random.next_bounded_i32(2) as usize];
    state_with_props(stairs, &[("facing", facing), ("half", half)])
}

/// Vanilla `BlockAgeProcessor.getRandomBlock` (BlockAgeProcessor.java:113-121):
/// one `nextFloat()` against mossiness picks the array, one `nextInt(len)`
/// picks the entry.
fn pick_replacement(
    mossiness: f32,
    random: &mut LegacyRand,
    non_mossy: &[&'static BlockState; 2],
    mossy: &[&'static BlockState; 2],
) -> &'static BlockState {
    let pool = if random.next_f32() < mossiness {
        mossy
    } else {
        non_mossy
    };
    pool[random.next_bounded_i32(2) as usize]
}

/// Vanilla `BlockAgeProcessor.maybeReplaceFullStoneBlock`
/// (BlockAgeProcessor.java:71-78). Draw order: keep-check `nextFloat() >= 0.5`,
/// then both stairs candidates are constructed eagerly (line 75-76: facing +
/// half each), then the mossiness float and the index draw (line 77).
fn maybe_replace_full_stone_block(
    mossiness: f32,
    random: &mut LegacyRand,
) -> Option<&'static BlockState> {
    if random.next_f32() >= 0.5 {
        return None;
    }
    let non_mossy = [
        Block::CRACKED_STONE_BRICKS.default_state,
        random_facing_stairs(random, &Block::STONE_BRICK_STAIRS),
    ];
    let mossy = [
        Block::MOSSY_STONE_BRICKS.default_state,
        random_facing_stairs(random, &Block::MOSSY_STONE_BRICK_STAIRS),
    ];
    Some(pick_replacement(mossiness, random, &non_mossy, &mossy))
}

/// Vanilla `BlockAgeProcessor.maybeReplaceStairs` (BlockAgeProcessor.java:80-86):
/// keep-check `nextFloat() >= 0.5`, then a pick between the static
/// `NON_MOSSY_REPLACEMENTS` (`{STONE_SLAB, STONE_BRICK_SLAB}`,
/// BlockAgeProcessor.java:41) and mossy stairs/slab.
fn maybe_replace_stairs(
    mossiness: f32,
    state: &'static BlockState,
    random: &mut LegacyRand,
) -> Option<&'static BlockState> {
    if random.next_f32() >= 0.5 {
        return None;
    }
    let non_mossy = [
        Block::STONE_SLAB.default_state,
        Block::STONE_BRICK_SLAB.default_state,
    ];
    let mossy = [
        with_properties_of(&Block::MOSSY_STONE_BRICK_STAIRS, state),
        Block::MOSSY_STONE_BRICK_SLAB.default_state,
    ];
    Some(pick_replacement(mossiness, random, &non_mossy, &mossy))
}

/// Vanilla `BlockAgeProcessor.processBlock` dispatch (BlockAgeProcessor.java:49-67):
/// stone/stone bricks/chiseled stone bricks, then `#stairs`, `#slabs`,
/// `#walls`, then obsidian (0.15, BlockAgeProcessor.java:102-107). Returns
/// `None` when the block is left unchanged.
fn apply_block_age(
    mossiness: f32,
    state: &'static BlockState,
    random: &mut LegacyRand,
) -> Option<&'static BlockState> {
    let block_id = state.id.to_block_id();
    if block_id == Block::STONE_BRICKS.id
        || block_id == Block::STONE.id
        || block_id == Block::CHISELED_STONE_BRICKS.id
    {
        maybe_replace_full_stone_block(mossiness, random)
    } else if block_id.has_tag(tag::Block::MINECRAFT_STAIRS) {
        maybe_replace_stairs(mossiness, state, random)
    } else if block_id.has_tag(tag::Block::MINECRAFT_SLABS) {
        // BlockAgeProcessor.java:88-93.
        (random.next_f32() < mossiness)
            .then(|| with_properties_of(&Block::MOSSY_STONE_BRICK_SLAB, state))
    } else if block_id.has_tag(tag::Block::MINECRAFT_WALLS) {
        // BlockAgeProcessor.java:95-100.
        (random.next_f32() < mossiness)
            .then(|| with_properties_of(&Block::MOSSY_STONE_BRICK_WALL, state))
    } else if block_id == Block::OBSIDIAN.id {
        // BlockAgeProcessor.java:102-107: fixed 0.15 probability.
        (random.next_f32() < 0.15).then_some(Block::CRYING_OBSIDIAN.default_state)
    } else {
        None
    }
}

/// Vanilla `BlackstoneReplaceProcessor` replacement map
/// (BlackstoneReplaceProcessor.java:32-54).
fn blackstone_target(block_id: BlockId) -> Option<&'static Block> {
    let map: [(&'static Block, &'static Block); 23] = [
        (&Block::COBBLESTONE, &Block::BLACKSTONE),
        (&Block::MOSSY_COBBLESTONE, &Block::BLACKSTONE),
        (&Block::STONE, &Block::POLISHED_BLACKSTONE),
        (&Block::STONE_BRICKS, &Block::POLISHED_BLACKSTONE_BRICKS),
        (
            &Block::MOSSY_STONE_BRICKS,
            &Block::POLISHED_BLACKSTONE_BRICKS,
        ),
        (&Block::COBBLESTONE_STAIRS, &Block::BLACKSTONE_STAIRS),
        (&Block::MOSSY_COBBLESTONE_STAIRS, &Block::BLACKSTONE_STAIRS),
        (&Block::STONE_STAIRS, &Block::POLISHED_BLACKSTONE_STAIRS),
        (
            &Block::STONE_BRICK_STAIRS,
            &Block::POLISHED_BLACKSTONE_BRICK_STAIRS,
        ),
        (
            &Block::MOSSY_STONE_BRICK_STAIRS,
            &Block::POLISHED_BLACKSTONE_BRICK_STAIRS,
        ),
        (&Block::COBBLESTONE_SLAB, &Block::BLACKSTONE_SLAB),
        (&Block::MOSSY_COBBLESTONE_SLAB, &Block::BLACKSTONE_SLAB),
        (&Block::SMOOTH_STONE_SLAB, &Block::POLISHED_BLACKSTONE_SLAB),
        (&Block::STONE_SLAB, &Block::POLISHED_BLACKSTONE_SLAB),
        (
            &Block::STONE_BRICK_SLAB,
            &Block::POLISHED_BLACKSTONE_BRICK_SLAB,
        ),
        (
            &Block::MOSSY_STONE_BRICK_SLAB,
            &Block::POLISHED_BLACKSTONE_BRICK_SLAB,
        ),
        (
            &Block::STONE_BRICK_WALL,
            &Block::POLISHED_BLACKSTONE_BRICK_WALL,
        ),
        (
            &Block::MOSSY_STONE_BRICK_WALL,
            &Block::POLISHED_BLACKSTONE_BRICK_WALL,
        ),
        (&Block::COBBLESTONE_WALL, &Block::BLACKSTONE_WALL),
        (&Block::MOSSY_COBBLESTONE_WALL, &Block::BLACKSTONE_WALL),
        (
            &Block::CHISELED_STONE_BRICKS,
            &Block::CHISELED_POLISHED_BLACKSTONE,
        ),
        (
            &Block::CRACKED_STONE_BRICKS,
            &Block::CRACKED_POLISHED_BLACKSTONE_BRICKS,
        ),
        (&Block::IRON_BARS, &Block::IRON_CHAIN),
    ];
    map.iter()
        .find(|(from, _)| from.id == block_id)
        .map(|(_, to)| *to)
}

/// Vanilla `BlackstoneReplaceProcessor.processBlock`
/// (BlackstoneReplaceProcessor.java:61-78): map the block, then copy the
/// stair `facing`/`half` and slab `type` properties when the source has them
/// (lines 68-76). Name-based copying is equivalent here because every mapped
/// source that has these properties is a stair/slab/wall sharing the same
/// property definitions. Returns `None` for unmapped blocks.
fn blackstone_replacement(state: &'static BlockState) -> Option<&'static BlockState> {
    let target = blackstone_target(state.id.to_block_id())?;
    let source_block = Block::from_state_id(state.id);
    let source_props = source_block
        .properties(state.id)
        .map_or_else(Vec::new, |props| props.to_props());
    let mut overrides = Vec::new();
    for key in ["facing", "half", "type"] {
        if let Some((_, value)) = source_props.iter().find(|(name, _)| *name == key) {
            overrides.push((key, *value));
        }
    }
    Some(state_with_props(target, &overrides))
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
    /// GravityProcessor.java:33: both fields are optional in the codec, with
    /// defaults `WORLD_SURFACE_WG` and `0`.
    #[serde(rename = "minecraft:gravity")]
    Gravity {
        heightmap: Option<HeightMap>,
        offset: Option<i32>,
    },
    /// BlockIgnoreProcessor.java:35-36: `blocks` is a list of block states
    /// compared by block only (`WEIRD_BLOCK_STATE_CODEC`).
    #[serde(rename = "minecraft:block_ignore")]
    BlockIgnore { blocks: Vec<RawOutputState> },
    #[serde(rename = "minecraft:jigsaw_replacement")]
    JigsawReplacement,
    #[serde(rename = "minecraft:block_age")]
    BlockAge { mossiness: f32 },
    #[serde(rename = "minecraft:lava_submerged_block")]
    LavaSubmergedBlock,
    #[serde(rename = "minecraft:blackstone_replace")]
    BlackstoneReplace,
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
        RawProcessor::Gravity { heightmap, offset } => Some(StructureProcessor::Gravity {
            // Codec defaults (GravityProcessor.java:33).
            heightmap: heightmap.unwrap_or(HeightMap::WorldSurfaceWg),
            offset: offset.unwrap_or(0),
        }),
        RawProcessor::BlockIgnore { blocks } => Some(StructureProcessor::BlockIgnore(
            blocks
                .iter()
                .filter_map(|raw| {
                    let name = raw.name.strip_prefix("minecraft:").unwrap_or(&raw.name);
                    let block = Block::from_name(name);
                    if block.is_none() {
                        tracing::warn!("Unknown block in block_ignore processor: {}", raw.name);
                    }
                    block.map(|block| block.id)
                })
                .collect(),
        )),
        RawProcessor::JigsawReplacement => Some(StructureProcessor::JigsawReplacement),
        RawProcessor::BlockAge { mossiness } => Some(StructureProcessor::BlockAge { mossiness }),
        RawProcessor::LavaSubmergedBlock => Some(StructureProcessor::LavaSubmergedBlock),
        RawProcessor::BlackstoneReplace => Some(StructureProcessor::BlackstoneReplace),
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

    fn parse_list(json: &str) -> Vec<StructureProcessor> {
        let raw: RawProcessorList = serde_json::from_str(json).expect("processor list must parse");
        raw.processors
            .into_iter()
            .filter_map(convert_raw_processor)
            .collect()
    }

    fn state_props(state: &'static BlockState) -> Vec<(&'static str, &'static str)> {
        let block = Block::from_state_id(state.id);
        block
            .properties(state.id)
            .map_or_else(Vec::new, |props| props.to_props())
    }

    fn overworld_chunk() -> crate::ProtoChunk {
        let world_gen = crate::generation::get_world_gen(
            pumpkin_util::world_seed::Seed(0),
            pumpkin_data::dimension::Dimension::OVERWORLD,
            false,
            Vec::new(),
            String::new(),
        );
        crate::ProtoChunk::new(0, 0, &world_gen)
    }

    #[test]
    fn parses_all_code_driven_processor_types() {
        let list = parse_list(
            r#"{"processors": [
                {"processor_type": "minecraft:gravity", "heightmap": "OCEAN_FLOOR_WG", "offset": -1},
                {"processor_type": "minecraft:gravity"},
                {"processor_type": "minecraft:block_ignore", "blocks": [{"Name": "minecraft:air"}]},
                {"processor_type": "minecraft:jigsaw_replacement"},
                {"processor_type": "minecraft:block_age", "mossiness": 0.35},
                {"processor_type": "minecraft:lava_submerged_block"},
                {"processor_type": "minecraft:blackstone_replace"},
                {"processor_type": "minecraft:capped", "limit": 2,
                 "delegate": {"processor_type": "minecraft:block_age", "mossiness": 1.0}}
            ]}"#,
        );
        assert_eq!(list.len(), 8);
        assert!(matches!(
            list[0],
            StructureProcessor::Gravity {
                heightmap: HeightMap::OceanFloorWg,
                offset: -1
            }
        ));
        // Codec defaults: WORLD_SURFACE_WG and 0 (GravityProcessor.java:33).
        assert!(matches!(
            list[1],
            StructureProcessor::Gravity {
                heightmap: HeightMap::WorldSurfaceWg,
                offset: 0
            }
        ));
        assert!(
            matches!(&list[2], StructureProcessor::BlockIgnore(blocks) if blocks == &[Block::AIR.id])
        );
        assert!(matches!(list[3], StructureProcessor::JigsawReplacement));
        assert!(
            matches!(list[4], StructureProcessor::BlockAge { mossiness } if (mossiness - 0.35).abs() < f32::EPSILON)
        );
        assert!(matches!(list[5], StructureProcessor::LavaSubmergedBlock));
        assert!(matches!(list[6], StructureProcessor::BlackstoneReplace));
        assert!(matches!(
            &list[7],
            StructureProcessor::Capped { limit: 2, delegate } if matches!(&**delegate, StructureProcessor::BlockAge { .. })
        ));
    }

    #[test]
    fn unknown_processor_type_fails_the_whole_list_parse() {
        // Contract: unsupported processor types must fail loudly at parse
        // time (the serde error aborts the list; `load_processor_list` logs
        // it and yields an empty list instead of silently skipping).
        let result = serde_json::from_str::<RawProcessorList>(
            r#"{"processors": [{"processor_type": "minecraft:not_a_processor"}]}"#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn block_age_slab_and_wall_follow_mossiness_bounds() {
        // BlockAgeProcessor.java:88-100: `nextFloat() < mossiness` — always
        // true for 1.0 (nextFloat < 1.0), never for 0.0.
        let slab = state_with_props(&Block::STONE_BRICK_SLAB, &[("type", "top")]);
        for seed in 0..32u64 {
            let mut random = LegacyRand::from_seed(seed);
            let aged = apply_block_age(1.0, slab, &mut random).expect("mossiness 1.0 always swaps");
            assert_eq!(
                Block::from_state_id(aged.id).id,
                Block::MOSSY_STONE_BRICK_SLAB.id
            );
            // withPropertiesOf keeps the shared `type` property.
            assert!(state_props(aged).contains(&("type", "top")));

            let mut random = LegacyRand::from_seed(seed);
            assert!(apply_block_age(0.0, slab, &mut random).is_none());

            let mut random = LegacyRand::from_seed(seed);
            let wall = apply_block_age(1.0, Block::COBBLESTONE_WALL.default_state, &mut random)
                .expect("mossiness 1.0 always swaps");
            assert_eq!(
                Block::from_state_id(wall.id).id,
                Block::MOSSY_STONE_BRICK_WALL.id
            );
        }
    }

    #[test]
    fn block_age_full_stone_stays_in_the_vanilla_replacement_pools() {
        // BlockAgeProcessor.java:71-78: with mossiness 1.0 only the mossy pool
        // {mossy_stone_bricks, mossy_stone_brick_stairs} is reachable, and the
        // keep-check drops half of all draws.
        let mut changed = 0u32;
        let mut kept = 0u32;
        for seed in 0..200u64 {
            let mut random = LegacyRand::from_seed(seed);
            match apply_block_age(1.0, Block::STONE.default_state, &mut random) {
                None => kept += 1,
                Some(aged) => {
                    changed += 1;
                    let block = Block::from_state_id(aged.id);
                    assert!(
                        block.id == Block::MOSSY_STONE_BRICKS.id
                            || block.id == Block::MOSSY_STONE_BRICK_STAIRS.id,
                        "unexpected block-age output: {}",
                        block.name
                    );
                }
            }
        }
        assert!(changed > 0, "some seeds must mossify");
        assert!(kept > 0, "some seeds must keep the block");
    }

    #[test]
    fn block_age_stairs_use_the_static_non_mossy_slabs() {
        // BlockAgeProcessor.java:41+80-86: mossiness 0.0 can only produce the
        // static NON_MOSSY_REPLACEMENTS {stone_slab, stone_brick_slab}.
        for seed in 0..200u64 {
            let mut random = LegacyRand::from_seed(seed);
            if let Some(aged) =
                apply_block_age(0.0, Block::STONE_BRICK_STAIRS.default_state, &mut random)
            {
                let block = Block::from_state_id(aged.id);
                assert!(
                    block.id == Block::STONE_SLAB.id || block.id == Block::STONE_BRICK_SLAB.id,
                    "unexpected stairs replacement: {}",
                    block.name
                );
            }
        }
    }

    #[test]
    fn block_age_obsidian_rate_is_fifteen_percent() {
        // BlockAgeProcessor.java:102-107: fixed 0.15 probability, independent
        // of mossiness. 10000 fixed seeds -> deterministic count near 1500.
        let mut replaced = 0u32;
        for seed in 0..10_000u64 {
            let mut random = LegacyRand::from_seed(seed);
            if let Some(aged) = apply_block_age(0.0, Block::OBSIDIAN.default_state, &mut random) {
                assert_eq!(Block::from_state_id(aged.id).id, Block::CRYING_OBSIDIAN.id);
                replaced += 1;
            }
        }
        assert!(
            (1300..=1700).contains(&replaced),
            "obsidian aging rate off: {replaced}/10000"
        );
    }

    #[test]
    fn blackstone_replace_maps_blocks_and_copies_properties() {
        // BlackstoneReplaceProcessor.java:32-54 map entries.
        let simple = blackstone_replacement(Block::COBBLESTONE.default_state).unwrap();
        assert_eq!(Block::from_state_id(simple.id).id, Block::BLACKSTONE.id);

        let bars = blackstone_replacement(Block::IRON_BARS.default_state).unwrap();
        assert_eq!(Block::from_state_id(bars.id).id, Block::IRON_CHAIN.id);

        // BlackstoneReplaceProcessor.java:68-76: facing/half/type are copied.
        let stairs = state_with_props(
            &Block::STONE_BRICK_STAIRS,
            &[("facing", "south"), ("half", "top")],
        );
        let replaced = blackstone_replacement(stairs).unwrap();
        assert_eq!(
            Block::from_state_id(replaced.id).id,
            Block::POLISHED_BLACKSTONE_BRICK_STAIRS.id
        );
        let props = state_props(replaced);
        assert!(props.contains(&("facing", "south")));
        assert!(props.contains(&("half", "top")));

        let slab = state_with_props(&Block::STONE_SLAB, &[("type", "double")]);
        let replaced_slab = blackstone_replacement(slab).unwrap();
        assert_eq!(
            Block::from_state_id(replaced_slab.id).id,
            Block::POLISHED_BLACKSTONE_SLAB.id
        );
        assert!(state_props(replaced_slab).contains(&("type", "double")));

        // Unmapped blocks stay untouched.
        assert!(blackstone_replacement(Block::DIRT.default_state).is_none());
    }

    #[test]
    fn gravity_snaps_blocks_to_the_heightmap() {
        let mut chunk = overworld_chunk();
        // Surface at y=63 -> getHeight (first free) = 64.
        chunk.set_block_state(1, 63, 2, Block::STONE.default_state);

        let processor = StructureProcessor::Gravity {
            heightmap: HeightMap::WorldSurfaceWg,
            offset: -1,
        };
        // GravityProcessor.java:43-46: y = height + offset + templateLocalY.
        let processed = processor
            .process(
                &chunk,
                Vector3::new(1, 100, 2),
                Vector3::new(0, 2, 0),
                Block::DIRT.default_state,
            )
            .expect("gravity never drops blocks");
        assert_eq!(processed.pos, Vector3::new(1, 65, 2));
        assert_eq!(processed.state.id, Block::DIRT.default_state.id);
    }

    #[test]
    fn block_ignore_drops_only_listed_blocks() {
        let chunk = overworld_chunk();
        let processor =
            StructureProcessor::BlockIgnore(vec![Block::AIR.id, Block::STRUCTURE_BLOCK.id]);
        let pos = Vector3::new(4, 10, 4);
        assert!(
            processor
                .process(&chunk, pos, pos, Block::AIR.default_state)
                .is_none()
        );
        let kept = processor
            .process(&chunk, pos, pos, Block::STONE.default_state)
            .expect("unlisted blocks pass through");
        assert_eq!(kept.state.id, Block::STONE.default_state.id);
    }

    #[test]
    fn lava_submerged_block_floods_non_full_cubes() {
        let mut chunk = overworld_chunk();
        let pos = Vector3::new(3, 40, 5);
        chunk.set_block_state(3, 40, 5, Block::LAVA.default_state);

        let processor = StructureProcessor::LavaSubmergedBlock;
        // LavaSubmergedBlockProcessor.java:29: non-full-cube over lava -> lava.
        let flooded = processor
            .process(&chunk, pos, pos, Block::OAK_STAIRS.default_state)
            .expect("processor never drops blocks");
        assert_eq!(flooded.state.id, Block::LAVA.default_state.id);

        // Full cubes stay.
        let kept = processor
            .process(&chunk, pos, pos, Block::STONE.default_state)
            .expect("processor never drops blocks");
        assert_eq!(kept.state.id, Block::STONE.default_state.id);

        // No lava in the world -> untouched.
        let dry_pos = Vector3::new(3, 41, 5);
        let dry = processor
            .process(&chunk, dry_pos, dry_pos, Block::OAK_STAIRS.default_state)
            .expect("processor never drops blocks");
        assert_eq!(dry.state.id, Block::OAK_STAIRS.default_state.id);
    }
}
