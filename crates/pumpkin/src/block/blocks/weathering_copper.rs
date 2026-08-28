use std::sync::Arc;

use pumpkin_data::block_properties::{
    BlockProperties, ChestLikeProperties, ChestType, CopperBulbLikeProperties,
    CopperGolemStatueLikeProperties, DoubleBlockHalf, IronChainLikeProperties,
    LanternLikeProperties, MangroveRootsLikeProperties, OakDoorLikeProperties,
    OakFenceLikeProperties, OakStairsLikeProperties, OakTrapdoorLikeProperties,
    ResinBrickSlabLikeProperties,
};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockId, BlockState, BlockStateId, Mirror, Rotation};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

use crate::block::blocks::doors::DoorBlock;
use crate::block::blocks::slabs::SlabBlock;
use crate::block::blocks::stairs::StairBlock;
use crate::block::blocks::trapdoor::TrapDoorBlock;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockMetadata, BrokenArgs, CanPlaceAtArgs, CanUpdateAtArgs,
    GetStateForNeighborUpdateArgs, NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs,
    OnStateReplacedArgs, PlacedArgs, RandomTickArgs,
};
use crate::world::World;

/// Base chance per random tick to attempt degradation (~5.69%).
pub const BASE_DEGRADATION_CHANCE: f32 = 0.056_888_89;

/// Scan distance in Manhattan metric for neighboring copper blocks.
pub const SCAN_DISTANCE: i32 = 4;

/// Weathering states for oxidizable copper blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WeatherState {
    Unaffected = 0,
    Exposed = 1,
    Weathered = 2,
    Oxidized = 3,
}

impl WeatherState {
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        *self as usize
    }

    #[must_use]
    pub const fn from_ordinal(ordinal: usize) -> Option<Self> {
        match ordinal {
            0 => Some(Self::Unaffected),
            1 => Some(Self::Exposed),
            2 => Some(Self::Weathered),
            3 => Some(Self::Oxidized),
            _ => None,
        }
    }

    #[must_use]
    pub const fn next(&self) -> Option<Self> {
        match self {
            Self::Unaffected => Some(Self::Exposed),
            Self::Exposed => Some(Self::Weathered),
            Self::Weathered => Some(Self::Oxidized),
            Self::Oxidized => None,
        }
    }

    #[must_use]
    pub const fn previous(&self) -> Option<Self> {
        match self {
            Self::Unaffected => None,
            Self::Exposed => Some(Self::Unaffected),
            Self::Weathered => Some(Self::Exposed),
            Self::Oxidized => Some(Self::Weathered),
        }
    }

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Unaffected => "unaffected",
            Self::Exposed => "exposed",
            Self::Weathered => "weathered",
            Self::Oxidized => "oxidized",
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "unaffected" => Some(Self::Unaffected),
            "exposed" => Some(Self::Exposed),
            "weathered" => Some(Self::Weathered),
            "oxidized" => Some(Self::Oxidized),
            _ => None,
        }
    }
}

/// Generic interface for blocks that change over time based on random ticks.
pub trait ChangeOverTimeBlock<T> {
    const SCAN_DISTANCE: i32 = SCAN_DISTANCE;
    const BASE_DEGRADATION_CHANCE: f32 = BASE_DEGRADATION_CHANCE;

    fn get_age(&self, block: &Block) -> Option<T>;
    fn get_chance_modifier(&self, age: T) -> f32;
    fn get_next(&self, block: &Block) -> Option<&'static Block>;
    fn get_previous(&self, block: &Block) -> Option<&'static Block>;
    fn get_first(&self, block: &Block) -> Option<&'static Block>;
}

/// Trait implemented by all weathering copper blocks.
pub trait WeatheringCopper: ChangeOverTimeBlock<WeatherState> {
    fn get_age(&self, block: &Block) -> Option<WeatherState> {
        get_weather_state(block)
    }

