use pumpkin_data::{Block, BlockId, BlockState, tag::Taggable};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::{
    math::{int_provider::IntProvider, vector3::Vector3},
    random::{RandomImpl, hash_block_pos, legacy_rand::LegacyRand},
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use super::{BlockPlacer, BlockStateResolver, PaletteEntry};

/// Structure processors are used to dynamically transform blocks in structure templates
/// during world generation or placement.
///
/// See <https://minecraft.wiki/w/Processor_list> for specification.
#[derive(Clone, Debug)]
pub enum StructureProcessor {
    /// Replaces blocks matching rule tests with custom states and block entity modifiers.
    Rule(Vec<ProcessorRule>),
    /// Randomly removes blocks based on integrity, preserving preexisting world blocks.
    BlockRot {
        integrity: f32,
        rottable_blocks: Option<RottableBlocks>,
    },
    /// Ages stone, cobblestone, stairs, slabs, walls, and obsidian blocks with mossy/cracked variants.
    BlockAge { mossiness: f32 },
    /// Removes specified blocks (such as structure void / structure blocks / air), preserving preexisting world blocks.
    BlockIgnore(Vec<IgnoredBlock>),
    /// Modifies the Y-level of blocks relative to terrain heightmap.
    Gravity {
        heightmap: HeightmapType,
        offset: i32,
    },
    /// Protects specific world blocks from being overridden by the structure.
    ProtectedBlocks(String),
    /// Replaces stone variants with blackstone variants and iron bars with chains.
    BlackstoneReplace,
    /// Replaces jigsaw blocks with their target `final_state`.
    JigsawReplacement,
    /// Blocks with incomplete outline shapes cannot override lava in the world.
    LavaSubmergedBlock,
    /// Limits the maximum number of blocks modified by a delegate processor.
    Capped {
        limit: IntProvider,
        delegate: Box<Self>,
    },
    /// No-op processor that leaves blocks unchanged.
    Nop,
}

#[derive(Clone, Debug)]
pub enum RottableBlocks {
    Tag(String),
    Block(BlockId),
    List(Vec<BlockId>),
}

impl RottableBlocks {
    #[must_use]
    pub fn matches(&self, block_id: BlockId) -> bool {
        match self {
            Self::Tag(tag) => check_block_has_tag(block_id, tag),
            Self::Block(id) => *id == block_id,
            Self::List(list) => list.contains(&block_id),
        }
    }
}

#[derive(Clone, Debug)]
pub struct IgnoredBlock {
    pub block_id: BlockId,
    pub properties: Option<Vec<(String, String)>>,
}

impl IgnoredBlock {
    #[must_use]
    pub fn matches(&self, state: &'static BlockState) -> bool {
        if state.id.to_block_id() != self.block_id {
            return false;
        }
        if let Some(props) = &self.properties {
            let block = Block::from_id(self.block_id);
            if let Some(state_props) = block.properties(state.id) {
                let actual = state_props.to_props();
                for (k, v) in props {
                    if !actual.iter().any(|(ak, av)| ak == k && av == v) {
                        return false;
                    }
                }
            } else {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HeightmapType {
    #[default]
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

#[derive(Clone, Debug)]
pub struct ProcessorRule {
    pub position_predicate: PosRuleTest,
    pub input_predicate: RuleTest,
    pub location_predicate: RuleTest,
    pub output_state: &'static BlockState,
    pub block_entity_modifier: Option<BlockEntityModifier>,
}

#[derive(Clone, Debug)]
pub enum RuleTest {
    AlwaysTrue,
    BlockMatch(BlockId),
    BlockStateMatch(BlockStateMatch),
    RandomBlockMatch {
        block: BlockId,
        probability: f32,
    },
    RandomBlockStateMatch {
        match_state: BlockStateMatch,
        probability: f32,
    },
    TagMatch(String),
}

#[derive(Clone, Debug)]
pub struct BlockStateMatch {
    pub block_id: BlockId,
    pub properties: Vec<(String, String)>,
}

impl BlockStateMatch {
    #[must_use]
    pub fn matches(&self, state: &'static BlockState) -> bool {
        if state.id.to_block_id() != self.block_id {
            return false;
        }
        if self.properties.is_empty() {
            return true;
        }
        let block = Block::from_id(self.block_id);
        if let Some(state_props) = block.properties(state.id) {
            let actual = state_props.to_props();
            for (k, v) in &self.properties {
                if !actual.iter().any(|(ak, av)| ak == k && av == v) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

impl RuleTest {
    #[must_use]
    pub fn test(&self, state: &'static BlockState, rng: &mut impl RandomImpl) -> bool {
        match self {
            Self::AlwaysTrue => true,
            Self::BlockMatch(block_id) => state.id.to_block_id() == *block_id,
            Self::BlockStateMatch(matcher) => matcher.matches(state),
            Self::RandomBlockMatch { block, probability } => {
                state.id.to_block_id() == *block && rng.next_f32() < *probability
            }
            Self::RandomBlockStateMatch {
                match_state,
                probability,
            } => match_state.matches(state) && rng.next_f32() < *probability,
            Self::TagMatch(tag) => check_block_has_tag(state.id.to_block_id(), tag),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum PosRuleTest {
    #[default]
    AlwaysTrue,
    LinearPos {
        min_dist: i32,
        max_dist: i32,
        min_chance: f32,
        max_chance: f32,
    },
    AxisAlignedLinearPos {
        axis: Axis,
        min_dist: i32,
        max_dist: i32,
        min_chance: f32,
        max_chance: f32,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Axis {
    X,
    #[default]
    Y,
    Z,
}

impl PosRuleTest {
    #[must_use]
    pub fn test(
        &self,
        pos: Vector3<i32>,
        structure_start: Vector3<i32>,
        rng: &mut impl RandomImpl,
    ) -> bool {
        match self {
            Self::AlwaysTrue => true,
            Self::LinearPos {
                min_dist,
                max_dist,
                min_chance,
                max_chance,
            } => {
                let dx = (pos.x - structure_start.x).abs();
                let dy = (pos.y - structure_start.y).abs();
                let dz = (pos.z - structure_start.z).abs();
                let dist = dx + dy + dz;
                let chance =
                    calculate_linear_chance(dist, *min_dist, *max_dist, *min_chance, *max_chance);
                rng.next_f32() <= chance
            }
            Self::AxisAlignedLinearPos {
                axis,
                min_dist,
                max_dist,
                min_chance,
                max_chance,
            } => {
                let dist = match axis {
                    Axis::X => (pos.x - structure_start.x).abs(),
                    Axis::Y => (pos.y - structure_start.y).abs(),
                    Axis::Z => (pos.z - structure_start.z).abs(),
                };
                let chance =
                    calculate_linear_chance(dist, *min_dist, *max_dist, *min_chance, *max_chance);
                rng.next_f32() <= chance
            }
        }
    }
}

fn calculate_linear_chance(
    dist: i32,
    min_dist: i32,
    max_dist: i32,
    min_chance: f32,
    max_chance: f32,
) -> f32 {
    if max_dist <= min_dist {
        if dist <= min_dist {
            min_chance.clamp(0.0, 1.0)
        } else {
            max_chance.clamp(0.0, 1.0)
        }
    } else if dist <= min_dist {
        min_chance.clamp(0.0, 1.0)
    } else if dist >= max_dist {
        max_chance.clamp(0.0, 1.0)
    } else {
        let t = (dist - min_dist) as f32 / (max_dist - min_dist) as f32;
        (t * (max_chance - min_chance) + min_chance).clamp(0.0, 1.0)
    }
}

#[derive(Clone, Debug)]
pub enum BlockEntityModifier {
    AppendLoot { loot_table: String },
    AppendStatic { data: NbtCompound },
    Clear,
    Passthrough,
}

impl BlockEntityModifier {
    pub fn apply(
        &self,
        nbt: &mut Option<NbtCompound>,
        _pos: Vector3<i32>,
        rng: &mut impl RandomImpl,
    ) {
        match self {
            Self::AppendLoot { loot_table } => {
                let compound = nbt.get_or_insert_with(NbtCompound::new);
                compound.put_string("LootTable", loot_table.clone());
                compound.put_long("LootTableSeed", rng.next_i64());
            }
            Self::AppendStatic { data } => {
                let compound = nbt.get_or_insert_with(NbtCompound::new);
                for (k, v) in &data.child_tags {
                    compound.child_tags.insert(k.clone(), v.clone());
                }
            }
            Self::Clear => {
                *nbt = None;
            }
            Self::Passthrough => {}
        }
    }
}

#[must_use]
pub fn check_block_has_tag(block_id: BlockId, tag: &str) -> bool {
    let raw_tag = tag.strip_prefix('#').unwrap_or(tag);
    let block = Block::from_id(block_id);
    if block.is_tagged_with(raw_tag).unwrap_or(false) {
        return true;
    }
    if !raw_tag.contains(':') {
        let with_namespace = format!("minecraft:{raw_tag}");
        if block.is_tagged_with(&with_namespace).unwrap_or(false) {
            return true;
        }
    }
    false
}

/// Execution context passed during structure template processing.
#[derive(Clone, Debug, Default)]
pub struct ProcessorContext {
    pub structure_start: Vector3<i32>,
    pub capped_limits: Vec<i32>,
    pub capped_counts: Vec<i32>,
}

impl ProcessorContext {
    #[must_use]
    pub fn new(
        structure_start: Vector3<i32>,
        processors: &[StructureProcessor],
        rng: &mut impl RandomImpl,
    ) -> Self {
        let mut capped_limits = Vec::new();
        let mut capped_counts = Vec::new();
        collect_capped_limits(processors, &mut capped_limits, &mut capped_counts, rng);
        Self {
            structure_start,
            capped_limits,
            capped_counts,
        }
    }
}

fn collect_capped_limits(
    processors: &[StructureProcessor],
    limits: &mut Vec<i32>,
    counts: &mut Vec<i32>,
    rng: &mut impl RandomImpl,
) {
    for processor in processors {
        if let StructureProcessor::Capped { limit, delegate } = processor {
            limits.push(limit.get(rng));
            counts.push(0);
            collect_capped_limits(std::slice::from_ref(delegate.as_ref()), limits, counts, rng);
        }
    }
}

impl StructureProcessor {
    #[must_use]
    pub fn process(
        &self,
        placer: &impl BlockPlacer,
        world_pos: Vector3<i32>,
        state: &'static BlockState,
    ) -> Option<&'static BlockState> {
        let mut nbt = None;
        let mut context = ProcessorContext::default();
        let mut capped_idx = 0;
        let mut rng =
            LegacyRand::from_seed(hash_block_pos(world_pos.x, world_pos.y, world_pos.z) as u64);
        self.process_with_context(
            placer,
            world_pos,
            state,
            &mut nbt,
            &mut context,
            &mut capped_idx,
            &mut rng,
        )
    }

    #[must_use]
    #[expect(clippy::too_many_arguments)]
    pub fn process_with_context(
        &self,
        placer: &impl BlockPlacer,
        world_pos: Vector3<i32>,
        state: &'static BlockState,
        nbt: &mut Option<NbtCompound>,
        context: &mut ProcessorContext,
        capped_idx: &mut usize,
        rng: &mut impl RandomImpl,
    ) -> Option<&'static BlockState> {
        match self {
            Self::Rule(rules) => {
                let world_state_id = placer.get_block_state(&world_pos);
                let world_state = BlockState::from_id(world_state_id);
                for rule in rules {
                    if rule
                        .position_predicate
                        .test(world_pos, context.structure_start, rng)
                        && rule.input_predicate.test(state, rng)
                        && rule.location_predicate.test(world_state, rng)
                    {
                        if let Some(modifier) = &rule.block_entity_modifier {
                            modifier.apply(nbt, world_pos, rng);
                        }
                        return Some(rule.output_state);
                    }
                }
                Some(state)
            }
            Self::BlockRot {
                integrity,
                rottable_blocks,
            } => {
                let input_block = state.id.to_block_id();
                if let Some(rottable) = rottable_blocks
                    && !rottable.matches(input_block)
                {
                    return Some(state);
                }
                (rng.next_f32() <= *integrity).then_some(state)
            }
            Self::BlockAge { mossiness } => Some(process_block_age(state, *mossiness, rng)),
            Self::BlockIgnore(ignored_blocks) => {
                if ignored_blocks.iter().any(|ignored| ignored.matches(state)) {
                    None
                } else {
                    Some(state)
                }
            }
            Self::Gravity { .. } | Self::Nop => Some(state),
            Self::ProtectedBlocks(tag) => {
                let world_state_id = placer.get_block_state(&world_pos);
                if check_block_has_tag(world_state_id.to_block_id(), tag) {
                    None
                } else {
                    Some(state)
                }
            }
            Self::BlackstoneReplace => Some(process_blackstone_replace(state)),
            Self::JigsawReplacement => {
                if state.id.to_block_id() == BlockId::JIGSAW {
                    let final_state_str = nbt
                        .as_ref()
                        .and_then(|n| n.get_string("final_state"))
                        .map(str::to_string);
                    *nbt = None;
                    let final_str = final_state_str.as_deref().unwrap_or("minecraft:air");
                    let palette_entry = PaletteEntry::from_string(final_str);
                    BlockStateResolver::resolve_simple(&palette_entry)
                        .or(Some(Block::AIR.default_state))
                } else {
                    Some(state)
                }
            }
            Self::LavaSubmergedBlock => {
                let world_state_id = placer.get_block_state(&world_pos);
                if world_state_id.to_block_id() == BlockId::LAVA
                    && (!state.is_full_cube() || !state.is_solid_block())
                {
                    return None;
                }
                Some(state)
            }
            Self::Capped { delegate, .. } => {
                let idx = *capped_idx;
                *capped_idx += 1;
                if idx < context.capped_limits.len() {
                    let limit = context.capped_limits[idx];
                    let count = context.capped_counts[idx];
                    if count < limit {
                        let prev_state = state;
                        let prev_nbt = nbt.clone();
                        let res = delegate.process_with_context(
                            placer, world_pos, state, nbt, context, capped_idx, rng,
                        );
                        if res != Some(prev_state) || *nbt != prev_nbt {
                            context.capped_counts[idx] += 1;
                        }
                        return res;
                    }
                    Some(state)
                } else {
                    delegate.process_with_context(
                        placer, world_pos, state, nbt, context, capped_idx, rng,
                    )
                }
            }
        }
    }
}

fn replace_preserving_properties(
    old_state: &'static BlockState,
    target_block: &'static Block,
) -> &'static BlockState {
    let old_block = Block::from_id(old_state.id.to_block_id());
    old_block
        .properties(old_state.id)
        .map_or(target_block.default_state, |props_source| {
            let props: Vec<(&str, &str)> = props_source
                .to_props()
                .iter()
                .map(|(k, v)| (*k, *v))
                .collect();
            let new_state_id = target_block
                .from_properties(&props)
                .to_state_id(target_block);
            BlockState::from_id(new_state_id)
        })
}

fn get_random_facing_stairs(
    rng: &mut impl RandomImpl,
    stair_block: &'static Block,
) -> &'static BlockState {
    let facings = ["north", "south", "east", "west"];
    let halves = ["bottom", "top"];
    let facing = facings[rng.next_bounded_i32(4) as usize];
    let half = halves[rng.next_bounded_i32(2) as usize];
    let props = [("facing", facing), ("half", half)];
    let state_id = stair_block.from_properties(&props).to_state_id(stair_block);
    BlockState::from_id(state_id)
}

fn process_block_age(
    state: &'static BlockState,
    mossiness: f32,
    rng: &mut impl RandomImpl,
) -> &'static BlockState {
    let block_id = state.id.to_block_id();

    if matches!(
        block_id,
        BlockId::STONE_BRICKS
            | BlockId::STONE
            | BlockId::CHISELED_STONE_BRICKS
            | BlockId::INFESTED_STONE
            | BlockId::INFESTED_STONE_BRICKS
            | BlockId::INFESTED_CHISELED_STONE_BRICKS
    ) {
        if rng.next_f32() < 0.5 {
            let non_mossy = [
                Block::CRACKED_STONE_BRICKS.default_state,
                get_random_facing_stairs(rng, &Block::STONE_BRICK_STAIRS),
            ];
            let mossy = [
                Block::MOSSY_STONE_BRICKS.default_state,
                get_random_facing_stairs(rng, &Block::MOSSY_STONE_BRICK_STAIRS),
            ];
            let idx = rng.next_bounded_i32(2) as usize;
            if rng.next_f32() < mossiness {
                return mossy[idx];
            }
            return non_mossy[idx];
        }
    } else if check_block_has_tag(block_id, "stairs") {
        if rng.next_f32() < 0.5 {
            let non_mossy = [
                Block::STONE_SLAB.default_state,
                Block::STONE_BRICK_SLAB.default_state,
            ];
            let mossy = [
                replace_preserving_properties(state, &Block::MOSSY_STONE_BRICK_STAIRS),
                Block::MOSSY_STONE_BRICK_SLAB.default_state,
            ];
            let idx = rng.next_bounded_i32(2) as usize;
            if rng.next_f32() < mossiness {
                return mossy[idx];
            }
            return non_mossy[idx];
        }
    } else if check_block_has_tag(block_id, "slabs") {
        if rng.next_f32() < mossiness {
            return replace_preserving_properties(state, &Block::MOSSY_STONE_BRICK_SLAB);
        }
    } else if check_block_has_tag(block_id, "walls") {
        if rng.next_f32() < mossiness {
            return replace_preserving_properties(state, &Block::MOSSY_STONE_BRICK_WALL);
        }
    } else if block_id == BlockId::OBSIDIAN && rng.next_f32() < 0.15 {
        return Block::CRYING_OBSIDIAN.default_state;
    }

    state
}

fn process_blackstone_replace(state: &'static BlockState) -> &'static BlockState {
    let block_id = state.id.to_block_id();
    match block_id {
        BlockId::COBBLESTONE | BlockId::MOSSY_COBBLESTONE | BlockId::INFESTED_COBBLESTONE => {
            Block::BLACKSTONE.default_state
        }
        BlockId::STONE | BlockId::INFESTED_STONE => Block::POLISHED_BLACKSTONE.default_state,
        BlockId::STONE_BRICKS
        | BlockId::MOSSY_STONE_BRICKS
        | BlockId::INFESTED_STONE_BRICKS
        | BlockId::INFESTED_MOSSY_STONE_BRICKS => Block::POLISHED_BLACKSTONE_BRICKS.default_state,
        BlockId::CRACKED_STONE_BRICKS | BlockId::INFESTED_CRACKED_STONE_BRICKS => {
            Block::CRACKED_POLISHED_BLACKSTONE_BRICKS.default_state
        }
        BlockId::CHISELED_STONE_BRICKS | BlockId::INFESTED_CHISELED_STONE_BRICKS => {
            Block::CHISELED_POLISHED_BLACKSTONE.default_state
        }
        BlockId::STONE_BRICK_STAIRS | BlockId::MOSSY_STONE_BRICK_STAIRS => {
            replace_preserving_properties(state, &Block::POLISHED_BLACKSTONE_BRICK_STAIRS)
        }
        BlockId::COBBLESTONE_STAIRS | BlockId::MOSSY_COBBLESTONE_STAIRS | BlockId::STONE_STAIRS => {
            replace_preserving_properties(state, &Block::BLACKSTONE_STAIRS)
        }
        BlockId::STONE_BRICK_SLAB | BlockId::MOSSY_STONE_BRICK_SLAB => {
            replace_preserving_properties(state, &Block::POLISHED_BLACKSTONE_BRICK_SLAB)
        }
        BlockId::COBBLESTONE_SLAB | BlockId::MOSSY_COBBLESTONE_SLAB | BlockId::STONE_SLAB => {
            replace_preserving_properties(state, &Block::BLACKSTONE_SLAB)
        }
        BlockId::STONE_BRICK_WALL | BlockId::MOSSY_STONE_BRICK_WALL => {
            replace_preserving_properties(state, &Block::POLISHED_BLACKSTONE_BRICK_WALL)
        }
        BlockId::COBBLESTONE_WALL | BlockId::MOSSY_COBBLESTONE_WALL => {
            replace_preserving_properties(state, &Block::BLACKSTONE_WALL)
        }
        BlockId::IRON_BARS => replace_preserving_properties(state, &Block::IRON_CHAIN),
        _ => state,
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum RawProcessorListWrapper {
    Object { processors: Vec<RawProcessor> },
    Array(Vec<RawProcessor>),
}

#[derive(Deserialize, Debug)]
#[serde(tag = "processor_type")]
enum RawProcessor {
    #[serde(rename = "minecraft:rule", alias = "rule")]
    Rule { rules: Vec<RawRule> },
    #[serde(rename = "minecraft:block_rot", alias = "block_rot")]
    BlockRot {
        integrity: f32,
        #[serde(default)]
        rottable_blocks: Option<RawRottableBlocks>,
    },
    #[serde(rename = "minecraft:block_age", alias = "block_age")]
    BlockAge { mossiness: f32 },
    #[serde(rename = "minecraft:block_ignore", alias = "block_ignore")]
    BlockIgnore { blocks: Vec<RawBlockStateOrName> },
    #[serde(rename = "minecraft:gravity", alias = "gravity")]
    Gravity {
        #[serde(default = "default_gravity_heightmap")]
        heightmap: String,
        #[serde(default)]
        offset: i32,
    },
    #[serde(rename = "minecraft:protected_blocks", alias = "protected_blocks")]
    ProtectedBlocks { value: String },
    #[serde(rename = "minecraft:blackstone_replace", alias = "blackstone_replace")]
    BlackstoneReplace,
    #[serde(rename = "minecraft:jigsaw_replacement", alias = "jigsaw_replacement")]
    JigsawReplacement,
    #[serde(
        rename = "minecraft:lava_submerged_block",
        alias = "lava_submerged_block"
    )]
    LavaSubmergedBlock,
    #[serde(rename = "minecraft:capped", alias = "capped")]
    Capped {
        #[serde(alias = "value")]
        limit: IntProvider,
        delegate: Box<Self>,
    },
    #[serde(rename = "minecraft:nop", alias = "nop")]
    Nop,
}

fn default_gravity_heightmap() -> String {
    "WORLD_SURFACE_WG".to_string()
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum RawRottableBlocks {
    Single(String),
    List(Vec<String>),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum RawBlockStateOrName {
    State(RawOutputState),
    Name(String),
}

#[derive(Deserialize, Debug)]
struct RawRule {
    #[serde(default)]
    position_predicate: Option<RawPosRuleTest>,
    input_predicate: RawRuleTest,
    #[serde(default)]
    location_predicate: Option<RawRuleTest>,
    output_state: RawOutputState,
    #[serde(default)]
    block_entity_modifier: Option<RawBlockEntityModifier>,
}

#[derive(Deserialize, Debug)]
struct RawOutputState {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Properties", default)]
    properties: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "predicate_type")]
enum RawRuleTest {
    #[serde(rename = "minecraft:always_true", alias = "always_true")]
    AlwaysTrue,
    #[serde(rename = "minecraft:block_match", alias = "block_match")]
    BlockMatch { block: String },
    #[serde(rename = "minecraft:blockstate_match", alias = "blockstate_match")]
    BlockStateMatch {
        #[serde(default)]
        block: Option<String>,
        #[serde(default)]
        block_state: Option<RawOutputState>,
    },
    #[serde(rename = "minecraft:random_block_match", alias = "random_block_match")]
    RandomBlockMatch { block: String, probability: f32 },
    #[serde(
        rename = "minecraft:random_blockstate_match",
        alias = "random_blockstate_match"
    )]
    RandomBlockStateMatch {
        #[serde(default)]
        block: Option<String>,
        #[serde(default)]
        block_state: Option<RawOutputState>,
        probability: f32,
    },
    #[serde(rename = "minecraft:tag_match", alias = "tag_match")]
    TagMatch { tag: String },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "predicate_type")]
enum RawPosRuleTest {
    #[serde(rename = "minecraft:always_true", alias = "always_true")]
    AlwaysTrue,
    #[serde(rename = "minecraft:linear_pos", alias = "linear_pos")]
    LinearPos {
        #[serde(default)]
        min_dist: i32,
        #[serde(default)]
        max_dist: i32,
        #[serde(default)]
        min_chance: f32,
        #[serde(default)]
        max_chance: f32,
    },
    #[serde(
        rename = "minecraft:axis_aligned_linear_pos",
        alias = "axis_aligned_linear_pos"
    )]
    AxisAlignedLinearPos {
        #[serde(default = "default_axis")]
        axis: String,
        #[serde(default)]
        min_dist: i32,
        #[serde(default)]
        max_dist: i32,
        #[serde(default)]
        min_chance: f32,
        #[serde(default)]
        max_chance: f32,
    },
}

fn default_axis() -> String {
    "y".to_string()
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum RawBlockEntityModifier {
    #[serde(rename = "minecraft:append_loot", alias = "append_loot")]
    AppendLoot { loot_table: String },
    #[serde(rename = "minecraft:append_static", alias = "append_static")]
    AppendStatic { data: serde_json::Value },
    #[serde(rename = "minecraft:clear", alias = "clear")]
    Clear,
    #[serde(rename = "minecraft:passthrough", alias = "passthrough")]
    Passthrough,
}

fn resolve_output_state(raw: &RawOutputState) -> Option<&'static BlockState> {
    let name = raw.name.strip_prefix("minecraft:").unwrap_or(&raw.name);
    let block = Block::from_name(name).or_else(|| Block::from_registry_key(name))?;
    if raw.properties.is_empty() {
        Some(block.default_state)
    } else {
        let props_vec: Vec<(&str, &str)> = raw
            .properties
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let state_id = block.from_properties(&props_vec).to_state_id(block);
        Some(BlockState::from_id(state_id))
    }
}

fn json_to_nbt(value: serde_json::Value) -> NbtTag {
    match value {
        serde_json::Value::Null => NbtTag::Byte(0),
        serde_json::Value::Bool(b) => NbtTag::Byte(i8::from(b)),
        serde_json::Value::Number(num) => num.as_i64().map_or_else(
            || num.as_f64().map_or(NbtTag::Int(0), NbtTag::Double),
            |i| i32::try_from(i).map_or(NbtTag::Long(i), NbtTag::Int),
        ),
        serde_json::Value::String(s) => NbtTag::String(s.into_boxed_str()),
        serde_json::Value::Array(arr) => {
            let tags: Vec<NbtTag> = arr.into_iter().map(json_to_nbt).collect();
            NbtTag::List(tags)
        }
        serde_json::Value::Object(map) => {
            let mut compound = NbtCompound::new();
            for (k, v) in map {
                compound.child_tags.insert(k.into(), json_to_nbt(v));
            }
            NbtTag::Compound(compound)
        }
    }
}

fn convert_raw_rule_test(raw: RawRuleTest) -> Option<RuleTest> {
    match raw {
        RawRuleTest::AlwaysTrue => Some(RuleTest::AlwaysTrue),
        RawRuleTest::BlockMatch { block } => {
            let block_name = block.strip_prefix("minecraft:").unwrap_or(&block);
            let block_obj =
                Block::from_name(block_name).or_else(|| Block::from_registry_key(block_name))?;
            Some(RuleTest::BlockMatch(block_obj.id))
        }
        RawRuleTest::BlockStateMatch { block, block_state } => {
            if let Some(raw_state) = block_state {
                let name = raw_state
                    .name
                    .strip_prefix("minecraft:")
                    .unwrap_or(&raw_state.name);
                let block_obj =
                    Block::from_name(name).or_else(|| Block::from_registry_key(name))?;
                Some(RuleTest::BlockStateMatch(BlockStateMatch {
                    block_id: block_obj.id,
                    properties: raw_state.properties.into_iter().collect(),
                }))
            } else if let Some(b) = block {
                let block_name = b.strip_prefix("minecraft:").unwrap_or(&b);
                let block_obj = Block::from_name(block_name)
                    .or_else(|| Block::from_registry_key(block_name))?;
                Some(RuleTest::BlockMatch(block_obj.id))
            } else {
                None
            }
        }
        RawRuleTest::RandomBlockMatch { block, probability } => {
            let block_name = block.strip_prefix("minecraft:").unwrap_or(&block);
            let block_obj =
                Block::from_name(block_name).or_else(|| Block::from_registry_key(block_name))?;
            Some(RuleTest::RandomBlockMatch {
                block: block_obj.id,
                probability,
            })
        }
        RawRuleTest::RandomBlockStateMatch {
            block,
            block_state,
            probability,
        } => {
            if let Some(raw_state) = block_state {
                let name = raw_state
                    .name
                    .strip_prefix("minecraft:")
                    .unwrap_or(&raw_state.name);
                let block_obj =
                    Block::from_name(name).or_else(|| Block::from_registry_key(name))?;
                Some(RuleTest::RandomBlockStateMatch {
                    match_state: BlockStateMatch {
                        block_id: block_obj.id,
                        properties: raw_state.properties.into_iter().collect(),
                    },
                    probability,
                })
            } else if let Some(b) = block {
                let block_name = b.strip_prefix("minecraft:").unwrap_or(&b);
                let block_obj = Block::from_name(block_name)
                    .or_else(|| Block::from_registry_key(block_name))?;
                Some(RuleTest::RandomBlockMatch {
                    block: block_obj.id,
                    probability,
                })
            } else {
                None
            }
        }
        RawRuleTest::TagMatch { tag } => Some(RuleTest::TagMatch(tag)),
    }
}

fn convert_raw_pos_rule_test(raw: Option<RawPosRuleTest>) -> PosRuleTest {
    match raw {
        None | Some(RawPosRuleTest::AlwaysTrue) => PosRuleTest::AlwaysTrue,
        Some(RawPosRuleTest::LinearPos {
            min_dist,
            max_dist,
            min_chance,
            max_chance,
        }) => PosRuleTest::LinearPos {
            min_dist,
            max_dist,
            min_chance,
            max_chance,
        },
        Some(RawPosRuleTest::AxisAlignedLinearPos {
            axis,
            min_dist,
            max_dist,
            min_chance,
            max_chance,
        }) => {
            let axis = match axis.to_lowercase().as_str() {
                "x" => Axis::X,
                "z" => Axis::Z,
                _ => Axis::Y,
            };
            PosRuleTest::AxisAlignedLinearPos {
                axis,
                min_dist,
                max_dist,
                min_chance,
                max_chance,
            }
        }
    }
}

#[expect(clippy::too_many_lines)]
fn convert_raw_processor(raw: RawProcessor) -> Option<StructureProcessor> {
    match raw {
        RawProcessor::BlockRot {
            integrity,
            rottable_blocks,
        } => {
            let rottable = rottable_blocks.and_then(|r| match r {
                RawRottableBlocks::Single(s) => {
                    if s.starts_with('#') {
                        Some(RottableBlocks::Tag(s))
                    } else {
                        let name = s.strip_prefix("minecraft:").unwrap_or(&s);
                        Block::from_name(name)
                            .or_else(|| Block::from_registry_key(name))
                            .map(|b| RottableBlocks::Block(b.id))
                            .or(Some(RottableBlocks::Tag(s)))
                    }
                }
                RawRottableBlocks::List(list) => {
                    let ids: Vec<BlockId> = list
                        .into_iter()
                        .filter_map(|s| {
                            let name = s.strip_prefix("minecraft:").unwrap_or(&s);
                            Block::from_name(name)
                                .or_else(|| Block::from_registry_key(name))
                                .map(|b| b.id)
                        })
                        .collect();
                    Some(RottableBlocks::List(ids))
                }
            });
            Some(StructureProcessor::BlockRot {
                integrity,
                rottable_blocks: rottable,
            })
        }
        RawProcessor::BlockAge { mossiness } => Some(StructureProcessor::BlockAge { mossiness }),
        RawProcessor::BlockIgnore { blocks } => {
            let ignored: Vec<IgnoredBlock> = blocks
                .into_iter()
                .filter_map(|b| match b {
                    RawBlockStateOrName::State(raw_state) => {
                        let name = raw_state
                            .name
                            .strip_prefix("minecraft:")
                            .unwrap_or(&raw_state.name);
                        let block =
                            Block::from_name(name).or_else(|| Block::from_registry_key(name))?;
                        Some(IgnoredBlock {
                            block_id: block.id,
                            properties: if raw_state.properties.is_empty() {
                                None
                            } else {
                                Some(raw_state.properties.into_iter().collect())
                            },
                        })
                    }
                    RawBlockStateOrName::Name(name_str) => {
                        let name = name_str.strip_prefix("minecraft:").unwrap_or(&name_str);
                        let block =
                            Block::from_name(name).or_else(|| Block::from_registry_key(name))?;
                        Some(IgnoredBlock {
                            block_id: block.id,
                            properties: None,
                        })
                    }
                })
                .collect();
            Some(StructureProcessor::BlockIgnore(ignored))
        }
        RawProcessor::Gravity { heightmap, offset } => {
            let heightmap = match heightmap.as_str() {
                "WORLD_SURFACE" => HeightmapType::WorldSurface,
                "OCEAN_FLOOR_WG" => HeightmapType::OceanFloorWg,
                "OCEAN_FLOOR" => HeightmapType::OceanFloor,
                "MOTION_BLOCKING" => HeightmapType::MotionBlocking,
                "MOTION_BLOCKING_NO_LEAVES" => HeightmapType::MotionBlockingNoLeaves,
                _ => HeightmapType::WorldSurfaceWg,
            };
            Some(StructureProcessor::Gravity { heightmap, offset })
        }
        RawProcessor::ProtectedBlocks { value } => Some(StructureProcessor::ProtectedBlocks(value)),
        RawProcessor::BlackstoneReplace => Some(StructureProcessor::BlackstoneReplace),
        RawProcessor::JigsawReplacement => Some(StructureProcessor::JigsawReplacement),
        RawProcessor::LavaSubmergedBlock => Some(StructureProcessor::LavaSubmergedBlock),
        RawProcessor::Rule { rules } => {
            let converted_rules: Vec<ProcessorRule> = rules
                .into_iter()
                .filter_map(|rule| {
                    let output_state = resolve_output_state(&rule.output_state)?;
                    let input_predicate = convert_raw_rule_test(rule.input_predicate)?;
                    let location_predicate = rule
                        .location_predicate
                        .and_then(convert_raw_rule_test)
                        .unwrap_or(RuleTest::AlwaysTrue);
                    let position_predicate = convert_raw_pos_rule_test(rule.position_predicate);
                    let block_entity_modifier = rule.block_entity_modifier.map(|m| match m {
                        RawBlockEntityModifier::AppendLoot { loot_table } => {
                            BlockEntityModifier::AppendLoot { loot_table }
                        }
                        RawBlockEntityModifier::AppendStatic { data } => {
                            if let NbtTag::Compound(compound) = json_to_nbt(data) {
                                BlockEntityModifier::AppendStatic { data: compound }
                            } else {
                                BlockEntityModifier::AppendStatic {
                                    data: NbtCompound::new(),
                                }
                            }
                        }
                        RawBlockEntityModifier::Clear => BlockEntityModifier::Clear,
                        RawBlockEntityModifier::Passthrough => BlockEntityModifier::Passthrough,
                    });

                    Some(ProcessorRule {
                        position_predicate,
                        input_predicate,
                        location_predicate,
                        output_state,
                        block_entity_modifier,
                    })
                })
                .collect();
            Some(StructureProcessor::Rule(converted_rules))
        }
        RawProcessor::Capped { limit, delegate } => {
            convert_raw_processor(*delegate).map(|proc| StructureProcessor::Capped {
                limit,
                delegate: Box::new(proc),
            })
        }
        RawProcessor::Nop => Some(StructureProcessor::Nop),
    }
}

#[must_use]
pub fn load_processor_list(name: &str) -> Arc<[StructureProcessor]> {
    static CACHE: LazyLock<dashmap::DashMap<String, Arc<[StructureProcessor]>>> =
        LazyLock::new(dashmap::DashMap::new);

    let name_key = name.strip_prefix("minecraft:").unwrap_or(name);

    if let Some(processors) = CACHE.get(name_key) {
        return Arc::clone(&processors);
    }

    let Some(json) = super::cache::get_processor_list_json(name) else {
        tracing::warn!("Unknown structure processor list: {name}");
        return Arc::from([]);
    };
    let raw_list: Vec<RawProcessor> = match serde_json::from_str::<RawProcessorListWrapper>(json) {
        Ok(
            RawProcessorListWrapper::Object { processors }
            | RawProcessorListWrapper::Array(processors),
        ) => processors,
        Err(error) => {
            tracing::error!("Failed to parse structure processor list {name}: {error}");
            return Arc::from([]);
        }
    };

    let processors = raw_list
        .into_iter()
        .filter_map(convert_raw_processor)
        .collect::<Arc<[_]>>();
    CACHE.insert(name_key.to_owned(), Arc::clone(&processors));
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
    fn parses_outpost_rot_without_a_block_filter() {
        let processors = load_processor_list("minecraft:outpost_rot");
        assert!(matches!(
            processors.as_ref(),
            [StructureProcessor::BlockRot {
                integrity,
                rottable_blocks: None,
            }] if (*integrity - 0.05).abs() < f32::EPSILON
        ));
    }
}