    fn get_chance_modifier(&self, age: WeatherState) -> f32 {
        get_chance_modifier(age)
    }

    fn get_next(&self, block: &Block) -> Option<&'static Block> {
        get_next(block)
    }

    fn get_previous(&self, block: &Block) -> Option<&'static Block> {
        get_previous(block)
    }

    fn get_first(&self, block: &Block) -> Option<&'static Block> {
        get_first(block)
    }
}

/// All 14 copper families with their 4 weathering stages (Unaffected, Exposed, Weathered, Oxidized).
const COPPER_PROGRESSIONS: &[(&Block, &Block, &Block, &Block)] = &[
    (
        &Block::COPPER_BLOCK,
        &Block::EXPOSED_COPPER,
        &Block::WEATHERED_COPPER,
        &Block::OXIDIZED_COPPER,
    ),
    (
        &Block::CUT_COPPER,
        &Block::EXPOSED_CUT_COPPER,
        &Block::WEATHERED_CUT_COPPER,
        &Block::OXIDIZED_CUT_COPPER,
    ),
    (
        &Block::CHISELED_COPPER,
        &Block::EXPOSED_CHISELED_COPPER,
        &Block::WEATHERED_CHISELED_COPPER,
        &Block::OXIDIZED_CHISELED_COPPER,
    ),
    (
        &Block::CUT_COPPER_SLAB,
        &Block::EXPOSED_CUT_COPPER_SLAB,
        &Block::WEATHERED_CUT_COPPER_SLAB,
        &Block::OXIDIZED_CUT_COPPER_SLAB,
    ),
    (
        &Block::CUT_COPPER_STAIRS,
        &Block::EXPOSED_CUT_COPPER_STAIRS,
        &Block::WEATHERED_CUT_COPPER_STAIRS,
        &Block::OXIDIZED_CUT_COPPER_STAIRS,
    ),
    (
        &Block::COPPER_DOOR,
        &Block::EXPOSED_COPPER_DOOR,
        &Block::WEATHERED_COPPER_DOOR,
        &Block::OXIDIZED_COPPER_DOOR,
    ),
    (
        &Block::COPPER_TRAPDOOR,
        &Block::EXPOSED_COPPER_TRAPDOOR,
        &Block::WEATHERED_COPPER_TRAPDOOR,
        &Block::OXIDIZED_COPPER_TRAPDOOR,
    ),
    (
        &Block::COPPER_GRATE,
        &Block::EXPOSED_COPPER_GRATE,
        &Block::WEATHERED_COPPER_GRATE,
        &Block::OXIDIZED_COPPER_GRATE,
    ),
    (
        &Block::COPPER_BULB,
        &Block::EXPOSED_COPPER_BULB,
        &Block::WEATHERED_COPPER_BULB,
        &Block::OXIDIZED_COPPER_BULB,
    ),
    (
        &Block::COPPER_LANTERN,
        &Block::EXPOSED_COPPER_LANTERN,
        &Block::WEATHERED_COPPER_LANTERN,
        &Block::OXIDIZED_COPPER_LANTERN,
    ),
    (
        &Block::COPPER_CHEST,
        &Block::EXPOSED_COPPER_CHEST,
        &Block::WEATHERED_COPPER_CHEST,
        &Block::OXIDIZED_COPPER_CHEST,
    ),
    (
        &Block::COPPER_GOLEM_STATUE,
        &Block::EXPOSED_COPPER_GOLEM_STATUE,
        &Block::WEATHERED_COPPER_GOLEM_STATUE,
        &Block::OXIDIZED_COPPER_GOLEM_STATUE,
    ),
    (
        &Block::COPPER_BARS,
        &Block::EXPOSED_COPPER_BARS,
        &Block::WEATHERED_COPPER_BARS,
        &Block::OXIDIZED_COPPER_BARS,
    ),
    (
        &Block::COPPER_CHAIN,
        &Block::EXPOSED_COPPER_CHAIN,
        &Block::WEATHERED_COPPER_CHAIN,
        &Block::OXIDIZED_COPPER_CHAIN,
    ),
];

/// Returns the next oxidized block in sequence, or `None` if already oxidized or not a weathering copper block.
#[must_use]
pub fn get_next(block: &Block) -> Option<&'static Block> {
    for &(unaffected, exposed, weathered, oxidized) in COPPER_PROGRESSIONS {
        if block == unaffected {
            return Some(exposed);
        }
        if block == exposed {
            return Some(weathered);
        }
        if block == weathered {
            return Some(oxidized);
        }
    }
    None
}

/// Returns the previous deoxidized block in sequence, or `None` if unaffected or not a weathering copper block.
#[must_use]
pub fn get_previous(block: &Block) -> Option<&'static Block> {
    for &(unaffected, exposed, weathered, oxidized) in COPPER_PROGRESSIONS {
        if block == oxidized {
            return Some(weathered);
        }
        if block == weathered {
            return Some(exposed);
        }
        if block == exposed {
            return Some(unaffected);
        }
    }
    None
}

/// Returns the first unaffected block in this copper family, or `None` if not a weathering copper block.
#[must_use]
pub fn get_first(block: &Block) -> Option<&'static Block> {
    for &(unaffected, exposed, weathered, oxidized) in COPPER_PROGRESSIONS {
        if block == unaffected || block == exposed || block == weathered || block == oxidized {
            return Some(unaffected);
        }
    }
    None
}

/// Returns the `WeatherState` of the given block, or `None` if it is not an unwaxed weathering copper block.
#[must_use]
pub fn get_weather_state(block: &Block) -> Option<WeatherState> {
    for &(unaffected, exposed, weathered, oxidized) in COPPER_PROGRESSIONS {
        if block == unaffected {
            return Some(WeatherState::Unaffected);
        }
        if block == exposed {
            return Some(WeatherState::Exposed);
        }
        if block == weathered {
            return Some(WeatherState::Weathered);
        }
        if block == oxidized {
            return Some(WeatherState::Oxidized);
        }
    }
    None
}

/// Returns true if the block can weather further (has a next state).
#[must_use]
pub fn is_weathering(block: &Block) -> bool {
    get_next(block).is_some()
}

/// Returns chance modifier for a given weather state (0.75 for Unaffected, 1.0 otherwise).
#[must_use]
pub const fn get_chance_modifier(state: WeatherState) -> f32 {
    match state {
        WeatherState::Unaffected => 0.75,
        WeatherState::Exposed | WeatherState::Weathered | WeatherState::Oxidized => 1.0,
    }
}

/// Converts a block state ID of `from_block` to the corresponding state ID of `to_block`, preserving all block properties.
#[must_use]
pub fn with_properties_of(
    from_block: &Block,
    from_state_id: BlockStateId,
    to_block: &Block,
) -> BlockStateId {
    // 1. Full blocks without properties
    if from_block == &Block::COPPER_BLOCK
        || from_block == &Block::EXPOSED_COPPER
        || from_block == &Block::WEATHERED_COPPER
        || from_block == &Block::OXIDIZED_COPPER
        || from_block == &Block::CUT_COPPER
        || from_block == &Block::EXPOSED_CUT_COPPER
        || from_block == &Block::WEATHERED_CUT_COPPER
        || from_block == &Block::OXIDIZED_CUT_COPPER
        || from_block == &Block::CHISELED_COPPER
        || from_block == &Block::EXPOSED_CHISELED_COPPER
        || from_block == &Block::WEATHERED_CHISELED_COPPER
        || from_block == &Block::OXIDIZED_CHISELED_COPPER
    {
        return to_block.default_state.id;
    }

    // 2. Stairs
    if from_block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_STAIRS) {
        let props = OakStairsLikeProperties::from_state_id(from_state_id, from_block);
        return props.to_state_id(to_block);
    }

    // 3. Slabs
    if from_block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_SLABS) {
        let props = ResinBrickSlabLikeProperties::from_state_id(from_state_id, from_block);
        return props.to_state_id(to_block);
    }

    // 4. Doors
    if from_block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_DOORS) {
        let props = OakDoorLikeProperties::from_state_id(from_state_id, from_block);
        return props.to_state_id(to_block);
    }

    // 5. Trapdoors
    if from_block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_TRAPDOORS) {
        let props = OakTrapdoorLikeProperties::from_state_id(from_state_id, from_block);
        return props.to_state_id(to_block);
    }

    // 6. Copper Bulbs
    if from_block == &Block::COPPER_BULB
        || from_block == &Block::EXPOSED_COPPER_BULB
        || from_block == &Block::WEATHERED_COPPER_BULB
        || from_block == &Block::OXIDIZED_COPPER_BULB
    {
        let props = CopperBulbLikeProperties::from_state_id(from_state_id, from_block);
        return props.to_state_id(to_block);
    }

    // 7. Copper Grates
    if from_block == &Block::COPPER_GRATE
        || from_block == &Block::EXPOSED_COPPER_GRATE
        || from_block == &Block::WEATHERED_COPPER_GRATE
        || from_block == &Block::OXIDIZED_COPPER_GRATE
    {
        let props = MangroveRootsLikeProperties::from_state_id(from_state_id, from_block);
        return props.to_state_id(to_block);
    }

    // 8. Copper Lanterns
    if from_block == &Block::COPPER_LANTERN
        || from_block == &Block::EXPOSED_COPPER_LANTERN
        || from_block == &Block::WEATHERED_COPPER_LANTERN
        || from_block == &Block::OXIDIZED_COPPER_LANTERN
    {
        let props = LanternLikeProperties::from_state_id(from_state_id, from_block);
        return props.to_state_id(to_block);
    }

    // 9. Copper Chests
    if from_block == &Block::COPPER_CHEST
        || from_block == &Block::EXPOSED_COPPER_CHEST
        || from_block == &Block::WEATHERED_COPPER_CHEST
        || from_block == &Block::OXIDIZED_COPPER_CHEST
    {
        let props = ChestLikeProperties::from_state_id(from_state_id, from_block);
        return props.to_state_id(to_block);
    }

    // 10. Copper Golem Statues
    if from_block == &Block::COPPER_GOLEM_STATUE
        || from_block == &Block::EXPOSED_COPPER_GOLEM_STATUE
        || from_block == &Block::WEATHERED_COPPER_GOLEM_STATUE
        || from_block == &Block::OXIDIZED_COPPER_GOLEM_STATUE
    {
        let props = CopperGolemStatueLikeProperties::from_state_id(from_state_id, from_block);
        return props.to_state_id(to_block);
    }

    // 11. Copper Bars
    if from_block == &Block::COPPER_BARS
        || from_block == &Block::EXPOSED_COPPER_BARS
        || from_block == &Block::WEATHERED_COPPER_BARS
        || from_block == &Block::OXIDIZED_COPPER_BARS
    {
        let props = OakFenceLikeProperties::from_state_id(from_state_id, from_block);
        return props.to_state_id(to_block);
    }

    // 12. Copper Chains
    if from_block == &Block::COPPER_CHAIN
        || from_block == &Block::EXPOSED_COPPER_CHAIN
        || from_block == &Block::WEATHERED_COPPER_CHAIN
        || from_block == &Block::OXIDIZED_COPPER_CHAIN
    {
        let props = IronChainLikeProperties::from_state_id(from_state_id, from_block);
        return props.to_state_id(to_block);
    }

    // Fallback: use relative state index
    let offset = from_state_id
        .as_u16()
        .saturating_sub(from_block.default_state.id.as_u16()) as usize;
    to_block
        .states
        .get(offset)
        .map_or(to_block.default_state.id, |s| s.id)
}

/// Returns the next block state for a weathering copper block, preserving all properties.
#[must_use]
pub fn get_next_state(current_block: &Block, state_id: BlockStateId) -> Option<BlockStateId> {
    let next_block = get_next(current_block)?;
    Some(with_properties_of(current_block, state_id, next_block))
}

/// Returns the previous block state for a weathering copper block, preserving all properties.
#[must_use]
pub fn get_previous_state(current_block: &Block, state_id: BlockStateId) -> Option<BlockStateId> {
    let prev_block = get_previous(current_block)?;
    Some(with_properties_of(current_block, state_id, prev_block))
}

/// Returns the first (unaffected) block state in this family, preserving all properties.
#[must_use]
pub fn get_first_state(current_block: &Block, state_id: BlockStateId) -> BlockStateId {
    get_first(current_block).map_or(state_id, |first_block| {
        if first_block == current_block {
            state_id
        } else {
            with_properties_of(current_block, state_id, first_block)
        }
    })
}

/// Scans neighbors within Manhattan distance 4 for other weathering copper blocks.
///
/// Returns `None` if any neighbor has a lower oxidation level (which completely suppresses oxidation).
/// Returns `Some((same_age_count, higher_age_count))` otherwise.
#[must_use]
pub fn scan_neighbor_oxidation_levels(
    world: &World,
    center: &BlockPos,
    current_age: WeatherState,
) -> Option<(usize, usize)> {
    let mut same_age_count = 0;
    let mut higher_age_count = 0;

    for dx in -SCAN_DISTANCE..=SCAN_DISTANCE {
        for dy in -SCAN_DISTANCE..=SCAN_DISTANCE {
            for dz in -SCAN_DISTANCE..=SCAN_DISTANCE {
                let manhattan_dist = dx.abs() + dy.abs() + dz.abs();
                if manhattan_dist > SCAN_DISTANCE || manhattan_dist == 0 {
                    continue;
                }

                let neighbor_pos = BlockPos(Vector3::new(
                    center.0.x + dx,
                    center.0.y + dy,
                    center.0.z + dz,
                ));

                let neighbor_block = world.get_block(&neighbor_pos);
                if let Some(neighbor_age) = get_weather_state(neighbor_block) {
                    if neighbor_age < current_age {
                        // Suppressed by a lower-age neighbor nearby
                        return None;
                    }
                    if neighbor_age > current_age {
                        higher_age_count += 1;
                    } else {
                        same_age_count += 1;
                    }
                }
            }
        }
    }

    Some((same_age_count, higher_age_count))
}

/// Executes a random tick change-over-time attempt on a weathering copper block using vanilla's probability formula.
pub fn change_over_time(world: &Arc<World>, position: &BlockPos, block: &Block) {
    use rand::RngExt;

    // 1. Roll base degradation chance (~5.69%)
    if rand::rng().random::<f32>() >= BASE_DEGRADATION_CHANCE {
        return;
    }

    // 2. Must be an oxidizable block with a next state
    let Some(current_age) = get_weather_state(block) else {
        return;
    };
    let Some(next_block) = get_next(block) else {
        return;
    };

    // 3. Scan neighbors within Manhattan distance 4
    let Some((same_age_count, higher_age_count)) =
        scan_neighbor_oxidation_levels(world, position, current_age)
    else {
        return;
    };

    // 4. Calculate chance: ((higher + 1) / (higher + same + 1))^2 * chance_modifier
    let ratio = (higher_age_count + 1) as f32 / (higher_age_count + same_age_count + 1) as f32;
    let chance = ratio * ratio * get_chance_modifier(current_age);

    if rand::rng().random::<f32>() >= chance {
        return;
    }

    // 5. Apply state change
    let current_state_id = world.get_block_state_id(position);
    let new_state_id = with_properties_of(block, current_state_id, next_block);

    world.set_block_state(position, new_state_id, BlockFlags::NOTIFY_ALL);

    // Special handling for multi-block structures:
    // Door: update upper half if present
    if block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_DOORS) {
        let door_props = OakDoorLikeProperties::from_state_id(current_state_id, block);
        if door_props.half == DoubleBlockHalf::Lower {
            let top_pos = position.up();
            let (top_block, top_state_id) = world.get_block_and_state_id(&top_pos);
            if top_block == block {
                let top_new_state_id = with_properties_of(top_block, top_state_id, next_block);
                world.set_block_state(&top_pos, top_new_state_id, BlockFlags::NOTIFY_ALL);
            }
        }
    }
    // Chest: update right companion chest if double chest
    else if block == &Block::COPPER_CHEST
        || block == &Block::EXPOSED_COPPER_CHEST
        || block == &Block::WEATHERED_COPPER_CHEST
    {
        let chest_props = ChestLikeProperties::from_state_id(current_state_id, block);
        if chest_props.r#type == ChestType::Left {
            let right_dir = chest_props.facing.rotate_clockwise();
            let right_pos = position.offset(right_dir.to_offset());
            let (right_block, right_state_id) = world.get_block_and_state_id(&right_pos);
            if right_block == block {
                let right_new_state_id =
                    with_properties_of(right_block, right_state_id, next_block);
                world.set_block_state(&right_pos, right_new_state_id, BlockFlags::NOTIFY_LISTENERS);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Block implementations
// ---------------------------------------------------------------------------

/// Handles standard full weathering copper blocks, grates, bars, chains, lanterns, and statues.
#[derive(Default)]
pub struct WeatheringCopperBlock;

impl ChangeOverTimeBlock<WeatherState> for WeatheringCopperBlock {
    fn get_age(&self, block: &Block) -> Option<WeatherState> {
        get_weather_state(block)
    }

    fn get_chance_modifier(&self, age: WeatherState) -> f32 {
        get_chance_modifier(age)
    }

    fn get_next(&self, block: &Block) -> Option<&'static Block> {
        get_next(block)
    }

    fn get_previous(&self, block: &Block) -> Option<&'static Block> {
        get_previous(block)
    }

    fn get_first(&self, block: &Block) -> Option<&'static Block> {
        get_first(block)
    }
}

impl WeatheringCopper for WeatheringCopperBlock {}

impl BlockMetadata for WeatheringCopperBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::COPPER_BLOCK,
            BlockId::EXPOSED_COPPER,
            BlockId::WEATHERED_COPPER,
            BlockId::OXIDIZED_COPPER,
            BlockId::CUT_COPPER,
            BlockId::EXPOSED_CUT_COPPER,
            BlockId::WEATHERED_CUT_COPPER,
            BlockId::OXIDIZED_CUT_COPPER,
            BlockId::CHISELED_COPPER,
            BlockId::EXPOSED_CHISELED_COPPER,
            BlockId::WEATHERED_CHISELED_COPPER,
            BlockId::OXIDIZED_CHISELED_COPPER,
            BlockId::COPPER_BARS,
            BlockId::EXPOSED_COPPER_BARS,
            BlockId::WEATHERED_COPPER_BARS,
            BlockId::OXIDIZED_COPPER_BARS,
            BlockId::COPPER_CHAIN,
            BlockId::EXPOSED_COPPER_CHAIN,
            BlockId::WEATHERED_COPPER_CHAIN,
            BlockId::OXIDIZED_COPPER_CHAIN,
            BlockId::COPPER_LANTERN,
            BlockId::EXPOSED_COPPER_LANTERN,
            BlockId::WEATHERED_COPPER_LANTERN,
            BlockId::OXIDIZED_COPPER_LANTERN,
            BlockId::COPPER_GOLEM_STATUE,
            BlockId::EXPOSED_COPPER_GOLEM_STATUE,
            BlockId::WEATHERED_COPPER_GOLEM_STATUE,
            BlockId::OXIDIZED_COPPER_GOLEM_STATUE,
        ]
        .into()
    }
}

impl BlockBehaviour for WeatheringCopperBlock {
    fn random_tick(&self, args: RandomTickArgs<'_>) {
        change_over_time(args.world, args.position, args.block);
    }
}

/// Weathering copper stair blocks.
#[derive(Default)]
pub struct WeatheringCopperStairBlock;

impl ChangeOverTimeBlock<WeatherState> for WeatheringCopperStairBlock {
    fn get_age(&self, block: &Block) -> Option<WeatherState> {
        get_weather_state(block)
    }

    fn get_chance_modifier(&self, age: WeatherState) -> f32 {
        get_chance_modifier(age)
    }

    fn get_next(&self, block: &Block) -> Option<&'static Block> {
        get_next(block)
    }

    fn get_previous(&self, block: &Block) -> Option<&'static Block> {
        get_previous(block)
    }

    fn get_first(&self, block: &Block) -> Option<&'static Block> {
        get_first(block)
    }
}

impl WeatheringCopper for WeatheringCopperStairBlock {}

impl BlockMetadata for WeatheringCopperStairBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::CUT_COPPER_STAIRS,
            BlockId::EXPOSED_CUT_COPPER_STAIRS,
            BlockId::WEATHERED_CUT_COPPER_STAIRS,
            BlockId::OXIDIZED_CUT_COPPER_STAIRS,
        ]
        .into()
    }
}

impl BlockBehaviour for WeatheringCopperStairBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        StairBlock.on_place(args)
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        StairBlock.on_neighbor_update(args);
    }

    fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState {
        StairBlock.rotate(block, state_id, rotation)
    }

    fn mirror(&self, block: &Block, state_id: BlockStateId, mirror: Mirror) -> &'static BlockState {
        StairBlock.mirror(block, state_id, mirror)
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        change_over_time(args.world, args.position, args.block);
    }
}

/// Weathering copper trapdoor blocks.
#[derive(Default)]
pub struct WeatheringCopperTrapDoorBlock;

impl ChangeOverTimeBlock<WeatherState> for WeatheringCopperTrapDoorBlock {
    fn get_age(&self, block: &Block) -> Option<WeatherState> {
        get_weather_state(block)
    }

    fn get_chance_modifier(&self, age: WeatherState) -> f32 {
        get_chance_modifier(age)
    }

    fn get_next(&self, block: &Block) -> Option<&'static Block> {
        get_next(block)
    }

    fn get_previous(&self, block: &Block) -> Option<&'static Block> {
        get_previous(block)
    }

    fn get_first(&self, block: &Block) -> Option<&'static Block> {
        get_first(block)
    }
}

impl WeatheringCopper for WeatheringCopperTrapDoorBlock {}

impl BlockMetadata for WeatheringCopperTrapDoorBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::COPPER_TRAPDOOR,
            BlockId::EXPOSED_COPPER_TRAPDOOR,
            BlockId::WEATHERED_COPPER_TRAPDOOR,
            BlockId::OXIDIZED_COPPER_TRAPDOOR,
        ]
        .into()
    }
}

impl BlockBehaviour for WeatheringCopperTrapDoorBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        TrapDoorBlock.on_place(args)
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        TrapDoorBlock.normal_use(args)
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        TrapDoorBlock.on_neighbor_update(args);
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        change_over_time(args.world, args.position, args.block);
    }
}

/// Weathering copper slab blocks.
#[derive(Default)]
pub struct WeatheringCopperSlabBlock;

impl ChangeOverTimeBlock<WeatherState> for WeatheringCopperSlabBlock {
    fn get_age(&self, block: &Block) -> Option<WeatherState> {
        get_weather_state(block)
    }

    fn get_chance_modifier(&self, age: WeatherState) -> f32 {
        get_chance_modifier(age)
    }

    fn get_next(&self, block: &Block) -> Option<&'static Block> {
        get_next(block)
    }

    fn get_previous(&self, block: &Block) -> Option<&'static Block> {
        get_previous(block)
    }

    fn get_first(&self, block: &Block) -> Option<&'static Block> {
        get_first(block)
    }
}

impl WeatheringCopper for WeatheringCopperSlabBlock {}

impl BlockMetadata for WeatheringCopperSlabBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::CUT_COPPER_SLAB,
            BlockId::EXPOSED_CUT_COPPER_SLAB,
            BlockId::WEATHERED_CUT_COPPER_SLAB,
            BlockId::OXIDIZED_CUT_COPPER_SLAB,
        ]
        .into()
    }
}

impl BlockBehaviour for WeatheringCopperSlabBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        SlabBlock.on_place(args)
    }

    fn can_update_at(&self, args: CanUpdateAtArgs<'_>) -> bool {
        SlabBlock.can_update_at(args)
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        change_over_time(args.world, args.position, args.block);
    }
}

/// Weathering copper door blocks.
#[derive(Default)]
pub struct WeatheringCopperDoorBlock;

impl ChangeOverTimeBlock<WeatherState> for WeatheringCopperDoorBlock {
    fn get_age(&self, block: &Block) -> Option<WeatherState> {
        get_weather_state(block)
    }

    fn get_chance_modifier(&self, age: WeatherState) -> f32 {
        get_chance_modifier(age)
    }

    fn get_next(&self, block: &Block) -> Option<&'static Block> {
        get_next(block)
    }

    fn get_previous(&self, block: &Block) -> Option<&'static Block> {
        get_previous(block)
    }

    fn get_first(&self, block: &Block) -> Option<&'static Block> {
        get_first(block)
    }
}

impl WeatheringCopper for WeatheringCopperDoorBlock {}

impl BlockMetadata for WeatheringCopperDoorBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::COPPER_DOOR,
            BlockId::EXPOSED_COPPER_DOOR,
            BlockId::WEATHERED_COPPER_DOOR,
            BlockId::OXIDIZED_COPPER_DOOR,
        ]
        .into()
    }
}

impl BlockBehaviour for WeatheringCopperDoorBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        DoorBlock.on_place(args)
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        DoorBlock.normal_use(args)
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        DoorBlock.can_place_at(args)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        DoorBlock.placed(args);
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        DoorBlock.broken(args);
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        DoorBlock.on_neighbor_update(args);
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        DoorBlock.get_state_for_neighbor_update(args)
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        DoorBlock.on_state_replaced(args);
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.position);
        let door_props = OakDoorLikeProperties::from_state_id(state_id, args.block);
        if door_props.half == DoubleBlockHalf::Lower {
            change_over_time(args.world, args.position, args.block);
        }
    }
}

/// Weathering copper grate blocks.
#[derive(Default)]
pub struct WeatheringCopperGrateBlock;

impl ChangeOverTimeBlock<WeatherState> for WeatheringCopperGrateBlock {
    fn get_age(&self, block: &Block) -> Option<WeatherState> {
        get_weather_state(block)
    }

    fn get_chance_modifier(&self, age: WeatherState) -> f32 {
        get_chance_modifier(age)
    }

    fn get_next(&self, block: &Block) -> Option<&'static Block> {
        get_next(block)
    }

    fn get_previous(&self, block: &Block) -> Option<&'static Block> {
        get_previous(block)
    }

    fn get_first(&self, block: &Block) -> Option<&'static Block> {
        get_first(block)
    }
}

impl WeatheringCopper for WeatheringCopperGrateBlock {}

impl BlockMetadata for WeatheringCopperGrateBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::COPPER_GRATE,
            BlockId::EXPOSED_COPPER_GRATE,
            BlockId::WEATHERED_COPPER_GRATE,
            BlockId::OXIDIZED_COPPER_GRATE,
        ]
        .into()
    }
}

impl BlockBehaviour for WeatheringCopperGrateBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = MangroveRootsLikeProperties::default(args.block);
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        change_over_time(args.world, args.position, args.block);
    }
}
